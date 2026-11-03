//! Trainers and their stables (調教師・厩舎).

use super::*;

// ---- trainers (調教師) and their stables (厩舎) ----

#[test]
fn a_horse_runs_better_over_its_trainers_distance() {
    let plain = fit_runner(4, 4, (1000, 3600));
    let mut coached = plain.clone();
    coached.trainer_distance = Some(trainer::Distance::Sprint);
    let stats = |e: &crate::rivals::Entrant, m: u32| {
        Game::run_stats(e, Surface::Turf, m, TrackCondition::Good)
    };
    let (s0, t0) = stats(&plain, 1200);
    assert_eq!(
        stats(&coached, 1200),
        (
            (s0 as f64 * trainer::DISTANCE_BONUS).round() as u32,
            (t0 as f64 * trainer::DISTANCE_BONUS).round() as u32
        )
    );
    assert_eq!(stats(&coached, 1600), stats(&plain, 1600));
    coached.trainer_distance = Some(trainer::Distance::Any);
    assert_eq!(stats(&coached, 1200), stats(&plain, 1200));
}

#[test]
fn the_race_card_marks_the_horses_running_over_their_trainers_distance() {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    let fit = |g: &Game| {
        g.race_card(Some(&key))
            .unwrap()
            .entrants
            .iter()
            .find(|e| e.name == mine)
            .unwrap()
            .trainer_fit
    };
    g.field[i].trainer = trainer_with(|t| t.distance.favours(race.distance));
    assert!(fit(&g));
    g.field[i].trainer = trainer_with(|t| {
        t.distance != trainer::Distance::Any && !t.distance.favours(race.distance)
    });
    assert!(!fit(&g));
    g.field[i].trainer = trainer::STANDARD;
    assert!(!fit(&g));
    let card = g.race_card(Some(&key)).unwrap();
    assert!(card
        .entrants
        .iter()
        .filter(|e| !e.mine)
        .all(|e| !e.trainer_fit));
}

#[test]
fn the_screen_is_told_how_much_a_trainers_distance_is_worth() {
    let v = serde_json::to_value(Game::new().state()).unwrap();
    assert_eq!(v["trainer_bonus_percent"], 3);
}

#[test]
fn the_trainers_distance_shows_in_the_odds() {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    let odds = |g: &Game| {
        g.race_card(Some(&key))
            .unwrap()
            .entrants
            .iter()
            .find(|e| e.name == mine)
            .unwrap()
            .odds
    };
    g.field[i].trainer = trainer::STANDARD;
    let without = odds(&g);
    g.field[i].trainer = trainer_with(|t| t.distance.favours(race.distance));
    assert!(odds(&g) <= without);
}

fn stable_count(g: &Game, id: u8) -> usize {
    g.all(|h| h.trainer).iter().filter(|&&t| t == id).count()
}

/// The trainer with these ways.
fn trainer_with(pick: impl Fn(&trainer::Trainer) -> bool) -> u8 {
    TRAINERS.iter().find(|t| pick(t)).unwrap().id
}

#[test]
fn every_horse_starts_with_a_trainer_and_the_stable_is_shared_out() {
    for seed in [1u64, 2, 3] {
        let g = Game::with_seed(seed);
        assert!(g
            .all(|h| h.trainer)
            .iter()
            .all(|&t| (t as usize) < TRAINERS.len()));
        for t in &TRAINERS {
            assert!(stable_count(&g, t.id) <= STABLE_CAPACITY);
        }
        let used: std::collections::HashSet<u8> = g.all(|h| h.trainer).iter().copied().collect();
        assert!(used.len() >= 2);
    }
    assert_ne!(
        Game::with_seed(1).all(|h| h.trainer),
        Game::with_seed(2).all(|h| h.trainer)
    );
}

