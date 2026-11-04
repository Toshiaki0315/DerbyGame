//! Prize money, racing records and race classes (賞金・戦績).

use super::*;

// ---- prize money for the owner (賞金) ----

fn runner_spec(mine: &[&str], all: &[&str]) -> RaceSpec {
    RaceSpec {
        race: None,
        distance: 2000,
        surface: crate::calendar::Surface::Turf,
        track: TrackCondition::Good,
        entrants: all
            .iter()
            .map(|n| {
                let mut e = crate::rivals::Entrant {
                    jump: 0,
                    name: n.to_string(),
                    speed: 70,
                    stamina: 70,
                    condition: Condition::Normal,
                    sex: Sex::Male,
                    age: 4,
                    dirt: 0,
                    power: 60,
                    peak_age: 4,
                    apt_min: 1000,
                    apt_max: 3600,
                    style: None,
                    temper: 50,
                    coat_color: "鹿毛".into(),
                    gear: Gear::default(),
                    fatigue: 0,
                    trainer_distance: None,
                    course_fit: None,
                    jockey: None,
                    weight: None,
                    star: false,
                    mine: false,
                };
                e.mine = mine.contains(n);
                e
            })
            .collect(),
    }
}

#[test]
fn prize_money_is_shared_as_in_real_racing() {
    assert_eq!(PRIZE_SHARES, [1.0, 0.4, 0.25, 0.15, 0.10]);
    assert_eq!(PRIZE_RATE, 0.05);
}

#[test]
fn a_horse_of_mine_earns_its_share_of_the_first_prize() {
    let spec = runner_spec(&["A", "C"], &["A", "B", "C", "D", "E", "F"]);
    let order = finish(&["C", "B", "A", "D", "E", "F"]);
    let won = Game::owner_prizes(&spec, &order, 10_000); // first prize 10,000 x 0.05 = 500
    assert_eq!(
        won,
        vec![
            PrizeWon {
                name: "C".into(),
                rank: 1,
                amount: 500
            },
            PrizeWon {
                name: "A".into(),
                rank: 3,
                amount: 125
            }, // 25%
        ]
    );
}

#[test]
fn the_five_prize_places_pay_100_40_25_15_and_10_percent() {
    let names = ["A", "B", "C", "D", "E", "F", "G"];
    let spec = runner_spec(&names, &names);
    let won = Game::owner_prizes(&spec, &finish(&names), 10_000);
    let amounts: Vec<u32> = won.iter().map(|w| w.amount).collect();
    assert_eq!(amounts, vec![500, 200, 125, 75, 50]);
    let ranks: Vec<u32> = won.iter().map(|w| w.rank).collect();
    assert_eq!(ranks, vec![1, 2, 3, 4, 5]);
}

#[test]
fn rivals_and_horses_outside_the_prize_places_earn_nothing() {
    let spec = runner_spec(&["F"], &["A", "B", "C", "D", "E", "F"]);
    let order = finish(&["A", "B", "C", "D", "E", "F"]);
    assert!(Game::owner_prizes(&spec, &order, 10_000).is_empty()); // F is 6th
    let spec = runner_spec(&[], &["A", "B", "C", "D", "E", "F"]);
    assert!(Game::owner_prizes(&spec, &order, 10_000).is_empty()); // none of mine
}

#[test]
fn a_race_with_no_prize_money_pays_nothing() {
    let spec = runner_spec(&["A"], &["A", "B"]);
    assert!(Game::owner_prizes(&spec, &finish(&["A", "B"]), 0).is_empty());
}

#[test]
fn amounts_are_rounded_to_whole_units() {
    let spec = runner_spec(&["A"], &["A", "B", "C", "D", "E"]);
    // 4,300 x 0.05 = 215; 15% of that is 32.25, rounded to 32
    let won = Game::owner_prizes(&spec, &finish(&["B", "C", "D", "A", "E"]), 4_300);
    assert_eq!(won[0].amount, 32);
}

#[test]
fn winning_a_calendar_race_pays_the_owner_the_first_prize() {
    let mut g = Game::new();
    g.stars.clear();
    g.week = 47;
    let race = calendar_race("国際友好ステークス"); // 30,000 x 0.05 = 1,500
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let seed = seed_where(&g, &key, &mine, 1);
    let rival = a_rival_in(&g, &key);
    let fees = race_fees(&g, &key);
    let out = g
        .play_combo(Some(&key), &[rival], 10, seed, BetKind::Win)
        .unwrap();
    assert_eq!(
        out.prizes,
        vec![PrizeWon {
            name: mine,
            rank: 1,
            amount: 1500
        }]
    );
    assert_eq!(out.prize_total, 1500);
    // the stake went out, any bet payout came in, and the prize on top
    assert_eq!(
        g.money as i64,
        START_MONEY as i64 - 10 + out.payout + 1500 - g.weekly_upkeep() as i64 - fees as i64
    );
    assert_eq!(out.money, g.money);
}

