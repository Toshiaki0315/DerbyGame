//! Betting tickets (馬券).

use super::*;

// ---- betting tickets: 単勝 複勝 馬単 馬連 三連単 三連複 ----

const ALL_KINDS: [BetKind; 6] = [
    BetKind::Win,
    BetKind::Place,
    BetKind::Exacta,
    BetKind::Quinella,
    BetKind::Trifecta,
    BetKind::Trio,
];

/// The order the free race finishes in for this seed (a fresh game has the same field).
fn free_order(seed: u64) -> Vec<String> {
    let g = Game::new();
    let spec = g.race_spec(None).unwrap();
    Game::run_race(&spec, seed)
        .1
        .iter()
        .map(|r| r.name.clone())
        .collect()
}

fn bet_free(kind: BetKind, picks: &[String], seed: u64) -> Result<RoundOutcome, GameError> {
    Game::new().play_combo(None, picks, 100, seed, kind)
}

#[test]
fn every_kind_of_ticket_wins_exactly_when_it_matches_the_result() {
    for seed in [1u64, 2, 3, 4, 5] {
        let o = free_order(seed);
        let hit = |kind: BetKind, picks: &[&String]| {
            let p: Vec<String> = picks.iter().map(|s| (*s).clone()).collect();
            bet_free(kind, &p, seed).unwrap().won
        };
        // 単勝
        assert!(hit(BetKind::Win, &[&o[0]]));
        assert!(!hit(BetKind::Win, &[&o[1]]));
        // 複勝
        assert!(hit(BetKind::Place, &[&o[2]]));
        assert!(!hit(BetKind::Place, &[&o[3]]));
        // 馬単
        assert!(hit(BetKind::Exacta, &[&o[0], &o[1]]));
        assert!(!hit(BetKind::Exacta, &[&o[1], &o[0]]));
        assert!(!hit(BetKind::Exacta, &[&o[0], &o[2]]));
        // 馬連
        assert!(hit(BetKind::Quinella, &[&o[1], &o[0]]));
        assert!(hit(BetKind::Quinella, &[&o[0], &o[1]]));
        assert!(!hit(BetKind::Quinella, &[&o[0], &o[2]]));
        // 三連単
        assert!(hit(BetKind::Trifecta, &[&o[0], &o[1], &o[2]]));
        assert!(!hit(BetKind::Trifecta, &[&o[0], &o[2], &o[1]]));
        assert!(!hit(BetKind::Trifecta, &[&o[1], &o[0], &o[2]]));
        // 三連複
        assert!(hit(BetKind::Trio, &[&o[2], &o[0], &o[1]]));
        assert!(hit(BetKind::Trio, &[&o[0], &o[1], &o[2]]));
        assert!(!hit(BetKind::Trio, &[&o[0], &o[1], &o[3]]));
    }
}

#[test]
fn the_result_the_ticket_is_checked_against_is_the_one_shown() {
    let o = free_order(7);
    let out = bet_free(BetKind::Trifecta, &o[..3], 7).unwrap();
    let shown: Vec<String> = out.results.iter().map(|r| r.name.clone()).collect();
    assert_eq!(shown, o);
    assert!(out.won);
}

#[test]
fn a_winning_ticket_pays_the_stake_times_the_quoted_odds_for_every_kind() {
    let o = free_order(11);
    for kind in ALL_KINDS {
        let picks: Vec<String> = o[..bets::picks(kind)].to_vec(); // the real finish: always a hit
        let mut g = Game::new();
        let odds = g.quote_bet(None, kind, &picks).unwrap();
        let out = g.play_combo(None, &picks, 100, 11, kind).unwrap();
        assert!(out.won, "{kind:?}");
        assert_eq!(
            out.payout,
            (100.0 * odds).round() as i64,
            "{kind:?} odds {odds}"
        );
        assert_eq!(
            g.money as i64,
            START_MONEY as i64 - 100 + out.payout,
            "{kind:?}"
        );
    }
}

#[test]
fn a_losing_ticket_costs_the_stake_and_pays_nothing() {
    let o = free_order(12);
    for (kind, picks) in [
        (BetKind::Win, strs(&[&o[1]])),
        (BetKind::Exacta, strs(&[&o[1], &o[0]])),
        (BetKind::Trifecta, strs(&[&o[2], &o[1], &o[0]])),
        (BetKind::Trio, strs(&[&o[0], &o[1], &o[5]])),
    ] {
        let mut g = Game::new();
        let out = g.play_combo(None, &picks, 100, 12, kind).unwrap();
        assert!(!out.won && out.payout == 0, "{kind:?}");
        assert_eq!(g.money, START_MONEY - 100);
    }
}

#[test]
fn single_horse_tickets_are_priced_like_the_cards_odds() {
    let mut g = Game::new();
    let card = g.race_card(None).unwrap();
    let e = &card.entrants[3];
    let name = e.name.clone();
    assert_eq!(
        g.quote_bet(None, BetKind::Win, &[name.clone()]).unwrap(),
        e.odds
    );
    assert_eq!(
        g.quote_bet(None, BetKind::Place, &[name.clone()]).unwrap(),
        e.place_odds
    );
    assert!(g.play_combo(None, &[name], 10, 1, BetKind::Place).is_ok());
}

