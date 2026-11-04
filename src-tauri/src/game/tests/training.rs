//! Workouts, equipment, fatigue, advice and leaving training to the trainer (調教).

use super::*;

// ---- talking with the trainer (調教師との相談) ----

fn advice_on(g: &Game, name: &str) -> Advice {
    g.advice().into_iter().find(|a| a.horse == name).unwrap()
}

#[test]
fn every_horse_has_a_word_from_its_trainer() {
    let g = Game::with_seed(1);
    let advice = g.advice();
    assert_eq!(advice.len(), g.field.len());
    assert!(advice.iter().all(|a| !a.comment.is_empty()));
}

#[test]
fn the_trainer_speaks_up_for_a_tired_horse_and_proposes_what_the_trainer_would_do() {
    let mut g = Game::with_seed(1);
    g.money = 100_000;
    let name = g.field[0].name.clone();
    g.field[0].fatigue = 100;
    let a = advice_on(&g, &name);
    assert!(a.comment.contains("疲れ"), "{}", a.comment);
    assert_eq!(a.proposal.as_deref(), Some("放牧に出す"));
    g.accept_advice(&name).unwrap();
    assert!(g.field[0].away.is_some());
}

#[test]
fn the_trainer_says_how_a_horse_is_coming_along_for_its_age() {
    let mut g = Game::with_seed(1);
    breed_default(&mut g).unwrap();
    let name = "ニューホープ";
    let i = g.field.iter().position(|h| h.name == name).unwrap();
    g.field[i].age = 2;
    let id = g.field[i].blood_id;
    let peak =
        |g: &mut Game, age: u8| g.bred.iter_mut().find(|h| h.id == id).unwrap().peak_age = age;
    peak(&mut g, 3);
    assert!(advice_on(&g, name).comment.contains("早くから"));
    peak(&mut g, 6);
    assert!(advice_on(&g, name).comment.contains("晩成"));
}

#[test]
fn a_foal_at_the_farm_needs_nothing_decided() {
    let (g, i) = with_a_newborn();
    let a = advice_on(&g, &g.field[i].name);
    assert!(a.comment.contains("牧場"));
    assert_eq!(a.proposal, None);
}

#[test]
fn a_yearling_is_only_trained() {
    let (mut g, i) = with_a_newborn();
    g.field[i].away = None;
    g.field[i].age = 1;
    let a = advice_on(&g, &g.field[i].name);
    assert!(a.comment.contains("デビュー"), "{}", a.comment);
    assert!(a.proposal.unwrap().contains("調教"));
}

#[test]
fn the_trainer_proposes_a_race_this_week_and_accepting_enters_the_horse() {
    let mut g = Game::with_seed(1);
    g.money = 100_000;
    // the first week with a race a trainer would pick for one of the horses
    let (week, i) = (1..=52u8)
        .find_map(|w| {
            g.week = w;
            (0..g.field.len())
                .find(|&i| g.auto_race(i, &[], None).is_some())
                .map(|i| (w, i))
        })
        .unwrap();
    g.week = week;
    g.field[i].fatigue = 0;
    g.field[i].since_race = NEVER_RACED;
    let name = g.field[i].name.clone();
    let race = g.auto_race(i, &[], None).unwrap();
    let a = advice_on(&g, &name);
    assert_eq!(a.proposal, Some(format!("{}に出走", race.name)));
    assert!(a.comment.contains(&race.name));
    g.accept_advice(&name).unwrap();
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    assert!(
        card.my_horses
            .iter()
            .find(|h| h.name == name)
            .unwrap()
            .entered
    );
}

#[test]
fn a_horse_already_entered_is_not_proposed_for_the_race_again() {
    let mut g = Game::with_seed(1);
    g.money = 100_000;
    let (week, i) = (1..=52u8)
        .find_map(|w| {
            g.week = w;
            (0..g.field.len())
                .find(|&i| g.auto_race(i, &[], None).is_some())
                .map(|i| (w, i))
        })
        .unwrap();
    g.week = week;
    g.field[i].fatigue = 0;
    g.field[i].since_race = NEVER_RACED;
    let name = g.field[i].name.clone();
    let race = g.auto_race(i, &[], None).unwrap();
    g.accept_advice(&name).unwrap();
    let a = advice_on(&g, &name);
    assert_ne!(a.proposal, Some(format!("{}に出走", race.name)));
    assert!(
        a.comment.contains(&race.name) && a.comment.contains("登録"),
        "{}",
        a.comment
    );
}

