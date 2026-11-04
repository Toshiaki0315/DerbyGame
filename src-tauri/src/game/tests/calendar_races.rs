//! Calendar races: weeks, years, eligibility and the other runners (週・年・出走条件).

use super::*;

// ---- calendar races (週・年・出走条件・相手馬) ----

fn names_of(card: &RaceCard) -> Vec<String> {
    card.entrants.iter().map(|e| e.name.clone()).collect()
}

#[test]
fn new_game_starts_in_week_one_of_year_one() {
    let g = Game::new();
    assert_eq!((g.week, g.year), (1, 1));
    let st = g.state();
    assert_eq!((st.week, st.year), (1, 1));
}

#[test]
fn advancing_the_week_moves_forward_and_rerolls_conditions() {
    let mut g = Game::with_seed(2);
    let mut seen = vec![g.all(|h| h.condition)];
    for expected in 2..=12u8 {
        g.advance_week();
        assert_eq!(g.week, expected);
        seen.push(g.all(|h| h.condition));
    }
    assert!(seen.windows(2).any(|w| w[0] != w[1]));
}

#[test]
fn the_year_rolls_over_after_week_52_and_horses_age() {
    let mut g = Game::new();
    let before: Vec<u8> = g.state().field.iter().map(|h| h.age).collect();
    g.week = 52;
    g.advance_week();
    assert_eq!((g.week, g.year), (1, 2));
    let after: Vec<u8> = g.state().field.iter().map(|h| h.age).collect();
    assert!(before.iter().zip(&after).all(|(b, a)| *a == b + 1));
    // ordinary week changes do not age anyone
    g.advance_week();
    let later: Vec<u8> = g.state().field.iter().map(|h| h.age).collect();
    assert_eq!(after, later);
}

#[test]
fn stable_horses_have_ages_and_dirt_aptitudes() {
    let st = Game::new().state();
    assert!(st.field.iter().all(|h| (3..=6).contains(&h.age)));
    assert!(st.field.iter().all(|h| (-2..=2).contains(&h.dirt)));
    assert!(st.field.iter().any(|h| h.dirt > 0), "no dirt specialist");
    assert!(st.field.iter().any(|h| h.dirt < 0), "no turf specialist");
}

#[test]
fn foals_are_two_year_olds_and_inherit_dirt_aptitude() {
    let mut g = Game::with_seed(1);
    for i in [0usize, 4] {
        g.field[i].dirt = 2;
    }
    breed_default(&mut g).unwrap();
    let foal = &g.state().field[5];
    assert_eq!(foal.age, 2);
    assert!((1..=2).contains(&foal.dirt), "dirt {}", foal.dirt);
}

#[test]
fn old_saves_without_week_age_or_dirt_still_load() {
    let g = Game::with_seed(4);
    let mut v: serde_json::Value = old_save(&g);
    for k in ["week", "year", "ages", "dirts", "world_seed"] {
        v.as_object_mut().unwrap().remove(k);
    }
    let old = Game::from_json(&v.to_string()).unwrap();
    assert_eq!((old.week, old.year), (1, 1));
    assert_eq!(old.state().field.len(), 8);
    assert!(old.state().field.iter().all(|h| h.age >= 3));
}

#[test]
fn week_year_and_ages_survive_save_and_load() {
    let mut g = Game::with_seed(4);
    g.week = 52;
    g.advance_week();
    g.advance_week();
    let restored = Game::from_json(&g.to_json()).unwrap();
    assert_eq!((restored.week, restored.year), (g.week, g.year));
    assert_eq!(restored.state(), g.state());
}