// ---- quotes

fn top_by_odds(card: &RaceCard, n: usize) -> Vec<String> {
    let mut es = card.entrants.clone();
    es.sort_by(|a, b| a.odds.total_cmp(&b.odds));
    es.iter().take(n).map(|e| e.name.clone()).collect()
}

#[test]
fn quotes_are_repeatable_and_rounded_to_one_decimal() {
    let g = Game::new();
    let card = g.race_card(None).unwrap();
    let picks = top_by_odds(&card, 3);
    for kind in ALL_KINDS {
        let p = &picks[..bets::picks(kind)];
        let a = g.quote_bet(None, kind, p).unwrap();
        assert_eq!(a, g.quote_bet(None, kind, p).unwrap(), "{kind:?}");
        assert!(((a * 10.0).round() - a * 10.0).abs() < 1e-6, "{kind:?} {a}");
    }
}

#[test]
fn harder_tickets_on_the_same_horses_pay_more() {
    let g = Game::new();
    let card = g.race_card(None).unwrap();
    let p = top_by_odds(&card, 3);
    let odds = |k: BetKind, n: usize| g.quote_bet(None, k, &p[..n]).unwrap();
    assert!(odds(BetKind::Win, 1) < odds(BetKind::Exacta, 2));
    assert!(odds(BetKind::Quinella, 2) <= odds(BetKind::Exacta, 2));
    assert!(odds(BetKind::Exacta, 2) < odds(BetKind::Trifecta, 3));
    assert!(odds(BetKind::Trio, 3) <= odds(BetKind::Trifecta, 3));
    assert!(odds(BetKind::Place, 1) < odds(BetKind::Win, 1));
}

#[test]
fn the_order_matters_for_exacta_and_trifecta_prices_but_not_for_the_unordered_ones() {
    let g = Game::new();
    let card = g.race_card(None).unwrap();
    // the favourite, the next and one from the middle of the field: far enough apart
    let by_odds = top_by_odds(&card, entries::FULL_FIELD);
    let p = [
        by_odds[0].clone(),
        by_odds[1].clone(),
        by_odds[entries::FULL_FIELD / 2].clone(),
    ];
    let rev = |v: &[String]| v.iter().rev().cloned().collect::<Vec<_>>();
    let q = |k: BetKind, v: &[String]| g.quote_bet(None, k, v).unwrap();
    assert_eq!(
        q(BetKind::Quinella, &p[..2]),
        q(BetKind::Quinella, &rev(&p[..2]))
    );
    assert_eq!(q(BetKind::Trio, &p), q(BetKind::Trio, &rev(&p)));
    // the favourite winning and a lesser horse second is likelier than the reverse
    assert!(
        q(
            BetKind::Exacta,
            &p[0..=0].iter().chain(&p[2..]).cloned().collect::<Vec<_>>()
        ) < q(
            BetKind::Exacta,
            &rev(&p[0..=0].iter().chain(&p[2..]).cloned().collect::<Vec<_>>())
        )
    );
    assert!(q(BetKind::Trifecta, &p) < q(BetKind::Trifecta, &rev(&p)));
}

#[test]
fn a_pair_of_favourites_pays_less_than_a_pair_of_outsiders() {
    let g = Game::new();
    let card = g.race_card(None).unwrap();
    let mut es = card.entrants.clone();
    es.sort_by(|a, b| a.odds.total_cmp(&b.odds));
    let fav: Vec<String> = es.iter().take(2).map(|e| e.name.clone()).collect();
    let out: Vec<String> = es.iter().rev().take(2).map(|e| e.name.clone()).collect();
    for kind in [BetKind::Exacta, BetKind::Quinella] {
        assert!(
            g.quote_bet(None, kind, &fav).unwrap() < g.quote_bet(None, kind, &out).unwrap(),
            "{kind:?}"
        );
    }
}

#[test]
fn quoted_odds_stay_between_the_minimum_and_9999_9() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let spec = g.race_spec(Some(&key_of(&race))).unwrap();
    let n = spec.entrants.len();
    assert_eq!(n, entries::FULL_FIELD);
    // price the field once, then every combination from those chances
    let priced = Game::priced(&spec);
    let mut top = 0.0f64;
    let mut price = |kind: BetKind, idx: &[usize]| {
        let o = Game::ticket_odds(&priced, kind, idx);
        assert!((MIN_ODDS..=9999.9).contains(&o), "{kind:?} {idx:?} {o}");
        assert!(((o * 10.0).round() - o * 10.0).abs() < 1e-6, "{o}");
        top = top.max(o);
    };
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            price(BetKind::Exacta, &[i, j]);
            price(BetKind::Quinella, &[i, j]);
            for k in 0..n {
                if k != i && k != j {
                    price(BetKind::Trifecta, &[i, j, k]);
                    price(BetKind::Trio, &[i, j, k]);
                }
            }
        }
    }
    assert_eq!(
        top, 9999.9,
        "long shots should reach the cap in a 12-horse field"
    );
}