#[test]
fn accepting_advice_for_an_unknown_horse_is_refused() {
    let mut g = Game::with_seed(1);
    assert_eq!(g.accept_advice("Nobody"), Err(GameError::UnknownHorse));
}

#[test]
fn accepting_when_nothing_is_proposed_changes_nothing() {
    let (mut g, i) = with_a_newborn();
    let before = g.state();
    let name = g.field[i].name.clone();
    g.accept_advice(&name).unwrap();
    assert_eq!(g.state(), before);
}

// ---- training menus (調教) ----

#[test]
fn each_workout_changes_its_abilities() {
    use Workout::*;
    for (w, want) in [
        (Hill, (60, 61, 63, 50)),
        (Wood, (63, 61, 60, 50)),
        (Pair, (62, 60, 62, 48)),
        (Pool, (60, 63, 60, 51)),
        (Walk, (60, 60, 60, 53)),
        (Dirt, (60, 60, 61, 50)),
    ] {
        let mut g = mid_horse();
        let name = g.field[0].name.clone();
        g.workout(&name, w).unwrap();
        assert_eq!(abilities(&g, 0), want, "{w:?}");
    }
}

#[test]
fn every_workout_costs_one_session() {
    let mut g = mid_horse();
    g.field[0].jumper = true;
    let name = g.field[0].name.clone();
    for w in Workout::ALL {
        let cost = g.train_cost(&name).unwrap();
        let money = g.money;
        g.workout(&name, w).unwrap();
        assert_eq!(g.money, money - cost, "{w:?}");
    }
    assert_eq!(g.field[0].trainings, Workout::ALL.len() as u32);
}

#[test]
fn walking_in_hand_calms_a_horse_up_to_the_limit() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.field[0].temper = MAX_STAT - 1;
    g.workout(&name, Workout::Walk).unwrap();
    assert_eq!(g.field[0].temper, MAX_STAT);
    assert_eq!(g.workout(&name, Workout::Walk), Err(GameError::MaxedOut));
}

#[test]
fn working_alongside_another_horse_never_takes_temperament_below_one() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.field[0].temper = 2;
    g.workout(&name, Workout::Pair).unwrap();
    assert_eq!(g.field[0].temper, MIN_TEMPER);
}

#[test]
fn a_side_gain_stops_at_the_limit_and_a_mutant_stat_is_not_cut() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.field[0].stamina = 110; // a mutation took it past 100
    g.workout(&name, Workout::Wood).unwrap();
    assert_eq!(g.field[0].stamina, 110);
    g.field[0].stamina = MAX_STAT;
    g.workout(&name, Workout::Wood).unwrap();
    assert_eq!(g.field[0].stamina, MAX_STAT);
}

#[test]
fn three_sessions_on_dirt_raise_dirt_aptitude_a_step() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.field[0].dirt = 0;
    g.workout(&name, Workout::Dirt).unwrap();
    g.workout(&name, Workout::Dirt).unwrap();
    assert_eq!((g.field[0].dirt, g.field[0].dirt_progress), (0, 2));
    assert_eq!(g.state().field[0].dirt_progress, 2);
    g.workout(&name, Workout::Dirt).unwrap();
    assert_eq!((g.field[0].dirt, g.field[0].dirt_progress), (1, 0));
}

#[test]
fn a_dirt_specialist_has_nothing_more_to_learn_on_dirt() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.field[0].dirt = MAX_DIRT;
    assert_eq!(g.workout(&name, Workout::Dirt), Err(GameError::MaxedOut));
}

#[test]
fn trained_temperament_is_what_the_horse_races_and_breeds_with() {
    let mut g = mid_horse();
    g.studs.clear(); // without the starting pair
    let name = g.field[0].name.clone();
    g.workout(&name, Workout::Walk).unwrap();
    assert_eq!(g.stable_entrant(0).temper, 53);
    assert_eq!(g.stable_blood(0).unwrap().stats.temper, 53.0);
    assert_eq!(g.state().field[0].temper, 53);
    // and it goes to stud with it
    g.retire(&name).unwrap();
    assert_eq!(g.studs[0].temper, 53);
    assert_eq!(g.parent(&name).unwrap().0.stats.temper, 53.0);
}

