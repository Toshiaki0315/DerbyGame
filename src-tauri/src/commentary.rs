use crate::race::RaceResult;
use serde::{Deserialize, Serialize};

/// One line of race commentary, shown once the replay reaches `frame`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentLine {
    pub frame: usize,
    pub text: String,
}

/// Minimum number of frames between two "takes the lead" calls.
const LEAD_CALL_GAP: usize = 5;
/// A runner must gain at least this many places within `CHARGE_WINDOW` frames to be called a charge.
const CHARGE_MIN_GAIN: usize = 2;
const CHARGE_WINDOW: usize = 10;
const CHARGE_CALL_GAP: usize = 8;
/// A lead of at least this many metres at the 200m mark is a breakaway.
const BREAKAWAY_GAP: f64 = 5.0;
/// Finishing times closer than this (seconds), a head or so, are called a photo finish.
const PHOTO_FINISH_SECS: f64 = 0.03;

/// Runner indices ordered front to back (ties keep field order).
fn order(row: &[f64]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..row.len()).collect();
    idx.sort_by(|&a, &b| row[b].total_cmp(&row[a]).then(a.cmp(&b)));
    idx
}

/// Builds the commentary for a race from its recorded positions.
/// `frames[f][i]` is runner `i`'s position (metres) after tick `f`.
pub fn generate(
    names: &[String],
    frames: &[Vec<f64>],
    distance: f64,
    results: &[RaceResult],
) -> Vec<CommentLine> {
    let runners = names.len().min(frames.first().map_or(0, |r| r.len()));
    if runners == 0 || frames.is_empty() {
        return vec![];
    }
    let mut lines: Vec<CommentLine> = vec![];
    let mut say = |frame: usize, text: String| lines.push(CommentLine { frame, text });

    let orders: Vec<Vec<usize>> = frames.iter().map(|r| order(&r[..runners])).collect();
    let leader_at = |f: usize| orders[f][0];

    say(
        0,
        format!(
            "各馬一斉にスタート! {}が好スタートを切りました。",
            names[leader_at(0)]
        ),
    );

    let milestones: [(f64, &str); 3] = [
        (distance * 0.5, "半分"),
        (distance - 400.0, "残り400m"),
        (distance - 200.0, "残り200m"),
    ];
    let mut milestone_done = [false; 3];
    let mut announced_leader = leader_at(0);
    let mut last_lead_call: Option<usize> = None;
    let mut charged = vec![false; runners];
    let mut last_charge_call: Option<usize> = None;

    for f in 0..frames.len() {
        let row = &frames[f];
        let leader = orders[f][0];
        let lead_pos = row[leader];

        // change of lead
        let cooled = last_lead_call.is_none_or(|c| f >= c + LEAD_CALL_GAP);
        if leader != announced_leader && cooled && lead_pos < distance {
            say(f, format!("{}が先頭に立った!", names[leader]));
            announced_leader = leader;
            last_lead_call = Some(f);
        }

        // distance markers, called when the leader reaches them
        for (m, (at, label)) in milestones.iter().enumerate() {
            if milestone_done[m] || lead_pos < *at {
                continue;
            }
            milestone_done[m] = true;
            let second = orders[f].get(1).copied();
            let text = match (m, second) {
                (0, Some(s)) => format!(
                    "レースは{}を通過、先頭は{}、2番手は{}。",
                    label, names[leader], names[s]
                ),
                (0, None) => format!("レースは{}を通過、先頭は{}。", label, names[leader]),
                (1, Some(s)) => {
                    format!("{}! 先頭は{}、2番手に{}。", label, names[leader], names[s])
                }
                (1, None) => format!("{}! 先頭は{}。", label, names[leader]),
                (_, Some(s)) if lead_pos - row[s] >= BREAKAWAY_GAP => {
                    format!("{}! {}が抜け出した!", label, names[leader])
                }
                (_, Some(s)) => format!(
                    "{}! {}と{}が激しい叩き合い!",
                    label, names[leader], names[s]
                ),
                (_, None) => format!("{}! {}が先頭。", label, names[leader]),
            };
            say(f, text);
        }

        // a late charge from the back
        let cooled = last_charge_call.is_none_or(|c| f >= c + CHARGE_CALL_GAP);
        if cooled && f >= CHARGE_WINDOW && lead_pos >= distance * 2.0 / 3.0 && lead_pos < distance {
            let now = &orders[f];
            let before = &orders[f - CHARGE_WINDOW];
            for (rank_now, &h) in now.iter().enumerate().take(3) {
                let rank_before = before.iter().position(|&x| x == h).unwrap_or(rank_now);
                if !charged[h] && rank_before >= rank_now + CHARGE_MIN_GAIN {
                    say(f, format!("{}が後方から一気に追い込んでくる!", names[h]));
                    charged[h] = true;
                    last_charge_call = Some(f);
                    break;
                }
            }
        }
    }

    // finish: top three at the moment each crosses the line
    let finish_frame = |name: &str| -> Option<usize> {
        let i = names.iter().position(|n| n == name)?;
        if i >= runners {
            return None;
        }
        frames.iter().position(|r| r[i] >= distance - 1e-6)
    };
    if let [first, second, ..] = results {
        if second.finish_time - first.finish_time < PHOTO_FINISH_SECS {
            if let Some(f) = finish_frame(&first.name) {
                say(f, "ゴール前は大接戦! 写真判定になりそうです!".to_string());
            }
        }
    }
    for r in results.iter().take(3) {
        if let Some(f) = finish_frame(&r.name) {
            let text = match r.rank {
                1 => format!("{}、1着でゴールイン!", r.name),
                2 => format!("2着は{}。", r.name),
                _ => format!("3着は{}。", r.name),
            };
            say(f, text);
        }
    }

    lines.sort_by_key(|l| l.frame); // stable: keeps generation order within a frame
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(n: &[&str]) -> Vec<String> {
        n.iter().map(|s| s.to_string()).collect()
    }

    /// Frames for runners moving at constant speeds (m/tick), until all finish.
    fn steady(speeds: &[f64], distance: f64) -> Vec<Vec<f64>> {
        let ticks = (distance / speeds.iter().cloned().fold(f64::MAX, f64::min)).ceil() as usize;
        (1..=ticks)
            .map(|t| {
                speeds
                    .iter()
                    .map(|s| (s * t as f64).min(distance))
                    .collect()
            })
            .collect()
    }

    fn results_for(names: &[String], frames: &[Vec<f64>], distance: f64) -> Vec<RaceResult> {
        let mut finished: Vec<(usize, f64, &String)> = names
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let f = frames.iter().position(|fr| fr[i] >= distance).unwrap();
                // sub-tick finish time: earlier index in field order wins ties
                (i, f as f64 + 1.0 + i as f64 * 0.001, n)
            })
            .collect();
        finished.sort_by(|a, b| a.1.total_cmp(&b.1));
        finished
            .iter()
            .enumerate()
            .map(|(r, (_, t, n))| RaceResult {
                rank: r as u32 + 1,
                name: (*n).clone(),
                finish_time: *t,
                margin: String::new(),
            })
            .collect()
    }

    fn run(speeds: &[f64], ns: &[&str], distance: f64) -> (Vec<CommentLine>, Vec<Vec<f64>>) {
        let names = names(ns);
        let frames = steady(speeds, distance);
        let results = results_for(&names, &frames, distance);
        (generate(&names, &frames, distance, &results), frames)
    }

    #[test]
    fn opens_with_the_start_and_names_the_early_leader() {
        let (lines, _) = run(
            &[20.0, 18.0, 16.0],
            &["アルファ", "ベータ", "ガンマ"],
            2400.0,
        );
        assert_eq!(lines[0].frame, 0);
        assert!(lines[0].text.contains("スタート"));
        assert!(lines[0].text.contains("アルファ"));
    }

    #[test]
    fn lines_are_in_frame_order_and_within_the_race() {
        let (lines, frames) = run(
            &[20.0, 18.0, 16.0],
            &["アルファ", "ベータ", "ガンマ"],
            2400.0,
        );
        assert!(lines.windows(2).all(|w| w[0].frame <= w[1].frame));
        assert!(lines.iter().all(|l| l.frame < frames.len()));
        assert!(lines.iter().all(|l| !l.text.is_empty()));
    }

    #[test]
    fn announces_the_top_three_in_order_at_the_moment_they_finish() {
        let ns = ["アルファ", "ベータ", "ガンマ", "デルタ"];
        let (lines, frames) = run(&[20.0, 19.0, 18.0, 16.0], &ns, 2400.0);
        let finish = |i: usize| frames.iter().position(|f| f[i] >= 2400.0).unwrap();
        let find = |needle: &str| lines.iter().find(|l| l.text.contains(needle)).unwrap();
        let first = find("1着");
        let second = find("2着");
        let third = find("3着");
        assert!(first.text.contains("アルファ") && first.frame == finish(0));
        assert!(second.text.contains("ベータ") && second.frame == finish(1));
        assert!(third.text.contains("ガンマ") && third.frame == finish(2));
        assert!(!lines.iter().any(|l| l.text.contains("4着")));
    }

    #[test]
    fn marks_the_half_way_point_and_the_last_400_and_200_metres_once_each() {
        let (lines, frames) = run(
            &[20.0, 18.0, 16.0],
            &["アルファ", "ベータ", "ガンマ"],
            2400.0,
        );
        for (needle, at) in [("半分", 1200.0), ("残り400m", 2000.0), ("残り200m", 2200.0)] {
            let hits: Vec<_> = lines.iter().filter(|l| l.text.contains(needle)).collect();
            assert_eq!(hits.len(), 1, "{needle}");
            let expected = frames.iter().position(|f| f[0] >= at).unwrap();
            assert_eq!(hits[0].frame, expected, "{needle}");
            assert!(
                hits[0].text.contains("アルファ"),
                "{needle} should name the leader"
            );
        }
    }

    #[test]
    fn close_race_at_200m_is_a_dogfight_and_a_clear_lead_is_a_breakaway() {
        let (tight, _) = run(&[20.0, 19.97], &["アルファ", "ベータ"], 2400.0);
        let tight200 = tight.iter().find(|l| l.text.contains("残り200m")).unwrap();
        assert!(tight200.text.contains("叩き合い"), "{}", tight200.text);

        let (clear, _) = run(&[24.0, 16.0], &["アルファ", "ベータ"], 2400.0);
        let clear200 = clear.iter().find(|l| l.text.contains("残り200m")).unwrap();
        assert!(clear200.text.contains("抜け出"), "{}", clear200.text);
    }

    #[test]
    fn announces_a_change_of_lead_naming_the_new_leader() {
        // A leads early, B overtakes around tick 40
        let distance = 2400.0;
        let frames: Vec<Vec<f64>> = (1..=130)
            .map(|t| {
                let t = t as f64;
                vec![
                    (21.0 * t).min(distance),
                    (18.0 * t + if t > 40.0 { 6.0 * (t - 40.0) } else { 0.0 }).min(distance),
                ]
            })
            .collect();
        let ns = names(&["アルファ", "ベータ"]);
        let results = results_for(&ns, &frames, distance);
        let lines = generate(&ns, &frames, distance, &results);
        let change = lines
            .iter()
            .find(|l| l.text.contains("ベータが先頭に立った"))
            .expect("no lead change line");
        let real = frames.iter().position(|f| f[1] > f[0]).unwrap();
        assert!(
            change.frame >= real && change.frame <= real + 6,
            "frame {} vs {}",
            change.frame,
            real
        );
    }

    #[test]
    fn lead_change_lines_are_not_spammed() {
        // two runners trade the lead every tick
        let distance = 1200.0;
        let frames: Vec<Vec<f64>> = (1..=60)
            .map(|t| {
                let base = 20.0 * t as f64;
                let (a, b) = if t % 2 == 0 {
                    (base + 1.0, base)
                } else {
                    (base, base + 1.0)
                };
                vec![a.min(distance), b.min(distance)]
            })
            .collect();
        let ns = names(&["アルファ", "ベータ"]);
        let results = results_for(&ns, &frames, distance);
        let lines = generate(&ns, &frames, distance, &results);
        let changes = lines
            .iter()
            .filter(|l| l.text.contains("先頭に立った"))
            .count();
        assert!(
            changes <= frames.len() / 5 + 1,
            "{changes} lead-change lines"
        );
    }

    #[test]
    fn calls_out_a_late_charge_from_the_back() {
        // C starts last and runs the field down in the final third
        let distance = 2400.0;
        let frames: Vec<Vec<f64>> = (1..=140)
            .map(|t| {
                let t = t as f64;
                let c = if t < 90.0 {
                    14.0 * t
                } else {
                    14.0 * 90.0 + 40.0 * (t - 90.0)
                };
                vec![
                    (20.0 * t).min(distance),
                    (19.0 * t).min(distance),
                    (18.0 * t).min(distance),
                    c.min(distance),
                ]
            })
            .collect();
        let ns = names(&["アルファ", "ベータ", "ガンマ", "デルタ"]);
        let results = results_for(&ns, &frames, distance);
        let lines = generate(&ns, &frames, distance, &results);
        let charge = lines
            .iter()
            .find(|l| l.text.contains("追い込"))
            .expect("no charge line");
        assert!(charge.text.contains("デルタ"));
        let final_third = frames
            .iter()
            .position(|f| f[0] >= distance * 2.0 / 3.0)
            .unwrap();
        assert!(charge.frame >= final_third);
    }

    #[test]
    fn mentions_a_photo_finish_only_when_the_result_is_that_close() {
        let ns = names(&["アルファ", "ベータ"]);
        let frames = steady(&[20.0, 19.9], 1200.0);
        let close = vec![
            RaceResult {
                rank: 1,
                name: "アルファ".into(),
                finish_time: 60.0,
                margin: String::new(),
            },
            RaceResult {
                rank: 2,
                name: "ベータ".into(),
                finish_time: 60.02,
                margin: String::new(),
            },
        ];
        let far = vec![
            RaceResult {
                rank: 1,
                name: "アルファ".into(),
                finish_time: 60.0,
                margin: String::new(),
            },
            RaceResult {
                rank: 2,
                name: "ベータ".into(),
                finish_time: 61.5,
                margin: String::new(),
            },
        ];
        let has = |r: &[RaceResult]| {
            generate(&ns, &frames, 1200.0, r)
                .iter()
                .any(|l| l.text.contains("大接戦"))
        };
        assert!(has(&close));
        assert!(!has(&far));
    }

    #[test]
    fn only_mentions_horses_in_the_field() {
        let ns = ["アルファ", "ベータ", "ガンマ"];
        let (lines, _) = run(&[20.0, 18.0, 16.0], &ns, 2400.0);
        let all: String = lines.iter().map(|l| l.text.clone()).collect();
        assert!(ns.iter().any(|n| all.contains(n)));
        assert!(!all.contains("デルタ"));
    }

    #[test]
    fn handles_empty_input_without_panicking() {
        assert!(generate(&[], &[], 2400.0, &[]).is_empty());
        let ns = names(&["アルファ"]);
        assert!(generate(&ns, &[], 2400.0, &[]).is_empty());
    }

    #[test]
    fn is_deterministic() {
        let (a, _) = run(
            &[20.0, 18.0, 16.0],
            &["アルファ", "ベータ", "ガンマ"],
            2400.0,
        );
        let (b, _) = run(
            &[20.0, 18.0, 16.0],
            &["アルファ", "ベータ", "ガンマ"],
            2400.0,
        );
        assert_eq!(a, b);
    }
}