#[test]
fn combination_tickets_pay_back_close_to_the_payout_rate_on_average() {
    // Price a field with the production number of simulations, then play many
    // races and see what each popular ticket really returns per unit staked.
    // Fixed seeds, so this is repeatable; it guards against a systematic edge
    // (a ticket that pays far more, or far less, than it should).
    let mut deviation = 0.0;
    let mut tickets = 0.0;
    for (week, name, seed) in [
        (47u8, "国際友好ステークス", 2u64),
        (13, "春雷スプリント", 4),
    ] {
        let mut g = Game::with_seed(seed);
        g.week = week;
        let race = calendar_race(name);
        let spec = g.race_spec(Some(&key_of(&race))).unwrap();
        let priced = Game::priced_n(&spec, 1000);
        let chances = &priced.chances;
        let names: Vec<String> = spec.entrants.iter().map(|e| e.name.clone()).collect();
        let mut fav: Vec<usize> = (0..names.len()).collect();
        fav.sort_by(|&a, &b| chances[b].0.total_cmp(&chances[a].0));
        let orders: Vec<Vec<String>> = (0..3000u64)
            .map(|s| {
                Game::run_race(&spec, 5_000_000 + s)
                    .1
                    .iter()
                    .map(|r| r.name.clone())
                    .collect()
            })
            .collect();
        let mut candidates: Vec<(BetKind, Vec<usize>)> = vec![];
        for a in 0..5 {
            for b in 0..5 {
                if a != b {
                    candidates.push((BetKind::Exacta, vec![fav[a], fav[b]]));
                }
                if a < b {
                    candidates.push((BetKind::Quinella, vec![fav[a], fav[b]]));
                    for c in (b + 1)..5 {
                        candidates.push((BetKind::Trio, vec![fav[a], fav[b], fav[c]]));
                    }
                }
            }
        }
        for (kind, idx) in candidates {
            let picks: Vec<String> = idx.iter().map(|&i| names[i].clone()).collect();
            let hits = orders
                .iter()
                .filter(|o| bets::is_hit(kind, &picks, o))
                .count() as f64;
            if hits < 80.0 {
                continue; // too few hits to measure
            }
            let back = hits / orders.len() as f64 * Game::ticket_odds(&priced, kind, &idx);
            assert!(
                (0.45..=1.5).contains(&back),
                "{name} {kind:?} {picks:?} returns {back:.2}"
            );
            deviation += (back / PAYOUT_RATE).ln().abs();
            tickets += 1.0;
        }
    }
    assert!(tickets >= 30.0, "only {tickets} tickets could be measured");
    assert!(
        deviation / tickets < 0.25,
        "average miss {:.3}",
        deviation / tickets
    );
}

#[test]
fn a_calendar_race_can_be_quoted_and_a_missing_one_is_refused() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    let p = top_by_odds(&card, 3);
    assert!(g.quote_bet(Some(&key_of(&race)), BetKind::Trio, &p).is_ok());
    g.week = 1;
    assert_eq!(
        g.quote_bet(Some(&key_of(&race)), BetKind::Trio, &p),
        Err(GameError::NoSuchRace)
    );
}

// ---- validating a selection

#[test]
fn the_number_of_horses_must_match_the_kind_of_ticket() {
    let g = Game::new();
    let n: Vec<String> = g.field.iter().map(|h| h.name.clone()).collect();
    let cases: [(BetKind, usize); 8] = [
        (BetKind::Win, 0),
        (BetKind::Win, 2),
        (BetKind::Place, 3),
        (BetKind::Exacta, 1),
        (BetKind::Quinella, 3),
        (BetKind::Trifecta, 2),
        (BetKind::Trio, 1),
        (BetKind::Trio, 4),
    ];
    for (kind, count) in cases {
        assert_eq!(
            g.quote_bet(None, kind, &n[..count]),
            Err(GameError::InvalidSelection),
            "{kind:?} with {count}"
        );
    }
}

#[test]
fn the_same_horse_cannot_be_picked_twice() {
    let g = Game::new();
    let a = g.field[0].name.clone();
    let b = g.field[1].name.clone();
    for (kind, picks) in [
        (BetKind::Exacta, strs(&[&a, &a])),
        (BetKind::Quinella, strs(&[&b, &b])),
        (BetKind::Trifecta, strs(&[&a, &b, &a])),
        (BetKind::Trio, strs(&[&a, &a, &b])),
    ] {
        assert_eq!(
            g.quote_bet(None, kind, &picks),
            Err(GameError::DuplicateHorse),
            "{kind:?}"
        );
    }
}

