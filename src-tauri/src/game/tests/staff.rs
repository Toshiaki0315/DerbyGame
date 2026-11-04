//! Jockeys and trainers coming and going (騎手・調教師の入れ替わり).

use super::*;

/// Goes on to the turn of the year, however many times.
fn years_pass(g: &mut Game, years: u32) {
    for _ in 0..years {
        g.week = calendar::WEEKS_PER_YEAR;
        g.money = 100_000;
        g.advance_week();
    }
}

fn active_jockeys(g: &Game) -> Vec<u8> {
    g.jockeys_now().iter().map(|j| j.id).collect()
}

fn active_trainers(g: &Game) -> Vec<u8> {
    g.trainers_now().iter().map(|t| t.id).collect()
}

#[test]
fn a_new_game_starts_with_the_familiar_faces() {
    let g = Game::new();
    assert_eq!(active_jockeys(&g), (0..10).collect::<Vec<u8>>());
    assert_eq!(active_trainers(&g), (0..10).collect::<Vec<u8>>());
}

#[test]
fn over_the_years_jockeys_and_trainers_retire_and_others_come_in() {
    let mut g = Game::new();
    years_pass(&mut g, 20);
    assert!(
        active_jockeys(&g).iter().any(|&id| id >= 10),
        "{:?}",
        active_jockeys(&g)
    );
    assert!(
        active_trainers(&g).iter().any(|&id| id >= 10),
        "{:?}",
        active_trainers(&g)
    );
}

#[test]
fn the_number_of_each_and_the_balance_of_east_and_west_stays_the_same() {
    let mut g = Game::with_seed(5);
    for _ in 0..30 {
        years_pass(&mut g, 1);
        assert_eq!(g.jockeys_now().len(), 10);
        assert_eq!(g.trainers_now().len(), 10);
        let east = g
            .jockeys_now()
            .iter()
            .filter(|j| j.region == course::Region::East)
            .count();
        assert_eq!(east, 5, "year {}", g.year);
        let east = g
            .trainers_now()
            .iter()
            .filter(|t| t.region == course::Region::East)
            .count();
        assert_eq!(east, 5, "year {}", g.year);
    }
}

#[test]
fn nobody_comes_back_once_they_have_retired() {
    let mut g = Game::with_seed(5);
    let mut gone = std::collections::HashSet::new();
    let mut last = active_jockeys(&g);
    for _ in 0..30 {
        years_pass(&mut g, 1);
        let now = active_jockeys(&g);
        for id in &last {
            if !now.contains(id) {
                gone.insert(*id);
            }
        }
        assert!(now.iter().all(|id| !gone.contains(id)));
        last = now;
    }
    assert!(!gone.is_empty());
}

#[test]
fn the_same_seed_brings_the_same_changes() {
    let (mut a, mut b) = (Game::with_seed(3), Game::with_seed(3));
    years_pass(&mut a, 15);
    years_pass(&mut b, 15);
    assert_eq!(active_jockeys(&a), active_jockeys(&b));
    assert_eq!(active_trainers(&a), active_trainers(&b));
}

#[test]
fn a_new_jockey_starts_green_and_gets_better() {
    let mut g = Game::with_seed(5);
    while active_jockeys(&g).iter().all(|&id| id < 10) {
        years_pass(&mut g, 1);
    }
    let id = *active_jockeys(&g).iter().find(|&&id| id >= 10).unwrap();
    let best = jockey::jockey_by_id(id).unwrap().skill;
    let skill = |g: &Game| g.jockeys_now().iter().find(|j| j.id == id).unwrap().skill;
    assert!(skill(&g) < best);
    let first = skill(&g);
    years_pass(&mut g, 1);
    assert!(skill(&g) > first);
    years_pass(&mut g, 4);
    assert_eq!(skill(&g), best);
}