#[test]
fn second_place_earns_forty_percent() {
    let mut g = Game::new();
    g.stars.clear();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let seed = seed_where(&g, &key, &mine, 2);
    let rival = a_rival_in(&g, &key);
    let out = g
        .play_combo(Some(&key), &[rival], 10, seed, BetKind::Win)
        .unwrap();
    // another of the stable's horses may place as well
    let own: Vec<_> = out
        .prizes
        .iter()
        .filter(|p| p.name == mine)
        .cloned()
        .collect();
    assert_eq!(
        own,
        vec![PrizeWon {
            name: mine,
            rank: 2,
            amount: 600
        }]
    );
}

#[test]
fn a_horse_that_does_not_place_earns_nothing() {
    let mut g = Game::new();
    g.stars.clear();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let seed = seed_where(&g, &key, &mine, 9);
    let rival = a_rival_in(&g, &key);
    let out = g
        .play_combo(Some(&key), &[rival], 10, seed, BetKind::Win)
        .unwrap();
    assert!(out.prizes.is_empty());
    assert_eq!(out.prize_total, 0);
}

#[test]
fn horses_that_are_not_entered_earn_nothing_even_if_they_are_the_best() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    g.set_entries(Some(&key), &[]).unwrap(); // none of mine run
    let rival = a_rival_in(&g, &key);
    let out = g
        .play_combo(Some(&key), &[rival], 10, 3, BetKind::Win)
        .unwrap();
    assert!(out.prizes.is_empty());
}

#[test]
fn the_free_race_pays_no_prize_even_for_a_win() {
    let g = Game::new();
    let spec = g.race_spec(None).unwrap();
    // the stable's best chance in this race
    let chances = Game::chances_n(&spec, 200);
    let mine = spec
        .entrants
        .iter()
        .zip(&chances)
        .filter(|(e, _)| e.mine)
        .max_by(|a, b| a.1 .0.total_cmp(&b.1 .0))
        .unwrap()
        .0
        .name
        .clone();
    let seed = (0..2000u64)
        .find(|s| Game::run_race(&spec, *s).1[0].name == mine)
        .expect("one of mine wins sometimes");
    let mut g = Game::new();
    let rival = spec.entrants.iter().find(|e| !e.mine).unwrap().name.clone();
    let out = g
        .play_combo(None, &[rival], 10, seed, BetKind::Win)
        .unwrap();
    assert_eq!(out.results[0].name, mine);
    assert!(out.prizes.is_empty() && out.prize_total == 0);
}

#[test]
fn every_horse_of_mine_in_the_prize_places_is_paid() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let spec = g.race_spec(Some(&key)).unwrap();
    // find a race in which at least two of mine are in the first five
    let seed = (0..2000u64)
        .find(|s| {
            Game::run_race(&spec, *s)
                .1
                .iter()
                .take(5)
                .filter(|r| spec.entrants.iter().any(|e| e.mine && e.name == r.name))
                .count()
                >= 2
        })
        .expect("two of mine place sometimes");
    let results = Game::run_race(&spec, seed).1;
    let mut expect = Vec::new();
    for (i, r) in results.iter().take(5).enumerate() {
        if spec.entrants.iter().any(|e| e.mine && e.name == r.name) {
            expect.push((
                r.name.clone(),
                i as u32 + 1,
                (1500.0 * PRIZE_SHARES[i]).round() as u32,
            ));
        }
    }
    let rival = a_rival_in(&g, &key);
    let out = g
        .play_combo(Some(&key), &[rival], 10, seed, BetKind::Win)
        .unwrap();
    let got: Vec<(String, u32, u32)> = out
        .prizes
        .iter()
        .map(|p| (p.name.clone(), p.rank, p.amount))
        .collect();
    assert_eq!(got, expect);
    assert_eq!(out.prize_total, expect.iter().map(|e| e.2).sum::<u32>());
    assert!(out.prizes.len() >= 2);
}

