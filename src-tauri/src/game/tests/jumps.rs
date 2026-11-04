//! Jump racing and horses pulling up (障害レースと競走中止).

use super::*;

fn jump_race(grade: Grade) -> CalendarRace {
    calendar::shared()
        .races
        .iter()
        .find(|r| r.surface == Surface::Jump && r.grade == grade)
        .unwrap()
        .clone()
}

/// A game with a horse ready to run over fences.
fn with_jumper() -> (Game, usize, String) {
    let mut g = Game::new();
    let i = g.field.iter().position(|h| h.age >= 4).unwrap();
    g.field[i].jumper = true;
    g.field[i].jump = 50;
    g.field[i].jump_license = true;
    g.field[i].fatigue = 0;
    let name = g.field[i].name.clone();
    (g, i, name)
}

// ---- pulling up ----

#[test]
fn a_horse_that_pulls_up_counts_as_a_start_and_earns_nothing() {
    let mut g = Game::new();
    let r = calendar::shared()
        .races
        .iter()
        .find(|r| {
            r.grade == Grade::Open
                && r.surface != Surface::Jump
                && r.eligibility.age == AgeRule::ThreeUp
        })
        .unwrap()
        .clone();
    g.week = r.week;
    let i = (0..g.field.len())
        .find(|&i| {
            r.eligibility
                .allows(g.field[i].age, g.field[i].sex == Sex::Female)
        })
        .unwrap();
    let name = g.field[i].name.clone();
    g.set_entries(Some(&RaceKey::of(&r)), std::slice::from_ref(&name))
        .unwrap();
    let spec = g.race_spec(Some(&RaceKey::of(&r))).unwrap();
    let results: Vec<RaceResult> = spec
        .entrants
        .iter()
        .filter(|e| !e.mine)
        .enumerate()
        .map(|(n, e)| RaceResult {
            rank: n as u32 + 1,
            name: e.name.clone(),
            finish_time: 100.0 + n as f64,
            margin: String::new(),
        })
        .collect();
    let (starts, wins) = (g.field[i].record.starts, g.field[i].record.wins);
    assert!(Game::owner_prizes(&spec, &results, 1000).is_empty());
    g.add_to_records(&spec, &results);
    assert_eq!(g.field[i].record.starts, starts + 1);
    assert_eq!(g.field[i].record.wins, wins);
    assert!(
        g.news
            .iter()
            .any(|n| n.contains(&name) && n.contains("競走中止")),
        "{:?}",
        g.news
    );
}

#[test]
fn the_outcome_names_those_that_did_not_finish() {
    let mut g = Game::new();
    let out = g.watch_race(None, 3).unwrap();
    assert_eq!(out.results.len() + out.stopped.len(), out.runners.len());
    let spec = g.race_spec(None).unwrap();
    assert_eq!(Game::did_not_finish(&spec, &out.results), out.stopped);
}

// ---- jumping ability in the blood ----

#[test]
fn every_record_of_the_database_has_a_jumping_ability() {
    let g = Game::new();
    for h in g.base_db().iter() {
        assert!((0.0..=100.0).contains(&h.jump), "{}", h.jump);
    }
    let v = serde_json::to_value(g.state()).unwrap();
    assert!(v["field"][0]["blood"]["jump"].is_number());
}

#[test]
fn foals_take_after_their_parents_at_fences() {
    let g = Game::new();
    let mut parents: Vec<BloodHorse> = g.base_db().iter().take(2).cloned().collect();
    parents[0].gender = Sex::Male;
    parents[1].gender = Sex::Female;
    let foals = |jump: f64| {
        let (mut s, mut d) = (parents[0].clone(), parents[1].clone());
        s.jump = jump;
        d.jump = jump;
        let mut rng = rivals::Rng::new(9);
        let total: f64 = (0..300)
            .map(|n| {
                bloodline::breed(
                    &s,
                    &d,
                    11,
                    90_000 + n,
                    "ハナミズキ".to_string(),
                    &std::collections::HashMap::new(),
                    &mut rng,
                )
                .jump
            })
            .sum();
        total / 300.0
    };
    assert!(foals(90.0) > foals(10.0) + 30.0);
}

#[test]
fn the_jumping_ability_was_added_without_changing_the_rest_of_the_database() {
    // the older records stay as they were: a save knows its horses by them
    let db = bloodline::base_database(12345);
    assert_eq!(db.len(), bloodline::base_database(12345).len());
    let old: BloodHorse = {
        let mut v = serde_json::to_value(&db[0]).unwrap();
        v.as_object_mut().unwrap().remove("jump");
        serde_json::from_value(v).unwrap()
    };
    assert_eq!(old.jump, bloodline::NEUTRAL_JUMP);
}

// ---- going over to jumps ----

