//! Whether the stable has horses to follow the ones it has (後継の馬).

use super::*;

/// A stable of this many horses of each age.
fn stable_of(ages: &[u8]) -> Game {
    let mut g = Game::new();
    let horse = g.field[0].clone();
    g.field = ages
        .iter()
        .enumerate()
        .map(|(i, &age)| StableHorse {
            name: format!("カリ{i}"),
            age,
            ..horse.clone()
        })
        .collect();
    g
}

/// Arrives at the week the breeding season opens.
fn into_the_season(g: &mut Game) {
    g.week = BREEDING_START_WEEK - 1;
    g.advance_week();
}

fn warned(g: &Game) -> bool {
    g.news.iter().any(|n| n.contains("後継"))
}

#[test]
fn only_horses_old_enough_count_as_racers() {
    let g = stable_of(&[0, 1, DEBUT_AGE, 5, 8]);
    assert_eq!(g.racers(), 3);
}

#[test]
fn a_horse_away_for_a_while_still_counts_as_a_racer() {
    let mut g = stable_of(&[4, 4]);
    g.send_to_pasture("カリ0").unwrap();
    assert_eq!(g.racers(), 2);
}

#[test]
fn successors_are_those_still_racing_when_a_foal_mated_now_would_debut() {
    let last = RETIRE_AGE - FOAL_LEAD_YEARS - 1;
    assert_eq!(stable_of(&[0, 1, last]).successors(), 3);
    assert_eq!(stable_of(&[last + 1, RETIRE_AGE - 1]).successors(), 0);
}

#[test]
fn a_foal_on_the_way_is_a_successor() {
    let (mut g, s, d, _) = in_season();
    let before = g.successors();
    g.money = 100_000;
    g.breed(&s, &d, "", "ハナミズキ").unwrap();
    assert_eq!(g.successors(), before + g.pregnancies.len());
}

#[test]
fn the_state_says_when_the_stable_is_short_of_horses() {
    let short = RETIRE_AGE - 1;
    let v = serde_json::to_value(stable_of(&[short; MIN_RACERS - 1]).state()).unwrap();
    assert_eq!(v["racers"], MIN_RACERS - 1);
    assert_eq!(v["successors"], 0);
    assert_eq!(v["min_racers"], MIN_RACERS);
    assert_eq!(v["short_of_racers"], true);
    assert_eq!(v["short_of_successors"], true);

    let v = serde_json::to_value(stable_of(&[3; MIN_RACERS]).state()).unwrap();
    assert_eq!(v["short_of_racers"], false);
    assert_eq!(v["short_of_successors"], false);
}

#[test]
fn a_new_game_is_not_short_of_anything() {
    for seed in 1..=5 {
        let g = Game::with_seed(seed);
        assert!(g.racers() >= MIN_RACERS);
        assert!(g.successors() >= MIN_RACERS);
    }
}

#[test]
fn the_warning_comes_as_the_breeding_season_opens() {
    let mut g = stable_of(&[7; 8]);
    into_the_season(&mut g);
    assert!(warned(&g), "{:?}", g.news);
    g.advance_week();
    assert!(!warned(&g), "{:?}", g.news);
}

#[test]
fn no_warning_outside_the_opening_of_the_season() {
    let mut g = stable_of(&[7; 8]);
    g.week = BREEDING_END_WEEK;
    g.advance_week();
    assert!(!warned(&g), "{:?}", g.news);
}

#[test]
fn no_warning_while_there_are_successors_enough() {
    let mut g = stable_of(&[3; MIN_RACERS]);
    into_the_season(&mut g);
    assert!(!warned(&g), "{:?}", g.news);
}

#[test]
fn the_warning_says_how_many_successors_there_are() {
    let mut g = stable_of(&[3, 7, 7, 7]);
    into_the_season(&mut g);
    let line = g.news.iter().find(|n| n.contains("後継")).unwrap();
    assert!(line.contains("1頭"), "{line}");
}

#[test]
fn a_stable_left_alone_is_warned_years_ahead_and_short_before_it_is_empty() {
    for seed in 1..=8u64 {
        let mut g = Game::with_seed(seed);
        g.entry_rules = true;
        g.set_all_auto(true);
        let (mut warned_in, mut short_in, mut empty_in) = (None, None, None);
        for _ in 0..8 * calendar::WEEKS_PER_YEAR as u32 {
            g.advance_week();
            if g.is_game_over() {
                break;
            }
            if warned(&g) {
                warned_in.get_or_insert(g.year);
            }
            if g.state().short_of_racers {
                short_in.get_or_insert(g.year);
            }
            if g.field.is_empty() {
                empty_in.get_or_insert(g.year);
            }
        }
        let warned_in = warned_in.expect("never warned");
        let short_in = short_in.expect("never short");
        assert!(
            warned_in + 2 <= short_in,
            "seed {seed}: warned {warned_in}, short {short_in}"
        );
        if let Some(empty_in) = empty_in {
            assert!(
                short_in < empty_in,
                "seed {seed}: short {short_in}, empty {empty_in}"
            );
        }
    }
}