#[test]
fn a_foal_starts_with_its_own_temperament_and_no_dirt_progress() {
    let mut g = mid_horse();
    let (sire, dam) = stallion_and_mare(&g);
    let slot = g
        .field
        .iter()
        .position(|h| h.name != sire && h.name != dam)
        .unwrap();
    g.field[slot].dirt_progress = 2;
    let replace = g.field[slot].name.clone();
    g.breed_now(&sire, &dam, &replace, "ハナミズキ").unwrap();
    let foal = g.blood_record(g.field[slot].blood_id).unwrap();
    assert_eq!(g.field[slot].temper, temper_of(foal.stats.temper));
    assert_eq!(g.field[slot].dirt_progress, 0);
}

#[test]
fn temperament_and_dirt_progress_survive_a_save_and_old_saves_take_them_from_the_records() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.workout(&name, Workout::Walk).unwrap();
    g.workout(&name, Workout::Dirt).unwrap();
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(
        (back.all(|h| h.temper), back.all(|h| h.dirt_progress)),
        (g.all(|h| h.temper), g.all(|h| h.dirt_progress))
    );
    let mut v = old_save(&Game::new());
    v.as_object_mut().unwrap().remove("tempers");
    v.as_object_mut().unwrap().remove("dirt_progress");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert_eq!(old.all(|h| h.temper), Game::new().all(|h| h.temper));
    assert_eq!(old.all(|h| h.dirt_progress), vec![0; old.field.len()]);
}

#[test]
fn the_state_lists_the_workouts_for_the_ui() {
    let g = Game::new();
    let v = serde_json::to_value(g.state()).unwrap();
    let names: Vec<&str> = v["workouts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "ウッドコース追い",
            "坂路",
            "プール",
            "併せ馬",
            "曳き運動",
            "ダートコース",
            "障害練習",
            "ポリトラック",
            "ゲート練習",
        ]
    );
    let hill = &v["workouts"][1];
    assert_eq!(hill["workout"], "hill");
    assert_eq!(
        hill["gains"][0],
        serde_json::json!({"ability": "power", "amount": 3})
    );
    assert!(!hill["description"].as_str().unwrap().is_empty());
    assert_eq!(v["dirt_sessions"], 3);
    assert!(v["field"][0]["temper"].as_u64().is_some());
}

// ---- racing equipment (馬具) ----

#[test]
fn a_horse_races_in_the_gear_it_is_given_until_it_is_changed() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    assert_eq!(g.stable_entrant(0).gear, Gear::default());
    let both = Gear {
        menko: true,
        shadow_roll: true,
        ..Gear::default()
    };
    g.set_gear(&name, both).unwrap();
    assert_eq!(g.stable_entrant(0).gear, both);
    g.advance_week();
    assert_eq!(g.field[0].gear, both);
    assert_eq!(g.set_gear("いない馬", both), Err(GameError::UnknownHorse));
}

#[test]
fn the_card_shows_everyones_gear_and_the_odds_take_it_into_account() {
    let mut g = Game::new();
    let card = g.race_card(None).unwrap();
    // the likeliest of mine, whose price shows the difference most
    let mine = card
        .entrants
        .iter()
        .filter(|e| e.mine)
        .min_by(|a, b| a.odds.total_cmp(&b.odds))
        .unwrap()
        .name
        .clone();
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    g.field[i].temper = 5; // hot enough to pull often
    let before = g.race_card(None).unwrap();
    g.set_gear(
        &mine,
        Gear {
            menko: true,
            shadow_roll: false,
            ..Gear::default()
        },
    )
    .unwrap();
    let after = g.race_card(None).unwrap();
    let entry = |c: &RaceCard| c.entrants.iter().find(|e| e.name == mine).unwrap().clone();
    assert!(entry(&after).gear.menko);
    assert!(
        after
            .my_horses
            .iter()
            .find(|h| h.name == mine)
            .unwrap()
            .gear
            .menko
    );
    assert_ne!(entry(&before).odds, entry(&after).odds);
    let v = serde_json::to_value(&after).unwrap();
    assert!(v["entrants"][0]["gear"]["shadow_roll"].is_boolean());
    assert!(v["my_horses"][0]["gear"]["menko"].is_boolean());
}