#[test]
fn dirt_aptitude_changes_speed_only_on_dirt() {
    use crate::calendar::Surface;
    let mk = |dirt: i8| crate::rivals::Entrant {
        jump: 0,
        name: "テスト".into(),
        speed: 80,
        stamina: 70,
        condition: Condition::Normal,
        sex: Sex::Male,
        age: 4,
        dirt,
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
    assert_eq!(Game::run_speed(&mk(2), Surface::Turf), 80);
    assert_eq!(Game::run_speed(&mk(-2), Surface::Turf), 80);
    assert!(Game::run_speed(&mk(2), Surface::Dirt) > 80);
    assert!(Game::run_speed(&mk(-2), Surface::Dirt) < 80);
    assert_eq!(Game::run_speed(&mk(0), Surface::Dirt), 80);
}

#[test]
fn the_free_race_is_a_twelve_horse_2400m_turf_race_with_a_few_of_mine() {
    let g = Game::new();
    let card = g.race_card(None).unwrap();
    assert!(card.race.is_none());
    assert_eq!(
        (card.distance, card.surface),
        (2400, crate::calendar::Surface::Turf)
    );
    assert_eq!(card.entrants.len(), entries::FULL_FIELD);
    assert_eq!(
        card.entrants.iter().filter(|e| e.mine).count(),
        MAX_OWN_ENTRIES
    );
    assert!(card.entrants.iter().any(|e| !e.mine));
}

#[test]
fn a_calendar_race_card_describes_the_race_and_fills_the_field() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    assert_eq!(card.race.as_ref().unwrap().name, "国際友好ステークス");
    assert_eq!(
        (card.distance, card.surface),
        (2400, crate::calendar::Surface::Turf)
    );
    assert_eq!(card.entrants.len(), entries::FULL_FIELD);
    let mut names = names_of(&card);
    names.sort();
    names.dedup();
    assert_eq!(names.len(), entries::FULL_FIELD, "names must be unique");
    assert!(card
        .entrants
        .iter()
        .all(|e| e.odds >= MIN_ODDS && e.place_odds >= 1.0));
}

#[test]
fn only_eligible_stable_horses_enter_and_are_marked_mine() {
    let mut g = Game::new();
    g.week = 20;
    // 4yo+ fillies and mares only: just the 4-year-old mare qualifies
    let race = calendar_race("薫風ヴィーナスマイル");
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    let mine: Vec<&str> = card
        .entrants
        .iter()
        .filter(|e| e.mine)
        .map(|e| e.name.as_str())
        .collect();
    assert_eq!(mine, vec!["ヘキクウブレイブ"]);
    assert_eq!(card.entrants.len(), entries::FULL_FIELD);
    assert!(card
        .entrants
        .iter()
        .all(|e| race.eligibility.allows(e.age, e.sex == Sex::Female)));

    // a 3yo+ open race could take the whole stable, but only the limit runs
    g.week = 47;
    let open = calendar_race("国際友好ステークス");
    let card = g.race_card(Some(&key_of(&open))).unwrap();
    assert_eq!(
        card.entrants.iter().filter(|e| e.mine).count(),
        MAX_OWN_ENTRIES
    );
    assert_eq!(card.entrants.len(), entries::FULL_FIELD);
    assert!(card.my_horses.iter().all(|h| h.eligible));
}

#[test]
fn a_race_without_eligible_stable_horses_is_all_rivals() {
    let mut g = Game::new();
    g.week = 51;
    let race = calendar_race("ジュニアマイルステークス"); // 2yo only
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    assert!(card.entrants.iter().all(|e| !e.mine && e.age == 2));
    assert_eq!(card.entrants.len(), entries::FULL_FIELD);
}

#[test]
fn rivals_are_stable_for_a_race_but_differ_between_races() {
    let mut g = Game::new();
    g.week = 20;
    let (a, b) = (
        calendar_race("初夏スプリントカップ"),
        calendar_race("若葉の森ステークス"),
    );
    let card_a = g.race_card(Some(&key_of(&a))).unwrap();
    assert_eq!(card_a, g.race_card(Some(&key_of(&a))).unwrap());
    let card_b = g.race_card(Some(&key_of(&b))).unwrap();
    assert_ne!(names_of(&card_a), names_of(&card_b));
}

#[test]
fn a_race_that_is_not_on_this_week_is_rejected() {
    let g = Game::new(); // week 1
    let race = calendar_race("薫風ヴィーナスマイル"); // week 20
    assert_eq!(
        g.race_card(Some(&key_of(&race))),
        Err(GameError::NoSuchRace)
    );
}

#[test]
fn card_odds_are_the_odds_that_get_paid() {
    let mut won_checked = false;
    for seed in 0..80u64 {
        let mut g = Game::new();
        g.week = 47;
        let race = calendar_race("国際友好ステークス");
        let key = key_of(&race);
        let card = g.race_card(Some(&key)).unwrap();
        // bet on the favourite, so a win is likely within a few dozen tries
        let pick = card
            .entrants
            .iter()
            .min_by(|a, b| a.odds.total_cmp(&b.odds))
            .unwrap()
            .clone();
        let out = g
            .play_bet_in(Some(&key), &pick.name, 100, seed, BetKind::Win)
            .unwrap();
        if out.won {
            assert_eq!(out.payout, (100.0 * pick.odds).round() as i64);
            won_checked = true;
        }
    }
    assert!(won_checked, "never saw a winning bet");
}

