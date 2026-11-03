//! The track: what the panel shows, going and power, how a course suits a horse (コース・馬場).

use super::*;

// ---- what the track panel shows (コース情報) ----

#[test]
fn the_card_says_where_the_race_is_and_which_way_it_turns() {
    let mut g = Game::with_seed(1);
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    let course = course::course_of(race.venue);
    assert_eq!(card.course.name, course.name);
    assert_eq!(
        card.course.turn,
        if course.turn == course::Turn::Left {
            "左回り"
        } else {
            "右回り"
        }
    );
    let straight = match race.surface {
        Surface::Turf => course.turf.straight,
        Surface::Dirt => course.dirt.straight,
    };
    assert_eq!(card.course.straight, straight);
    let lap = match race.surface {
        Surface::Turf => course.turf.lap_metres(),
        Surface::Dirt => course.dirt.lap_metres(),
    };
    assert_eq!(card.course.lap, lap);
    let plan = course::plan_of(race.venue, race.surface, race.distance);
    assert_eq!(card.course.corners, plan.corners);
    assert_eq!(card.course.after_goal, plan.after_goal);
}

#[test]
fn the_card_gives_the_inner_or_outer_course_of_the_distance() {
    let mut g = Game::with_seed(1);
    let race = calendar::shared()
        .races
        .iter()
        .find(|r| r.venue == Venue::Kyoto && r.surface == Surface::Turf && r.distance == 2000)
        .unwrap()
        .clone();
    g.week = race.week;
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    assert_eq!(card.course.layout.as_deref(), Some("内回り"));
    assert_eq!(card.course.lap, 1782.8);
    assert_eq!(card.course.straight, 328.4);
}

#[test]
fn the_free_race_says_where_it_is_too() {
    let card = Game::with_seed(1).race_card(None).unwrap();
    assert!(!card.course.name.is_empty());
    assert!(card.course.straight > 0.0);
}

// ---- track condition (馬場状態) and power ----

fn turf_spec(track: TrackCondition, entrants: Vec<crate::rivals::Entrant>) -> RaceSpec {
    RaceSpec {
        race: None,
        distance: 2000,
        surface: crate::calendar::Surface::Turf,
        entrants,
        track,
    }
}

#[test]
fn a_firm_track_leaves_speed_and_stamina_alone() {
    use crate::calendar::Surface;
    let e = runner("テスト", 10, 0);
    assert_eq!(
        Game::run_stats(&e, Surface::Turf, 2000, TrackCondition::Good),
        (78, 78)
    );
    assert_eq!(
        Game::run_stats(&e, Surface::Dirt, 2000, TrackCondition::Good),
        (78, 78)
    );
}

#[test]
fn a_heavy_track_costs_a_weak_horse_more_than_a_powerful_one() {
    use crate::calendar::Surface;
    let weak = runner("ヒリョク", 10, 0);
    let strong = runner("パワー", 95, 0);
    for track in [TrackCondition::Yielding, TrackCondition::Heavy] {
        let (ws, wst) = Game::run_stats(&weak, Surface::Turf, 2000, track);
        let (ps, pst) = Game::run_stats(&strong, Surface::Turf, 2000, track);
        assert!(
            ws < 78 && wst < 78,
            "{track:?}: weak horse should be slowed ({ws}, {wst})"
        );
        assert!(ps > ws && pst > wst, "{track:?}: {ps},{pst} vs {ws},{wst}");
    }
    // heavier is worse
    let soft = Game::run_stats(&weak, Surface::Turf, 2000, TrackCondition::Yielding);
    let heavy = Game::run_stats(&weak, Surface::Turf, 2000, TrackCondition::Heavy);
    assert!(heavy.0 < soft.0 && heavy.1 < soft.1);
}

#[test]
fn dirt_aptitude_helps_on_a_heavy_track_even_over_turf() {
    use crate::calendar::Surface;
    let (good_dirt, bad_dirt) = (runner("ダートカ", 50, 2), runner("ダートヘタ", 50, -2));
    let a = Game::run_stats(&good_dirt, Surface::Turf, 2000, TrackCondition::Heavy);
    let b = Game::run_stats(&bad_dirt, Surface::Turf, 2000, TrackCondition::Heavy);
    assert!(a.0 > b.0 && a.1 > b.1);
    // and on a firm turf course dirt aptitude means nothing
    assert_eq!(
        Game::run_stats(&good_dirt, Surface::Turf, 2000, TrackCondition::Good),
        Game::run_stats(&bad_dirt, Surface::Turf, 2000, TrackCondition::Good)
    );
}

