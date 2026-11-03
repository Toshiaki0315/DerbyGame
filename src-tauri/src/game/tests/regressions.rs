//! Things found by reviews and the coverage report.

use super::*;

// ---- review fixes ----

#[test]
fn a_place_ticket_pays_the_odds_the_card_shows() {
    let g = Game::with_seed(1);
    let card = g.race_card(None).unwrap();
    let outsider = card
        .entrants
        .iter()
        .max_by(|a, b| a.place_odds.total_cmp(&b.place_odds))
        .unwrap();
    let mut spec = g.race_spec(None).unwrap();
    // a runner with no chance at all: the shown odds are at their cap
    spec.entrants
        .iter_mut()
        .find(|e| e.name == outsider.name)
        .unwrap()
        .speed = 1;
    let table = Game::odds_table_for(&spec);
    let at = spec
        .entrants
        .iter()
        .position(|e| e.name == outsider.name)
        .unwrap();
    assert_eq!(table[at].1, MAX_PLACE_ODDS);
    let priced = Game::priced(&spec);
    assert_eq!(
        Game::ticket_odds(&priced, BetKind::Place, &[at]),
        table[at].1
    );
}

#[test]
fn winnings_never_overflow_the_purse() {
    let mut g = Game::with_seed(1);
    g.money = u32::MAX - 5;
    g.earn(100);
    assert_eq!(g.money, u32::MAX);
    g.money = 10;
    g.earn(u32::MAX as i64 + 7);
    assert_eq!(g.money, u32::MAX);
}

#[test]
fn a_horse_that_leaves_the_stable_leaves_nothing_behind() {
    let mut g = Game::with_seed(1);
    let name = g.field[0].name.clone();
    g.orders.insert(name.clone(), Style::Nige);
    g.priority.push(name.clone());
    g.trial_rights.push(TrialRight {
        horse: name.clone(),
        year: g.year,
        race: "疾風賞".to_string(),
    });
    g.sell_horse(&name).unwrap();
    assert!(!g.orders.contains_key(&name));
    assert!(!g.priority.contains(&name));
    assert!(g.trial_rights.iter().all(|t| t.horse != name));
}

/// A pregnancy about to deliver, for a place held by `replace`.
fn foal_for(g: &mut Game, replace: &str) -> Pregnancy {
    let mut foal = g.blood_record(g.field[0].blood_id).unwrap();
    foal.id = g.next_blood_id();
    foal.name = "アラタナホシ".to_string();
    Pregnancy {
        foal,
        legs: 70,
        sire: "チチ".to_string(),
        dam: "ハハ".to_string(),
        replace: replace.to_string(),
        weeks_left: 0,
    }
}

#[test]
fn a_horse_that_makes_way_for_a_foal_leaves_nothing_behind_either() {
    let mut g = Game::with_seed(1);
    let name = g.field[1].name.clone();
    g.orders.insert(name.clone(), Style::Nige);
    g.priority.push(name.clone());
    let p = foal_for(&mut g, &name);
    g.deliver(p);
    assert!(g.field.iter().all(|h| h.name != name));
    assert!(!g.orders.contains_key(&name));
    assert!(!g.priority.contains(&name));
}

#[test]
fn a_great_horse_that_makes_way_for_a_foal_still_goes_into_the_hall() {
    let mut g = Game::with_seed(1);
    let name = g.field[1].name.clone();
    g.field[1].career.earnings = HALL_EARNINGS;
    let p = foal_for(&mut g, &name);
    g.deliver(p);
    assert!(g.hall.iter().any(|h| h.name == name));
}

#[test]
fn rights_from_the_trials_lapse_with_the_year() {
    let mut g = Game::with_seed(1);
    g.trial_rights.push(TrialRight {
        horse: g.field[0].name.clone(),
        year: g.year,
        race: "疾風賞".to_string(),
    });
    g.week = calendar::WEEKS_PER_YEAR;
    g.advance_week();
    assert!(g.trial_rights.is_empty());
}

#[test]
fn a_foal_born_to_an_empty_stable_is_not_left_to_the_trainer_unasked() {
    let mut g = Game::with_seed(1);
    while g.field.len() > 1 {
        g.remove_slot(0);
    }
    g.remove_slot(0);
    let p = foal_for_empty(&g);
    g.deliver(p);
    assert_eq!(g.field.len(), 1);
    assert!(!g.field[0].auto_training && !g.field[0].auto_entry);
}