#[test]
fn playing_a_calendar_race_uses_its_distance_and_advances_the_week() {
    let mut g = Game::new();
    g.week = 22;
    let race = calendar_race("王冠ステークス"); // 2400m, 3yo
    let key = key_of(&race);
    let card = g.race_card(Some(&key)).unwrap();
    let pick = card.entrants[0].name.clone();
    let out = g
        .play_bet_in(Some(&key), &pick, 10, 3, BetKind::Place)
        .unwrap();
    assert_eq!(out.distance, 2400.0);
    assert_eq!(out.runners.len(), card.entrants.len());
    assert_eq!(out.results.len(), card.entrants.len());
    assert!(out.frames.last().unwrap().iter().all(|p| *p == 2400.0));
    assert_eq!(out.race_name.as_deref(), Some("王冠ステークス"));
    assert_eq!(g.week, 23);
}

#[test]
fn a_sprint_is_decided_over_its_own_distance() {
    let mut g = Game::new();
    g.week = 13;
    let race = calendar_race("春雷スプリント"); // 1200m
    let key = key_of(&race);
    let card = g.race_card(Some(&key)).unwrap();
    let out = g
        .play_bet_in(Some(&key), &card.entrants[0].name, 10, 5, BetKind::Win)
        .unwrap();
    assert_eq!(out.distance, 1200.0);
    assert!(out.frames.last().unwrap().iter().all(|p| *p == 1200.0));
    assert!(
        out.frames.len() < 100,
        "a 1200m sprint should be short, got {}",
        out.frames.len()
    );
}

#[test]
fn history_records_the_race_name() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let pick = g.race_card(Some(&key)).unwrap().entrants[0].name.clone();
    g.play_bet_in(Some(&key), &pick, 10, 1, BetKind::Win)
        .unwrap();
    assert_eq!(
        g.state().history.last().unwrap().race.as_deref(),
        Some("国際友好ステークス")
    );
    // free-race rounds carry no race name
    g.play_round(&g.field[0].name.clone(), 10, 2).unwrap();
    assert_eq!(g.state().history.last().unwrap().race, None);
}

#[test]
fn cannot_bet_on_a_horse_that_is_not_running() {
    let mut g = Game::new();
    g.week = 51;
    let race = calendar_race("ジュニアマイルステークス"); // 2yo only
    let stable_horse = g.field[0].name.clone();
    let before = (g.money, g.week);
    assert_eq!(
        g.play_bet_in(Some(&key_of(&race)), &stable_horse, 10, 1, BetKind::Win),
        Err(GameError::UnknownHorse)
    );
    assert_eq!((g.money, g.week), before);
}

#[test]
fn a_rejected_calendar_bet_changes_nothing() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let pick = g.race_card(Some(&key)).unwrap().entrants[0].name.clone();
    let before = g.state();
    assert_eq!(
        g.play_bet_in(Some(&key), &pick, 0, 1, BetKind::Win),
        Err(GameError::InvalidAmount)
    );
    assert_eq!(
        g.play_bet_in(Some(&key), &pick, 99999, 1, BetKind::Win),
        Err(GameError::InsufficientFunds)
    );
    assert_eq!(g.state(), before);
}

#[test]
fn the_free_race_does_not_advance_the_week() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.play_round(&name, 10, 1).unwrap();
    assert_eq!((g.week, g.year), (1, 1));
}