#[test]
fn every_horse_picked_must_be_in_the_race() {
    let g = Game::new();
    let a = g.field[0].name.clone();
    assert_eq!(
        g.quote_bet(None, BetKind::Win, &strs(&["Nobody"])),
        Err(GameError::UnknownHorse)
    );
    assert_eq!(
        g.quote_bet(None, BetKind::Exacta, &strs(&[&a, "Nobody"])),
        Err(GameError::UnknownHorse)
    );
    assert_eq!(
        g.quote_bet(
            None,
            BetKind::Trio,
            &strs(&["Nobody", &a, &g.field[1].name])
        ),
        Err(GameError::UnknownHorse)
    );
}

#[test]
fn a_ticket_cannot_include_a_stable_horse_that_is_not_running() {
    let mut g = Game::new();
    g.week = 51;
    let race = calendar_race("ジュニアマイルステークス"); // 2yo only
    let key = key_of(&race);
    let rival = g.race_card(Some(&key)).unwrap().entrants[0].name.clone();
    let stable = g.field[0].name.clone();
    assert_eq!(
        g.play_combo(Some(&key), &[rival, stable], 10, 1, BetKind::Exacta),
        Err(GameError::UnknownHorse)
    );
}

#[test]
fn a_refused_ticket_changes_nothing() {
    let mut g = Game::new();
    // two horses that are actually in the free race
    let card = g.race_card(None).unwrap();
    let (a, b) = (card.entrants[0].name.clone(), card.entrants[1].name.clone());
    let before = g.state();
    assert_eq!(
        g.play_combo(None, &[a.clone()], 10, 1, BetKind::Exacta),
        Err(GameError::InvalidSelection)
    );
    assert_eq!(
        g.play_combo(None, &[a.clone(), a.clone()], 10, 1, BetKind::Quinella),
        Err(GameError::DuplicateHorse)
    );
    assert_eq!(
        g.play_combo(None, &[a.clone(), b.clone()], 0, 1, BetKind::Exacta),
        Err(GameError::InvalidAmount)
    );
    assert_eq!(
        g.play_combo(None, &[a, b], 99999, 1, BetKind::Exacta),
        Err(GameError::InsufficientFunds)
    );
    assert_eq!(g.state(), before);
}

#[test]
fn nothing_can_be_bought_when_the_game_is_over() {
    let mut g = Game::new();
    let (a, b) = (g.field[0].name.clone(), g.field[1].name.clone());
    g.money = 0;
    assert_eq!(
        g.play_combo(None, &[a, b], 1, 1, BetKind::Quinella),
        Err(GameError::GameOver)
    );
}

// ---- history

#[test]
fn history_records_the_kind_and_how_the_selection_is_written() {
    let o = free_order(21);
    for (kind, label) in [
        (BetKind::Win, o[0].clone()),
        (BetKind::Place, o[0].clone()),
        (BetKind::Exacta, format!("{}→{}", o[0], o[1])),
        (BetKind::Quinella, format!("{}-{}", o[0], o[1])),
        (BetKind::Trifecta, format!("{}→{}→{}", o[0], o[1], o[2])),
        (BetKind::Trio, format!("{}-{}-{}", o[0], o[1], o[2])),
    ] {
        let picks: Vec<String> = o[..bets::picks(kind)].to_vec();
        let mut g = Game::new();
        g.play_combo(None, &picks, 10, 21, kind).unwrap();
        let e = g.state().history.pop().unwrap();
        assert_eq!(e.kind, kind);
        assert_eq!(e.horse, label, "{kind:?}");
        assert_eq!(e.horses, picks);
        assert!(e.won);
    }
}

#[test]
fn old_history_entries_without_the_list_of_horses_still_load() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.play_round(&name, 10, 1).unwrap();
    let mut v: serde_json::Value = old_save(&g);
    v["history"][0].as_object_mut().unwrap().remove("horses");
    let old = Game::from_json(&v.to_string()).unwrap();
    let e = &old.state().history[0];
    assert_eq!(e.horse, name);
    assert!(e.horses.is_empty());
}

// ---- tickets in a calendar race

#[test]
fn a_trifecta_on_a_calendar_race_is_decided_over_the_whole_field() {
    let mut g = Game::new();
    g.stars.clear();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let spec = g.race_spec(Some(&key)).unwrap();
    let order: Vec<String> = Game::run_race(&spec, 5)
        .1
        .iter()
        .map(|r| r.name.clone())
        .collect();
    assert_eq!(order.len(), entries::FULL_FIELD);
    let out = g
        .play_combo(Some(&key), &order[..3], 10, 5, BetKind::Trifecta)
        .unwrap();
    assert!(out.won);
    assert_eq!(out.race_name.as_deref(), Some("国際友好ステークス"));
    assert_eq!(g.week, 48);
    // a trifecta this long pays a lot
    assert!(out.payout >= 10 * 50, "{}", out.payout);
}