#[test]
fn higher_grades_pay_more() {
    let owner_first = |name: &str, week: u8| {
        let mut g = Game::new();
        g.stars.clear();
        g.week = week;
        let race = calendar_race(name);
        let key = key_of(&race);
        let mine = my_runner(&g, &key);
        let seed = seed_where(&g, &key, &mine, 1);
        let rival = a_rival_in(&g, &key);
        g.play_combo(Some(&key), &[rival], 10, seed, BetKind::Win)
            .unwrap()
            .prize_total
    };
    let g1 = owner_first("国際友好ステークス", 47); // G1, 3歳以上
    let g3 = owner_first("武蔵野ハンデキャップ", 24); // G3
    assert!(g1 > g3 * 3, "{g1} vs {g3}");
    assert_eq!(g3, (4300.0 * PRIZE_RATE).round() as u32);
}

#[test]
fn the_history_records_the_prize_and_old_history_has_none() {
    let mut g = Game::new();
    g.stars.clear();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let seed = seed_where(&g, &key, &mine, 1);
    let rival = a_rival_in(&g, &key);
    g.play_combo(Some(&key), &[rival], 10, seed, BetKind::Win)
        .unwrap();
    assert_eq!(g.state().history.last().unwrap().prize, 1500);
    let mut v: serde_json::Value = old_save(&g);
    v["history"][0].as_object_mut().unwrap().remove("prize");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert_eq!(old.state().history[0].prize, 0);
}

#[test]
fn the_card_shows_the_owner_prize_for_a_win() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    assert_eq!(g.race_card(Some(&key_of(&race))).unwrap().first_prize, 1500);
    g.week = 24;
    let g3 = calendar_race("武蔵野ハンデキャップ");
    assert_eq!(g.race_card(Some(&key_of(&g3))).unwrap().first_prize, 215);
    // the free race pays nothing
    assert_eq!(g.race_card(None).unwrap().first_prize, 0);
    let v = serde_json::to_value(g.race_card(None).unwrap()).unwrap();
    assert_eq!(v["first_prize"], 0);
}

#[test]
fn the_outcome_json_carries_the_prizes() {
    let mut g = Game::new();
    g.stars.clear();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let seed = seed_where(&g, &key, &mine, 1);
    let rival = a_rival_in(&g, &key);
    let out = g
        .play_combo(Some(&key), &[rival], 10, seed, BetKind::Win)
        .unwrap();
    let v = serde_json::to_value(&out).unwrap();
    assert_eq!(v["prize_total"], 1500);
    assert_eq!(v["prizes"][0]["name"], mine);
    assert_eq!(v["prizes"][0]["rank"], 1);
    assert_eq!(v["prizes"][0]["amount"], 1500);
}

// ---- racing records (戦績) and race classes ----

/// A class race of the given grade this week (setting the week to one that has it).
fn class_race(
    g: &mut Game,
    grade: Grade,
    age: crate::calendar::AgeRule,
) -> crate::calendar::CalendarRace {
    let r = crate::calendar::shared()
        .races
        .iter()
        .find(|r| r.grade == grade && r.eligibility.age == age)
        .unwrap_or_else(|| panic!("no {grade:?} race for {age:?}"))
        .clone();
    g.week = r.week;
    r
}

#[test]
fn the_founding_horses_come_with_records() {
    let g = Game::new();
    let rec = |n: &str| g.field[g.field.iter().position(|h| h.name == n).unwrap()].record;
    assert_eq!(
        rec("ギンセイストーム"),
        RaceRecord {
            starts: 12,
            wins: 4
        }
    );
    assert_eq!(rec("サクラフブキ"), RaceRecord { starts: 0, wins: 0 });
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(
        v["field"][1]["record"],
        serde_json::json!({"starts": 12, "wins": 4})
    );
}

#[test]
fn a_calendar_race_counts_in_the_records_of_the_stables_runners() {
    let mut g = Game::new();
    g.stars.clear();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    let before = g.field[i].record;
    let seed = seed_where(&g, &key, &mine, 1);
    let rival = a_rival_in(&g, &key);
    let runners: Vec<String> = g
        .race_spec(Some(&key))
        .unwrap()
        .entrants
        .iter()
        .filter(|e| e.mine)
        .map(|e| e.name.clone())
        .collect();
    let others: Vec<(usize, RaceRecord)> = runners
        .iter()
        .filter(|n| **n != mine)
        .map(|n| {
            let j = g.field.iter().position(|h| &h.name == n).unwrap();
            (j, g.field[j].record)
        })
        .collect();
    g.play_combo(Some(&key), &[rival], 10, seed, BetKind::Win)
        .unwrap();
    assert_eq!(
        g.field[i].record,
        RaceRecord {
            starts: before.starts + 1,
            wins: before.wins + 1
        }
    );
    for (j, r) in others {
        assert_eq!(
            g.field[j].record,
            RaceRecord {
                starts: r.starts + 1,
                wins: r.wins
            }
        );
    }
}