#[test]
fn a_horse_of_three_or_more_can_go_over_to_jumps() {
    let mut g = Game::new();
    let i = g.field.iter().position(|h| h.age >= 3).unwrap();
    let name = g.field[i].name.clone();
    g.field[i].age = 2;
    assert_eq!(g.go_over_to_jumps(&name), Err(GameError::TooYoung));
    g.field[i].age = 3;
    g.go_over_to_jumps(&name).unwrap();
    assert!(g.field[i].jumper);
    assert!(g.field[i].jump > 0 && g.field[i].jump < JUMP_EXAM_LEVEL);
    assert_eq!(g.go_over_to_jumps(&name), Err(GameError::AlreadyJumper));
}

#[test]
fn jump_schooling_is_for_jumpers_and_builds_jumping_up_to_what_is_in_the_horse() {
    let mut g = Game::new();
    let i = 0;
    let name = g.field[i].name.clone();
    g.money = 1_000_000;
    assert_eq!(g.workout(&name, Workout::Jump), Err(GameError::NotJumper));
    g.field[i].age = 4;
    g.go_over_to_jumps(&name).unwrap();
    let before = g.field[i].jump;
    g.workout(&name, Workout::Jump).unwrap();
    assert!(g.field[i].jump > before);
    let ceiling = g.jump_ceiling(i);
    g.field[i].jump = ceiling;
    g.field[i].fatigue = 0;
    assert_eq!(g.workout(&name, Workout::Jump), Err(GameError::MaxedOut));
    assert!((JUMP_EXAM_LEVEL..=100).contains(&ceiling));
}

#[test]
fn the_exam_needs_enough_jumping_and_costs_money() {
    let (mut g, i, name) = with_jumper();
    g.field[i].jump_license = false;
    g.field[i].jump = JUMP_EXAM_LEVEL - 1;
    assert_eq!(g.take_jump_exam(&name), Err(GameError::JumpTooLow));
    g.field[i].jump = JUMP_EXAM_LEVEL;
    let money = g.money;
    g.take_jump_exam(&name).unwrap();
    assert!(g.field[i].jump_license);
    assert_eq!(g.money, money - JUMP_EXAM_COST);
    assert_eq!(g.take_jump_exam(&name), Err(GameError::AlreadyLicensed));
    let flat = (0..g.field.len()).find(|&n| !g.field[n].jumper).unwrap();
    let flat = g.field[flat].name.clone();
    assert_eq!(g.take_jump_exam(&flat), Err(GameError::NotJumper));
}

// ---- jump races ----

#[test]
fn the_calendar_has_jump_races_from_maidens_to_g1s_at_courses_with_fences() {
    let races: Vec<&CalendarRace> = calendar::shared()
        .races
        .iter()
        .filter(|r| r.surface == Surface::Jump)
        .collect();
    let count = |g: Grade| races.iter().filter(|r| r.grade == g).count();
    assert_eq!(count(Grade::G1), 2);
    assert!(
        count(Grade::G2) >= 3 && count(Grade::G3) >= 3,
        "{} {}",
        count(Grade::G2),
        count(Grade::G3)
    );
    assert!(count(Grade::Maiden) >= 20 && count(Grade::Open) >= 20);
    for r in &races {
        assert!(
            r.distance >= 2800 && r.distance <= 4250,
            "{} {}",
            r.name,
            r.distance
        );
        assert!(!r.eligibility.fillies_only);
        assert!(!matches!(r.eligibility.age, AgeRule::Two), "{}", r.name);
    }
    let g1 = jump_race(Grade::G1);
    assert_eq!(g1.venue, Venue::Nakayama);
    assert!(g1.prize < calendar_race("疾風賞").prize);
}

#[test]
fn only_a_licensed_jumper_may_run_over_fences() {
    let r = jump_race(Grade::Maiden);
    let (mut g, i, _) = with_jumper();
    g.week = r.week;
    let other = (0..g.field.len())
        .find(|&n| !g.field[n].jumper && g.field[n].age >= 3)
        .unwrap();
    assert!(g.eligible_horses(&r).contains(&i));
    assert!(!g.eligible_horses(&r).contains(&other));
    g.field[i].jump_license = false;
    assert!(!g.eligible_horses(&r).contains(&i));
}

#[test]
fn the_classes_over_fences_go_by_wins_over_fences() {
    let maiden = jump_race(Grade::Maiden);
    let open = jump_race(Grade::Open);
    let stakes = jump_race(Grade::G3);
    let (mut g, i, _) = with_jumper();
    // wins on the flat count for nothing here
    g.field[i].record = RaceRecord {
        starts: 20,
        wins: 5,
    };
    g.week = maiden.week;
    assert!(g.eligible_horses(&maiden).contains(&i));
    g.week = open.week;
    assert!(g.eligible_horses(&open).contains(&i));
    g.week = stakes.week;
    assert!(!g.eligible_horses(&stakes).contains(&i));
    g.field[i].jump_wins = 1;
    assert!(g.eligible_horses(&stakes).contains(&i));
    g.week = maiden.week;
    assert!(!g.eligible_horses(&maiden).contains(&i));
}