#[test]
fn dirt_races_favour_dirt_specialists() {
    use crate::calendar::Surface;
    // two identical horses except for dirt aptitude, racing on dirt
    let mk = |name: &str, dirt: i8| crate::rivals::Entrant {
        jump: 0,
        name: name.into(),
        speed: 78,
        stamina: 78,
        condition: Condition::Normal,
        sex: Sex::Male,
        age: 5,
        dirt,
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
    let spec = RaceSpec {
        race: None,
        distance: 1800,
        surface: Surface::Dirt,
        entrants: vec![mk("ダートオー", 2), mk("シバオー", -2)],
        track: TrackCondition::Good,
    };
    let wins = (0..100u64)
        .filter(|seed| Game::run_race(&spec, *seed).1[0].name == "ダートオー")
        .count();
    assert!(wins >= 80, "dirt specialist won only {wins}/100 on dirt");
}

#[test]
fn the_frontend_race_key_and_card_json_shapes_match() {
    // exactly what the UI sends for a race ...
    let key: RaceKey =
        serde_json::from_str(r#"{"day":"sat","venue":"kyoto","race_no":11}"#).unwrap();
    assert_eq!(
        key,
        RaceKey {
            day: Day::Sat,
            venue: Venue::Kyoto,
            race_no: 11
        }
    );
    // ... and what it expects back
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    let v = serde_json::to_value(&card).unwrap();
    assert_eq!(v["race"]["grade"], "g1");
    assert_eq!(v["surface"], "turf");
    let e = &v["entrants"][0];
    for field in [
        "name",
        "speed",
        "stamina",
        "condition",
        "sex",
        "age",
        "dirt",
        "mine",
        "odds",
        "place_odds",
    ] {
        assert!(e.get(field).is_some(), "missing {field}");
    }
    assert!(["male", "female"].contains(&e["sex"].as_str().unwrap()));
    let free = serde_json::to_value(g.race_card(None).unwrap()).unwrap();
    assert!(free["race"].is_null());
    let out = serde_json::to_value(
        g.play_bet_in(
            None,
            &g.race_card(None).unwrap().entrants[0].name,
            10,
            1,
            BetKind::Win,
        )
        .unwrap(),
    )
    .unwrap();
    assert!(out["race_name"].is_null());
    assert!(out["runners"].is_array());
}

// ---- local courses and the races open to all (地方・交流重賞) ----

fn local_races() -> Vec<CalendarRace> {
    calendar::shared()
        .races
        .iter()
        .filter(|r| r.venue.is_local())
        .cloned()
        .collect()
}

#[test]
fn the_local_tracks_run_stakes_races_on_dirt_at_each_course() {
    let races = local_races();
    assert!(races.len() >= 8, "{}", races.len());
    for r in &races {
        assert_eq!(r.surface, Surface::Dirt, "{}", r.name);
        assert!(
            matches!(r.grade, Grade::G1 | Grade::G2 | Grade::G3),
            "{}",
            r.name
        );
        assert!(
            calendar::course_distances(r.venue, Surface::Dirt).contains(&r.distance),
            "{}",
            r.name
        );
    }
    for v in [Venue::Oi, Venue::Kawasaki, Venue::Funabashi, Venue::Sonoda] {
        assert!(races.iter().any(|r| r.venue == v), "{v:?}");
    }
}

#[test]
fn the_ten_national_courses_are_not_local() {
    for v in crate::calendar::ALL_VENUES {
        assert!(!v.is_local(), "{v:?}");
    }
}

#[test]
fn a_local_race_can_be_entered_and_run() {
    let r = local_races()
        .into_iter()
        .find(|r| r.grade == Grade::G3)
        .unwrap();
    let mut g = Game::new();
    g.week = r.week;
    let key = RaceKey::of(&r);
    let i = (0..g.field.len())
        .find(|&i| {
            r.eligibility
                .allows(g.field[i].age, g.field[i].sex == Sex::Female)
        })
        .unwrap();
    g.field[i].record = RaceRecord { starts: 5, wins: 3 };
    let name = g.field[i].name.clone();
    g.set_entries(Some(&key), std::slice::from_ref(&name))
        .unwrap();
    let card = g.race_card(Some(&key)).unwrap();
    assert!(card.entrants.iter().any(|e| e.name == name));
    assert!(card.course.lap > 0.0 && card.course.straight > 0.0);
    assert!(g.watch_race(Some(&key), 1).is_ok());
}

#[test]
fn a_local_course_starts_no_more_than_its_gate_and_at_least_a_race() {
    for r in local_races() {
        let mut g = Game::new();
        g.entry_rules = true;
        g.week = r.week;
        let card = g.race_card(Some(&RaceKey::of(&r))).unwrap();
        let gate = entries::full_gate(r.venue);
        assert!(
            card.entrants.len() <= gate,
            "{}: {}",
            r.name,
            card.entrants.len()
        );
        assert!(card.entrants.len() >= entries::MIN_FIELD, "{}", r.name);
        assert_eq!(card.gate, gate);
    }
}

#[test]
fn local_races_are_not_in_a_meeting_of_their_own() {
    assert!(calendar::shared()
        .meetings
        .iter()
        .all(|m| !m.venue.is_local()));
}