fn foal_for_empty(g: &Game) -> Pregnancy {
    let base = g.base_db();
    let mut foal = base[0].clone();
    foal.id = g.next_blood_id();
    foal.name = "アラタナホシ".to_string();
    Pregnancy {
        foal,
        legs: 70,
        sire: "チチ".to_string(),
        dam: "ハハ".to_string(),
        replace: String::new(),
        weeks_left: 0,
    }
}

#[test]
fn a_save_with_an_unknown_expedition_or_trainer_is_refused_not_a_crash_later() {
    let g = Game::with_seed(1);
    let mut v: serde_json::Value = old_save(&g);
    v["expeditions"] = serde_json::json!([{ "horse": "アサギリライズ", "race": 99, "year": 1 }]);
    assert!(Game::from_json(&v.to_string()).is_err());
    let mut v: serde_json::Value = old_save(&g);
    v["trainer_of"][0] = serde_json::json!(200);
    assert!(Game::from_json(&v.to_string()).is_err());
}

#[test]
fn a_save_that_lost_track_of_a_list_says_which() {
    let g = Game::with_seed(1);
    let mut v: serde_json::Value = old_save(&g);
    v["fatigue"] = serde_json::json!([1]);
    let err = Game::from_json(&v.to_string()).unwrap_err();
    assert!(err.contains("fatigue"), "{err}");
}

// ---- things the coverage report showed nobody was checking ----

#[test]
fn every_refusal_has_a_message_of_its_own() {
    use GameError::*;
    let all = [
        UnknownHorse,
        InvalidAmount,
        InsufficientFunds,
        GameOver,
        MaxedOut,
        WrongSex,
        InvalidName,
        DuplicateName,
        NoSuchRace,
        InvalidSelection,
        DuplicateHorse,
        TooManyEntries,
        NotEligible,
        NoTickets,
        TooManyTickets,
        LastHorse,
        StudFull,
        StableFull,
        OutOfSeason,
        MareInFoal,
        DamNotRetired,
        SireNotRetired,
        TooYoung,
        SaleClosed,
        UnknownLot,
        LotClosed,
        FacilityFull,
        UnknownRace,
        BidTooLow,
        UnknownJockey,
        Away,
        NotOnPasture,
        UnknownTrainer,
        TrainerFull,
        NotForSale,
        TooTired,
        Resting,
    ];
    let messages: std::collections::HashSet<String> = all.iter().map(|e| e.to_string()).collect();
    assert_eq!(messages.len(), all.len());
    assert!(messages.iter().all(|m| !m.is_empty()));
}

/// What a horse's trainer says with something set up on it.
fn says(set_up: impl FnOnce(&mut Game)) -> String {
    let mut g = Game::with_seed(1);
    set_up(&mut g);
    g.advice()[0].comment.clone()
}

#[test]
fn a_trainer_says_where_a_horse_that_is_away_has_gone_and_for_how_long() {
    for (kind, label, word) in [
        (AwayKind::Expedition, "遠征", "海外に遠征中です。あと3週"),
        (
            AwayKind::Pasture,
            "放牧",
            "放牧でリフレッシュ中です。あと3週",
        ),
        (AwayKind::Injured, "骨折", "骨折の治療中です。あと3週"),
        (AwayKind::Growing, "牧場", "牧場で育っています。あと3週"),
    ] {
        let said = says(|g| g.field[0].away = away(kind, label));
        assert!(said.contains(word), "{said}");
    }
}

#[test]
fn a_horse_that_is_away_has_no_proposal() {
    let mut g = Game::with_seed(1);
    g.field[0].away = away(AwayKind::Pasture, "放牧");
    assert_eq!(g.advice()[0].proposal, None);
}

#[test]
fn a_trainer_says_a_horse_is_past_its_best() {
    let said = says(|g| {
        let peak = g.stable_entrant(0).peak_age;
        g.field[0].age = peak + 2;
    });
    assert!(said.contains("盛りを過ぎてきました"), "{said}");
}

#[test]
fn a_trainer_says_a_horse_needs_rest_and_proposes_it() {
    let mut g = Game::with_seed(1);
    let rest_from = g.trainer_of_horse(0).training.rest_from();
    g.field[0].fatigue = rest_from.min(AUTO_PASTURE_FATIGUE - 1);
    let a = &g.advice()[0];
    assert!(a.comment.contains("少し疲れが見えます"), "{}", a.comment);
    assert!(a.proposal.is_some());
    let name = g.field[0].name.clone();
    g.accept_advice(&name).unwrap();
    assert!(g.field[0].resting);
}

