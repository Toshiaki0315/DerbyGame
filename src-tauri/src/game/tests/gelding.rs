//! Gelding a horse, and racing and retiring as a gelding (去勢).

use super::*;

fn colt() -> (Game, String) {
    let mut g = Game::new();
    let i = g.field.iter().position(|h| h.sex == Sex::Male).unwrap();
    g.field[i].age = 4;
    g.field[i].temper = 30;
    let name = g.field[i].name.clone();
    (g, name)
}

fn horse<'a>(g: &'a Game, name: &str) -> &'a StableHorse {
    g.field.iter().find(|h| h.name == name).unwrap()
}

#[test]
fn gelding_calms_a_colt_costs_money_and_takes_time_to_get_over() {
    let (mut g, name) = colt();
    let money = g.money;
    g.geld(&name).unwrap();
    let h = horse(&g, &name);
    assert!(h.gelded);
    assert_eq!(h.temper, 30 + GELDING_CALM);
    assert_eq!(h.away.as_ref().unwrap().kind, AwayKind::Pasture);
    assert_eq!(h.away.as_ref().unwrap().weeks, GELDING_WEEKS);
    assert_eq!(g.money, money - GELDING_COST);
}

#[test]
fn temperament_stays_on_the_scale() {
    let (mut g, name) = colt();
    let i = g.slot(&name).unwrap();
    g.field[i].temper = 95;
    g.geld(&name).unwrap();
    assert_eq!(horse(&g, &name).temper, 100);
}

#[test]
fn only_a_colt_can_be_gelded_and_only_once() {
    let (mut g, name) = colt();
    let mare = g
        .field
        .iter()
        .find(|h| h.sex == Sex::Female)
        .unwrap()
        .name
        .clone();
    assert_eq!(g.geld(&mare), Err(GameError::WrongSex));
    assert_eq!(g.geld("ダレモイナイ"), Err(GameError::UnknownHorse));
    g.geld(&name).unwrap();
    assert_eq!(g.geld(&name), Err(GameError::AlreadyGelded));
}

#[test]
fn a_horse_away_or_a_stable_short_of_money_cannot_have_it_done() {
    let (mut g, name) = colt();
    g.send_to_pasture(&name).unwrap();
    assert_eq!(g.geld(&name), Err(GameError::Away));
    let (mut g, name) = colt();
    g.money = GELDING_COST;
    assert_eq!(g.geld(&name), Err(GameError::InsufficientFunds));
    assert!(!horse(&g, &name).gelded);
}

#[test]
fn a_gelding_retires_without_going_to_stud() {
    let (mut g, name) = colt();
    let studs = g.studs.len();
    g.geld(&name).unwrap();
    g.retire(&name).unwrap();
    assert!(g.field.iter().all(|h| h.name != name));
    assert_eq!(g.studs.len(), studs);
}

#[test]
fn an_old_gelding_leaves_at_the_turn_of_the_year_and_no_stud_makes_way_for_it() {
    let (mut g, name) = colt();
    g.geld(&name).unwrap();
    while g.studs.len() < MAX_STUDS {
        let m = g
            .field
            .iter()
            .find(|h| h.name != name)
            .unwrap()
            .name
            .clone();
        g.retire(&m).unwrap();
    }
    let longest = g.studs[0].name.clone();
    let i = g.slot(&name).unwrap();
    g.field[i].age = RETIRE_AGE;
    g.retire_the_old();
    assert!(g.field.iter().all(|h| h.name != name));
    assert!(g.studs.iter().any(|s| s.name == longest));
    assert!(
        g.news
            .iter()
            .any(|n| n.contains(&name) && n.contains("引退")),
        "{:?}",
        g.news
    );
}

#[test]
fn the_classics_are_closed_to_geldings() {
    let race = calendar_race(TRIPLE_CROWN[0]);
    let (mut g, name) = colt();
    let i = g.slot(&name).unwrap();
    g.week = race.week;
    g.field[i].age = 3;
    g.field[i].fatigue = 0;
    g.field[i].record = RaceRecord { starts: 3, wins: 2 };
    assert!(g.eligible_horses(&race).contains(&i));
    g.geld(&name).unwrap();
    g.field[i].away = None;
    assert!(!g.eligible_horses(&race).contains(&i));
    let card = g.race_card(Some(&RaceKey::of(&race))).unwrap();
    let mine = card.my_horses.iter().find(|h| h.name == name).unwrap();
    assert_eq!(mine.barred.as_deref(), Some("セン馬不可"));
}

#[test]
fn geldings_may_run_in_other_big_races() {
    let race = calendar::shared()
        .races
        .iter()
        .find(|r| {
            r.grade == Grade::G1
                && r.eligibility.age == AgeRule::ThreeUp
                && !r.eligibility.fillies_only
        })
        .unwrap()
        .clone();
    let (mut g, name) = colt();
    let i = g.slot(&name).unwrap();
    g.week = race.week;
    g.field[i].record = RaceRecord { starts: 3, wins: 2 };
    g.geld(&name).unwrap();
    g.field[i].away = None;
    assert!(g.eligible_horses(&race).contains(&i));
}

#[test]
fn the_screen_is_told_who_is_a_gelding() {
    let (mut g, name) = colt();
    g.geld(&name).unwrap();
    let v = serde_json::to_value(g.state()).unwrap();
    let h = v["field"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["name"] == name.as_str())
        .unwrap();
    assert_eq!(h["gelded"], true);
    assert_eq!(v["gelding_cost"], GELDING_COST);
    let i = g.slot(&name).unwrap();
    g.field[i].away = None;
    g.set_entries(None, std::slice::from_ref(&name)).unwrap();
    let card = g.race_card(None).unwrap();
    assert!(card.entrants.iter().any(|e| e.name == name && e.gelded));
}

#[test]
fn being_a_gelding_survives_a_save_and_old_saves_have_none() {
    let (mut g, name) = colt();
    g.geld(&name).unwrap();
    let back = Game::from_json(&g.to_json()).unwrap();
    assert!(horse(&back, &name).gelded);
    let old = Game::from_json(&old_save(&Game::new()).to_string()).unwrap();
    assert!(old.field.iter().all(|h| !h.gelded));
    let mut save: serde_json::Value = serde_json::from_str(&Game::new().to_json()).unwrap();
    save["field"][0].as_object_mut().unwrap().remove("gelded");
    assert!(!Game::from_json(&save.to_string()).unwrap().field[0].gelded);
}