#[test]
fn gear_is_kept_in_a_save_and_old_saves_race_without_any() {
    let mut g = Game::new();
    let name = g.field[1].name.clone();
    g.set_gear(
        &name,
        Gear {
            menko: false,
            shadow_roll: true,
            ..Gear::default()
        },
    )
    .unwrap();
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.all(|h| h.gear), g.all(|h| h.gear));
    assert!(back.state().field[1].gear.shadow_roll);
    let mut v = old_save(&Game::new());
    v.as_object_mut().unwrap().remove("gear");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert!(old.all(|h| h.gear).iter().all(|g| *g == Gear::default()));
}

#[test]
fn a_foal_starts_without_gear() {
    let mut g = Game::new();
    let (sire, dam) = stallion_and_mare(&g);
    let slot = g
        .field
        .iter()
        .position(|h| h.name != sire && h.name != dam)
        .unwrap();
    let replace = g.field[slot].name.clone();
    g.set_gear(
        &replace,
        Gear {
            menko: true,
            shadow_roll: true,
            ..Gear::default()
        },
    )
    .unwrap();
    g.breed_now(&sire, &dam, &replace, "ハナミズキ").unwrap();
    assert_eq!(g.field[slot].gear, Gear::default());
}

// ---- fatigue (疲労度) and rest (休養) ----

#[test]
fn workouts_tire_a_horse_by_how_hard_they_are() {
    let mut g = rich();
    g.field[0].jumper = true;
    let name = g.field[0].name.clone();
    for w in Workout::ALL {
        g.field[0].fatigue = 0;
        g.workout(&name, w).unwrap();
        assert_eq!(g.field[0].fatigue, w.fatigue(), "{w:?}");
    }
    assert!(Workout::Pair.fatigue() > Workout::Wood.fatigue());
    assert!(Workout::Pool.fatigue() < Workout::Hill.fatigue());
    assert_eq!(Workout::Walk.fatigue(), 0);
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["workouts"][1]["fatigue"], Workout::Hill.fatigue());
}

#[test]
fn a_tired_horse_cannot_be_worked() {
    let mut g = rich();
    let name = g.field[0].name.clone();
    g.field[0].fatigue = TRAIN_FATIGUE_LIMIT;
    assert_eq!(g.workout(&name, Workout::Wood), Err(GameError::TooTired));
    g.field[0].fatigue = TRAIN_FATIGUE_LIMIT - 1;
    g.workout(&name, Workout::Pair).unwrap();
    assert!(g.field[0].fatigue <= MAX_FATIGUE);
}

#[test]
fn running_in_a_calendar_race_tires_a_horse_more_the_longer_the_race() {
    assert!(race_fatigue(2400) > race_fatigue(1200));
    assert_eq!(race_fatigue(2000), 45);
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine: Vec<String> = g
        .race_spec(Some(&key))
        .unwrap()
        .entrants
        .iter()
        .filter(|e| e.mine)
        .map(|e| e.name.clone())
        .collect();
    let rival = a_rival_in(&g, &key);
    g.play_combo(Some(&key), &[rival], 10, 1, BetKind::Win)
        .unwrap();
    // the race moved the week on, so a week's recovery came off already
    for n in &mine {
        let i = g.field.iter().position(|h| &h.name == n).unwrap();
        assert_eq!(
            g.field[i].fatigue,
            race_fatigue(race.distance).saturating_sub(WEEKLY_RECOVERY),
            "{n}"
        );
    }
    let others = g
        .field
        .iter()
        .enumerate()
        .filter(|(_, h)| !mine.contains(&h.name));
    for (i, _) in others {
        assert_eq!(g.field[i].fatigue, 0);
    }
}

#[test]
fn the_free_race_tires_a_horse_half_as_much() {
    let mut g = Game::new();
    let card = g.race_card(None).unwrap();
    let mine = card.entrants.iter().find(|e| e.mine).unwrap().name.clone();
    let rival = card.entrants.iter().find(|e| !e.mine).unwrap().name.clone();
    g.play_combo(None, &[rival], 10, 1, BetKind::Win).unwrap();
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    assert_eq!(g.field[i].fatigue, race_fatigue(FREE_DISTANCE) / 2);
}

