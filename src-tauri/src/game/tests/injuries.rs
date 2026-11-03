//! Injuries and time at pasture (故障・放牧).

use super::*;

// ---- injuries and time at pasture (故障・放牧) ----

#[test]
fn every_horse_has_legs_of_its_own_that_stay_the_same() {
    let g = Game::new();
    assert!(g
        .all(|h| h.legs)
        .iter()
        .all(|l| (LEGS_MIN..=LEGS_MAX).contains(l)));
    assert!(
        g.all(|h| h.legs)
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            > 1
    );
    assert_eq!(Game::new().all(|h| h.legs), g.all(|h| h.legs));
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["field"][0]["legs"], g.field[0].legs);
}

#[test]
fn tired_horses_and_weak_legs_are_more_likely_to_break_down() {
    let c = |f, l| injury_chance(RACE_INJURY, f, l);
    assert!(c(60, 60) > c(20, 60));
    assert!(c(40, 40) > c(40, 80));
    assert!(c(100, LEGS_MIN) < 0.05, "still rare: {}", c(100, LEGS_MIN));
    assert!(injury_chance(WORKOUT_INJURY, 40, 60) < c(40, 60));
}

#[test]
fn an_injury_keeps_a_horse_out_costs_a_vet_bill_and_may_leave_a_mark() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    let money = g.money;
    g.injure(0, Injury::Soe);
    assert_eq!(g.money, money - Injury::Soe.vet_fee());
    let a = g.field[0].away.clone().unwrap();
    assert_eq!(
        (a.kind, a.label.as_str(), a.weeks),
        (AwayKind::Injured, "ソエ", 4)
    );
    assert!(g
        .news
        .iter()
        .any(|n| n.contains(&name) && n.contains("ソエ")));
    assert_eq!(g.field[0].speed, 60);
    g.field[0].away = None;
    g.injure(0, Injury::Tendon);
    assert_eq!(g.field[0].speed, 60 - Injury::Tendon.speed_loss());
    assert_eq!(g.field[0].away.as_ref().unwrap().label, "屈腱炎");
}

#[test]
fn a_vet_bill_does_not_finish_a_stable_off() {
    let mut g = mid_horse();
    g.money = 10;
    g.injure(0, Injury::Tendon);
    assert_eq!(g.money, 1);
    assert!(!g.is_game_over());
}

#[test]
fn a_horse_that_is_away_can_not_race_train_or_rest() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.injure(0, Injury::Soe);
    assert_eq!(g.workout(&name, Workout::Wood), Err(GameError::Away));
    assert_eq!(g.rest(&name), Err(GameError::Away));
    assert_eq!(
        g.set_entries(None, std::slice::from_ref(&name)),
        Err(GameError::NotEligible)
    );
    let card = g.race_card(None).unwrap();
    let mine = card.my_horses.iter().find(|h| h.name == name).unwrap();
    assert!(!mine.eligible && !mine.entered);
    assert_eq!(mine.away.as_ref().unwrap().label, "ソエ");
    assert!(card.entrants.iter().all(|e| e.name != name));
}

#[test]
fn a_horse_comes_back_when_its_time_is_up_rested_and_in_form() {
    let mut g = mid_horse();
    g.money = 100_000;
    let name = g.field[0].name.clone();
    g.field[0].fatigue = 90;
    g.injure(0, Injury::Soe);
    for left in [3, 2, 1] {
        g.advance_week();
        assert_eq!(g.field[0].away.as_ref().unwrap().weeks, left);
    }
    g.advance_week();
    assert_eq!(g.field[0].away, None);
    assert_eq!(g.field[0].fatigue, 0);
    assert_eq!(g.field[0].condition, Condition::Good);
    assert!(
        g.news
            .iter()
            .any(|n| n.contains(&name) && n.contains("復帰")),
        "{:?}",
        g.news
    );
    assert!(g.workout(&name, Workout::Wood).is_ok() || g.field[0].away.is_some());
}