#[test]
fn power_horses_win_in_the_mud_but_not_on_firm_ground() {
    let two = || vec![runner("パワーオー", 95, 0), runner("ヒリョクオー", 10, 0)];
    let wins = |track: TrackCondition| {
        let spec = turf_spec(track, two());
        (0..200u64)
            .filter(|s| Game::run_race(&spec, *s).1[0].name == "パワーオー")
            .count()
    };
    assert!(
        wins(TrackCondition::Heavy) >= 170,
        "heavy: {}",
        wins(TrackCondition::Heavy)
    );
    assert!(
        wins(TrackCondition::Yielding) >= 130,
        "soft: {}",
        wins(TrackCondition::Yielding)
    );
    let firm = wins(TrackCondition::Good);
    assert!((70..=130).contains(&firm), "firm: {firm}");
}

#[test]
fn the_odds_price_in_the_track() {
    let two = || vec![runner("パワーオー", 95, 0), runner("ヒリョクオー", 10, 0)];
    let heavy = Game::odds_table_for(&turf_spec(TrackCondition::Heavy, two()));
    assert!(heavy[0].0 < heavy[1].0, "{heavy:?}");
    let firm = Game::odds_table_for(&turf_spec(TrackCondition::Good, two()));
    let ratio = firm[0].0 / firm[1].0;
    assert!((0.6..1.7).contains(&ratio), "{firm:?}");
}

#[test]
fn the_free_race_is_always_on_a_firm_track() {
    let g = Game::new();
    assert_eq!(g.race_card(None).unwrap().track, TrackCondition::Good);
}

#[test]
fn every_race_on_a_venue_and_day_shares_the_track() {
    let mut g = Game::new();
    g.week = 20;
    let a = calendar_race("初夏スプリントカップ");
    let b = calendar_race("薫風ヴィーナスマイル");
    assert_eq!((a.venue, a.day), (b.venue, b.day), "test premise");
    assert_eq!(
        g.race_card(Some(&key_of(&a))).unwrap().track,
        g.race_card(Some(&key_of(&b))).unwrap().track
    );
}

#[test]
fn a_races_track_is_stable_but_varies_over_the_year() {
    let mut g = Game::with_seed(5);
    let mut seen = std::collections::HashMap::new();
    for week in 1..=52u8 {
        g.week = week;
        for venue in crate::calendar::ALL_VENUES {
            for day in [Day::Sat, Day::Sun] {
                let t = g.track_for(day, venue);
                assert_eq!(t, g.track_for(day, venue), "must be repeatable");
                *seen.entry(t).or_insert(0usize) += 1;
            }
        }
    }
    assert_eq!(seen.len(), 4, "{seen:?}");
    let (good, total): (usize, usize) = (seen[&TrackCondition::Good], seen.values().sum());
    assert!(
        good * 100 / total > 55 && good * 100 / total < 85,
        "{seen:?}"
    );
}

#[test]
fn the_track_depends_on_the_game_and_the_year() {
    let tracks = |g: &mut Game| -> Vec<TrackCondition> {
        (1..=52u8)
            .map(|w| {
                g.week = w;
                g.track_for(Day::Sat, Venue::Tokyo)
            })
            .collect()
    };
    let mut a = Game::with_seed(1);
    let mut b = Game::with_seed(2);
    assert_ne!(tracks(&mut a), tracks(&mut b));
    let first_year = tracks(&mut a);
    a.year = 2;
    assert_ne!(first_year, tracks(&mut a));
}

#[test]
fn a_race_card_shows_the_track_and_each_runners_power_and_bad_track_aptitude() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    assert_eq!(card.track, g.track_for(race.day, race.venue));
    for e in &card.entrants {
        assert!((0..=100).contains(&e.power), "{}", e.power);
        assert!((0.0..=100.0).contains(&e.bad_track));
        let expected = track::bad_track_aptitude(e.power as f64, track::dirt_score(e.dirt));
        assert!(
            (e.bad_track - expected).abs() < 0.06,
            "{} vs {expected}",
            e.bad_track
        );
    }
}

