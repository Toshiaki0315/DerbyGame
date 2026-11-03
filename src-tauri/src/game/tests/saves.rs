//! The save format, and the saves older versions left behind (セーブ形式と旧セーブ).

use super::*;

/// A save written by version 1 after a year and a half left to the trainers.
const VERSION_1_SAVE: &str = include_str!("save_version_1.json");

#[test]
fn a_new_save_keeps_each_horses_details_with_the_horse() {
    let g = Game::new();
    let save: serde_json::Value = serde_json::from_str(&g.to_json()).unwrap();
    assert_eq!(save["save_version"], SAVE_VERSION);
    assert_eq!(save["field"][0]["age"], g.field[0].age);
    assert_eq!(save["field"][0]["blood_id"], g.field[0].blood_id);
    for list in ["ages", "trainings", "fatigue", "blood_ids", "careers"] {
        assert!(
            save.get(list).is_none(),
            "{list} is still a list of its own"
        );
    }
}

#[test]
fn a_save_keeps_everything_about_every_horse() {
    let mut g = Game::with_seed(7);
    g.set_all_auto(true);
    for _ in 0..60 {
        g.advance_week();
    }
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(
        serde_json::to_value(&back.field).unwrap(),
        serde_json::to_value(&g.field).unwrap()
    );
    assert_eq!(
        serde_json::to_value(back.state()).unwrap(),
        serde_json::to_value(g.state()).unwrap()
    );
}

#[test]
fn a_real_version_1_save_loads_with_every_list_on_the_right_horse() {
    let old: serde_json::Value = serde_json::from_str(VERSION_1_SAVE).unwrap();
    let g = Game::from_json(VERSION_1_SAVE).unwrap();
    let horses = serde_json::to_value(&g.field).unwrap();
    assert_eq!(
        horses.as_array().unwrap().len(),
        old["field"].as_array().unwrap().len()
    );
    assert!(g.field.len() > 1);
    let lists = [
        ("trainings", "trainings"),
        ("conditions", "condition"),
        ("sexes", "sex"),
        ("blood_ids", "blood_id"),
        ("ages", "age"),
        ("dirts", "dirt"),
        ("powers", "power"),
        ("tempers", "temper"),
        ("dirt_progress", "dirt_progress"),
        ("records", "record"),
        ("gear", "gear"),
        ("fatigue", "fatigue"),
        ("resting", "resting"),
        ("auto_training", "auto_training"),
        ("auto_entry", "auto_entry"),
        ("trainer_of", "trainer"),
        ("since_race", "since_race"),
        ("legs", "legs"),
        ("away", "away"),
        ("jockey_of", "jockey"),
        ("partner", "partner"),
        ("careers", "career"),
    ];
    for i in 0..g.field.len() {
        assert_eq!(horses[i]["name"], old["field"][i]["name"]);
        assert_eq!(horses[i]["speed"], old["field"][i]["speed"]);
        assert_eq!(horses[i]["stamina"], old["field"][i]["stamina"]);
        for (list, key) in lists {
            assert_eq!(horses[i][key], old[list][i], "{list} of horse {i}");
        }
    }
    assert_eq!(g.money as u64, old["money"].as_u64().unwrap());
    assert_eq!(g.year as u64, old["year"].as_u64().unwrap());
}

#[test]
fn a_version_1_save_is_written_back_in_the_new_format() {
    let g = Game::from_json(VERSION_1_SAVE).unwrap();
    let again: serde_json::Value = serde_json::from_str(&g.to_json()).unwrap();
    assert_eq!(again["save_version"], SAVE_VERSION);
    assert!(again.get("trainings").is_none());
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(
        serde_json::to_value(back.state()).unwrap(),
        serde_json::to_value(g.state()).unwrap()
    );
}

#[test]
fn a_game_goes_on_from_a_version_1_save() {
    let mut g = Game::from_json(VERSION_1_SAVE).unwrap();
    let (year, week) = (g.year, g.week);
    g.advance_week();
    assert!(g.year > year || g.week > week);
}

#[test]
fn a_save_made_by_a_newer_version_is_refused() {
    let mut save: serde_json::Value = serde_json::from_str(&Game::new().to_json()).unwrap();
    save["save_version"] = serde_json::json!(SAVE_VERSION + 1);
    let err = Game::from_json(&save.to_string()).err().unwrap();
    assert!(err.contains("新しいバージョン"), "{err}");
}

#[test]
fn a_horse_missing_part_of_its_details_spoils_a_new_save() {
    let mut save: serde_json::Value = serde_json::from_str(&Game::new().to_json()).unwrap();
    save["field"][0].as_object_mut().unwrap().remove("age");
    let err = Game::from_json(&save.to_string()).err().unwrap();
    assert!(err.contains("セーブデータが不正です"), "{err}");
}

#[test]
fn a_version_1_save_with_a_list_too_short_names_the_list() {
    for list in ["fatigue", "ages", "careers", "trainings"] {
        let mut save = old_save(&Game::new());
        save[list].as_array_mut().unwrap().pop();
        let err = Game::from_json(&save.to_string()).err().unwrap();
        assert!(err.contains(list), "{list}: {err}");
    }
}

#[test]
fn things_that_are_not_saves_are_refused_rather_than_crashing() {
    for text in [
        "",
        "[]",
        "[1, 2]",
        "3",
        "null",
        "\"a\"",
        "{}",
        "{\"field\": 3}",
    ] {
        let err = Game::from_json(text)
            .err()
            .unwrap_or_else(|| panic!("{text} loaded"));
        assert!(err.contains("セーブデータが不正です"), "{text}: {err}");
    }
}
