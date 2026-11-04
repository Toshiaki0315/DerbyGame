//! Body weight and the extra training menus (馬体重と調教メニュー).

use super::*;

#[test]
fn every_horse_starts_at_its_best_weight() {
    let g = Game::new();
    for h in &g.field {
        assert_eq!(h.body_weight, ideal_weight(h.blood_id));
        assert!((430..=530).contains(&h.body_weight), "{}", h.body_weight);
    }
}

#[test]
fn a_horse_near_its_best_weight_runs_as_well_as_it_can_and_one_far_off_does_not() {
    assert_eq!(body_form(480, 480), 1.0);
    assert_eq!(body_form(488, 480), 1.0);
    assert_eq!(body_form(472, 480), 1.0);
    assert!(body_form(500, 480) < 1.0);
    assert!(body_form(460, 480) < 1.0);
    assert!(body_form(520, 480) < body_form(500, 480));
    assert!(body_form(700, 480) >= BODY_FORM_FLOOR);
}

#[test]
fn a_heavy_horse_runs_slower_in_a_race() {
    let mut g = Game::new();
    g.field[0].body_weight = ideal_weight(g.field[0].blood_id) + 40;
    let heavy = g.stable_entrant(0);
    g.field[0].body_weight = ideal_weight(g.field[0].blood_id);
    let right = g.stable_entrant(0);
    let speed = |e: &crate::rivals::Entrant| {
        Game::run_stats(e, Surface::Turf, 2000, TrackCondition::Good).0
    };
    assert!(speed(&heavy) < speed(&right));
}

#[test]
fn hard_work_takes_weight_off_and_walking_does_not() {
    let mut g = Game::new();
    g.money = 100_000;
    let name = g.field[0].name.clone();
    let before = g.field[0].body_weight;
    g.workout(&name, Workout::Hill).unwrap();
    assert!(g.field[0].body_weight < before);
    let before = g.field[0].body_weight;
    g.workout(&name, Workout::Walk).unwrap();
    assert_eq!(g.field[0].body_weight, before);
}

#[test]
fn a_race_takes_weight_off_and_the_card_shows_the_change_since_the_last_race() {
    let r = calendar::shared()
        .races
        .iter()
        .find(|r| {
            r.grade == Grade::Open
                && r.surface != Surface::Jump
                && r.eligibility.age == AgeRule::ThreeUp
        })
        .unwrap()
        .clone();
    let mut g = Game::new();
    g.week = r.week;
    let key = RaceKey::of(&r);
    let i = (0..g.field.len())
        .find(|&i| {
            r.eligibility
                .allows(g.field[i].age, g.field[i].sex == Sex::Female)
        })
        .unwrap();
    let name = g.field[i].name.clone();
    g.field[i].last_race_weight = g.field[i].body_weight - 6;
    g.set_entries(Some(&key), std::slice::from_ref(&name))
        .unwrap();
    let card = g.race_card(Some(&key)).unwrap();
    let me = card.entrants.iter().find(|e| e.name == name).unwrap();
    assert_eq!(me.body_weight, g.field[i].body_weight);
    assert_eq!(me.body_change, 6);
    assert!(
        card.entrants
            .iter()
            .all(|e| (400..=560).contains(&e.body_weight)),
        "{:?}",
        card.entrants
            .iter()
            .map(|e| e.body_weight)
            .collect::<Vec<_>>()
    );
    let spec = g.race_spec(Some(&key)).unwrap();
    let before = g.field[i].body_weight;
    g.tire_runners(&spec, 10);
    assert!(g.field[i].body_weight < before);
    assert_eq!(g.field[i].last_race_weight, before);
}

#[test]
fn weight_comes_back_to_where_it_should_be_week_by_week() {
    let mut g = Game::new();
    let best = ideal_weight(g.field[0].blood_id);
    g.field[0].body_weight = best - 10;
    g.advance_week();
    let a = g.field[0].body_weight;
    assert!(a > best - 10, "{a}");
    g.field[0].body_weight = best + 10;
    g.advance_week();
    assert!(g.field[0].body_weight < best + 10);
    g.field[0].body_weight = best;
    g.advance_week();
    assert_eq!(g.field[0].body_weight, best);
}

#[test]
fn a_foal_starts_light_and_grows_into_its_weight() {
    let (mut g, s, d, _) = in_season();
    g.money = 100_000;
    g.breed(&s, &d, "", "ハナミズキ").unwrap();
    weeks_pass(&mut g, GESTATION_WEEKS);
    if let Some(h) = g.field.iter().find(|h| h.name == "ハナミズキ") {
        assert_eq!(h.body_weight, ideal_weight(h.blood_id));
    }
}

#[test]
fn the_new_menus_work_a_horse_their_own_way() {
    let mut g = Game::new();
    g.money = 100_000;
    let name = g.field[0].name.clone();
    g.field[0].speed = 50;
    g.field[0].stamina = 50;
    g.field[0].temper = 50;
    g.workout(&name, Workout::Poly).unwrap();
    assert_eq!((g.field[0].speed, g.field[0].stamina), (52, 52));
    assert!(Workout::Poly.fatigue() < Workout::Wood.fatigue());
    g.workout(&name, Workout::Gate).unwrap();
    assert_eq!(g.field[0].temper, 52);
    assert_eq!(g.field[0].speed, 53);
    assert!(Workout::Gate.fatigue() < Workout::Poly.fatigue());
}

#[test]
fn the_screen_is_told_the_weight_and_old_saves_start_at_the_best_weight() {
    let g = Game::new();
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["field"][0]["body_weight"], g.field[0].body_weight);
    assert_eq!(
        v["field"][0]["ideal_weight"],
        ideal_weight(g.field[0].blood_id)
    );
    let old = Game::from_json(&old_save(&Game::new()).to_string()).unwrap();
    assert!(old
        .field
        .iter()
        .all(|h| h.body_weight == ideal_weight(h.blood_id)));
    let mut save: serde_json::Value = serde_json::from_str(&Game::new().to_json()).unwrap();
    save["save_version"] = serde_json::json!(2);
    for h in save["field"].as_array_mut().unwrap() {
        let o = h.as_object_mut().unwrap();
        o.remove("body_weight");
        o.remove("last_race_weight");
        o.remove("specials");
    }
    let two = Game::from_json(&save.to_string()).unwrap();
    assert!(two
        .field
        .iter()
        .all(|h| h.body_weight == ideal_weight(h.blood_id)));
    assert_eq!(two.save_version, SAVE_VERSION);
}