#[test]
fn a_trainer_gives_a_horse_time_after_a_race() {
    let said = says(|g| {
        let t = trainer::TRAINERS
            .iter()
            .find(|t| t.rotation == trainer::Rotation::Spaced)
            .unwrap();
        g.field[0].trainer = t.id;
        g.field[0].since_race = 1;
        g.field[0].age = 4;
    });
    assert!(said.contains("前走の疲れを抜いて"), "{said}");
}

#[test]
fn a_save_file_that_is_not_there_or_not_a_save_is_refused_with_a_reason() {
    let dir = std::env::temp_dir().join(format!("derby_save_{}", std::process::id()));
    let path = dir.join("nested").join("save.json");
    assert_eq!(
        Game::load_from(&path).unwrap_err(),
        "セーブデータがありません"
    );
    let g = Game::with_seed(3);
    g.save_to(&path).unwrap();
    assert_eq!(Game::load_from(&path).unwrap().state(), g.state());
    std::fs::write(&path, "壊れたファイル").unwrap();
    assert!(Game::load_from(&path)
        .unwrap_err()
        .contains("セーブデータが不正です"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_save_from_a_week_that_does_not_exist_starts_the_year_again() {
    let g = Game::with_seed(1);
    let mut v: serde_json::Value = old_save(&g);
    v["week"] = serde_json::json!(99);
    assert_eq!(Game::from_json(&v.to_string()).unwrap().week, 1);
}

#[test]
fn a_save_whose_horse_has_no_record_in_the_database_is_refused() {
    let g = Game::with_seed(1);
    let mut v: serde_json::Value = old_save(&g);
    v["blood_ids"][0] = serde_json::json!(999_999_999u32);
    assert!(Game::from_json(&v.to_string())
        .unwrap_err()
        .contains("血統データベース"));
    let mut v: serde_json::Value = old_save(&g);
    v["studs"][0]["blood_id"] = serde_json::json!(999_999_999u32);
    assert!(Game::from_json(&v.to_string())
        .unwrap_err()
        .contains("繁殖馬"));
}

#[test]
fn a_save_from_before_studs_had_a_temperament_gets_one_from_the_records() {
    let g = Game::with_seed(1);
    let mut v: serde_json::Value = old_save(&g);
    v["studs"][0]["temper"] = serde_json::json!(0);
    v["studs"][0]["legs"] = serde_json::json!(0);
    let back = Game::from_json(&v.to_string()).unwrap();
    assert_eq!(back.studs[0].temper, g.studs[0].temper);
    assert_eq!(back.studs[0].legs, g.studs[0].legs);
}

#[test]
fn too_many_tickets_are_refused_when_bought_as_when_quoted() {
    let mut g = Game::with_seed(1);
    let card = g.race_card(None).unwrap();
    let ticket = Ticket {
        kind: BetKind::Win,
        horses: vec![card.entrants[0].name.clone()],
        amount: 1,
    };
    let tickets = vec![ticket; MAX_TICKETS + 1];
    assert_eq!(
        g.quote_tickets(None, &tickets),
        Err(GameError::TooManyTickets)
    );
    assert_eq!(
        g.play_tickets(None, &tickets, 1).map(|_| ()),
        Err(GameError::TooManyTickets)
    );
}

#[test]
fn a_small_field_is_paid_on_two_places_in_the_race_itself() {
    // a real race of seven or fewer: the third horse home does not pay
    let mut g = Game::with_seed(1);
    g.entry_rules = true;
    let race = calendar::shared()
        .races
        .iter()
        .find(|r| {
            g.week = r.week;
            g.race_spec(Some(&RaceKey::of(r))).unwrap().entrants.len() <= 7
        })
        .expect("a small field somewhere in the year")
        .clone();
    g.week = race.week;
    let key = RaceKey::of(&race);
    let card = g.race_card(Some(&key)).unwrap();
    assert_eq!(card.place_slots, 2);
    let mut hits = 0;
    for horse in card.entrants.iter().map(|e| e.name.clone()) {
        let mut g = g.clone();
        let out = g
            .play_tickets(
                Some(&key),
                &[Ticket {
                    kind: BetKind::Place,
                    horses: vec![horse],
                    amount: 1,
                }],
                11,
            )
            .unwrap();
        hits += out.won as usize;
    }
    assert_eq!(hits, 2);
}
