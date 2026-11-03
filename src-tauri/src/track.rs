//! Track condition (馬場状態) and how well a horse copes with a heavy track.
//!
//! Ported from the pedigree script's "bad track" logic: a horse's 道悪適性
//! (bad-track aptitude) is mostly its power and partly its dirt aptitude, and
//! on a 重 or 不良 track it softens how much speed is lost and how much faster
//! stamina drains.

use crate::bloodline::Stats;
use crate::rivals::Rng;
use serde::{Deserialize, Serialize};

/// Share of 道悪適性 that comes from power (the rest is dirt aptitude).
const POWER_WEIGHT: f64 = 0.7;
const DIRT_WEIGHT: f64 = 0.3;
/// Stamina drain is relieved only half as much as speed loss is ("anyone tires in mud").
const STAMINA_MITIGATION_SHARE: f64 = 0.5;

/// 良 (firm), 稍重 (slightly soft), 重 (soft) or 不良 (heavy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackCondition {
    Good,
    Soft,
    Yielding,
    Heavy,
}

pub const ALL_CONDITIONS: [TrackCondition; 4] = [
    TrackCondition::Good,
    TrackCondition::Soft,
    TrackCondition::Yielding,
    TrackCondition::Heavy,
];

impl TrackCondition {
    /// The most speed a horse can lose here (as a fraction), with no aptitude at all.
    fn max_speed_penalty(self) -> f64 {
        match self {
            TrackCondition::Good => 0.0,
            TrackCondition::Soft => 0.07,
            TrackCondition::Yielding => 0.15,
            TrackCondition::Heavy => 0.30,
        }
    }

    /// How many times faster stamina is used up here, with no aptitude at all.
    fn max_stamina_drain(self) -> f64 {
        match self {
            TrackCondition::Good => 1.0,
            TrackCondition::Soft => 1.1,
            TrackCondition::Yielding => 1.2,
            TrackCondition::Heavy => 1.5,
        }
    }
}