#[test]
fn the_json_for_the_new_kinds_and_history_matches_what_the_ui_expects() {
    let o = free_order(3);
    let mut g = Game::new();
    g.play_combo(None, &o[..2], 10, 3, BetKind::Exacta).unwrap();
    let st = serde_json::to_value(g.state()).unwrap();
    assert_eq!(st["history"][0]["kind"], "exacta");
    assert_eq!(st["history"][0]["horses"], serde_json::json!([o[0], o[1]]));
    assert_eq!(st["history"][0]["horse"], format!("{}→{}", o[0], o[1]));
    let k: BetKind = serde_json::from_str("\"quinella\"").unwrap();
    assert_eq!(k, BetKind::Quinella);
}

// ---- several tickets at once (複数馬券の同時購入) ----

fn tk(kind: BetKind, horses: &[&String], amount: u32) -> Ticket {
    Ticket {
        kind,
        horses: horses.iter().map(|h| (*h).clone()).collect(),
        amount,
    }
}

fn buy_free(tickets: &[Ticket], seed: u64) -> (Game, Result<RoundOutcome, GameError>) {
    let mut g = Game::new();
    let r = g.play_tickets(None, tickets, seed);
    (g, r)
}

#[test]
fn several_tickets_are_judged_one_by_one_and_paid_separately() {
    let o = free_order(31);
    let tickets = vec![
        tk(BetKind::Win, &[&o[0]], 100),               // hit
        tk(BetKind::Quinella, &[&o[1], &o[0]], 50),    // hit
        tk(BetKind::Exacta, &[&o[1], &o[0]], 30),      // miss: wrong order
        tk(BetKind::Trio, &[&o[0], &o[1], &o[5]], 20), // miss
    ];
    let g0 = Game::new();
    let quoted = g0.quote_tickets(None, &tickets).unwrap();
    let (g, out) = buy_free(&tickets, 31);
    let out = out.unwrap();
    assert_eq!(out.tickets.len(), 4);
    assert_eq!(
        out.tickets.iter().map(|t| t.won).collect::<Vec<_>>(),
        [true, true, false, false]
    );
    for (t, (src, q)) in out.tickets.iter().zip(tickets.iter().zip(&quoted)) {
        assert_eq!(
            (t.kind, &t.horses, t.amount),
            (src.kind, &src.horses, src.amount)
        );
        assert_eq!(t.odds, *q);
        let want = if t.won {
            (t.amount as f64 * t.odds).round() as i64
        } else {
            0
        };
        assert_eq!(t.payout, want);
    }
    let total_payout: i64 = out.tickets.iter().map(|t| t.payout).sum();
    assert_eq!(out.payout, total_payout);
    assert!(out.won);
    assert_eq!(g.money as i64, START_MONEY as i64 - 200 + total_payout);
}

#[test]
fn the_total_stake_is_taken_once_and_each_ticket_keeps_its_own_amount() {
    let o = free_order(32);
    let tickets = vec![
        tk(BetKind::Win, &[&o[3]], 10),
        tk(BetKind::Win, &[&o[4]], 25),
        tk(BetKind::Place, &[&o[5]], 65),
    ];
    let (g, out) = buy_free(&tickets, 32);
    let out = out.unwrap();
    assert!(out.tickets.iter().all(|t| !t.won)); // none of these placed
    assert_eq!(g.money, START_MONEY - 100);
    assert_eq!(
        out.tickets.iter().map(|t| t.amount).collect::<Vec<_>>(),
        [10, 25, 65]
    );
}

#[test]
fn the_outcome_is_a_loss_only_if_every_ticket_misses() {
    let o = free_order(33);
    let (_, out) = buy_free(
        &[
            tk(BetKind::Win, &[&o[2]], 10),
            tk(BetKind::Win, &[&o[3]], 10),
        ],
        33,
    );
    let out = out.unwrap();
    assert!(!out.won && out.payout == 0);
    let (_, out) = buy_free(
        &[
            tk(BetKind::Win, &[&o[2]], 10),
            tk(BetKind::Place, &[&o[2]], 10),
        ],
        33,
    );
    assert!(out.unwrap().won); // the place ticket hit
}

#[test]
fn the_race_runs_once_for_all_the_tickets_and_matches_a_single_ticket_run() {
    let o = free_order(34);
    let (_, many) = buy_free(
        &[
            tk(BetKind::Win, &[&o[0]], 10),
            tk(BetKind::Quinella, &[&o[0], &o[1]], 10),
        ],
        34,
    );
    let (_, one) = buy_free(&[tk(BetKind::Win, &[&o[0]], 10)], 34);
    let (many, one) = (many.unwrap(), one.unwrap());
    assert_eq!(many.results, one.results);
    assert_eq!(many.frames, one.frames);
    assert_eq!(many.runners, one.runners);
}

#[test]
fn a_single_ticket_is_just_a_purchase_of_one() {
    let o = free_order(35);
    let (_, out) = buy_free(&[tk(BetKind::Exacta, &[&o[0], &o[1]], 40)], 35);
    let out = out.unwrap();
    assert_eq!(out.tickets.len(), 1);
    assert!(out.tickets[0].won);
    assert_eq!(out.tickets[0].label, format!("{}→{}", o[0], o[1]));
}