#[test]
fn a_jump_race_takes_eight_to_fourteen() {
    for seed in 1..=10 {
        let mut g = Game::with_seed(seed);
        g.entry_rules = true;
        let r = jump_race(Grade::Open);
        g.week = r.week;
        let card = g.race_card(Some(&RaceKey::of(&r))).unwrap();
        assert!(
            (8..=14).contains(&card.entrants.len()),
            "{}",
            card.entrants.len()
        );
        assert_eq!(card.gate, 14);
    }
}

#[test]
fn a_jump_race_is_run_over_fences_with_top_weight_and_no_distance_penalty() {
    let r = jump_race(Grade::G1);
    let (mut g, i, name) = with_jumper();
    g.field[i].jump_wins = 1;
    g.week = r.week;
    g.set_entries(Some(&RaceKey::of(&r)), std::slice::from_ref(&name))
        .unwrap();
    let key = RaceKey::of(&r);
    let spec = g.race_spec(Some(&key)).unwrap();
    let me = spec.entrants.iter().find(|e| e.mine).unwrap();
    assert!(me.weight.unwrap() >= 60, "{:?}", me.weight);
    let race = Game::start_race(&spec, 1);
    assert!(race.runners.iter().all(|r| r.jump.is_some()));
    let card = g.race_card(Some(&key)).unwrap();
    assert_eq!(card.surface, Surface::Jump);
    assert!(card.fences >= 8);
    assert!(card.entrants.iter().any(|e| e.mine && e.jump > 0));
    assert_eq!(
        card.entrants.iter().find(|e| e.mine).unwrap().distance_fit,
        form::DistanceFit::Fit
    );
    assert!(g.watch_race(Some(&key), 1).is_ok());
}

#[test]
fn flat_races_have_no_fences_and_rivals_have_none_in_them() {
    let g = Game::new();
    let spec = g.race_spec(None).unwrap();
    assert!(Game::start_race(&spec, 1)
        .runners
        .iter()
        .all(|r| r.jump.is_none()));
    assert_eq!(g.race_card(None).unwrap().fences, 0);
}

#[test]
fn a_win_over_fences_goes_down_with_the_jumps_and_does_not_lift_it_in_class_on_the_flat() {
    let r = jump_race(Grade::Maiden);
    let (mut g, i, name) = with_jumper();
    g.week = r.week;
    g.set_entries(Some(&RaceKey::of(&r)), std::slice::from_ref(&name))
        .unwrap();
    let spec = g.race_spec(Some(&RaceKey::of(&r))).unwrap();
    let (starts, wins) = (g.field[i].record.starts, g.field[i].record.wins);
    g.add_to_records(
        &spec,
        &[RaceResult {
            rank: 1,
            name: name.clone(),
            finish_time: 200.0,
            margin: String::new(),
        }],
    );
    assert_eq!(g.field[i].jump_wins, 1);
    assert_eq!(g.field[i].year_jump_wins, 1);
    assert_eq!(
        (g.field[i].record.starts, g.field[i].record.wins),
        (starts, wins)
    );
}

#[test]
fn a_trainer_sends_a_licensed_jumper_over_fences_when_it_can_and_never_goes_over_on_its_own() {
    let r = jump_race(Grade::Maiden);
    let (mut g, i, _) = with_jumper();
    g.week = r.week;
    let chosen = g.auto_race(i, &[], None).unwrap();
    assert_eq!(chosen.surface, Surface::Jump);
    let mut left = Game::with_seed(2);
    left.set_all_auto(true);
    for _ in 0..104 {
        left.advance_week();
    }
    assert!(left.field.iter().all(|h| !h.jumper));
}

#[test]
fn a_trainer_schools_a_jumper_until_it_has_its_licence() {
    let (mut g, i, _) = with_jumper();
    g.field[i].jump_license = false;
    g.field[i].jump = 5;
    g.field[i].temper = 80;
    assert_eq!(g.auto_plan(i), Some(AutoPlan::Work(Workout::Jump)));
    g.field[i].jump_license = true;
    g.field[i].jump = 90;
    assert_ne!(g.auto_plan(i), Some(AutoPlan::Work(Workout::Jump)));
}

