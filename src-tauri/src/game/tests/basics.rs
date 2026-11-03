//! The basics: training, saving, history, condition and odds.

use super::*;

#[test]
fn training_raises_stat_and_costs_money() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let before = g.field[0].speed;
    let cost = g.train_cost(&name).unwrap();
    g.workout(&name, Workout::Wood).unwrap();
    assert_eq!(g.field[0].speed, before + TRAIN_GAIN);
    assert_eq!(g.money, START_MONEY - cost);
}

#[test]
fn training_cost_rises_with_each_session() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let c1 = g.train_cost(&name).unwrap();
    g.workout(&name, Workout::Pool).unwrap();
    assert!(g.train_cost(&name).unwrap() > c1);
}

#[test]
fn training_needs_money_and_a_known_horse() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.money = 1;
    assert_eq!(
        g.workout(&name, Workout::Wood),
        Err(GameError::InsufficientFunds)
    );
    assert_eq!(
        g.workout("Nobody", Workout::Wood),
        Err(GameError::UnknownHorse)
    );
    assert_eq!(g.money, 1);
}

#[test]
fn stats_are_capped_at_max() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.field[0].speed = MAX_STAT - 1;
    g.workout(&name, Workout::Wood).unwrap();
    assert_eq!(g.field[0].speed, MAX_STAT);
    assert_eq!(g.workout(&name, Workout::Wood), Err(GameError::MaxedOut));
}

#[test]
fn training_is_blocked_when_game_over() {
    let mut g = Game::new();
    g.money = 0;
    let name = g.field[0].name.clone();
    assert_eq!(g.workout(&name, Workout::Wood), Err(GameError::GameOver));
}

#[test]
fn reset_clears_training() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let c = g.train_cost(&name).unwrap();
    g.workout(&name, Workout::Wood).unwrap();
    g.reset();
    assert_eq!(g.train_cost(&name).unwrap(), c);
}

#[test]
fn json_round_trip_preserves_progress() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.workout(&name, Workout::Wood).unwrap();
    g.play_round(&name, 100, 4).unwrap();
    let restored = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(restored.state(), g.state());
    assert_eq!(restored.train_cost(&name), g.train_cost(&name));
}

#[test]
fn corrupt_or_inconsistent_save_is_rejected() {
    assert!(Game::from_json("not json").is_err());
    let mut v = old_save(&Game::new());
    v["trainings"] = serde_json::json!([1]);
    assert!(Game::from_json(&v.to_string()).is_err());
}

#[test]
fn file_save_and_load() {
    let dir = std::env::temp_dir().join(format!("derby_test_{}", std::process::id()));
    let path = dir.join("nested").join("save.json");
    let mut g = Game::new();
    g.money = 777;
    g.save_to(&path).unwrap();
    assert_eq!(Game::load_from(&path).unwrap().money, 777);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn loading_missing_file_fails() {
    let path = std::env::temp_dir().join("derby_definitely_missing.json");
    assert!(Game::load_from(&path).is_err());
}

#[test]
fn rounds_are_recorded_in_history() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let out = g.play_round(&name, 100, 4).unwrap();
    let h = &g.state().history;
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].round, 1);
    assert_eq!(h[0].horse, name);
    assert_eq!(h[0].stake, 100);
    assert_eq!(h[0].won, out.won);
    assert_eq!(h[0].payout, out.payout);
    assert_eq!(h[0].winner, out.results[0].name);
}

#[test]
fn rejected_bets_are_not_recorded() {
    let mut g = Game::new();
    let _ = g.play_round("Nobody", 100, 1);
    let name = g.field[0].name.clone();
    let _ = g.play_round(&name, 5000, 1);
    assert!(g.state().history.is_empty());
}

#[test]
fn history_keeps_only_the_latest_entries() {
    let mut g = Game::new();
    g.money = 1_000_000;
    let name = g.field[0].name.clone();
    for seed in 0..(HISTORY_LIMIT as u64 + 5) {
        g.play_round(&name, 1, seed).unwrap();
    }
    let h = g.state().history;
    assert_eq!(h.len(), HISTORY_LIMIT);
    assert_eq!(h.last().unwrap().round, HISTORY_LIMIT as u32 + 5);
}