#[test]
fn a_horse_can_be_moved_to_another_stable_free_of_charge() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let target = (g.field[0].trainer + 1) % TRAINERS.len() as u8;
    let money = g.money;
    g.set_trainer(&name, target).unwrap();
    assert_eq!(g.field[0].trainer, target);
    assert_eq!(g.money, money);
    assert_eq!(g.set_trainer("いない馬", 0), Err(GameError::UnknownHorse));
    assert_eq!(
        g.set_trainer(&name, TRAINERS.len() as u8),
        Err(GameError::UnknownTrainer)
    );
}

#[test]
fn a_full_stable_takes_no_more_horses() {
    let mut g = Game::new();
    let (sire, dam) = stallion_and_mare(&g);
    fill_stables(&mut g, &sire, &dam);
    for t in &TRAINERS {
        assert_eq!(stable_count(&g, t.id), STABLE_CAPACITY);
    }
    let name = g.field[0].name.clone();
    let own = g.field[0].trainer;
    let other = (own + 1) % TRAINERS.len() as u8;
    assert_eq!(g.set_trainer(&name, other), Err(GameError::TrainerFull));
    assert_eq!(g.set_trainer(&name, own), Ok(())); // already there
    assert_eq!(g.field[0].trainer, own);
}

#[test]
fn the_stables_hold_ten_each_in_all() {
    assert_eq!(STABLE_CAPACITY, 10);
    assert_eq!(STABLE_SIZE, TRAINERS.len() * STABLE_CAPACITY);
}

#[test]
fn a_foal_goes_to_the_emptiest_stable_or_stays_in_the_one_it_replaces() {
    let mut g = Game::new();
    g.money = 100_000;
    let (sire, dam) = stallion_and_mare(&g);
    g.breed_now(&sire, &dam, "", "ハナミズキ").unwrap();
    let foal = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    let least = TRAINERS
        .iter()
        .map(|t| stable_count(&g, t.id))
        .min()
        .unwrap();
    assert!(stable_count(&g, g.field[foal].trainer) <= least + 1);
    let slot = g
        .field
        .iter()
        .position(|h| h.name != sire && h.name != dam && h.name != "ハナミズキ")
        .unwrap();
    let (replace, keeper) = (g.field[slot].name.clone(), g.field[slot].trainer);
    g.field[slot].since_race = 1;
    g.breed_now(&sire, &dam, &replace, "サクラソウ").unwrap();
    assert_eq!(g.field[slot].trainer, keeper);
    assert_eq!(g.field[slot].since_race, NEVER_RACED);
}

#[test]
fn trainers_are_kept_in_a_save_and_an_old_save_gets_them() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.set_trainer(&name, 7).unwrap();
    g.field[0].since_race = 3;
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.all(|h| h.trainer), g.all(|h| h.trainer));
    assert_eq!(back.all(|h| h.since_race), g.all(|h| h.since_race));
    let mut old: serde_json::Value = serde_json::to_value(Game::new()).unwrap();
    let o = old.as_object_mut().unwrap();
    o.remove("trainer_of");
    o.remove("since_race");
    let old = Game::from_json(&old.to_string()).unwrap();
    assert!(old
        .field
        .iter()
        .all(|h| (h.trainer as usize) < TRAINERS.len()));
    assert!(old.all(|h| h.since_race).iter().all(|&w| w == NEVER_RACED));
}

#[test]
fn the_screen_is_told_of_the_trainers_and_each_horses_stable() {
    let g = Game::new();
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["trainers"].as_array().unwrap().len(), 10);
    assert_eq!(v["trainers"][4]["name"], "松原 誠一");
    assert_eq!(v["stable_capacity"], 10);
    assert_eq!(v["field"][0]["trainer"], g.field[0].trainer);
}

#[test]
fn a_sparta_trainer_works_a_horse_harder_than_a_kind_one() {
    let sparta = trainer_with(|t| t.training == trainer::Training::Sparta);
    let kind = trainer_with(|t| t.training == trainer::Training::Kind);
    let run = |id: u8| {
        let mut g = with_trainer(id);
        g.field[0].auto_entry = false;
        g.field[0].auto_training = true;
        g.field[0].fatigue = 15 + WEEKLY_RECOVERY;
        let before = g.field[0].trainings;
        g.advance_week();
        g.field[0].trainings - before
    };
    assert_eq!(run(sparta), 2);
    assert_eq!(run(kind), 0);
}