#[test]
fn a_jockey_has_a_hand_for_jumps_too() {
    let (j, k) = (&jockey::JOCKEYS[0], &jockey::JOCKEYS[9]);
    assert!((30..=100).contains(&j.jump_skill()));
    assert!((30..=100).contains(&k.jump_skill()));
    let v = serde_json::to_value(j.info()).unwrap();
    assert_eq!(v["jump"], j.jump_skill());
    assert!(jockey::JOCKEYS
        .iter()
        .any(|a| a.jump_skill() != j.jump_skill()));
}

#[test]
fn a_good_jump_jockey_helps_at_fences() {
    let r = jump_race(Grade::Maiden);
    let (mut g, i, name) = with_jumper();
    g.week = r.week;
    let (best, worst) = {
        let mut js: Vec<_> = g.jockeys_now();
        js.sort_by_key(|j| j.jump_skill());
        (js[js.len() - 1].id, js[0].id)
    };
    g.set_entries(Some(&RaceKey::of(&r)), std::slice::from_ref(&name))
        .unwrap();
    let jump_with = |g: &mut Game, id: u8| {
        g.set_jockey(&name, Some(id)).unwrap();
        let spec = g.race_spec(Some(&RaceKey::of(&r))).unwrap();
        let at = spec.entrants.iter().position(|e| e.mine).unwrap();
        Game::start_race(&spec, 1).runners[at].jump.unwrap()
    };
    assert!(jump_with(&mut g, best) > jump_with(&mut g, worst));
    assert_eq!(g.field[i].jockey, Some(worst));
}

#[test]
fn the_years_best_jumper_of_the_stable_gets_an_award() {
    let (mut g, i, name) = with_jumper();
    g.field[i].year_jump_wins = 2;
    g.choose_horse_of_the_year();
    assert!(g
        .awards
        .iter()
        .any(|a| a.title == TITLE_BEST_JUMPER && a.horse == name && a.mine));
    assert_eq!(g.field[i].year_jump_wins, 0);
}

#[test]
fn nobody_gets_it_for_doing_nothing() {
    let mut g = Game::new();
    g.choose_horse_of_the_year();
    assert!(g.awards.iter().all(|a| a.title != TITLE_BEST_JUMPER));
}

#[test]
fn being_a_jumper_survives_a_save_and_old_saves_have_none() {
    let (g, i, _) = with_jumper();
    let back = Game::from_json(&g.to_json()).unwrap();
    assert!(back.field[i].jumper && back.field[i].jump_license);
    assert_eq!(back.field[i].jump, 50);
    let old = Game::from_json(&old_save(&Game::new()).to_string()).unwrap();
    assert!(old.field.iter().all(|h| !h.jumper && h.jump == 0));
}

#[test]
fn the_screen_is_told_about_jumpers_and_the_exam() {
    let (g, i, _) = with_jumper();
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["field"][i]["jumper"], true);
    assert_eq!(v["field"][i]["jump"], 50);
    assert_eq!(v["field"][i]["jump_license"], true);
    assert_eq!(v["jump_exam_level"], JUMP_EXAM_LEVEL);
    assert_eq!(v["jump_exam_cost"], JUMP_EXAM_COST);
    assert_eq!(serde_json::to_value(Surface::Jump).unwrap(), "jump");
    assert_eq!(serde_json::to_value(Workout::Jump).unwrap(), "jump");
}

#[test]
fn the_commentary_calls_those_who_pull_up_and_blunder() {
    let r = jump_race(Grade::Open);
    let mut g = Game::new();
    g.week = r.week;
    let spec = g.race_spec(Some(&RaceKey::of(&r))).unwrap();
    let seed = (0..3000u64)
        .find(|&s| !Game::did_not_finish(&spec, &Game::finish_order(&spec, s)).is_empty())
        .expect("nobody ever fell");
    let (frames, results) = Game::run_race(&spec, seed);
    let stopped = Game::did_not_finish(&spec, &results);
    let mut lines = vec![];
    Game::call_mishaps(&mut lines, &spec, seed, &frames, &stopped);
    assert!(
        lines
            .iter()
            .any(|l| l.text == format!("{}が競走中止!", stopped[0])),
        "{lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.text.contains("飛越でミス")),
        "{lines:?}"
    );
    assert!(lines.windows(2).all(|w| w[0].frame <= w[1].frame));
}

#[test]
fn a_season_over_fences_is_run_by_the_trainers_from_schooling_to_winning() {
    let mut g = Game::with_seed(11);
    g.entry_rules = true;
    g.set_all_auto(true);
    g.money = 50_000;
    let name = g.field[1].name.clone();
    g.field[1].age = 4;
    g.go_over_to_jumps(&name).unwrap();
    for _ in 0..104 {
        g.advance_week();
    }
    if let Some(h) = g.field.iter().find(|h| h.name == name) {
        assert!(h.jump_license, "{} jump {}", h.name, h.jump);
    }
}