#[test]
fn a_week_passing_takes_some_fatigue_away() {
    let mut g = Game::new();
    g.field[0].fatigue = 50;
    g.field[1].fatigue = 5;
    g.advance_week();
    assert_eq!(
        (g.field[0].fatigue, g.field[1].fatigue),
        (50 - WEEKLY_RECOVERY, 0)
    );
}

#[test]
fn a_horse_that_raced_last_week_is_still_tired_this_week() {
    // 連闘: two long races in a row leave a horse too tired for a third
    let mut g = Game::new();
    g.field[0].fatigue = 0;
    g.field[0].fatigue += race_fatigue(2400);
    g.advance_week();
    g.field[0].fatigue += race_fatigue(2400);
    assert!(
        g.field[0].fatigue >= RACE_FATIGUE_LIMIT,
        "{}",
        g.field[0].fatigue
    );
}

#[test]
fn a_tired_horse_runs_below_its_ability() {
    use crate::calendar::Surface;
    assert_eq!(fatigue_form(0), 1.0);
    assert_eq!(fatigue_form(FATIGUE_FREE), 1.0);
    assert!((fatigue_form(MAX_FATIGUE) - 0.8).abs() < 1e-9);
    assert!(fatigue_form(60) < fatigue_form(40));
    let mut fresh = runner("F", 60, 0);
    fresh.fatigue = 0;
    let mut tired = fresh.clone();
    tired.fatigue = 90;
    let a = Game::run_stats(&fresh, Surface::Turf, 2000, TrackCondition::Good);
    let b = Game::run_stats(&tired, Surface::Turf, 2000, TrackCondition::Good);
    assert!(b.0 < a.0 && b.1 < a.1);
    let mut g = Game::new();
    g.field[0].fatigue = 80;
    assert_eq!(g.stable_entrant(0).fatigue, 80);
}

#[test]
fn a_horse_too_tired_cannot_be_entered_and_is_left_out_by_default() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    g.field[i].fatigue = RACE_FATIGUE_LIMIT;
    assert!(!g.eligible_horses(&race).contains(&i));
    assert_eq!(
        g.set_entries(Some(&key), std::slice::from_ref(&mine)),
        Err(GameError::NotEligible)
    );
    let card = g.race_card(Some(&key)).unwrap();
    assert!(card.entrants.iter().all(|e| e.name != mine));
    let h = card.my_horses.iter().find(|h| h.name == mine).unwrap();
    assert!(!h.eligible && h.fatigue == RACE_FATIGUE_LIMIT);
}

#[test]
fn resting_takes_fatigue_away_but_keeps_the_horse_out_for_the_week() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.field[0].fatigue = 70;
    g.rest(&name).unwrap();
    assert_eq!(g.field[0].fatigue, 70 - REST_RECOVERY);
    assert!(g.field[0].resting);
    assert_eq!(g.rest(&name), Err(GameError::Resting));
    assert_eq!(g.workout(&name, Workout::Walk), Err(GameError::Resting));
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    assert!(!g.eligible_horses(&race).contains(&0));
    assert!(g.race_card(None).unwrap().my_horses[0].resting);
    g.advance_week();
    assert!(!g.field[0].resting);
    assert_eq!(g.rest("いない馬"), Err(GameError::UnknownHorse));
}

#[test]
fn fatigue_and_rest_survive_a_save_and_old_saves_start_fresh() {
    let mut g = Game::new();
    let name = g.field[2].name.clone();
    g.field[2].fatigue = 40;
    g.rest(&name).unwrap();
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(
        (back.all(|h| h.fatigue), back.all(|h| h.resting)),
        (g.all(|h| h.fatigue), g.all(|h| h.resting))
    );
    let st = serde_json::to_value(back.state()).unwrap();
    assert_eq!(st["field"][2]["fatigue"], 10);
    assert_eq!(st["field"][2]["resting"], true);
    assert_eq!(st["race_fatigue_limit"], RACE_FATIGUE_LIMIT);
    let mut v = old_save(&Game::new());
    v.as_object_mut().unwrap().remove("fatigue");
    v.as_object_mut().unwrap().remove("resting");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert!(
        old.all(|h| h.fatigue).iter().all(|f| *f == 0) && old.all(|h| h.resting).iter().all(|r| !r)
    );
}