#[test]
fn many_tickets_in_a_calendar_race_move_one_week_and_pay_the_prize_once() {
    let mut g = Game::new();
    g.stars.clear();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let seed = seed_where(&g, &key, &mine, 1);
    let rivals: Vec<String> = g
        .race_card(Some(&key))
        .unwrap()
        .entrants
        .iter()
        .filter(|e| !e.mine)
        .map(|e| e.name.clone())
        .collect();
    let tickets: Vec<Ticket> = rivals[..3]
        .iter()
        .map(|r| Ticket {
            kind: BetKind::Win,
            horses: vec![r.clone()],
            amount: 10,
        })
        .collect();
    let bill = g.weekly_upkeep() as i64 + race_fees(&g, &key) as i64;
    let out = g.play_tickets(Some(&key), &tickets, seed).unwrap();
    assert_eq!(g.week, 48);
    assert_eq!(out.prize_total, 1500);
    assert_eq!(
        g.money as i64,
        START_MONEY as i64 - 30 + out.payout + 1500 - bill
    );
    assert_eq!(out.race_name.as_deref(), Some("国際友好ステークス"));
}

// ---- validating a purchase

#[test]
fn a_purchase_needs_at_least_one_ticket() {
    let mut g = Game::new();
    assert_eq!(g.play_tickets(None, &[], 1), Err(GameError::NoTickets));
    assert_eq!(g.quote_tickets(None, &[]), Err(GameError::NoTickets));
}

#[test]
fn at_most_two_hundred_tickets_can_be_bought_at_once() {
    let mut g = Game::new();
    g.money = 100_000;
    let o = free_order(36);
    let t = tk(BetKind::Win, &[&o[0]], 1);
    assert!(g
        .play_tickets(None, &vec![t.clone(); MAX_TICKETS], 36)
        .is_ok());
    let before = g.state();
    assert_eq!(
        g.play_tickets(None, &vec![t; MAX_TICKETS + 1], 36),
        Err(GameError::TooManyTickets)
    );
    assert_eq!(g.state(), before);
}

#[test]
fn one_bad_ticket_refuses_the_whole_purchase_and_changes_nothing() {
    let o = free_order(37);
    let mut g = Game::new();
    let before = g.state();
    let good = tk(BetKind::Win, &[&o[0]], 10);
    // wrong number of horses
    let two = Ticket {
        kind: BetKind::Win,
        horses: vec![o[0].clone(), o[1].clone()],
        amount: 10,
    };
    assert_eq!(
        g.play_tickets(None, &[good.clone(), two], 1),
        Err(GameError::InvalidSelection)
    );
    // the same horse twice
    let dup = Ticket {
        kind: BetKind::Quinella,
        horses: vec![o[0].clone(), o[0].clone()],
        amount: 10,
    };
    assert_eq!(
        g.play_tickets(None, &[good.clone(), dup], 1),
        Err(GameError::DuplicateHorse)
    );
    // a horse that is not in the race
    let ghost = Ticket {
        kind: BetKind::Win,
        horses: vec!["Nobody".into()],
        amount: 10,
    };
    assert_eq!(
        g.play_tickets(None, &[good.clone(), ghost], 1),
        Err(GameError::UnknownHorse)
    );
    // a ticket for nothing
    let zero = Ticket {
        amount: 0,
        ..good.clone()
    };
    assert_eq!(
        g.play_tickets(None, &[good.clone(), zero], 1),
        Err(GameError::InvalidAmount)
    );
    assert_eq!(g.state(), before);
}

#[test]
fn the_total_must_be_affordable_even_if_each_ticket_is() {
    let o = free_order(38);
    let mut g = Game::new();
    let tickets = vec![
        tk(BetKind::Win, &[&o[0]], 2000),
        tk(BetKind::Win, &[&o[1]], 2000),
    ];
    assert_eq!(
        g.play_tickets(None, &tickets, 1),
        Err(GameError::InsufficientFunds)
    );
    assert_eq!(g.money, START_MONEY);
    assert!(g.play_tickets(None, &tickets[..1], 1).is_ok());
}

#[test]
fn a_multiple_purchase_is_refused_when_the_game_is_over() {
    let o = free_order(39);
    let mut g = Game::new();
    g.money = 0;
    assert_eq!(
        g.play_tickets(None, &[tk(BetKind::Win, &[&o[0]], 1)], 1),
        Err(GameError::GameOver)
    );
}

#[test]
fn a_ticket_for_a_horse_that_is_not_entered_is_refused() {
    let mut g = Game::new();
    g.set_entries(None, &strs(&["サクラフブキ"])).unwrap();
    let ghost = g
        .field
        .iter()
        .find(|h| h.name == "ミナモムーン")
        .unwrap()
        .name
        .clone();
    let o = free_order(1);
    let t = vec![
        tk(BetKind::Win, &[&o[0]], 5),
        tk(BetKind::Win, &[&ghost], 5),
    ];
    assert_eq!(g.play_tickets(None, &t, 1), Err(GameError::UnknownHorse));
}