/// 道悪適性: 70% power, 30% dirt aptitude (each 0-100).
pub fn bad_track_aptitude(power: f64, dirt: f64) -> f64 {
    power * POWER_WEIGHT + dirt * DIRT_WEIGHT
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// A horse's dirt aptitude (-2 poor .. +2 specialist) on the 0-100 scale the
/// bad-track formula uses.
pub fn dirt_score(aptitude: i8) -> f64 {
    50.0 + 25.0 * aptitude as f64
}

/// Track condition for a race day, drawn from `seed`. Most days are firm;
/// the rainy seasons (weeks 21-27 and 37-40) are wetter and winter is drier.
pub fn forecast(seed: u64, week: u8) -> TrackCondition {
    let (firm, slightly, soft) = match week {
        21..=27 | 37..=40 => (0.45, 0.65, 0.85),
        1..=8 => (0.85, 0.93, 0.98),
        _ => (0.70, 0.84, 0.94),
    };
    let r = Rng::new(seed).next_f64();
    if r < firm {
        TrackCondition::Good
    } else if r < slightly {
        TrackCondition::Soft
    } else if r < soft {
        TrackCondition::Yielding
    } else {
        TrackCondition::Heavy
    }
}

/// A horse's speed and stamina on the given track.
///
/// On 良 nothing changes. Otherwise aptitude (0-100) cancels part of the loss:
/// full aptitude removes the speed penalty entirely, but even then stamina
/// drains somewhat faster. Results are rounded to one decimal.
pub fn apply_track_condition(stats: &Stats, condition: TrackCondition) -> (f64, f64) {
    apply_track(
        stats.speed,
        stats.stamina,
        stats.power,
        stats.dirt,
        condition,
    )
}

/// [`apply_track_condition`] on plain numbers.
pub fn apply_track(
    speed: f64,
    stamina: f64,
    power: f64,
    dirt: f64,
    condition: TrackCondition,
) -> (f64, f64) {
    if condition == TrackCondition::Good {
        return (speed, stamina);
    }
    let apt = bad_track_aptitude(power, dirt);
    let mitigation = (apt / 100.0).min(1.0);

    let speed_penalty = condition.max_speed_penalty() * (1.0 - mitigation);
    let speed = speed * (1.0 - speed_penalty);

    let drain =
        1.0 + (condition.max_stamina_drain() - 1.0) * (1.0 - mitigation * STAMINA_MITIGATION_SHARE);
    let stamina = stamina / drain;

    (round1(speed), round1(stamina))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionEffect {
    pub condition: TrackCondition,
    pub speed: f64,
    pub stamina: f64,
}

/// A horse's bad-track aptitude and its effective speed and stamina on every track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackEffects {
    pub aptitude: f64,
    pub conditions: Vec<ConditionEffect>,
}

pub fn track_effects(stats: &Stats) -> TrackEffects {
    TrackEffects {
        aptitude: round1(bad_track_aptitude(stats.power, stats.dirt)),
        conditions: ALL_CONDITIONS
            .iter()
            .map(|&condition| {
                let (speed, stamina) = apply_track_condition(stats, condition);
                ConditionEffect {
                    condition,
                    speed,
                    stamina,
                }
            })
            .collect(),
    }
}

/// The effect of each track on a horse with these four stats.
#[tauri::command]
pub fn get_track_effects(speed: f64, stamina: f64, power: f64, dirt: f64) -> TrackEffects {
    track_effects(&Stats {
        speed,
        stamina,
        temper: 0.0,
        power,
        turf: 0.0,
        dirt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use TrackCondition::*;

    fn stats(speed: f64, stamina: f64, power: f64, dirt: f64) -> Stats {
        Stats {
            speed,
            stamina,
            temper: 50.0,
            power,
            turf: 50.0,
            dirt,
        }
    }

    // The same horses and expected numbers as the original script printed.
    fn speedy() -> Stats {
        stats(90.0, 60.0, 30.0, 20.0)
    }
    fn power_type() -> Stats {
        stats(75.0, 65.0, 85.0, 80.0)
    }

    #[test]
    fn bad_track_aptitude_is_seventy_percent_power_and_thirty_percent_dirt() {
        assert_eq!(bad_track_aptitude(100.0, 0.0), 70.0);
        assert_eq!(bad_track_aptitude(0.0, 100.0), 30.0);
        assert_eq!(bad_track_aptitude(50.0, 50.0), 50.0);
        assert_eq!(bad_track_aptitude(0.0, 0.0), 0.0);
        assert_eq!(bad_track_aptitude(100.0, 100.0), 100.0);
        assert!((bad_track_aptitude(30.0, 20.0) - 27.0).abs() < 1e-9);
        assert!((bad_track_aptitude(85.0, 80.0) - 83.5).abs() < 1e-9);
    }

    #[test]
    fn a_firm_track_changes_nothing_and_does_not_round() {
        assert_eq!(
            apply_track_condition(&stats(85.123, 61.987, 10.0, 10.0), Good),
            (85.123, 61.987)
        );
        assert_eq!(apply_track_condition(&speedy(), Good), (90.0, 60.0));
    }

    #[test]
    fn matches_the_original_scripts_output_for_a_speedy_weak_horse() {
        assert_eq!(apply_track_condition(&speedy(), Yielding), (80.1, 51.2));
        assert_eq!(apply_track_condition(&speedy(), Heavy), (70.3, 41.9));
    }

    #[test]
    fn matches_the_original_scripts_output_for_a_powerful_dirt_horse() {
        assert_eq!(apply_track_condition(&power_type(), Yielding), (73.1, 58.2));
        assert_eq!(apply_track_condition(&power_type(), Heavy), (71.3, 50.3));
    }

    #[test]
    fn matches_the_original_scripts_output_at_the_extremes_and_the_middle() {
        let max = stats(80.0, 70.0, 100.0, 100.0);
        assert_eq!(apply_track_condition(&max, Yielding), (80.0, 63.6));
        assert_eq!(apply_track_condition(&max, Heavy), (80.0, 56.0));
        let zero = stats(80.0, 70.0, 0.0, 0.0);
        assert_eq!(apply_track_condition(&zero, Yielding), (68.0, 58.3));
        assert_eq!(apply_track_condition(&zero, Heavy), (56.0, 46.7));
        let mid = stats(66.6, 55.5, 50.0, 50.0);
        assert_eq!(apply_track_condition(&mid, Yielding), (61.6, 48.3));
        assert_eq!(apply_track_condition(&mid, Heavy), (56.6, 40.4));
    }

    #[test]
    fn full_aptitude_removes_the_speed_penalty_but_not_the_stamina_drain() {
        let max = stats(80.0, 70.0, 100.0, 100.0);
        for c in [Yielding, Heavy] {
            let (speed, stamina) = apply_track_condition(&max, c);
            assert_eq!(speed, 80.0);
            assert!(
                stamina < 70.0,
                "stamina should still drain on a {c:?} track"
            );
        }
    }

    #[test]
    fn no_aptitude_takes_the_full_penalty() {
        let zero = stats(100.0, 90.0, 0.0, 0.0);
        assert_eq!(apply_track_condition(&zero, Yielding), (85.0, 75.0)); // -15%, drain x1.2
        assert_eq!(apply_track_condition(&zero, Heavy), (70.0, 60.0)); // -30%, drain x1.5
    }

    #[test]
    fn better_aptitude_always_loses_less() {
        for c in [Yielding, Heavy] {
            let mut last = (0.0, 0.0);
            for power in (0..=100).step_by(10) {
                let r = apply_track_condition(&stats(80.0, 70.0, power as f64, power as f64), c);
                assert!(
                    r.0 >= last.0 && r.1 >= last.1,
                    "{c:?} at power {power}: {r:?} after {last:?}"
                );
                last = r;
            }
        }
    }

    #[test]
    fn heavier_tracks_are_never_easier() {
        for power in [0.0, 25.0, 50.0, 75.0, 100.0] {
            for dirt in [0.0, 50.0, 100.0] {
                let s = stats(77.7, 66.6, power, dirt);
                let good = apply_track_condition(&s, Good);
                let soft = apply_track_condition(&s, Yielding);
                let heavy = apply_track_condition(&s, Heavy);
                assert!(
                    good.0 >= soft.0 && soft.0 >= heavy.0,
                    "speed {power}/{dirt}"
                );
                assert!(
                    good.1 >= soft.1 && soft.1 >= heavy.1,
                    "stamina {power}/{dirt}"
                );
            }
        }
    }

    #[test]
    fn stamina_is_relieved_less_than_speed() {
        // at the same aptitude, the share of speed kept grows faster than the share of stamina
        let s = stats(100.0, 100.0, 80.0, 80.0);
        let (speed, stamina) = apply_track_condition(&s, Heavy);
        assert!(speed > stamina, "{speed} vs {stamina}");
    }

    #[test]
    fn a_power_horse_copes_with_mud_better_than_a_speed_horse() {
        let kept = |s: &Stats| {
            let (speed, stamina) = apply_track_condition(s, Heavy);
            (speed / s.speed, stamina / s.stamina)
        };
        let (sp_speed, sp_stamina) = kept(&speedy());
        let (pw_speed, pw_stamina) = kept(&power_type());
        assert!(pw_speed > sp_speed + 0.1, "{pw_speed} vs {sp_speed}");
        // stamina is relieved less than speed, so the gap is smaller there
        assert!(
            pw_stamina > sp_stamina + 0.05,
            "{pw_stamina} vs {sp_stamina}"
        );
        assert!(pw_speed - sp_speed > pw_stamina - sp_stamina);
        // and the speedy horse is faster on a firm track but not in the mud
        assert!(speedy().speed > power_type().speed);
        assert!(
            apply_track_condition(&speedy(), Heavy).0
                < apply_track_condition(&power_type(), Heavy).0
        );
    }

    #[test]
    fn results_are_rounded_to_one_decimal() {
        for c in [Yielding, Heavy] {
            let (speed, stamina) = apply_track_condition(&stats(71.37, 63.91, 33.3, 41.7), c);
            for v in [speed, stamina] {
                assert!(((v * 10.0).round() - v * 10.0).abs() < 1e-6, "{v}");
            }
        }
    }

    #[test]
    fn a_slightly_soft_track_costs_less_than_a_soft_one() {
        let (speed, stamina) = apply_track_condition(&speedy(), Soft);
        let (s2, t2) = apply_track_condition(&speedy(), Yielding);
        assert!(speed < 90.0 && speed > s2, "{speed} {s2}");
        assert!(stamina < 60.0 && stamina > t2, "{stamina} {t2}");
    }

    #[test]
    fn all_four_going_descriptions_come_up_and_the_rainy_season_is_wetter() {
        let count = |week: u8, c: TrackCondition| {
            (0..4000u64)
                .filter(|&seed| forecast(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15), week) == c)
                .count()
        };
        for c in ALL_CONDITIONS {
            assert!(count(30, c) > 0, "{c:?}");
        }
        assert!(count(24, Heavy) > count(3, Heavy));
        assert!(count(24, Good) < count(3, Good));
    }

    #[test]
    fn track_effects_list_every_condition_in_order() {
        let e = track_effects(&speedy());
        assert_eq!(e.aptitude, 27.0);
        let order: Vec<TrackCondition> = e.conditions.iter().map(|c| c.condition).collect();
        assert_eq!(order, vec![Good, Soft, Yielding, Heavy]);
        assert_eq!(
            (e.conditions[0].speed, e.conditions[0].stamina),
            (90.0, 60.0)
        );
        assert_eq!(
            (e.conditions[2].speed, e.conditions[2].stamina),
            (80.1, 51.2)
        );
        assert_eq!(
            (e.conditions[3].speed, e.conditions[3].stamina),
            (70.3, 41.9)
        );
    }

    #[test]
    fn aptitude_is_reported_to_one_decimal() {
        assert_eq!(track_effects(&stats(70.0, 70.0, 33.3, 41.7)).aptitude, 35.8);
    }

    #[test]
    fn the_command_and_json_shapes_match_what_the_ui_reads() {
        let e = get_track_effects(90.0, 60.0, 30.0, 20.0);
        assert_eq!(e, track_effects(&speedy()));
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["aptitude"], 27.0);
        assert_eq!(v["conditions"][0]["condition"], "good");
        assert_eq!(v["conditions"][1]["condition"], "soft");
        assert_eq!(v["conditions"][2]["condition"], "yielding");
        assert_eq!(v["conditions"][3]["condition"], "heavy");
        assert_eq!(v["conditions"][3]["speed"], 70.3);
        assert_eq!(v["conditions"][3]["stamina"], 41.9);
    }

    #[test]
    fn dirt_aptitude_maps_onto_a_zero_to_hundred_score() {
        assert_eq!(dirt_score(-2), 0.0);
        assert_eq!(dirt_score(-1), 25.0);
        assert_eq!(dirt_score(0), 50.0);
        assert_eq!(dirt_score(1), 75.0);
        assert_eq!(dirt_score(2), 100.0);
    }

    #[test]
    fn the_plain_number_form_matches_the_stats_form() {
        for s in [speedy(), power_type(), stats(66.6, 55.5, 50.0, 50.0)] {
            for c in ALL_CONDITIONS {
                assert_eq!(
                    apply_track(s.speed, s.stamina, s.power, s.dirt, c),
                    apply_track_condition(&s, c)
                );
            }
        }
    }

    fn share(week: u8, want: TrackCondition) -> f64 {
        (0..4000u64)
            .filter(|&seed| forecast(seed, week) == want)
            .count() as f64
            / 4000.0
    }

    #[test]
    fn the_forecast_is_a_pure_function_of_seed_and_week() {
        for seed in 0..50u64 {
            for week in [1u8, 25, 40] {
                assert_eq!(forecast(seed, week), forecast(seed, week));
            }
        }
    }

    #[test]
    fn most_race_days_are_firm_but_every_condition_happens() {
        let (good, slightly, soft, heavy) = (
            share(15, Good),
            share(15, Soft),
            share(15, Yielding),
            share(15, Heavy),
        );
        assert!((0.66..0.74).contains(&good), "良 {good}");
        assert!((0.10..0.18).contains(&slightly), "稍重 {slightly}");
        assert!((0.06..0.14).contains(&soft), "重 {soft}");
        assert!((0.02..0.10).contains(&heavy), "不良 {heavy}");
        assert!((good + slightly + soft + heavy - 1.0).abs() < 1e-9);
    }

    #[test]
    fn the_rainy_seasons_are_wetter_and_winter_is_drier() {
        let dry_winter = share(3, Good);
        let normal = share(15, Good);
        for rainy in [22u8, 25, 38] {
            let wet = share(rainy, Good);
            assert!(wet < normal - 0.15, "week {rainy}: {wet} vs {normal}");
            assert!(
                share(rainy, Heavy) > share(15, Heavy) + 0.05,
                "week {rainy}"
            );
        }
        assert!(dry_winter > normal + 0.08, "{dry_winter} vs {normal}");
    }

    #[test]
    fn works_for_every_horse_in_the_generated_pedigree() {
        for h in crate::bloodline::generate(1) {
            let e = track_effects(&h.stats);
            // a mutated power or dirt aptitude can lift it past 100
            assert!((0.0..=125.0).contains(&e.aptitude));
            for c in &e.conditions {
                assert!(c.speed >= 0.0 && c.speed <= h.stats.speed + 1e-9);
                assert!(c.stamina >= 0.0 && c.stamina <= h.stats.stamina + 1e-9);
            }
        }
    }
}