#[test]
fn a_kind_trainer_rests_a_tired_horse_sooner_than_a_sparta_one() {
    let sparta = trainer_with(|t| t.training == trainer::Training::Sparta);
    let kind = trainer_with(|t| t.training == trainer::Training::Kind);
    let mut g = with_trainer(kind);
    g.field[0].fatigue = 30;
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Rest));
    g.field[0].trainer = sparta;
    assert_ne!(g.auto_plan(0), Some(AutoPlan::Rest));
    g.field[0].fatigue = 60;
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Rest));
}

#[test]
fn a_trainer_brings_on_what_their_distance_calls_for() {
    let sprint = trainer_with(|t| {
        t.distance == trainer::Distance::Sprint && t.training != trainer::Training::Kind
    });
    let stayer = trainer_with(|t| {
        t.distance == trainer::Distance::Stayer && t.training != trainer::Training::Kind
    });
    let mut g = with_trainer(sprint);
    g.field[0].speed = 63; // a little ahead of the others, but this trainer wants more
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Work(Workout::Wood)));
    g.field[0].trainer = stayer;
    g.field[0].speed = 60;
    g.field[0].stamina = 63;
    g.field[0].power = 65;
    assert_eq!(g.auto_plan(0), Some(AutoPlan::Work(Workout::Pool)));
}

#[test]
fn a_standard_trainer_leaves_two_weeks_between_races() {
    let mut g = with_trainer(trainer::STANDARD);
    g.field[0].since_race = 2;
    g.week = (1..=52)
        .find(|&w| {
            g.week = w;
            !g.auto_choices(0, None).is_empty()
        })
        .unwrap();
    assert!(g.auto_entries(None).is_empty());
    g.field[0].since_race = 3;
    assert!(!g.auto_entries(None).is_empty());
}

/// Weeks a horse runs in, in a year with this trainer.
fn starts_in_a_year(id: u8) -> u32 {
    let mut g = with_trainer(id);
    for _ in 0..calendar::WEEKS_PER_YEAR {
        g.field[0].condition = Condition::Normal;
        g.advance_week();
    }
    g.field[0].record.starts
}

#[test]
fn a_busy_trainer_runs_a_horse_more_often_than_a_patient_one() {
    let busy = trainer_with(|t| t.rotation == trainer::Rotation::Busy);
    let regular = trainer::STANDARD;
    let spaced =
        trainer_with(|t| t.rotation == trainer::Rotation::Spaced && t.aim != trainer::Aim::Glory);
    let (b, r, s) = (
        starts_in_a_year(busy),
        starts_in_a_year(regular),
        starts_in_a_year(spaced),
    );
    assert!(b > r && r > s, "{b} {r} {s}");
}

#[test]
fn a_trainer_who_watches_form_runs_only_a_horse_in_form_again_next_week() {
    let id = trainer_with(|t| t.rotation == trainer::Rotation::ByForm);
    let mut g = with_trainer(id);
    g.field[0].speed = MAX_STAT;
    g.field[0].since_race = 1;
    for week in 1..=52 {
        g.week = week;
        if !g.auto_choices(0, None).is_empty() {
            break;
        }
    }
    g.field[0].condition = Condition::Poor;
    assert!(g.auto_entries(None).is_empty());
    g.field[0].condition = Condition::Great;
    assert!(!g.auto_entries(None).is_empty());
}

#[test]
fn the_players_own_race_counts_towards_the_interval() {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    let rival = a_rival_in(&g, &key);
    g.play_combo(Some(&key), &[rival], 10, 1, BetKind::Win)
        .unwrap();
    assert_eq!(g.field[i].since_race, 1);
}