// ---- history

#[test]
fn a_multiple_purchase_is_one_history_line_with_the_totals() {
    let o = free_order(40);
    let tickets = vec![
        tk(BetKind::Quinella, &[&o[0], &o[1]], 100), // hit
        tk(BetKind::Quinella, &[&o[0], &o[4]], 100),
        tk(BetKind::Quinella, &[&o[1], &o[4]], 100),
    ];
    let (g, out) = buy_free(&tickets, 40);
    let out = out.unwrap();
    let h = g.state().history;
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].kind, BetKind::Quinella);
    assert_eq!(h[0].horse, format!("{}-{} ほか2点", o[0], o[1]));
    assert_eq!(h[0].ticket_count, 3);
    assert!(!h[0].mixed_kinds);
    assert_eq!(h[0].stake, 300);
    assert_eq!(h[0].payout, out.payout);
    assert!(h[0].won);
}

#[test]
fn a_purchase_of_mixed_kinds_is_marked_so() {
    let o = free_order(41);
    let tickets = vec![
        tk(BetKind::Win, &[&o[0]], 10),
        tk(BetKind::Trio, &[&o[0], &o[1], &o[2]], 10),
    ];
    let (g, _) = buy_free(&tickets, 41);
    let h = &g.state().history[0];
    assert!(h.mixed_kinds);
    assert_eq!(h.ticket_count, 2);
    assert_eq!(h.kind, BetKind::Win);
}

#[test]
fn a_purchase_of_one_ticket_reads_as_before() {
    let o = free_order(42);
    let (g, _) = buy_free(&[tk(BetKind::Exacta, &[&o[0], &o[1]], 10)], 42);
    let h = &g.state().history[0];
    assert_eq!(h.horse, format!("{}→{}", o[0], o[1]));
    assert_eq!(h.ticket_count, 1);
    assert!(!h.mixed_kinds);
}

#[test]
fn old_history_entries_count_as_one_ticket() {
    let o = free_order(43);
    let (g, _) = buy_free(&[tk(BetKind::Win, &[&o[0]], 10)], 43);
    let mut v: serde_json::Value = old_save(&g);
    for k in ["ticket_count", "mixed_kinds"] {
        v["history"][0].as_object_mut().unwrap().remove(k);
    }
    let old = Game::from_json(&v.to_string()).unwrap();
    let h = &old.state().history[0];
    assert_eq!((h.ticket_count, h.mixed_kinds), (1, false));
}

// ---- quoting several tickets

#[test]
fn quotes_for_several_tickets_match_the_single_quotes() {
    let g = Game::new();
    let card = g.race_card(None).unwrap();
    let n: Vec<String> = card.entrants.iter().map(|e| e.name.clone()).collect();
    let tickets = vec![
        tk(BetKind::Win, &[&n[0]], 1),
        tk(BetKind::Place, &[&n[1]], 1),
        tk(BetKind::Exacta, &[&n[0], &n[1]], 1),
        tk(BetKind::Trio, &[&n[2], &n[3], &n[4]], 1),
    ];
    let many = g.quote_tickets(None, &tickets).unwrap();
    for (t, q) in tickets.iter().zip(many) {
        assert_eq!(q, g.quote_bet(None, t.kind, &t.horses).unwrap());
    }
}

#[test]
fn a_bad_ticket_in_a_quote_is_refused() {
    let g = Game::new();
    let t = Ticket {
        kind: BetKind::Exacta,
        horses: strs(&["Nobody", "Else"]),
        amount: 1,
    };
    assert_eq!(g.quote_tickets(None, &[t]), Err(GameError::UnknownHorse));
}

// ---- ボックス and 流し in a race

fn field_names(g: &Game) -> Vec<String> {
    g.race_card(None)
        .unwrap()
        .entrants
        .iter()
        .map(|e| e.name.clone())
        .collect()
}

#[test]
fn a_box_expands_to_every_combination_of_the_chosen_horses() {
    let g = Game::new();
    let n = field_names(&g);
    let t = g
        .expand_selection(None, BetStyle::Box, BetKind::Quinella, None, &n[..4])
        .unwrap();
    assert_eq!(t.len(), 6);
    let t = g
        .expand_selection(None, BetStyle::Box, BetKind::Trifecta, None, &n[..4])
        .unwrap();
    assert_eq!(t.len(), 24);
    assert!(t.iter().all(|x| x.iter().all(|h| n[..4].contains(h))));
}

#[test]
fn a_flow_expands_from_the_axis_horse() {
    let g = Game::new();
    let n = field_names(&g);
    let t = g
        .expand_selection(None, BetStyle::Flow, BetKind::Trio, Some(&n[0]), &n[1..5])
        .unwrap();
    assert_eq!(t.len(), 6); // pairs of the four partners
    assert!(t.iter().all(|x| x[0] == n[0] && x.len() == 3));
    let t = g
        .expand_selection(None, BetStyle::Flow, BetKind::Exacta, Some(&n[0]), &n[1..4])
        .unwrap();
    assert_eq!(
        t,
        vec![
            vec![n[0].clone(), n[1].clone()],
            vec![n[0].clone(), n[2].clone()],
            vec![n[0].clone(), n[3].clone()]
        ]
    );
}

