//! Growth, distance aptitude, running style and temperament (成長・適性・脚質・気性).

use super::*;

// ---- growth and distance aptitude ----

#[test]
fn a_horse_in_its_prime_and_on_its_distance_runs_at_full_stats() {
    use crate::calendar::Surface;
    let e = fit_runner(5, 5, (1600, 2400));
    let (s, st) = Game::run_stats(&e, Surface::Turf, 2000, TrackCondition::Good);
    assert_eq!((s, st), (78, 78));
}

#[test]
fn a_young_horse_that_peaks_late_runs_below_its_ability() {
    use crate::calendar::Surface;
    let prime = Game::run_stats(
        &fit_runner(6, 6, (1600, 2400)),
        Surface::Turf,
        2000,
        TrackCondition::Good,
    );
    let young = Game::run_stats(
        &fit_runner(3, 6, (1600, 2400)),
        Surface::Turf,
        2000,
        TrackCondition::Good,
    );
    assert!(
        young.0 < prime.0 && young.1 < prime.1,
        "{young:?} vs {prime:?}"
    );
}

#[test]
fn running_beyond_its_distance_costs_stamina_more_than_speed() {
    use crate::calendar::Surface;
    let at_home = Game::run_stats(
        &fit_runner(5, 5, (1600, 2000)),
        Surface::Turf,
        2000,
        TrackCondition::Good,
    );
    let too_far = Game::run_stats(
        &fit_runner(5, 5, (1600, 2000)),
        Surface::Turf,
        3000,
        TrackCondition::Good,
    );
    assert!(too_far.1 < at_home.1 - 10, "{too_far:?} vs {at_home:?}");
    assert!(too_far.0 < at_home.0);
    assert!((at_home.0 - too_far.0) < (at_home.1 - too_far.1));
}

#[test]
fn running_short_of_its_distance_costs_speed_only() {
    use crate::calendar::Surface;
    let at_home = Game::run_stats(
        &fit_runner(5, 5, (2400, 3000)),
        Surface::Turf,
        2400,
        TrackCondition::Good,
    );
    let too_short = Game::run_stats(
        &fit_runner(5, 5, (2400, 3000)),
        Surface::Turf,
        1200,
        TrackCondition::Good,
    );
    assert!(too_short.0 < at_home.0);
    assert_eq!(too_short.1, at_home.1);
}

#[test]
fn a_stayer_beats_a_sprinter_over_a_long_race_and_loses_over_a_short_one() {
    use crate::calendar::Surface;
    let spec = |distance: u32| RaceSpec {
        track: TrackCondition::Good,
        distance,
        surface: Surface::Turf,
        race: None,
        entrants: vec![
            {
                let mut e = fit_runner(5, 5, (1000, 1400));
                e.name = "短距離".into();
                e
            },
            {
                let mut e = fit_runner(5, 5, (2800, 3600));
                e.name = "長距離".into();
                e
            },
        ],
    };
    let wins = |distance: u32, who: &str| {
        let s = spec(distance);
        (0..200u64)
            .filter(|seed| Game::run_race(&s, *seed).1[0].name == who)
            .count()
    };
    assert!(
        wins(3200, "長距離") > 140,
        "stayer over 3200: {}",
        wins(3200, "長距離")
    );
    assert!(
        wins(1200, "短距離") > 100,
        "sprinter over 1200: {}",
        wins(1200, "短距離")
    );
}

#[test]
fn the_odds_reflect_growth_and_distance() {
    use crate::calendar::Surface;
    let mk = |age: u8, peak: u8, apt: (u32, u32), name: &str| {
        let mut e = fit_runner(age, peak, apt);
        e.name = name.into();
        e
    };
    let spec = RaceSpec {
        track: TrackCondition::Good,
        distance: 2400,
        surface: Surface::Turf,
        race: None,
        entrants: vec![
            mk(5, 5, (2000, 2800), "適性"),
            mk(5, 5, (1000, 1400), "短距離馬"),
            mk(2, 7, (2000, 2800), "未成熟"),
            mk(5, 5, (2000, 2800), "適性2"),
        ],
    };
    let chances = Game::chances_n(&spec, 200);
    assert!(chances[0].0 > chances[1].0, "sprinter should be longer");
    assert!(chances[0].0 > chances[2].0, "immature should be longer");
}

