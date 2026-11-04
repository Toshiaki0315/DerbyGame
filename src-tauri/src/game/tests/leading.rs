//! The year's leading jockey, trainer and sire (リーディング).

use super::*;

fn a_year_of_racing() -> Game {
    let mut g = Game::with_seed(4);
    g.entry_rules = true;
    g.set_all_auto(true);
    for _ in 0..calendar::WEEKS_PER_YEAR {
        g.advance_week();
    }
    g
}

fn descending(rows: &[LeadingRow]) -> bool {
    rows.windows(2).all(|p| p[0].wins >= p[1].wins)
}

#[test]
fn the_table_lists_every_jockey_and_trainer_in_order_of_wins() {
    let mut g = Game::new();
    g.week = 30;
    let t = g.leading();
    assert_eq!(t.jockeys.len(), 10);
    assert_eq!(t.trainers.len(), 10);
    assert!(!t.sires.is_empty());
    assert!(descending(&t.jockeys) && descending(&t.trainers) && descending(&t.sires));
}

#[test]
fn the_tables_build_up_through_the_year() {
    let mut g = Game::new();
    let total = |g: &Game| g.leading().jockeys.iter().map(|r| r.wins).sum::<u32>();
    g.week = 1;
    let early = total(&g);
    g.week = 40;
    assert!(total(&g) > early);
}

#[test]
fn a_better_jockey_tends_to_win_more() {
    let mut top = 0;
    for seed in 1..=20u64 {
        let g = Game::with_seed(seed);
        let t = g.leading();
        let wins = |name: &str| t.jockeys.iter().find(|r| r.name == name).unwrap().wins;
        let (best, worst) = (&jockey::JOCKEYS[0], &jockey::JOCKEYS[9]);
        top += (wins(best.name) > wins(worst.name)) as u32;
    }
    assert!(top >= 18, "{top}");
}

#[test]
fn a_win_for_the_stable_counts_for_its_jockey_trainer_and_sire() {
    let mut g = Game::new();
    let r = calendar::shared()
        .races
        .iter()
        .find(|r| r.grade == Grade::Open && r.eligibility.age == AgeRule::ThreeUp)
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
    let spec = {
        g.field[i].record = RaceRecord { starts: 5, wins: 3 };
        g.set_entries(Some(&RaceKey::of(&r)), std::slice::from_ref(&name))
            .unwrap();
        g.race_spec(Some(&RaceKey::of(&r))).unwrap()
    };
    let e = spec.entrants.iter().find(|e| e.mine).unwrap();
    let ride = e.jockey.clone().unwrap();
    let trainer = g.field[i].trainer;
    let before = g.year_wins.clone();
    g.add_to_records(
        &spec,
        &[RaceResult {
            rank: 1,
            name: name.clone(),
            finish_time: 100.0,
            margin: String::new(),
        }],
    );
    let count = |m: &std::collections::HashMap<u8, u32>, id: u8| m.get(&id).copied().unwrap_or(0);
    assert_eq!(
        count(&g.year_wins.jockeys, ride.id),
        count(&before.jockeys, ride.id) + 1
    );
    assert_eq!(
        count(&g.year_wins.trainers, trainer),
        count(&before.trainers, trainer) + 1
    );
    assert_eq!(
        g.year_wins.sires.values().sum::<u32>(),
        before.sires.values().sum::<u32>() + 1
    );
    let row = g
        .leading()
        .jockeys
        .into_iter()
        .find(|r| r.name == ride.name)
        .unwrap();
    assert_eq!(row.own_wins, 1);
}

#[test]
fn a_defeat_counts_for_nobody() {
    let mut g = Game::new();
    let spec = g.race_spec(None).unwrap();
    g.add_to_records(
        &spec,
        &[RaceResult {
            rank: 1,
            name: "ダレカ".to_string(),
            finish_time: 100.0,
            margin: String::new(),
        }],
    );
    assert!(g.year_wins.jockeys.is_empty() && g.year_wins.trainers.is_empty());
}

#[test]
fn at_the_end_of_the_year_the_leaders_are_named_and_the_count_starts_again() {
    let g = a_year_of_racing();
    assert_eq!(g.leading_years.len(), 1);
    let y = &g.leading_years[0];
    assert_eq!(y.year, 1);
    assert!(!y.jockey.is_empty() && !y.trainer.is_empty() && !y.sire.is_empty());
    assert!(
        g.news.iter().any(|n| n.contains("リーディング")),
        "{:?}",
        g.news
    );
    assert!(g.year_wins.jockeys.is_empty());
}

#[test]
fn the_screen_is_given_the_tables_and_the_past_leaders() {
    let g = a_year_of_racing();
    let v = serde_json::to_value(g.state()).unwrap();
    assert!(v["leading"]["jockeys"][0]["name"].is_string());
    assert!(v["leading"]["trainers"][0]["wins"].is_number());
    assert!(v["leading"]["sires"].is_array());
    assert_eq!(v["leading_years"][0]["year"], 1);
}

#[test]
fn the_tables_survive_a_save_and_old_saves_have_none() {
    let g = a_year_of_racing();
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.leading_years, g.leading_years);
    let old = Game::from_json(&old_save(&Game::new()).to_string()).unwrap();
    assert!(old.leading_years.is_empty());
}
