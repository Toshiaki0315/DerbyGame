//! Retirement and the stud book (引退・繁殖馬).

use super::*;

// ---- retirement (引退) and the stud book (繁殖) ----

fn retire_ok(g: &mut Game, name: &str) {
    g.retire(name)
        .unwrap_or_else(|e| panic!("retire {name}: {e}"));
}

#[test]
fn a_retired_horse_leaves_the_racing_stable_and_joins_the_stud_book() {
    let mut g = Game::new();
    let i = 3;
    let (name, horse, sex, age, power, dirt, id) = (
        g.field[i].name.clone(),
        g.field[i].clone(),
        g.field[i].sex,
        g.field[i].age,
        g.field[i].power,
        g.field[i].dirt,
        g.field[i].blood_id,
    );
    let before = g.field.len();
    retire_ok(&mut g, &name);
    assert_eq!(g.field.len(), before - 1);
    assert!(g.field.iter().all(|h| h.name != name));
    let stud = g.studs.iter().find(|s| s.name == name).unwrap();
    assert_eq!(
        (
            stud.speed,
            stud.stamina,
            stud.sex,
            stud.retired_age,
            stud.power,
            stud.dirt,
            stud.blood_id
        ),
        (horse.speed, horse.stamina, sex, age, power, dirt, id)
    );
    assert_eq!(stud.retired_year, g.year);
}

#[test]
fn a_retired_horse_no_longer_trains_or_runs() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    retire_ok(&mut g, &name);
    assert_eq!(
        g.workout(&name, Workout::Wood),
        Err(GameError::UnknownHorse)
    );
    let card = g.race_card(None).unwrap();
    assert!(card.entrants.iter().all(|e| e.name != name));
    assert!(card.my_horses.iter().all(|h| h.name != name));
}

#[test]
fn retiring_takes_the_horse_out_of_this_weeks_entries() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.set_entries(None, std::slice::from_ref(&name)).unwrap();
    retire_ok(&mut g, &name);
    assert!(g.entries.values().all(|v| !v.contains(&name)));
}

#[test]
fn the_last_racehorse_cannot_be_retired() {
    let mut g = Game::new();
    g.studs.clear();
    while g.field.len() > 1 {
        let name = g.field[0].name.clone();
        if g.studs.len() == MAX_STUDS {
            let s = g.studs[0].name.clone();
            g.release_stud(&s).unwrap();
        }
        retire_ok(&mut g, &name);
    }
    let last = g.field[0].name.clone();
    assert_eq!(g.retire(&last), Err(GameError::LastHorse));
    assert_eq!(g.retire("いない馬"), Err(GameError::UnknownHorse));
}

#[test]
fn the_stud_book_has_room_for_six_and_a_stud_can_be_released() {
    let mut g = Game::new();
    g.studs.clear(); // without the starting pair
    for _ in 0..MAX_STUDS {
        let name = g.field[0].name.clone();
        retire_ok(&mut g, &name);
    }
    let next = g.field[0].name.clone();
    assert_eq!(g.retire(&next), Err(GameError::StudFull));
    let first = g.studs[0].name.clone();
    let id = g.studs[0].blood_id;
    g.release_stud(&first).unwrap();
    assert_eq!(g.studs.len(), MAX_STUDS - 1);
    // the record stays in the database under its name, no longer owned
    let rec = g.blood_record(id).unwrap();
    assert_eq!(rec.name, first);
    assert!(!rec.owned);
    retire_ok(&mut g, &next);
    assert_eq!(g.release_stud("いない馬"), Err(GameError::UnknownHorse));
}

#[test]
fn studs_stay_owned_in_the_database() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let id = g.field[0].blood_id;
    retire_ok(&mut g, &name);
    assert!(g.blood_record(id).unwrap().owned);
    assert!(g.bloodline_all().iter().any(|h| h.id == id && h.owned));
}

#[test]
fn a_stud_can_sire_a_foal_that_fills_an_empty_place_in_the_stable() {
    let mut g = Game::new();
    let (sire, dam) = stallion_and_mare(&g);
    let sire_id = g.field[g.field.iter().position(|h| h.name == sire).unwrap()].blood_id;
    retire_ok(&mut g, &sire);
    let before = g.field.len();
    g.breed_now(&sire, &dam, "", "ハナミズキ").unwrap();
    assert_eq!(g.field.len(), before + 1);
    let i = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    assert_eq!(g.field[i].age, 2);
    let foal = g.blood_record(g.field[i].blood_id).unwrap();
    assert_eq!(foal.sire_id, Some(sire_id));
    assert!(g.state().field[i].pedigree.is_some());
}

#[test]
fn a_mare_in_the_stud_book_can_be_a_dam() {
    let mut g = Game::new();
    let (sire, dam) = stallion_and_mare(&g);
    retire_ok(&mut g, &dam);
    assert!(g.breeding_outlook(&sire, &dam).is_ok());
    g.breed_now(&sire, &dam, "", "ハルノソラ").unwrap();
    assert!(g.field.iter().any(|h| h.name == "ハルノソラ"));
}