#[test]
fn the_free_race_does_not_count_in_records() {
    let mut g = Game::new();
    let before = g.all(|h| h.record);
    let card = g.race_card(None).unwrap();
    let rival = card.entrants.iter().find(|e| !e.mine).unwrap().name.clone();
    g.play_combo(None, &[rival], 10, 1, BetKind::Win).unwrap();
    assert_eq!(g.all(|h| h.record), before);
}

#[test]
fn only_unraced_horses_run_in_a_debut_race() {
    let mut g = Game::new();
    let race = class_race(&mut g, Grade::Debut, crate::calendar::AgeRule::Three);
    // make two three-year-olds, one unraced
    g.field[0].age = 3;
    g.field[1].age = 3;
    g.field[0].record = RaceRecord { starts: 0, wins: 0 };
    g.field[1].record = RaceRecord { starts: 1, wins: 0 };
    let eligible = g.eligible_horses(&race);
    assert!(eligible.contains(&0) && !eligible.contains(&1));
}

#[test]
fn winners_cannot_drop_back_into_maiden_or_lower_classes() {
    let mut g = Game::new();
    let maiden = class_race(&mut g, Grade::Maiden, crate::calendar::AgeRule::Three);
    g.field[0].age = 3;
    g.field[0].record = RaceRecord { starts: 4, wins: 1 };
    assert!(!g.eligible_horses(&maiden).contains(&0));
    let key = key_of(&maiden);
    let name = g.field[0].name.clone();
    assert_eq!(
        g.set_entries(Some(&key), &[name]),
        Err(GameError::NotEligible)
    );
    assert!(!g.race_card(Some(&key)).unwrap().my_horses[0].eligible);
}

#[test]
fn a_maiden_may_try_a_higher_class_and_stakes_are_open_to_all() {
    let mut g = Game::new();
    let win2 = class_race(&mut g, Grade::Win2, crate::calendar::AgeRule::FourUp);
    g.field[0].age = 5;
    g.field[0].record = RaceRecord { starts: 6, wins: 0 };
    assert!(g.eligible_horses(&win2).contains(&0));
    g.field[0].record = RaceRecord {
        starts: 10,
        wins: 3,
    };
    assert!(!g.eligible_horses(&win2).contains(&0));
    let g1 = calendar_race("国際友好ステークス");
    g.field[0].record = RaceRecord { starts: 0, wins: 0 };
    assert!(g.eligible_horses(&g1).contains(&0));
}

#[test]
fn rivals_in_class_races_are_weaker_than_in_stakes() {
    let mut g = Game::new();
    let maiden = class_race(&mut g, Grade::Maiden, crate::calendar::AgeRule::Three);
    let card = g.race_card(Some(&key_of(&maiden))).unwrap();
    let mean = |c: &RaceCard| {
        let r: Vec<u32> = c
            .entrants
            .iter()
            .filter(|e| !e.mine)
            .map(|e| e.speed)
            .collect();
        r.iter().sum::<u32>() as f64 / r.len() as f64
    };
    let mut g2 = Game::new();
    g2.week = 47;
    let stakes = g2
        .race_card(Some(&key_of(&calendar_race("国際友好ステークス"))))
        .unwrap();
    assert!(
        mean(&card) + 10.0 < mean(&stakes),
        "{} vs {}",
        mean(&card),
        mean(&stakes)
    );
}

#[test]
fn records_go_to_stud_and_survive_a_save_and_old_saves_get_the_founders_records() {
    let mut g = Game::new();
    g.studs.clear(); // without the starting pair
    g.field[2].record = RaceRecord { starts: 9, wins: 5 };
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.all(|h| h.record), g.all(|h| h.record));
    let name = g.field[2].name.clone();
    g.retire(&name).unwrap();
    assert_eq!(g.studs[0].record, RaceRecord { starts: 9, wins: 5 });
    let mut v = old_save(&Game::new());
    v.as_object_mut().unwrap().remove("records");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert_eq!(old.all(|h| h.record), Game::new().all(|h| h.record));
}

#[test]
fn a_foal_starts_with_no_record() {
    let mut g = Game::new();
    let (sire, dam) = stallion_and_mare(&g);
    let slot = g
        .field
        .iter()
        .position(|h| h.name != sire && h.name != dam)
        .unwrap();
    let replace = g.field[slot].name.clone();
    g.breed_now(&sire, &dam, &replace, "ハナミズキ").unwrap();
    assert_eq!(g.field[slot].record, RaceRecord::default());
}