#[test]
fn the_weeks_since_a_race_count_up_and_a_free_race_does_not_count() {
    let mut g = with_trainer(trainer::STANDARD);
    g.field[0].auto_entry = false;
    g.field[0].since_race = 2;
    g.advance_week();
    assert_eq!(g.field[0].since_race, 3);
    let name = g.field[0].name.clone();
    g.set_entries(None, std::slice::from_ref(&name)).unwrap();
    g.play_combo(None, &[name], 10, 1, BetKind::Win).unwrap();
    assert_eq!(g.field[0].since_race, 3);
}

#[test]
fn a_trainer_sends_a_horse_over_their_distance_when_the_week_offers_it() {
    for (bent, band) in [
        (trainer::Distance::Sprint, 0..=1400u32),
        (trainer::Distance::Stayer, 2401..=9999),
    ] {
        let id = trainer_with(|t| t.distance == bent);
        let mut g = with_trainer(id);
        g.field[0].speed = MAX_STAT;
        let mut checked = 0;
        for week in 1..=52 {
            g.week = week;
            let offers = g.auto_choices(0, None);
            if offers.iter().any(|r| band.contains(&r.distance)) {
                let saving = g.saving_for_g1(0);
                if let (Some(r), false) = (g.auto_race(0, &[], None), saving) {
                    assert!(band.contains(&r.distance), "week {week}: {}m", r.distance);
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "{bent:?}");
    }
}

#[test]
fn a_trainer_runs_a_horse_at_other_distances_when_theirs_is_not_on_offer() {
    let id =
        trainer_with(|t| t.distance == trainer::Distance::Stayer && t.aim != trainer::Aim::Glory);
    let mut g = with_trainer(id);
    let mut found = 0;
    for week in 1..=52 {
        g.week = week;
        let offers = g.auto_choices(0, None);
        if !offers.is_empty() && !offers.iter().any(|r| r.distance > 2400) {
            assert!(g.auto_race(0, &[], None).is_some());
            found += 1;
        }
    }
    assert!(found > 0);
}

#[test]
fn a_trainer_with_an_eye_on_the_g1_sends_a_horse_the_others_would_not() {
    let g1 = rivals::speed_centre(Grade::G1) as u32;
    let mut checked = 0;
    for t in TRAINERS.iter().filter(|t| t.aim == trainer::Aim::Glory) {
        let mut g = with_trainer(t.id);
        g.field[0].speed = g1 - 4; // up to the rivals of a G1 only with the trainer's ambition
        for week in 1..=52 {
            g.week = week;
            let suits = g
                .auto_choices(0, None)
                .iter()
                .any(|r| r.grade == Grade::G1 && t.distance.covers(r.distance));
            if !suits {
                continue;
            }
            g.field[0].trainer = t.id;
            assert_eq!(g.auto_race(0, &[], None).unwrap().grade, Grade::G1);
            g.field[0].trainer = trainer::STANDARD;
            assert_ne!(g.auto_race(0, &[], None).unwrap().grade, Grade::G1);
            checked += 1;
        }
    }
    assert!(checked > 0);
}

#[test]
fn a_careful_trainer_keeps_a_horse_where_it_should_win() {
    let steady =
        trainer_with(|t| t.aim == trainer::Aim::Steady && t.distance == trainer::Distance::Any);
    let mut g = with_trainer(steady);
    // find a week where a class just at the horse's level is offered with an easier one
    let mut checked = 0;
    for week in 1..=52 {
        g.week = week;
        let offers = g.auto_choices(0, None);
        let top = offers.iter().map(|r| rivals::speed_centre(r.grade)).max();
        let low = offers.iter().map(|r| rivals::speed_centre(r.grade)).min();
        let (Some(top), Some(low)) = (top, low) else {
            continue;
        };
        if top - low < 4 {
            continue;
        }
        g.field[0].speed = top as u32;
        let picked = g.auto_race(0, &[], None).unwrap();
        assert!(rivals::speed_centre(picked.grade) <= top - 4);
        checked += 1;
    }
    assert!(checked > 0);
}

#[test]
fn a_trainer_aiming_at_the_g1_saves_the_horse_for_it() {
    let glory = trainer_with(|t| t.aim == trainer::Aim::Glory);
    let mut g = with_trainer(glory);
    g.field[0].speed = MAX_STAT;
    let mut checked = 0;
    for target in 2..=52u8 {
        g.week = target;
        let g1_then = g.auto_choices(0, None).iter().any(|r| r.grade == Grade::G1);
        g.week = target - 1;
        let before = g.auto_choices(0, None);
        if !g1_then || before.is_empty() || before.iter().any(|r| r.grade == Grade::G1) {
            continue;
        }
        g.field[0].trainer = glory;
        assert!(g.saving_for_g1(0));
        assert!(g.auto_entries(None).is_empty());
        g.field[0].trainer = trainer::STANDARD;
        assert!(!g.saving_for_g1(0));
        assert!(!g.auto_entries(None).is_empty());
        checked += 1;
    }
    assert!(checked > 0);
}

#[test]
fn a_stable_of_every_kind_of_trainer_plays_on_for_years() {
    let mut g = Game::new();
    for i in 0..g.field.len() {
        g.field[i].auto_entry = true;
        g.field[i].auto_training = true;
        g.field[i].trainer = (i % TRAINERS.len()) as u8;
    }
    for _ in 0..(3 * calendar::WEEKS_PER_YEAR as usize) {
        g.advance_week();
        assert!(!g.is_game_over());
    }
}

#[test]
fn a_foal_is_not_entered_automatically_and_a_retiree_drops_out() {
    let (mut g, name) = auto_runner();
    let (sire, dam) = stallion_and_mare(&g);
    let slot = g
        .field
        .iter()
        .position(|h| h.name != sire && h.name != dam)
        .unwrap();
    g.field[slot].auto_entry = true;
    let replace = g.field[slot].name.clone();
    g.breed_now(&sire, &dam, &replace, "ハナミズキ").unwrap();
    assert!(!g.field[slot].auto_entry);
    g.studs.clear();
    if g.field.iter().any(|h| h.name == name) {
        g.retire(&name).unwrap();
    }
}

#[test]
fn a_race_can_be_run_without_a_ticket() {
    let mut g = Game::with_seed(1);
    let money = g.money;
    let o = g.watch_race(None, 1).unwrap();
    assert!(o.tickets.is_empty());
    assert!(!o.won);
    assert_eq!(o.payout, 0);
    assert_eq!(o.money, money);
    assert_eq!(g.money, money);
    assert_eq!(g.round, 1);
    let h = g.history.last().unwrap();
    assert!(h.watched);
    assert_eq!((h.ticket_count, h.stake, h.payout), (0, 0, 0));
    assert!(h.horse.contains("着"));
}

#[test]
fn a_calendar_race_without_a_ticket_still_moves_the_week_on() {
    let mut g = Game::with_seed(1);
    let race = calendar_race("疾風賞");
    g.week = race.week;
    let o = g.watch_race(Some(&key_of(&race)), 1).unwrap();
    assert_eq!(g.week, race.week + 1);
    assert_eq!(o.race_name.as_deref(), Some("疾風賞"));
    assert_eq!(g.history.last().unwrap().race.as_deref(), Some("疾風賞"));
}

#[test]
fn no_race_can_be_watched_when_the_game_is_over() {
    let mut g = Game::with_seed(1);
    g.money = 0;
    assert_eq!(g.watch_race(None, 1), Err(GameError::GameOver));
}

#[test]
fn a_ticket_race_is_not_marked_as_watched() {
    let mut g = Game::with_seed(1);
    let name = g.field[0].name.clone();
    g.play_round(&name, 10, 1).unwrap();
    assert!(!g.history.last().unwrap().watched);
}