#[test]
fn history_survives_save_and_reset_clears_it() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.play_round(&name, 10, 1).unwrap();
    let restored = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(restored.state().history.len(), 1);
    g.reset();
    assert!(g.state().history.is_empty());
}

#[test]
fn old_saves_without_history_still_load() {
    let mut v = old_save(&Game::new());
    v.as_object_mut().unwrap().remove("history");
    let g = Game::from_json(&v.to_string()).unwrap();
    assert!(g.state().history.is_empty());
}

#[test]
fn condition_multipliers_are_ordered() {
    assert!(Condition::Great.multiplier() > Condition::Good.multiplier());
    assert!(Condition::Good.multiplier() > Condition::Normal.multiplier());
    assert!(Condition::Normal.multiplier() > Condition::Poor.multiplier());
    assert_eq!(Condition::Normal.multiplier(), 1.0);
}

#[test]
fn every_horse_has_a_condition_and_seed_decides_it() {
    let a = Game::with_seed(11);
    let b = Game::with_seed(11);
    assert_eq!(a.all(|h| h.condition), b.all(|h| h.condition));
    let differs =
        (0..20u64).any(|s| Game::with_seed(s).all(|h| h.condition) != a.all(|h| h.condition));
    assert!(differs);
}

#[test]
fn conditions_are_rerolled_after_each_round() {
    let mut g = Game::with_seed(5);
    let name = g.field[0].name.clone();
    let mut seen = vec![g.all(|h| h.condition)];
    for s in 0..10 {
        g.play_round(&name, 1, s).unwrap();
        seen.push(g.all(|h| h.condition));
    }
    assert!(seen.windows(2).any(|w| w[0] != w[1]));
}

#[test]
fn good_condition_lowers_odds_and_poor_raises_them() {
    let mut g = Game::new();
    let name = g.field[2].name.clone();
    g.field[2].condition = Condition::Normal;
    let normal = g.odds(&name).unwrap();
    g.field[2].condition = Condition::Great;
    let great = g.odds(&name).unwrap();
    g.field[2].condition = Condition::Poor;
    let poor = g.odds(&name).unwrap();
    assert!(great < normal && normal < poor);
}

#[test]
fn condition_affects_race_results() {
    let wins = (0..200u64)
        .filter(|seed| {
            let mut g = Game::new();
            // no family records: no growth or distance bias in this test,
            // and a trainer with no distance of their own
            let horse = |name, condition| StableHorse {
                condition,
                ..StableHorse::starter(name, 70, 70, trainer::STANDARD)
            };
            g.field = vec![horse("A", Condition::Great), horse("B", Condition::Poor)];
            let out = g.play_round("A", 1, *seed).unwrap();
            out.results[0].name == "A"
        })
        .count();
    assert!(wins >= 120, "great-condition horse won only {wins}/200");
}

#[test]
fn conditions_survive_save_and_old_saves_default_to_normal() {
    let g = Game::with_seed(9);
    let restored = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(restored.all(|h| h.condition), g.all(|h| h.condition));
    let mut v: serde_json::Value = old_save(&g);
    v.as_object_mut().unwrap().remove("conditions");
    v.as_object_mut().unwrap().remove("rng");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert!(old
        .all(|h| h.condition)
        .iter()
        .all(|c| *c == Condition::Normal));
}

#[test]
fn reset_rolls_fresh_conditions_instead_of_repeating() {
    let mut g = Game::with_seed(3);
    let first = g.all(|h| h.condition);
    let mut changed = false;
    for _ in 0..10 {
        g.reset();
        if g.all(|h| h.condition) != first {
            changed = true;
        }
    }
    assert!(changed);
}

#[test]
fn state_exposes_conditions() {
    let g = Game::with_seed(1);
    let st = g.state();
    assert_eq!(st.field[0].condition, g.field[0].condition);
}

#[test]
fn field_names_follow_the_naming_rule_and_are_unique() {
    let g = Game::new();
    assert!(g.field.iter().all(|h| crate::horse::is_valid_name(&h.name)));
    let mut names: Vec<&str> = g.field.iter().map(|h| h.name.as_str()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), g.field.len());
}