#[test]
fn a_horse_can_be_sent_to_pasture_and_called_back() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.field[0].fatigue = 80;
    g.send_to_pasture(&name).unwrap();
    let a = g.field[0].away.clone().unwrap();
    assert_eq!(
        (a.kind, a.label.as_str(), a.weeks),
        (AwayKind::Pasture, "放牧", PASTURE_WEEKS)
    );
    assert_eq!(g.send_to_pasture(&name), Err(GameError::Away));
    g.advance_week();
    assert_eq!(g.field[0].fatigue, 80 - WEEKLY_RECOVERY - REST_RECOVERY);
    assert_eq!(g.field[0].away.as_ref().unwrap().weeks, PASTURE_WEEKS - 1);
    g.recall(&name).unwrap();
    assert_eq!(g.field[0].away, None);
    assert_eq!(g.recall(&name), Err(GameError::NotOnPasture));
    assert_eq!(g.send_to_pasture("いない馬"), Err(GameError::UnknownHorse));
    assert_eq!(g.recall("いない馬"), Err(GameError::UnknownHorse));
}

#[test]
fn an_injured_horse_can_not_be_called_back() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.injure(0, Injury::Soe);
    assert_eq!(g.recall(&name), Err(GameError::NotOnPasture));
    assert_eq!(g.send_to_pasture(&name), Err(GameError::Away));
}

#[test]
fn pasture_takes_a_horse_out_of_its_entries_and_a_day_of_rest() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.set_entries(None, std::slice::from_ref(&name)).unwrap();
    g.field[0].resting = true;
    g.send_to_pasture(&name).unwrap();
    assert!(!g.field[0].resting);
    assert_eq!(g.entries["free"], Vec::<String>::new());
}

#[test]
fn a_horse_that_is_away_costs_half_as_much_to_keep() {
    let mut g = mid_horse();
    let bill = g.weekly_upkeep();
    let name = g.field[0].name.clone();
    g.send_to_pasture(&name).unwrap();
    assert_eq!(g.weekly_upkeep(), bill - UPKEEP_PER_HORSE / 2);
}

#[test]
fn an_injured_horse_sells_for_less() {
    let mut g = mid_horse();
    let fit = g.sale_price(0);
    g.send_to_pasture(&g.field[0].name.clone()).unwrap();
    assert_eq!(g.sale_price(0), fit);
    g.field[0].away = None;
    g.injure(0, Injury::Soe);
    assert_eq!(g.sale_price(0), (fit / 2 / 10) * 10);
    g.field[0].speed = 1;
    g.field[0].stamina = 1;
    g.field[0].power = 1;
    g.field[0].record = RaceRecord::default();
    assert_eq!(g.sale_price(0), SALE_MIN);
}

#[test]
fn horses_that_are_away_miss_the_trainers_plans() {
    let mut g = with_trainer(trainer::STANDARD);
    g.field[0].auto_training = true;
    g.week = (1..=52)
        .find(|&w| {
            g.week = w;
            !g.auto_choices(0, None).is_empty()
        })
        .unwrap();
    assert!(!g.auto_entries(None).is_empty());
    g.injure(0, Injury::Soe);
    assert!(g.auto_entries(None).is_empty());
    assert_eq!(g.auto_plan(0), None);
}

#[test]
fn a_trainer_turns_out_a_worn_horse() {
    let (mut g, name) = auto_horse();
    g.field[0].auto_entry = false;
    g.field[0].fatigue = AUTO_PASTURE_FATIGUE;
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Pasture));
    g.field[0].fatigue += WEEKLY_RECOVERY;
    g.advance_week();
    assert_eq!(g.field[0].away.as_ref().unwrap().kind, AwayKind::Pasture);
    assert_eq!(g.news, vec![format!("{name}はお任せで放牧に出ました。")]);
}