#[test]
fn a_foal_and_a_retiree_carry_no_fatigue_over() {
    let mut g = Game::new();
    let (sire, dam) = stallion_and_mare(&g);
    let slot = g
        .field
        .iter()
        .position(|h| h.name != sire && h.name != dam)
        .unwrap();
    g.field[slot].fatigue = 60;
    let replace = g.field[slot].name.clone();
    g.breed_now(&sire, &dam, &replace, "ハナミズキ").unwrap();
    assert_eq!((g.field[slot].fatigue, g.field[slot].resting), (0, false));
    g.studs.clear();
    let n = g.field[0].name.clone();
    g.retire(&n).unwrap();
}

// ---- leaving training to the trainer (お任せ調教) ----

#[test]
fn the_trainer_brings_on_the_weakest_ability() {
    let (mut g, _) = auto_horse();
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Work(Workout::Wood))); // all level: speed first
    g.field[0].speed = 70;
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Work(Workout::Hill))); // stamina and power level
    g.field[0].power = 70;
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Work(Workout::Pool)));
}

#[test]
fn the_trainer_settles_a_hot_horse_first() {
    let (mut g, _) = auto_horse();
    g.field[0].temper = AUTO_CALM_TEMPER - 1;
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Work(Workout::Walk)));
    g.field[0].temper = AUTO_CALM_TEMPER;
    assert_ne!(g.auto_plan(0), Some(AutoPlan::Work(Workout::Walk)));
}

#[test]
fn the_trainer_rests_a_horse_too_tired_to_race() {
    let (mut g, _) = auto_horse();
    g.field[0].fatigue = RACE_FATIGUE_LIMIT;
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Rest));
}

#[test]
fn the_trainer_does_not_work_a_horse_into_losing_form() {
    let (mut g, _) = auto_horse();
    g.field[0].fatigue = FATIGUE_FREE - Workout::Wood.fatigue();
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Work(Workout::Wood)));
    g.field[0].fatigue += 1;
    assert_eq!(g.auto_plan(0), None);
}

#[test]
fn the_trainer_has_nothing_to_do_for_a_horse_at_its_limits() {
    let (mut g, _) = auto_horse();
    g.field[0].speed = MAX_STAT;
    g.field[0].stamina = MAX_STAT;
    g.field[0].power = MAX_STAT;
    assert_eq!(g.auto_plan(0), None);
}

#[test]
fn a_new_week_trains_only_the_horses_left_to_the_trainer() {
    let (mut g, name) = auto_horse();
    let (money, cost) = (g.money, g.train_cost(&name).unwrap());
    let bill = g.weekly_upkeep();
    let other = abilities(&g, 1);
    g.advance_week();
    assert_eq!(abilities(&g, 0), (63, 61, 60, 50));
    assert_eq!(abilities(&g, 1), other);
    assert_eq!(g.money, money - bill - cost);
    assert_eq!(g.field[0].fatigue, Workout::Wood.fatigue());
    assert_eq!(
        g.news,
        vec![format!("{name}はお任せでウッドコース追いをしました。")]
    );
}

#[test]
fn nothing_happens_until_a_horse_is_left_to_the_trainer_and_it_can_be_taken_back() {
    let mut g = mid_horse();
    let bill = g.weekly_upkeep();
    let (before, money) = (abilities(&g, 0), g.money);
    g.advance_week();
    assert_eq!((abilities(&g, 0), g.money), (before, money - bill));
    let name = g.field[0].name.clone();
    g.set_auto_training(&name, true).unwrap();
    g.set_auto_training(&name, false).unwrap();
    g.advance_week();
    assert_eq!((abilities(&g, 0), g.money), (before, money - 2 * bill));
    assert_eq!(
        g.set_auto_training("いない馬", true),
        Err(GameError::UnknownHorse)
    );
}