#[test]
fn place_odds_never_exceed_win_odds_and_pay_at_least_the_stake() {
    let g = Game::new();
    for h in &g.field {
        let win = g.odds(&h.name).unwrap();
        let place = g.place_odds(&h.name).unwrap();
        assert!(place <= win, "{}: place {place} win {win}", h.name);
        assert!(place >= 1.0);
    }
    assert_eq!(g.place_odds("Nobody"), None);
}

#[test]
fn odds_match_simulated_chances() {
    // Over many fresh races, a horse's win rate should be near 85% / odds
    // (the payout rate), i.e. the odds are fair apart from the house edge.
    let g = Game::new();
    let i = 2;
    let name = g.field[i].name.clone();
    let odds = g.odds(&name).unwrap();
    let wins = (1000..1600u64)
        .filter(|seed| {
            let mut g = Game::new();
            g.play_round(&name, 1, *seed).unwrap().won
        })
        .count() as f64
        / 600.0;
    let implied = PAYOUT_RATE / odds;
    assert!(
        (wins - implied).abs() < 0.08,
        "win rate {wins:.2} vs implied {implied:.2}"
    );
}

#[test]
fn weaker_horses_get_higher_place_odds() {
    let mut g = Game::new();
    for h in &mut g.field {
        h.condition = Condition::Normal;
    }
    let strongest = (0..g.field.len())
        .max_by_key(|&i| g.effective_speed(i))
        .unwrap();
    let weakest = (0..g.field.len())
        .min_by_key(|&i| g.effective_speed(i))
        .unwrap();
    let s = g.place_odds(&g.field[strongest].name).unwrap();
    let w = g.place_odds(&g.field[weakest].name).unwrap();
    assert!(w > s);
}

#[test]
fn place_bet_wins_when_horse_finishes_in_top_three() {
    let mut checked_win_but_not_first = false;
    for seed in 0..60u64 {
        let mut g = Game::new();
        let name = g.field[3].name.clone();
        let place_odds = g.place_odds(&name).unwrap();
        let out = g.play_bet(&name, 100, seed, BetKind::Place).unwrap();
        let rank = out.results.iter().position(|r| r.name == name).unwrap() + 1;
        assert_eq!(out.won, rank <= 3, "seed {seed} rank {rank}");
        if out.won {
            assert_eq!(out.payout, (100.0 * place_odds).round() as i64);
            assert_eq!(g.money as i64, START_MONEY as i64 - 100 + out.payout);
            if rank > 1 {
                checked_win_but_not_first = true;
            }
        } else {
            assert_eq!(out.payout, 0);
            assert_eq!(g.money, START_MONEY - 100);
        }
    }
    assert!(
        checked_win_but_not_first,
        "never saw a 2nd/3rd place payout"
    );
}

#[test]
fn win_bet_still_requires_first_place() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let out = g.play_bet(&name, 10, 3, BetKind::Win).unwrap();
    assert_eq!(out.won, out.results[0].name == name);
}

#[test]
fn history_records_the_bet_kind() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.play_bet(&name, 10, 1, BetKind::Place).unwrap();
    g.play_round(&name, 10, 2).unwrap();
    let h = g.state().history;
    assert_eq!(h[0].kind, BetKind::Place);
    assert_eq!(h[1].kind, BetKind::Win);
}

#[test]
fn old_history_entries_default_to_win() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.play_round(&name, 10, 1).unwrap();
    let mut v: serde_json::Value = old_save(&g);
    v["history"][0].as_object_mut().unwrap().remove("kind");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert_eq!(old.state().history[0].kind, BetKind::Win);
}

#[test]
fn the_state_leaves_the_odds_to_the_race_card() {
    // odds depend on the race and its field, so they are on the card, not on each horse
    let v = serde_json::to_value(Game::new().state()).unwrap();
    assert!(v["field"][0].get("odds").is_none());
    assert!(v["field"][0].get("place_odds").is_none());
}

#[test]
fn state_exposes_a_pedigree_for_every_horse() {
    let g = Game::new();
    let st = g.state();
    assert!(st.field.iter().all(|h| h.pedigree.is_some()));
}