#[test]
fn work_and_racing_do_break_horses_down_in_the_end() {
    // the chance is small, so many tries find it
    let mut injured = 0;
    for seed in 0..40u64 {
        let mut g = Game::with_seed(seed + 1);
        g.money = 1_000_000;
        for i in 0..g.field.len() {
            g.field[i].auto_entry = true;
            g.field[i].trainer = trainer::STANDARD;
        }
        for _ in 0..52 {
            g.advance_week();
            injured += g.news.iter().filter(|n| n.contains("故障")).count();
        }
    }
    assert!(injured > 0);
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    g.money = 10_000_000;
    g.field[0].legs = LEGS_MIN;
    let mut hurt = false;
    for _ in 0..3000 {
        g.field[0].fatigue = 80;
        g.field[0].speed = 60;
        if g.workout(&name, Workout::Wood).is_ok() && g.field[0].away.is_some() {
            hurt = true;
            break;
        }
    }
    assert!(hurt);
}

#[test]
fn a_races_injury_is_told_after_the_race_moved_the_week_on() {
    let mut found = false;
    for seed in 0..200u64 {
        let mut g = Game::with_seed(seed + 1);
        g.money = 100_000;
        g.week = 47;
        let race = calendar_race("国際友好ステークス");
        let key = key_of(&race);
        let mine = my_runner(&g, &key);
        let i = g.field.iter().position(|h| h.name == mine).unwrap();
        g.field[i].fatigue = 49;
        g.field[i].legs = LEGS_MIN;
        let rival = a_rival_in(&g, &key);
        g.play_combo(Some(&key), &[rival], 10, 1, BetKind::Win)
            .unwrap();
        if g.field[i]
            .away
            .as_ref()
            .is_some_and(|a| a.kind == AwayKind::Injured)
        {
            assert!(
                g.news
                    .iter()
                    .any(|n| n.contains(&mine) && n.contains("故障")),
                "{:?}",
                g.news
            );
            found = true;
            break;
        }
    }
    assert!(found);
}

#[test]
fn a_foal_takes_after_its_parents_legs() {
    let mut g = Game::new();
    g.money = 10_000_000;
    let (sire, dam) = stallion_and_mare(&g);
    let (s, d) = (
        g.field.iter().position(|h| h.name == sire).unwrap(),
        g.field.iter().position(|h| h.name == dam).unwrap(),
    );
    g.field[s].legs = 90;
    g.field[d].legs = 90;
    let slot = g
        .field
        .iter()
        .position(|h| h.name != sire && h.name != dam)
        .unwrap();
    let replace = g.field[slot].name.clone();
    g.breed_now(&sire, &dam, &replace, "ハナミズキ").unwrap();
    assert!(
        (90 - FOAL_LEGS_SPREAD..=LEGS_MAX).contains(&g.field[slot].legs),
        "{}",
        g.field[slot].legs
    );
    assert_eq!(g.field[slot].away, None);
}

#[test]
fn a_retired_horse_keeps_its_legs_for_its_foals() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.field[0].legs = 77;
    g.retire(&name).unwrap();
    let stud = g.studs.iter().find(|s| s.name == name).unwrap();
    assert_eq!(stud.legs, 77);
    let v = serde_json::to_value(g.state()).unwrap();
    assert!(v["studs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["legs"] == 77));
    assert!(g
        .stud_market()
        .iter()
        .all(|o| (LEGS_MIN..=LEGS_MAX).contains(&o.legs)));
}

#[test]
fn injuries_and_legs_are_kept_in_a_save_and_old_saves_get_them() {
    let mut g = mid_horse();
    g.injure(0, Injury::Fracture);
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.all(|h| h.away.clone()), g.all(|h| h.away.clone()));
    assert_eq!(back.all(|h| h.legs), g.all(|h| h.legs));
    let mut old = old_save(&Game::new());
    let o = old.as_object_mut().unwrap();
    o.remove("legs");
    o.remove("away");
    for s in o["studs"].as_array_mut().unwrap() {
        s.as_object_mut().unwrap().remove("legs");
    }
    let old = Game::from_json(&old.to_string()).unwrap();
    assert!(old
        .all(|h| h.legs)
        .iter()
        .all(|l| (LEGS_MIN..=LEGS_MAX).contains(l)));
    assert!(old.all(|h| h.away.clone()).iter().all(|a| a.is_none()));
    assert!(old
        .studs
        .iter()
        .all(|s| (LEGS_MIN..=LEGS_MAX).contains(&s.legs)));
}