#[test]
fn a_rest_left_to_the_trainer_keeps_the_horse_out_for_the_week_and_is_free() {
    let (mut g, name) = auto_horse();
    let money = g.money - g.weekly_upkeep();
    g.field[0].fatigue = RACE_FATIGUE_LIMIT + WEEKLY_RECOVERY;
    g.advance_week();
    assert!(g.field[0].resting);
    assert_eq!(g.field[0].fatigue, RACE_FATIGUE_LIMIT - REST_RECOVERY);
    assert_eq!(g.money, money);
    assert_eq!(g.news, vec![format!("{name}はお任せで休養しました。")]);
}

#[test]
fn the_trainer_always_leaves_money_in_the_stable() {
    let (mut g, name) = auto_horse();
    let cost = g.train_cost(&name).unwrap();
    let bill = g.weekly_upkeep();
    let keep = g.auto_keep();
    g.money = keep + bill + cost - 1;
    g.advance_week();
    assert_eq!(g.money, keep + cost - 1);
    assert!(g.news.is_empty());
    g.money = keep + bill + cost;
    g.advance_week();
    assert_eq!(g.money, keep);
    // the trainer alone never spends the reserve
    g.money = keep + 2_000;
    let before = g.money;
    let trained = g.field[0].trainings;
    for _ in 0..3 {
        g.advance_week();
    }
    assert!(g.field[0].trainings > trained);
    assert!(g.money < before);
    assert!(!g.is_game_over());
}

#[test]
fn the_trainer_works_after_a_calendar_race_moves_the_week_on() {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    g.field[i].trainer = trainer::STANDARD;
    g.set_auto_training(&mine, true).unwrap();
    g.field[i].temper = AUTO_CALM_TEMPER - 10; // hot, so walked even though it has just raced
    let hot = g.field[i].temper;
    let rival = a_rival_in(&g, &key);
    let out = g
        .play_combo(Some(&key), &[rival], 10, 1, BetKind::Win)
        .unwrap();
    assert_eq!(g.field[i].temper, hot + 3);
    assert_eq!(
        g.field[i].fatigue,
        race_fatigue(race.distance) - WEEKLY_RECOVERY
    );
    assert!(g
        .news
        .contains(&format!("{mine}はお任せで曳き運動をしました。")));
    assert_eq!(out.money, g.money); // the outcome reports what is left after paying for it
    assert_eq!(g.week, 48);
}

#[test]
fn auto_training_is_kept_in_a_save_and_old_saves_have_none() {
    let (g, _) = auto_horse();
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.all(|h| h.auto_training), g.all(|h| h.auto_training));
    assert!(back.state().field[0].auto_training);
    let v = serde_json::to_value(back.state()).unwrap();
    assert_eq!(v["field"][0]["auto_training"], true);
    assert_eq!(v["auto_keep_money"], g.auto_keep());
    let mut old = old_save(&Game::new());
    old.as_object_mut().unwrap().remove("auto_training");
    let old = Game::from_json(&old.to_string()).unwrap();
    assert!(old.all(|h| h.auto_training).iter().all(|a| !a));
}

#[test]
fn a_foal_starts_with_the_trainer_not_in_charge_and_a_retiree_drops_out() {
    let (mut g, name) = auto_horse();
    let (sire, dam) = stallion_and_mare(&g);
    let slot = g
        .field
        .iter()
        .position(|h| h.name != sire && h.name != dam)
        .unwrap();
    g.field[slot].auto_training = true;
    let replace = g.field[slot].name.clone();
    g.breed_now(&sire, &dam, &replace, "ハナミズキ").unwrap();
    assert!(!g.field[slot].auto_training);
    g.studs.clear();
    if g.field.iter().any(|h| h.name == name) {
        g.retire(&name).unwrap();
    }
}

// ---- ブリンカー ----

#[test]
fn blinkers_are_kept_with_the_rest_of_the_gear_and_old_gear_has_none() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let gear = Gear {
        blinkers: true,
        ..Gear::default()
    };
    g.set_gear(&name, gear).unwrap();
    assert!(
        Game::from_json(&g.to_json()).unwrap().field[0]
            .gear
            .blinkers
    );
    let old: Gear = serde_json::from_str(r#"{"menko":true,"shadow_roll":false}"#).unwrap();
    assert!(old.menko && !old.blinkers);
}