#[test]
fn the_stables_horses_use_their_family_records_for_growth_and_distance() {
    let g = Game::new();
    for i in 0..g.field.len() {
        let e = g.stable_entrant(i);
        let r = g.blood_record(g.field[i].blood_id).unwrap();
        assert_eq!(
            (e.peak_age, e.apt_min, e.apt_max),
            (r.peak_age, r.aptitude_min, r.aptitude_max)
        );
    }
}

#[test]
fn the_race_card_shows_growth_and_distance_for_every_runner() {
    let g = Game::new();
    let card = g.race_card(None).unwrap();
    let v = serde_json::to_value(&card).unwrap();
    for e in v["entrants"].as_array().unwrap() {
        assert!(["rising", "peak", "declining"].contains(&e["growth"].as_str().unwrap()));
        assert!(["fit", "too_short", "too_long"].contains(&e["distance_fit"].as_str().unwrap()));
        let f = e["age_form"].as_f64().unwrap();
        assert!((0.75..=1.0).contains(&f));
        assert!(e["apt_min"].as_u64().unwrap() <= e["apt_max"].as_u64().unwrap());
    }
}

// ---- running style (脚質) and temperament (気性) ----

#[test]
fn the_stables_horses_race_with_their_records_style_and_temperament() {
    let g = Game::new();
    for i in 0..g.field.len() {
        let e = g.stable_entrant(i);
        let r = g.blood_record(g.field[i].blood_id).unwrap();
        assert_eq!(e.style, Style::from_label(&r.running_style));
        assert!(e.style.is_some(), "{}", r.running_style);
        assert_eq!(e.temper, r.stats.temper.round() as u32);
    }
}

#[test]
fn a_horse_without_a_record_races_with_no_style_and_a_neutral_temperament() {
    let mut g = Game::new();
    g.field[0].blood_id = 0;
    let e = g.stable_entrant(0);
    assert_eq!((e.style, e.temper), (None, g.field[0].temper));
}

#[test]
fn the_race_card_shows_style_and_temperament_for_every_runner() {
    let g = Game::new();
    let v = serde_json::to_value(g.race_card(None).unwrap()).unwrap();
    for e in v["entrants"].as_array().unwrap() {
        assert!(["nige", "senko", "sashi", "oikomi"].contains(&e["style"].as_str().unwrap()));
        assert!(e["temper"].as_u64().unwrap() <= 100);
    }
}

#[test]
fn running_style_changes_who_wins_once_front_runners_duel() {
    use crate::calendar::Surface;
    let mk = |name: &str, style: Style| {
        let mut e = runner(name, 60, 0);
        e.style = Some(style);
        e
    };
    let spec = RaceSpec {
        track: TrackCondition::Good,
        distance: 2000,
        surface: Surface::Turf,
        race: None,
        entrants: vec![
            mk("逃1", Style::Nige),
            mk("逃2", Style::Nige),
            mk("逃3", Style::Nige),
            mk("差", Style::Sashi),
        ],
    };
    let chances = Game::chances_n(&spec, 400);
    assert!(chances[3].0 > chances[0].0 + 0.1, "{chances:?}");
}

#[test]
fn the_commentary_calls_a_horse_that_pulls_for_its_head() {
    let mut g = Game::new();
    let spec = g.race_spec(None).unwrap();
    let seed = (0..500u64)
        .find(|s| !Game::pulled(&spec, *s).is_empty())
        .expect("someone pulls in some race");
    let pulled = Game::pulled(&spec, seed);
    let rival = spec.entrants.iter().find(|e| !e.mine).unwrap().name.clone();
    let out = g
        .play_combo(None, &[rival], 10, seed, BetKind::Win)
        .unwrap();
    for name in &pulled {
        let line = out
            .commentary
            .iter()
            .find(|l| l.text == format!("{name}が掛かっている、行きたがっています!"))
            .expect("a call for the horse that pulled");
        assert_eq!(line.frame, KAKARI_CALL_FRAME);
    }
    assert!(out.commentary[0].text.starts_with("各馬一斉にスタート"));
    assert!(out.commentary.windows(2).all(|w| w[0].frame <= w[1].frame));
}

#[test]
fn calm_horses_are_never_called_for_pulling() {
    let mut lines = vec![];
    Game::call_pulling(&mut lines, &[], 100);
    assert!(lines.is_empty());
    Game::call_pulling(&mut lines, &["A".to_string()], 2);
    assert_eq!(lines[0].frame, 1); // a very short replay calls it on its last frame
}
