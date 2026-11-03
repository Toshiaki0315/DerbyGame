//! Racing abroad (海外遠征).

use super::*;

// ---- racing abroad (海外遠征) ----

/// A game in the entry window for the Arc, with a G1 winner ready to go.
fn ready_to_go() -> (Game, usize, usize) {
    let mut g = Game::with_seed(1);
    g.money = 100_000;
    let race = OVERSEAS.iter().position(|r| r.name == "凱旋門賞").unwrap();
    g.week = OVERSEAS[race].week - 4;
    g.field[0].age = 4;
    g.field[0].fatigue = 0;
    g.field[0].career.stakes_wins.push(StakesWin {
        year: 1,
        race: "X".into(),
        grade: Grade::G1,
    });
    (g, 0, race)
}

#[test]
fn the_great_races_abroad_are_listed_with_when_they_can_be_entered() {
    let (g, _, race) = ready_to_go();
    let st = g.state();
    assert_eq!(st.overseas.len(), OVERSEAS.len());
    let arc = &st.overseas[race];
    assert_eq!(arc.country, "フランス");
    assert!(arc.open);
    assert_eq!(arc.candidates, vec![g.field[0].name.clone()]);
    let mut late = g.clone();
    late.week = OVERSEAS[race].week - EXPEDITION_LATEST + 1;
    assert!(!late.state().overseas[race].open);
    let mut early = g.clone();
    early.week = OVERSEAS[race].week - EXPEDITION_EARLIEST - 1;
    assert!(!early.state().overseas[race].open);
}

#[test]
fn only_a_horse_with_a_g1_or_great_earnings_may_go_abroad() {
    let (mut g, _, race) = ready_to_go();
    let other = g.field[1].name.clone();
    assert_eq!(g.go_abroad(&other, race), Err(GameError::NotEligible));
    g.field[1].career.earnings = EXPEDITION_MIN_EARNINGS;
    g.field[1].age = 4;
    g.go_abroad(&other, race).unwrap();
}

#[test]
fn going_abroad_costs_money_and_takes_the_horse_away() {
    let (mut g, i, race) = ready_to_go();
    let name = g.field[i].name.clone();
    g.go_abroad(&name, race).unwrap();
    assert_eq!(g.money, 100_000 - EXPEDITION_COST);
    assert_eq!(g.field[i].away.as_ref().unwrap().kind, AwayKind::Expedition);
    assert_eq!(g.state().expeditions[0].horse, name);
    assert_eq!(g.go_abroad(&name, race), Err(GameError::Away));
    assert_eq!(g.recall(&name), Err(GameError::NotOnPasture));
}

#[test]
fn a_trip_abroad_is_refused_when_it_cannot_be_paid_for_or_the_window_has_closed() {
    let (mut g, i, race) = ready_to_go();
    let name = g.field[i].name.clone();
    let money = EXPEDITION_COST;
    g.money = money;
    assert_eq!(g.go_abroad(&name, race), Err(GameError::InsufficientFunds));
    g.money = 100_000;
    g.week = OVERSEAS[race].week;
    assert_eq!(g.go_abroad(&name, race), Err(GameError::SaleClosed));
    assert_eq!(g.go_abroad(&name, 99), Err(GameError::UnknownRace));
}

#[test]
fn the_horse_runs_abroad_in_its_week_and_comes_home_after() {
    let (mut g, i, race) = ready_to_go();
    let name = g.field[i].name.clone();
    g.go_abroad(&name, race).unwrap();
    let starts = g.field[i].record.starts;
    let week = OVERSEAS[race].week;
    while g.week <= week {
        assert!(g.news.iter().all(|n| !n.contains("凱旋門賞に出走")));
        g.advance_week();
    }
    assert!(
        g.news
            .iter()
            .any(|n| n.contains(&name) && n.contains("凱旋門賞")),
        "{:?}",
        g.news
    );
    assert!(g.state().expeditions.is_empty());
    assert_eq!(g.field[i].record.starts, starts + 1);
    assert!(g.field[i].away.is_some());
    for _ in 0..EXPEDITION_RETURN_WEEKS {
        g.advance_week();
    }
    assert!(g.field[i].away.is_none());
}

#[test]
fn a_win_abroad_pays_and_is_remembered_as_a_goal() {
    // over many tries the strongest of horses wins sometimes
    let (base, i, race) = ready_to_go();
    let name = base.field[i].name.clone();
    let won = (0..300u64)
        .map(|seed| {
            let mut g = with_rng(&base, seed);
            g.world_seed ^= seed;
            g.field[i].speed = 125;
            g.field[i].stamina = 125;
            g.field[i].power = 125;
            g.go_abroad(&name, race).unwrap();
            while g.week <= OVERSEAS[race].week {
                g.advance_week();
            }
            g
        })
        .find(|g| {
            g.field[i]
                .career
                .stakes_wins
                .iter()
                .any(|w| w.race == "凱旋門賞")
        })
        .expect("a win in 300 tries");
    assert!(won.field[i]
        .career
        .titles
        .iter()
        .any(|t| t == "凱旋門賞制覇"));
    assert!(won
        .awards
        .iter()
        .any(|a| a.mine && a.title == "海外G1 凱旋門賞"));
    assert!(won
        .state()
        .goals
        .iter()
        .any(|x| x.name == "海外G1制覇" && x.done));
}

#[test]
fn a_horse_sold_before_its_race_abroad_does_not_run() {
    let (mut g, i, race) = ready_to_go();
    let name = g.field[i].name.clone();
    g.go_abroad(&name, race).unwrap();
    g.sell_horse(&name).unwrap();
    while g.week <= OVERSEAS[race].week {
        g.advance_week();
    }
    assert!(g.news.iter().all(|n| !n.contains(&name)));
}

#[test]
fn a_trip_abroad_survives_a_save() {
    let (mut g, i, race) = ready_to_go();
    let name = g.field[i].name.clone();
    g.go_abroad(&name, race).unwrap();
    let r = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(r.state(), g.state());
}