#[test]
fn a_full_stable_needs_a_horse_to_make_room_for_a_foal() {
    let mut g = Game::new();
    let (sire, dam) = stallion_and_mare(&g);
    fill_stables(&mut g, &sire, &dam);
    assert_eq!(g.field.len(), STABLE_SIZE);
    let money = g.money;
    assert_eq!(
        g.breed_now(&sire, &dam, "", "ハナミズキ"),
        Err(GameError::StableFull)
    );
    assert_eq!(g.money, money);
}

#[test]
fn a_foal_cannot_take_the_name_of_a_stud() {
    let mut g = Game::new();
    let (sire, dam) = stallion_and_mare(&g);
    let other = g
        .field
        .iter()
        .find(|h| h.name != sire && h.name != dam)
        .unwrap()
        .name
        .clone();
    retire_ok(&mut g, &other);
    assert_eq!(
        g.breed_now(&sire, &dam, "", &other),
        Err(GameError::DuplicateName)
    );
}

#[test]
fn horses_retire_to_stud_when_they_turn_nine() {
    let mut g = Game::new();
    g.field[1].age = RETIRE_AGE - 1;
    let old = g.field[1].name.clone();
    g.week = calendar::WEEKS_PER_YEAR;
    g.advance_week();
    assert!(g.field.iter().all(|h| h.name != old));
    assert!(g
        .studs
        .iter()
        .any(|s| s.name == old && s.retired_age == RETIRE_AGE));
    assert!(g
        .news
        .iter()
        .any(|n| n == &format!("{old}が引退し、繁殖入りしました。")));
    // the news is for that week only
    g.advance_week();
    assert!(g.news.is_empty());
}

#[test]
fn a_full_stud_book_lets_its_longest_serving_stud_go_for_a_new_retiree() {
    let mut g = Game::new();
    g.studs.clear(); // without the starting pair
    for _ in 0..MAX_STUDS {
        let name = g.field[0].name.clone();
        retire_ok(&mut g, &name);
    }
    let first = g.studs[0].name.clone();
    g.field[0].age = RETIRE_AGE - 1;
    let old = g.field[0].name.clone();
    g.week = calendar::WEEKS_PER_YEAR;
    g.advance_week();
    assert_eq!(g.studs.len(), MAX_STUDS);
    assert!(g.studs.iter().all(|s| s.name != first));
    assert!(g.studs.iter().any(|s| s.name == old));
    assert!(g
        .news
        .iter()
        .any(|n| n == &format!("{first}は繁殖を引退しました。")));
}

#[test]
fn the_whole_racing_stable_may_retire_at_once_and_races_go_on() {
    let mut g = Game::new();
    for h in &mut g.field {
        h.age = RETIRE_AGE - 1;
    }
    g.week = calendar::WEEKS_PER_YEAR;
    g.advance_week();
    assert!(g.field.is_empty());
    let card = g.race_card(None).unwrap();
    assert!(card.entrants.iter().all(|e| !e.mine));
    let rival = card.entrants[0].name.clone();
    g.play_combo(None, &[rival], 10, 1, BetKind::Win).unwrap();
    // the stud book can still breed a new horse into the empty stable
    let s = g
        .studs
        .iter()
        .find(|s| s.sex == Sex::Male)
        .unwrap()
        .name
        .clone();
    let d = g
        .studs
        .iter()
        .find(|s| s.sex == Sex::Female)
        .unwrap()
        .name
        .clone();
    g.breed_now(&s, &d, "", "アラタナミチ").unwrap();
    assert_eq!(g.field.len(), 1);
}

#[test]
fn studs_and_news_survive_a_save_and_old_saves_have_none() {
    let mut g = Game::new();
    let name = g.field[2].name.clone();
    retire_ok(&mut g, &name);
    g.news = vec!["テスト".into()];
    let back = Game::from_json(&serde_json::to_string(&g).unwrap()).unwrap();
    assert_eq!(back.studs, g.studs);
    assert_eq!(back.state(), g.state());
    let mut v = old_save(&Game::new());
    v.as_object_mut().unwrap().remove("studs");
    v.as_object_mut().unwrap().remove("news");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert!(old.studs.is_empty() && old.news.is_empty());
}

#[test]
fn a_save_with_a_stud_unknown_to_the_database_is_refused() {
    let mut g = Game::new();
    let name = g.field[2].name.clone();
    retire_ok(&mut g, &name);
    g.studs[0].blood_id = 999_999;
    assert!(Game::from_json(&serde_json::to_string(&g).unwrap()).is_err());
}

#[test]
fn the_state_json_lists_the_studs_and_the_news() {
    let mut g = Game::new();
    let name = g.field[2].name.clone();
    retire_ok(&mut g, &name);
    let v = serde_json::to_value(g.state()).unwrap();
    let s = &v["studs"][0];
    for k in [
        "name",
        "sex",
        "speed",
        "stamina",
        "power",
        "dirt",
        "retired_age",
        "retired_year",
    ] {
        assert!(!s[k].is_null(), "{k}");
    }
    assert!(s["blood"]["lineage"].as_str().is_some());
    assert!(v["news"].as_array().is_some());
    assert_eq!(v["max_studs"], MAX_STUDS);
    assert_eq!(v["stable_size"], STABLE_SIZE);
}