#[test]
fn single_kinds_cannot_be_expanded() {
    let g = Game::new();
    let n = field_names(&g);
    for kind in [BetKind::Win, BetKind::Place] {
        assert_eq!(
            g.expand_selection(None, BetStyle::Box, kind, None, &n[..4]),
            Err(GameError::InvalidSelection)
        );
    }
}

#[test]
fn too_few_horses_cannot_be_expanded() {
    let g = Game::new();
    let n = field_names(&g);
    assert_eq!(
        g.expand_selection(None, BetStyle::Box, BetKind::Trio, None, &n[..2]),
        Err(GameError::InvalidSelection)
    );
    assert_eq!(
        g.expand_selection(None, BetStyle::Flow, BetKind::Trio, Some(&n[0]), &n[1..2]),
        Err(GameError::InvalidSelection)
    );
}

#[test]
fn expansion_checks_the_horses_are_distinct_and_in_the_race() {
    let g = Game::new();
    let n = field_names(&g);
    let dup = vec![n[0].clone(), n[0].clone(), n[1].clone()];
    assert_eq!(
        g.expand_selection(None, BetStyle::Box, BetKind::Trio, None, &dup),
        Err(GameError::DuplicateHorse)
    );
    let ghost = vec![n[0].clone(), "Nobody".to_string(), n[1].clone()];
    assert_eq!(
        g.expand_selection(None, BetStyle::Box, BetKind::Trio, None, &ghost),
        Err(GameError::UnknownHorse)
    );
    // the axis may not also be a partner
    assert_eq!(
        g.expand_selection(
            None,
            BetStyle::Flow,
            BetKind::Quinella,
            Some(&n[0]),
            &n[..3]
        ),
        Err(GameError::DuplicateHorse)
    );
    // a flow needs an axis
    assert_eq!(
        g.expand_selection(None, BetStyle::Flow, BetKind::Quinella, None, &n[1..3]),
        Err(GameError::InvalidSelection)
    );
}

#[test]
fn an_expansion_larger_than_the_limit_is_refused() {
    let g = Game::new();
    let n = field_names(&g);
    // 8 horses boxed for a trifecta is 336 tickets
    assert_eq!(
        g.expand_selection(None, BetStyle::Box, BetKind::Trifecta, None, &n[..8]),
        Err(GameError::TooManyTickets)
    );
    // 6 horses is 120: fine
    assert_eq!(
        g.expand_selection(None, BetStyle::Box, BetKind::Trifecta, None, &n[..6])
            .unwrap()
            .len(),
        120
    );
}

#[test]
fn an_expanded_box_can_be_bought_and_hits_when_the_boxed_horses_finish_in_the_places() {
    let o = free_order(44);
    let g = Game::new();
    let boxed: Vec<String> = o[..4].to_vec();
    let combos = g
        .expand_selection(None, BetStyle::Box, BetKind::Trio, None, &boxed)
        .unwrap();
    let tickets: Vec<Ticket> = combos
        .into_iter()
        .map(|horses| Ticket {
            kind: BetKind::Trio,
            horses,
            amount: 10,
        })
        .collect();
    assert_eq!(tickets.len(), 4);
    let (g, out) = buy_free(&tickets, 44);
    let out = out.unwrap();
    // the top three are among the four boxed, so exactly one of the four trios hits
    assert_eq!(out.tickets.iter().filter(|t| t.won).count(), 1);
    assert_eq!(g.money as i64, START_MONEY as i64 - 40 + out.payout);
}

#[test]
fn the_json_shapes_for_tickets_match_what_the_ui_expects() {
    let o = free_order(45);
    let t: Ticket = serde_json::from_str(&format!(
        r#"{{"kind":"quinella","horses":["{}","{}"],"amount":50}}"#,
        o[0], o[1]
    ))
    .unwrap();
    assert_eq!((t.kind, t.amount), (BetKind::Quinella, 50));
    let (g, out) = buy_free(&[t], 45);
    let v = serde_json::to_value(out.unwrap()).unwrap();
    let r = &v["tickets"][0];
    for field in ["kind", "horses", "label", "amount", "odds", "won", "payout"] {
        assert!(r.get(field).is_some(), "missing {field}");
    }
    let st = serde_json::to_value(g.state()).unwrap();
    assert_eq!(st["history"][0]["ticket_count"], 1);
    assert_eq!(st["history"][0]["mixed_kinds"], false);
    for style in ["box", "flow"] {
        let s: BetStyle = serde_json::from_str(&format!("\"{style}\"")).unwrap();
        assert!(matches!(s, BetStyle::Box | BetStyle::Flow));
    }
}