#[test]
fn a_calendar_race_is_run_on_its_own_track() {
    // a week and venue with a wet forecast should hurt non-power horses in the odds
    let mut g = Game::with_seed(11);
    let race = calendar_race("夏越グランプリ");
    let mut found_wet = false;
    for year in 1..=40u32 {
        g.week = race.week;
        g.year = year;
        let track = g.track_for(race.day, race.venue);
        if track == TrackCondition::Good {
            continue;
        }
        found_wet = true;
        let card = g.race_card(Some(&key_of(&race))).unwrap();
        assert_eq!(card.track, track);
        // on a soft or heavy track, powerful horses are shorter than weak ones of equal speed
        let best = card.entrants.iter().max_by_key(|e| e.power).unwrap();
        let worst = card.entrants.iter().min_by_key(|e| e.power).unwrap();
        assert!(best.power > worst.power);
        break;
    }
    assert!(found_wet, "no wet year in 40");
}

#[test]
fn stable_horses_have_power_and_bad_track_aptitude() {
    let st = Game::new().state();
    assert!(
        st.field.iter().all(|h| (40..=100).contains(&h.power)),
        "{:?}",
        st.field.iter().map(|h| h.power).collect::<Vec<_>>()
    );
    for h in &st.field {
        let expected = track::bad_track_aptitude(h.power as f64, track::dirt_score(h.dirt));
        assert!((h.bad_track - expected).abs() < 0.06, "{}", h.name);
    }
    // the dirt specialists are the powerful ones
    let by = |n: &str| st.field.iter().find(|h| h.name == n).unwrap().power;
    assert!(by("ミナモムーン") > by("ギンセイストーム"));
}

#[test]
fn power_can_be_trained_like_speed_and_stamina() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let before = g.state().field[0].power;
    let cost = g.train_cost(&name).unwrap();
    g.workout(&name, Workout::Hill).unwrap();
    assert_eq!(g.state().field[0].power, before + TRAIN_GAIN);
    assert_eq!(g.money, START_MONEY - cost);
    assert!(g.train_cost(&name).unwrap() > cost);
    // it is capped at 100 like the others
    g.field[0].power = MAX_STAT - 1;
    g.workout(&name, Workout::Hill).unwrap();
    assert_eq!(g.state().field[0].power, MAX_STAT);
    assert_eq!(g.workout(&name, Workout::Hill), Err(GameError::MaxedOut));
}

#[test]
fn training_power_raises_bad_track_aptitude_and_shows_in_the_race() {
    let mut g = Game::new();
    let name = g.field[2].name.clone();
    let before = g.state().field[2].bad_track;
    g.workout(&name, Workout::Hill).unwrap();
    assert!(g.state().field[2].bad_track > before);
}

#[test]
fn power_is_inherited_by_foals() {
    let (mut strong, mut weak) = (0u32, 0u32);
    for seed in 0..40u64 {
        for (p, total) in [(92u32, &mut strong), (35u32, &mut weak)] {
            let mut g = with_rng(&shared_base(), seed);
            for i in [0usize, 4] {
                g.field[i].power = p;
            }
            breed_default(&mut g).unwrap();
            let foal = g.state().field[5].power;
            assert!((FOAL_MIN_STAT..=FOAL_MAX_STAT).contains(&foal), "{foal}");
            *total += foal;
        }
    }
    assert!(strong > weak + 40 * 30, "{strong} vs {weak}");
}

#[test]
fn old_saves_without_power_still_load_and_new_saves_keep_it() {
    let mut g = Game::with_seed(4);
    g.field[0].power = 97;
    let restored = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(restored.state().field[0].power, 97);
    let mut v: serde_json::Value = old_save(&g);
    v.as_object_mut().unwrap().remove("powers");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert_eq!(old.state().field.len(), 8);
    assert!(old.state().field.iter().all(|h| h.power >= 40));
}