#[test]
fn the_news_says_who_has_gone_and_who_has_come() {
    let mut g = Game::with_seed(5);
    let mut said = (false, false);
    for _ in 0..30 {
        g.week = calendar::WEEKS_PER_YEAR;
        g.money = 100_000;
        g.advance_week();
        said.0 |= g
            .news
            .iter()
            .any(|n| n.contains("騎手") && n.contains("引退") && n.contains("デビュー"));
        said.1 |= g
            .news
            .iter()
            .any(|n| n.contains("調教師") && n.contains("引退") && n.contains("厩舎"));
    }
    assert!(said.0 && said.1);
}

#[test]
fn a_horse_whose_jockey_retires_goes_back_to_its_trainers_booking() {
    let mut g = Game::with_seed(5);
    let first = active_jockeys(&g);
    let name = g.field[0].name.clone();
    while active_jockeys(&g) == first {
        // keep on the one who goes next
        for &id in &first {
            g.set_jockey(&name, Some(id)).unwrap();
            if g.jockey_staff
                .iter()
                .any(|s| s.id == id && s.until == g.year + 1)
            {
                break;
            }
        }
        let chosen = g.field[0].jockey;
        years_pass(&mut g, 1);
        if g.field.iter().all(|h| h.name != name) {
            return;
        }
        if let Some(id) = chosen {
            if !active_jockeys(&g).contains(&id) {
                assert_eq!(g.field[0].jockey, None);
                return;
            }
        }
    }
}

#[test]
fn only_a_jockey_or_trainer_still_working_can_be_chosen() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    assert_eq!(g.set_jockey(&name, Some(15)), Err(GameError::UnknownJockey));
    assert_eq!(g.set_trainer(&name, 15), Err(GameError::UnknownTrainer));
}

#[test]
fn a_new_trainer_takes_over_the_stable_of_one_who_retires() {
    let mut g = Game::with_seed(5);
    let old: Vec<u8> = active_trainers(&g);
    let horses = |g: &Game| g.field.len();
    let n = horses(&g);
    while active_trainers(&g) == old {
        for h in &mut g.field {
            h.age = 3;
        }
        years_pass(&mut g, 1);
    }
    assert_eq!(horses(&g), n);
    let now = active_trainers(&g);
    assert!(
        g.field.iter().all(|h| now.contains(&h.trainer)),
        "{:?}",
        g.all(|h| h.trainer)
    );
}

#[test]
fn every_trainer_has_a_record_and_a_new_one_starts_at_nothing() {
    let g = Game::new();
    assert_eq!(g.trainer_records.len(), trainer::ALL_TRAINERS);
}

#[test]
fn horses_run_for_a_new_trainer_and_are_ridden_by_a_new_jockey_after_a_save() {
    let mut g = Game::with_seed(5);
    g.set_all_auto(true);
    years_pass(&mut g, 12);
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(active_jockeys(&back), active_jockeys(&g));
    assert_eq!(active_trainers(&back), active_trainers(&g));
    assert_eq!(back.jockey_staff, g.jockey_staff);
}

#[test]
fn a_save_from_before_the_changes_starts_with_the_familiar_faces() {
    let mut save: serde_json::Value = serde_json::from_str(&Game::new().to_json()).unwrap();
    let o = save.as_object_mut().unwrap();
    o.remove("jockey_staff");
    o.remove("trainer_staff");
    o.insert(
        "trainer_records".to_string(),
        serde_json::json!(vec![TrainerRecord::default(); 10]),
    );
    let g = Game::from_json(&save.to_string()).unwrap();
    assert_eq!(active_jockeys(&g), (0..10).collect::<Vec<u8>>());
    assert_eq!(g.trainer_records.len(), trainer::ALL_TRAINERS);
    let old = Game::from_json(&old_save(&Game::new()).to_string()).unwrap();
    assert_eq!(active_trainers(&old), (0..10).collect::<Vec<u8>>());
}

#[test]
fn the_screen_is_shown_those_working_now() {
    let mut g = Game::with_seed(5);
    years_pass(&mut g, 25);
    let st = g.state();
    let ids: Vec<u8> = st.jockeys.iter().map(|j| j.id).collect();
    assert_eq!(ids, active_jockeys(&g));
    let ids: Vec<u8> = st.trainers.iter().map(|t| t.id).collect();
    assert_eq!(ids, active_trainers(&g));
}