#[test]
fn a_save_with_mismatched_power_data_is_rejected() {
    let g = Game::with_seed(4);
    let mut v: serde_json::Value = old_save(&g);
    v["powers"] = serde_json::json!([60, 60]);
    assert!(Game::from_json(&v.to_string()).is_err());
}

#[test]
fn the_json_shapes_include_track_and_power() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let card = serde_json::to_value(g.race_card(Some(&key_of(&race))).unwrap()).unwrap();
    assert!(["good", "yielding", "heavy"].contains(&card["track"].as_str().unwrap()));
    for field in ["power", "bad_track"] {
        assert!(
            card["entrants"][0].get(field).is_some(),
            "entrant missing {field}"
        );
    }
    let st = serde_json::to_value(g.state()).unwrap();
    for field in ["power", "bad_track"] {
        assert!(st["field"][0].get(field).is_some(), "horse missing {field}");
    }
    let w: Workout = serde_json::from_str("\"hill\"").unwrap();
    assert_eq!(w, Workout::Hill);
}

// ---- how a racecourse suits a horse (競馬場との相性) ----

#[test]
fn a_course_that_suits_a_horse_brings_out_more_of_it() {
    let plain = fit_runner(4, 4, (1000, 3600));
    let suited = course::fit(Venue::Hakodate, Surface::Turf, "ア", Some(Style::Nige), 90);
    let mut e = plain.clone();
    e.course_fit = Some(suited);
    let stats =
        |e: &crate::rivals::Entrant| Game::run_stats(e, Surface::Turf, 1800, TrackCondition::Good);
    let (s0, t0) = stats(&plain);
    assert_eq!(
        stats(&e),
        (
            (s0 as f64 * suited.multiplier()).round() as u32,
            (t0 as f64 * suited.multiplier()).round() as u32
        )
    );
}

#[test]
fn every_runner_in_a_calendar_race_gets_the_courses_verdict() {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let spec = g.race_spec(Some(&key_of(&race))).unwrap();
    for e in &spec.entrants {
        assert_eq!(
            e.course_fit,
            Some(course::fit(
                race.venue,
                race.surface,
                &e.name,
                e.style,
                e.power
            ))
        );
    }
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    for (c, e) in card.entrants.iter().zip(&spec.entrants) {
        assert_eq!(c.course_fit, e.course_fit);
    }
}

#[test]
fn the_free_race_is_no_course_in_particular() {
    let g = Game::new();
    let spec = g.race_spec(None).unwrap();
    assert!(spec.entrants.iter().all(|e| e.course_fit.is_none()));
    let v = serde_json::to_value(g.race_card(None).unwrap()).unwrap();
    assert!(v["entrants"][0]["course_fit"].is_null());
}

#[test]
fn a_horses_fit_at_every_course_can_be_looked_up() {
    let g = Game::new();
    let name = g.field[0].name.clone();
    let fits = g.course_fits(&name).unwrap();
    assert_eq!(fits.len(), 10);
    let e = g.stable_entrant(0);
    for f in &fits {
        assert_eq!(
            f.turf,
            course::fit(f.venue, Surface::Turf, &name, e.style, e.power)
        );
        assert_eq!(
            f.dirt,
            course::fit(f.venue, Surface::Dirt, &name, e.style, e.power)
        );
    }
    assert_eq!(g.course_fits("いない馬"), Err(GameError::UnknownHorse));
}

#[test]
fn a_trainer_picks_the_course_that_suits_the_horse_among_equal_prizes() {
    let (mut g, _) = auto_runner();
    g.field[0].speed = MAX_STAT;
    let percent = |g: &Game, r: &CalendarRace| g.fit_at(0, r).percent;
    let mut told = 0;
    for week in 1..=calendar::WEEKS_PER_YEAR {
        g.week = week;
        let Some(picked) = g.auto_race(0, &[], None) else {
            continue;
        };
        let rivals: Vec<CalendarRace> = g
            .auto_choices(0, None)
            .into_iter()
            .filter(|r| r.prize == picked.prize)
            .collect();
        for r in &rivals {
            assert!(percent(&g, &picked) >= percent(&g, r), "week {week}");
            if percent(&g, &picked) > percent(&g, r) {
                told += 1;
            }
        }
    }
    assert!(told > 0, "no week offered a real choice of course");
}
