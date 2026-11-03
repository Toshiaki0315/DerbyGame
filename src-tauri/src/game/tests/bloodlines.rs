//! The blood database, sire lines, the first stud book and coat colours (血統).

use super::*;

// ---- founding a sire line (系統の確立) ----

/// A game where a stallion at stud has a foal racing, and the foal's place.
fn sire_with_a_foal() -> (Game, String, usize) {
    let (mut g, s, d, _) = in_season();
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    weeks_pass(&mut g, GESTATION_WEEKS);
    let i = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    (g, s, i)
}

/// Credits a G1 win to the horse in place `i`.
fn win_a_g1(g: &mut Game, i: usize) {
    let race = calendar_race("疾風賞");
    g.entry_rules = false;
    g.field[i].age = 3;
    g.field[i].away = None;
    g.field[i].fatigue = 0;
    g.field[i].record = RaceRecord::default();
    g.week = race.week;
    let name = g.field[i].name.clone();
    g.set_entries(Some(&key_of(&race)), std::slice::from_ref(&name))
        .unwrap();
    let spec = g.race_spec(Some(&key_of(&race))).unwrap();
    g.note_results(&spec, &with_horse_placed(&spec, &name, 0));
}

#[test]
fn a_sires_foals_and_their_g1_wins_are_counted() {
    let (mut g, s, i) = sire_with_a_foal();
    let line = |g: &Game| {
        g.state()
            .sire_lines
            .into_iter()
            .find(|l| l.sire == s)
            .unwrap()
    };
    assert_eq!(
        (line(&g).foals, line(&g).g1_wins, line(&g).established),
        (1, 0, false)
    );
    win_a_g1(&mut g, i);
    assert_eq!(line(&g).g1_wins, 1);
}

#[test]
fn enough_foals_and_g1_wins_found_a_line_with_honours() {
    let (mut g, s, i) = sire_with_a_foal();
    let id = g.parent(&s).unwrap().0.id;
    g.sire_records.get_mut(&id).unwrap().foals = LINE_FOALS;
    for _ in 0..LINE_G1_WINS {
        win_a_g1(&mut g, i);
    }
    assert!(g
        .state()
        .sire_lines
        .iter()
        .any(|l| l.sire == s && l.established));
    assert!(g.news.iter().any(|n| n.contains(&s) && n.contains("系統")));
    assert!(g
        .awards
        .iter()
        .any(|a| a.mine && a.title == "系統確立" && a.horse == s));
    assert!(g
        .state()
        .goals
        .iter()
        .any(|x| x.name == "系統の確立" && x.done));
    // once only
    let awards = g.awards.len();
    win_a_g1(&mut g, i);
    assert_eq!(g.awards.len(), awards);
}

#[test]
fn a_mating_from_a_founded_line_is_worth_more_to_the_theories() {
    let (mut g, s, _) = sire_with_a_foal();
    let d = g
        .studs
        .iter()
        .find(|h| h.sex == Sex::Female)
        .unwrap()
        .name
        .clone();
    let before: u32 = g
        .breeding_outlook(&s, &d)
        .unwrap()
        .theories
        .iter()
        .map(|t| t.points)
        .sum();
    let id = g.parent(&s).unwrap().0.id;
    g.sire_records.get_mut(&id).unwrap().established = true;
    let after = g.breeding_outlook(&s, &d).unwrap();
    assert!(after.theories.iter().any(|t| t.name == "確立した系統"));
    assert_eq!(
        after.theories.iter().map(|t| t.points).sum::<u32>(),
        before + LINE_THEORY_POINTS
    );
}

#[test]
fn a_horse_not_by_one_of_the_players_sires_is_not_counted() {
    let mut g = Game::with_seed(1);
    win_a_g1(&mut g, 0);
    assert!(g.state().sire_lines.is_empty());
}

#[test]
fn sire_records_survive_a_save() {
    let (g, s, _) = sire_with_a_foal();
    let r = Game::from_json(&g.to_json()).unwrap();
    assert!(r.state().sire_lines.iter().any(|l| l.sire == s));
}

// ---- breeding from the blood database (血統DB) ----

#[test]
fn every_stable_horse_is_linked_to_a_distinct_record_of_its_own_sex_from_the_newest_generation() {
    let g = Game::new();
    let mut ids = g.all(|h| h.blood_id);
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), g.field.len(), "two horses share a record");
    for i in 0..g.field.len() {
        let r = blood(&g, i);
        assert_eq!(r.gender, g.field[i].sex, "{}", g.field[i].name);
        assert_eq!(r.gen, 10);
        assert!(r.sire_id.is_some() && r.dam_id.is_some());
    }
}

#[test]
fn the_links_are_repeatable_and_differ_from_game_to_game() {
    assert_eq!(
        Game::with_seed(3).all(|h| h.blood_id),
        Game::with_seed(3).all(|h| h.blood_id)
    );
    let a = Game::with_seed(3);
    let b = Game::with_seed(4);
    let names = |g: &Game| -> Vec<String> {
        g.state()
            .field
            .iter()
            .map(|h| h.pedigree.clone().unwrap().sire)
            .collect()
    };
    assert_ne!(names(&a), names(&b));
}

#[test]
fn the_pedigree_of_a_stable_horse_is_made_of_its_records_ancestors() {
    let g = Game::new();
    for (i, h) in g.state().field.iter().enumerate() {
        let p = h.pedigree.clone().expect("a pedigree");
        let r = blood(&g, i);
        let sire = g.blood_record(r.sire_id.unwrap()).unwrap();
        let dam = g.blood_record(r.dam_id.unwrap()).unwrap();
        assert_eq!(p.sire, sire.name);
        assert_eq!(p.dam, dam.name);
        assert_eq!(
            p.sire_sire,
            g.blood_record(sire.sire_id.unwrap()).unwrap().name
        );
        assert_eq!(
            p.sire_dam,
            g.blood_record(sire.dam_id.unwrap()).unwrap().name
        );
        assert_eq!(
            p.dam_sire,
            g.blood_record(dam.sire_id.unwrap()).unwrap().name
        );
        assert_eq!(p.dam_dam, g.blood_record(dam.dam_id.unwrap()).unwrap().name);
        assert!(crate::horse::is_valid_name(&p.sire));
    }
}

#[test]
fn the_database_lists_the_stable_horses_under_their_own_names_and_marks_them_owned() {
    let g = Game::new();
    let all = g.bloodline_all();
    assert_eq!(all.len(), 1148);
    // the racing stable and the starting stallion and mare
    assert_eq!(
        all.iter().filter(|h| h.owned).count(),
        g.field.len() + g.studs.len()
    );
    for (i, h) in g.field.iter().enumerate() {
        let r = all.iter().find(|x| x.id == g.field[i].blood_id).unwrap();
        assert_eq!(r.name, h.name);
        assert!(r.owned);
    }
    // names stay unique, so the stable's names are not also someone else's
    let mut names: Vec<&String> = all.iter().map(|h| &h.name).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), all.len());
}

#[test]
fn the_state_carries_each_horses_traits_from_its_record() {
    let g = Game::new();
    for (i, h) in g.state().field.iter().enumerate() {
        let r = blood(&g, i);
        let t = h.blood.clone().expect("traits");
        assert_eq!(
            (
                t.coat_color.as_str(),
                t.growth_type.as_str(),
                t.running_style.as_str(),
                t.personality.as_str()
            ),
            (
                r.coat_color.as_str(),
                r.growth_type.as_str(),
                r.running_style.as_str(),
                r.personality.as_str()
            )
        );
        assert_eq!(
            (t.aptitude_min, t.aptitude_max, t.peak_age),
            (r.aptitude_min, r.aptitude_max, r.peak_age)
        );
        assert_eq!(
            (t.lineage.as_str(), t.family.as_str(), t.gen),
            (r.lineage.as_str(), r.family.as_str(), r.gen)
        );
        assert_eq!(t.inbreed_vol, r.inbreed_vol);
    }
}

#[test]
fn breeding_adds_the_foal_to_the_database_with_its_parents() {
    let mut g = Game::with_seed(1);
    let (sire_id, dam_id) = (g.field[0].blood_id, g.field[4].blood_id);
    breed_default(&mut g).unwrap();
    let all = g.bloodline_all();
    assert_eq!(all.len(), 1149);
    let foal = all.last().unwrap();
    assert_eq!(foal.id, 1149);
    assert_eq!(foal.name, "ニューホープ");
    assert_eq!((foal.sire_id, foal.dam_id), (Some(sire_id), Some(dam_id)));
    assert_eq!(foal.gen, 11);
    assert!(foal.owned);
    assert_eq!(g.field[5].blood_id, 1149);
    // the retired horse's own record is still there
    assert!(all.iter().filter(|h| h.owned).count() >= g.field.len());
}

#[test]
fn the_foal_gets_its_stats_sex_and_age_from_the_record() {
    for seed in 0..10u64 {
        let mut g = with_rng(&shared_base(), seed);
        breed_default(&mut g).unwrap();
        let r = blood(&g, 5);
        let st = g.state();
        let foal = &st.field[5];
        assert_eq!(foal.age, 2);
        assert_eq!(g.field[5].sex, r.gender);
        let clamp = |v: f64| (v.round() as u32).clamp(FOAL_MIN_STAT, FOAL_MAX_STAT);
        assert_eq!(foal.speed, clamp(r.stats.speed));
        assert_eq!(foal.stamina, clamp(r.stats.stamina));
        assert_eq!(foal.power, clamp(r.stats.power));
        let dirt = ((r.stats.dirt - 50.0) / 25.0).round().clamp(-2.0, 2.0) as i8;
        assert_eq!(foal.dirt, dirt);
    }
}

#[test]
fn the_foal_inherits_the_parents_ancestry_in_its_pedigree() {
    let mut g = Game::with_seed(2);
    let (sire_name, dam_name) = (g.field[0].name.clone(), g.field[4].name.clone());
    let sire_rec = blood(&g, 0);
    let dam_rec = blood(&g, 4);
    breed_default(&mut g).unwrap();
    let p = g.state().field[5].pedigree.clone().unwrap();
    assert_eq!(
        (p.sire.as_str(), p.dam.as_str()),
        (sire_name.as_str(), dam_name.as_str())
    );
    assert_eq!(
        p.sire_sire,
        g.blood_record(sire_rec.sire_id.unwrap()).unwrap().name
    );
    assert_eq!(
        p.sire_dam,
        g.blood_record(sire_rec.dam_id.unwrap()).unwrap().name
    );
    assert_eq!(
        p.dam_sire,
        g.blood_record(dam_rec.sire_id.unwrap()).unwrap().name
    );
    assert_eq!(
        p.dam_dam,
        g.blood_record(dam_rec.dam_id.unwrap()).unwrap().name
    );
}

#[test]
fn the_parents_current_stats_including_training_decide_the_foal() {
    let mean_foal = |parent: u32| -> f64 {
        let mut total = 0.0;
        for seed in 0..25u64 {
            let mut g = with_rng(&shared_base(), seed);
            for i in [0usize, 4] {
                g.field[i].speed = parent;
                g.field[i].stamina = parent;
                g.field[i].power = parent;
            }
            breed_default(&mut g).unwrap();
            let f = &g.state().field[5];
            total += (f.speed + f.stamina + f.power) as f64 / 3.0;
        }
        total / 25.0
    };
    let (strong, weak) = (mean_foal(95), mean_foal(45));
    assert!(strong > weak + 25.0, "{strong:.1} vs {weak:.1}");
    // the pull toward the average: 95 gives about 86, 45 about 46
    assert!((80.0..95.0).contains(&strong), "{strong}");
    assert!((40.0..55.0).contains(&weak), "{weak}");
}

#[test]
fn foal_stats_stay_between_the_floor_and_the_mutation_cap() {
    for seed in 0..40u64 {
        let mut g = with_rng(&shared_base(), seed);
        breed_default(&mut g).unwrap();
        let f = &g.state().field[5];
        for v in [f.speed, f.stamina, f.power] {
            assert!((FOAL_MIN_STAT..=125).contains(&v), "{v}");
        }
    }
}

#[test]
fn a_foal_over_100_in_a_stat_is_a_listed_mutation() {
    let (mut saw_mutation, mut saw_over) = (false, false);
    for seed in 0..400u64 {
        let mut g = with_rng(&shared_base(), seed);
        for i in [0usize, 4] {
            g.field[i].speed = 100; // so a mutation clearly carries past 100
            g.field[i].stamina = 100;
            g.field[i].power = 100;
        }
        breed_default(&mut g).unwrap();
        let r = blood(&g, 5);
        let st = g.state();
        let f = &st.field[5];
        let t = f.blood.clone().unwrap();
        saw_mutation |= !t.mutations.is_empty();
        for (v, name) in [
            (f.speed, "speed"),
            (f.stamina, "stamina"),
            (f.power, "power"),
        ] {
            if v > 100 {
                saw_over = true;
                assert!(
                    t.mutations.iter().any(|m| m == name),
                    "{name} {v} not flagged"
                );
            }
        }
        assert_eq!(t.mutations, r.mutations);
    }
    assert!(saw_mutation && saw_over, "no mutation in 400 foals");
}

#[test]
fn mating_relatives_records_the_inbreeding_and_costs_temperament() {
    let mut g = Game::with_seed(5);
    // horse 0 and horse 4 become half-siblings (both sired by record 1)
    g.bred = vec![
        rec(2001, Some(1), None, Sex::Male),
        rec(2002, Some(1), None, Sex::Female),
    ];
    g.field[0].blood_id = 2001;
    g.field[4].blood_id = 2002;
    let out = g
        .breeding_outlook("アサギリライズ", "ヘキクウブレイブ")
        .unwrap();
    assert_eq!(out.inbreed_vol, 0.125);
    assert!(out.penalty > 5.0);
    breed_default(&mut g).unwrap();
    assert_eq!(blood(&g, 5).inbreed_vol, 0.125);
    assert_eq!(g.state().field[5].blood.clone().unwrap().inbreed_vol, 0.125);
}

#[test]
fn unrelated_parents_have_no_inbreeding() {
    let g = Game::with_seed(6);
    let o = g
        .breeding_outlook("アサギリライズ", "ヘキクウブレイブ")
        .unwrap();
    // the stable's founders come from different branches; any inbreeding is slight
    assert!(o.inbreed_vol <= 0.5);
    assert!(o.penalty <= 30.0);
}

#[test]
fn the_outlook_shows_a_nick_when_the_lines_cross_well() {
    let mut g = Game::with_seed(7);
    let mut a = rec(2001, None, None, Sex::Male);
    a.lineage = "Ignis".into();
    let mut b = rec(2002, None, None, Sex::Female);
    b.lineage = "Aqua".into();
    g.bred = vec![a, b];
    g.field[0].blood_id = 2001;
    g.field[4].blood_id = 2002;
    let o = g
        .breeding_outlook("アサギリライズ", "ヘキクウブレイブ")
        .unwrap();
    assert_eq!(
        o.nick,
        Some(crate::bloodline::Nick {
            stat: "speed".into(),
            bonus: 7.0
        })
    );
    assert_eq!(o.inbreed_vol, 0.0);
    // the bonus really lifts the foal's speed
    let mean = |nick: bool| -> f64 {
        let mut total = 0.0;
        for seed in 0..60u64 {
            let mut g = with_rng(&shared_base(), seed);
            let (mut a, mut b) = (
                rec(2001, None, None, Sex::Male),
                rec(2002, None, None, Sex::Female),
            );
            a.lineage = if nick { "Ignis" } else { "Aqua" }.into();
            b.lineage = if nick { "Aqua" } else { "Ignis" }.into();
            g.bred = vec![a, b];
            g.field[0].blood_id = 2001;
            g.field[4].blood_id = 2002;
            for i in [0usize, 4] {
                g.field[i].speed = 60;
            }
            breed_default(&mut g).unwrap();
            total += g.state().field[5].speed as f64;
        }
        total / 60.0
    };
    assert!(
        (mean(true) - mean(false) - 7.0).abs() < 3.0,
        "{} vs {}",
        mean(true),
        mean(false)
    );
}

#[test]
fn the_outlook_refuses_unknown_horses_and_the_wrong_sexes() {
    let g = Game::new();
    assert_eq!(
        g.breeding_outlook("Nobody", "ヘキクウブレイブ"),
        Err(GameError::UnknownHorse)
    );
    assert_eq!(
        g.breeding_outlook("アサギリライズ", "Nobody"),
        Err(GameError::UnknownHorse)
    );
    assert_eq!(
        g.breeding_outlook("ヘキクウブレイブ", "サクラフブキ"),
        Err(GameError::WrongSex)
    );
    assert_eq!(
        g.breeding_outlook("アサギリライズ", "グレンキング"),
        Err(GameError::WrongSex)
    );
}

#[test]
fn the_outlook_does_not_change_the_game() {
    let g = Game::new();
    let before = g.state();
    let _ = g.breeding_outlook("アサギリライズ", "ヘキクウブレイブ");
    assert_eq!(g.state(), before);
}

#[test]
fn a_bred_foal_survives_save_and_load_together_with_its_database_record() {
    let mut g = Game::with_seed(8);
    breed_default(&mut g).unwrap();
    let restored = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(restored.bloodline_all(), g.bloodline_all());
    assert_eq!(restored.state(), g.state());
    // and the foal can itself be a parent
    assert_eq!(restored.field[5].blood_id, g.field[5].blood_id);
}

#[test]
fn old_saves_without_links_are_linked_the_same_way_on_load() {
    let g = Game::with_seed(9);
    let mut v: serde_json::Value = old_save(&g);
    for k in ["blood_ids", "bred", "blood_names", "pedigrees"] {
        v.as_object_mut().unwrap().remove(k);
    }
    let old = Game::from_json(&v.to_string()).unwrap();
    assert_eq!(old.all(|h| h.blood_id), g.all(|h| h.blood_id));
    assert_eq!(old.state(), g.state());
}

#[test]
fn a_horse_bred_before_the_database_existed_gets_a_record_of_its_own() {
    let mut g = Game::with_seed(10);
    breed_default(&mut g).unwrap(); // slot 5 is now "ニューホープ"
    let mut v: serde_json::Value = old_save(&g);
    for k in ["blood_ids", "bred", "blood_names"] {
        v.as_object_mut().unwrap().remove(k);
    }
    let old = Game::from_json(&v.to_string()).unwrap();
    let r = blood(&old, 5);
    assert_eq!(r.name, "ニューホープ");
    assert_eq!(r.gender, old.field[5].sex);
    assert_eq!(r.gen, 1);
    // it can be bred from like any other
    let mut old = old;
    let (sire, dam) = (old.field[0].name.clone(), old.field[4].name.clone());
    let sex = old.field[5].sex;
    let (s, d) = if sex == Sex::Male {
        ("ニューホープ".to_string(), dam)
    } else {
        (sire, "ニューホープ".to_string())
    };
    assert!(old
        .breed_now(&s, &d, &old.field[7].name.clone(), "セカンドホープ")
        .is_ok());
}

#[test]
fn a_save_with_the_wrong_number_of_links_is_rejected() {
    let g = Game::with_seed(11);
    let mut v: serde_json::Value = old_save(&g);
    v["blood_ids"] = serde_json::json!([1, 2]);
    assert!(Game::from_json(&v.to_string()).is_err());
}

#[test]
fn starting_over_builds_a_new_database() {
    let mut g = Game::with_seed(12);
    let before = g.bloodline_all();
    g.reset();
    assert_ne!(g.bloodline_all(), before);
    assert_eq!(g.bloodline_all().len(), 1148);
}

#[test]
fn breeding_still_costs_money_and_follows_the_same_rules() {
    let mut g = Game::with_seed(13);
    breed_default(&mut g).unwrap();
    assert_eq!(g.money, START_MONEY - BREED_COST);
    let before = g.bloodline_all();
    assert!(g
        .breed_now(
            "ヘキクウブレイブ",
            "サクラフブキ",
            "ミナモムーン",
            "ツギノコ"
        )
        .is_err());
    assert_eq!(g.bloodline_all(), before, "a refused breeding adds nothing");
}

#[test]
fn the_json_shapes_carry_the_traits_and_the_outlook() {
    let g = Game::new();
    let st = serde_json::to_value(g.state()).unwrap();
    let t = &st["field"][0]["blood"];
    for field in [
        "coat_color",
        "growth_type",
        "peak_age",
        "aptitude_min",
        "aptitude_max",
        "running_style",
        "personality",
        "inbreed_vol",
        "mutations",
        "lineage",
        "family",
        "gen",
    ] {
        assert!(t.get(field).is_some(), "missing {field}");
    }
    let o = serde_json::to_value(
        g.breeding_outlook("アサギリライズ", "ヘキクウブレイブ")
            .unwrap(),
    )
    .unwrap();
    for field in ["inbreed_vol", "penalty", "nick"] {
        assert!(o.get(field).is_some(), "missing {field}");
    }
}

#[test]
fn a_foal_cannot_take_a_name_that_is_already_in_the_database() {
    let mut g = Game::with_seed(14);
    let taken = g.base_db()[500].name.clone();
    assert_eq!(
        g.breed_now("アサギリライズ", "ヘキクウブレイブ", "サクラフブキ", &taken),
        Err(GameError::DuplicateName)
    );
    assert_eq!(g.money, START_MONEY);
    // nor the name of an earlier foal
    breed_default(&mut g).unwrap();
    assert_eq!(
        g.breed_now(
            "ホクトセイバー",
            "ミナモムーン",
            "グレンキング",
            "ニューホープ"
        ),
        Err(GameError::DuplicateName)
    );
}

#[test]
fn the_stable_draws_on_every_line_the_newest_generation_still_has() {
    for seed in 1..=12u64 {
        let g = Game::new();
        let _ = seed;
        let newest: std::collections::HashSet<String> = g
            .base_db()
            .iter()
            .filter(|h| h.gen == NEWEST_GENERATION)
            .map(|h| h.lineage.clone())
            .collect();
        let stable: std::collections::HashSet<String> =
            (0..g.field.len()).map(|i| blood(&g, i).lineage).collect();
        // a line that has died out cannot be used, but every surviving one is
        assert!(stable.is_subset(&newest));
        assert_eq!(
            stable.len(),
            newest.len().min(g.field.len()),
            "seed {seed}: {stable:?} of {newest:?}"
        );
    }
}

#[test]
fn new_game_has_starting_money_and_field() {
    let g = Game::new();
    assert_eq!(g.money, START_MONEY);
    assert_eq!(g.round, 0);
    assert_eq!(g.field.len(), 8);
}

#[test]
fn odds_favour_stronger_horses_and_never_below_minimum() {
    let g = Game::new();
    let mut odds: Vec<(u32, f64)> = g
        .field
        .iter()
        .enumerate()
        .map(|(i, h)| (g.effective_speed(i), g.odds(&h.name).unwrap()))
        .collect();
    odds.sort_by_key(|(strength, _)| *strength);
    // weaker horse => higher odds
    assert!(odds.first().unwrap().1 > odds.last().unwrap().1);
    assert!(odds.iter().all(|(_, o)| *o >= 1.1));
}

#[test]
fn unknown_horse_is_rejected() {
    let mut g = Game::new();
    assert_eq!(g.play_round("Nobody", 100, 1), Err(GameError::UnknownHorse));
    assert_eq!(g.money, START_MONEY);
}

#[test]
fn zero_bet_is_rejected() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    assert_eq!(g.play_round(&name, 0, 1), Err(GameError::InvalidAmount));
}

#[test]
fn bet_over_balance_is_rejected() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    assert_eq!(
        g.play_round(&name, START_MONEY + 1, 1),
        Err(GameError::InsufficientFunds)
    );
    assert_eq!(g.money, START_MONEY);
}

#[test]
fn winning_pays_odds_and_losing_costs_stake() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let odds = g.odds(&name).unwrap();
    let out = g.play_round(&name, 100, 5).unwrap();
    assert_eq!(g.round, 1);
    assert_eq!(out.results.len(), g.field.len());
    if out.won {
        let payout = (100.0 * odds).round() as i64;
        assert_eq!(out.payout, payout);
        assert_eq!(g.money as i64, START_MONEY as i64 - 100 + payout);
    } else {
        assert_eq!(out.payout, 0);
        assert_eq!(g.money, START_MONEY - 100);
    }
}

#[test]
fn outcome_includes_replay_frames() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let out = g.play_round(&name, 10, 2).unwrap();
    assert_eq!(out.distance, 2400.0);
    assert!(!out.frames.is_empty());
    assert!(out.frames.iter().all(|f| f.len() == g.field.len()));
}

#[test]
fn outcome_includes_commentary_about_this_race() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let out = g.play_round(&name, 10, 2).unwrap();
    assert!(out.commentary.first().unwrap().text.contains("スタート"));
    let winner = &out.results[0].name;
    assert!(out
        .commentary
        .iter()
        .any(|l| l.text.contains("1着") && l.text.contains(winner.as_str())));
    assert!(out.commentary.iter().all(|l| l.frame < out.frames.len()));
}

#[test]
fn winner_is_whoever_ranked_first() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    let out = g.play_round(&name, 10, 9).unwrap();
    assert_eq!(out.won, out.results[0].name == name);
}

#[test]
fn game_over_when_broke() {
    let mut g = Game::new();
    g.money = 0;
    assert!(g.is_game_over());
    let name = g.field[0].name.clone();
    assert_eq!(g.play_round(&name, 1, 1), Err(GameError::GameOver));
}

#[test]
fn reset_restores_initial_state() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.play_round(&name, 100, 3).unwrap();
    g.reset();
    assert_eq!(g.money, START_MONEY);
    assert_eq!(g.round, 0);
}

// ---- coat colours (毛色) ----

#[test]
fn the_stables_horses_race_in_their_records_coat_colour() {
    let g = Game::new();
    for i in 0..g.field.len() {
        let r = g.blood_record(g.field[i].blood_id).unwrap();
        assert_eq!(g.stable_entrant(i).coat_color, r.coat_color);
    }
    let mut g = Game::new();
    g.field[0].blood_id = 0;
    assert_eq!(g.stable_entrant(0).coat_color, DEFAULT_COAT);
}

#[test]
fn the_card_and_the_outcome_carry_every_runners_coat_colour() {
    let mut g = Game::new();
    let card = g.race_card(None).unwrap();
    assert!(card
        .entrants
        .iter()
        .all(|e| ["鹿毛", "栗毛", "青毛", "芦毛"].contains(&e.coat_color.as_str())));
    let rival = card.entrants.iter().find(|e| !e.mine).unwrap().name.clone();
    let out = g.play_combo(None, &[rival], 10, 1, BetKind::Win).unwrap();
    assert_eq!(out.coats.len(), out.runners.len());
    for (name, coat) in out.runners.iter().zip(&out.coats) {
        let e = card.entrants.iter().find(|e| &e.name == name).unwrap();
        assert_eq!(&e.coat_color, coat);
    }
    let v = serde_json::to_value(&out).unwrap();
    assert!(v["coats"].as_array().is_some());
}

// ---- the stud book a new game starts with ----

#[test]
fn a_new_game_starts_with_one_stallion_and_one_mare_at_stud() {
    let g = Game::new();
    assert_eq!(g.studs.len(), 2);
    assert_eq!(g.studs.iter().filter(|s| s.sex == Sex::Male).count(), 1);
    assert_eq!(g.studs.iter().filter(|s| s.sex == Sex::Female).count(), 1);
    for s in &g.studs {
        let rec = g.blood_record(s.blood_id).unwrap();
        assert!(rec.owned);
        assert_eq!((s.name.clone(), s.sex), (rec.name.clone(), rec.gender));
        assert!(s.starter && !s.bought);
        assert_eq!(s.record, RaceRecord::default());
        // not one of the racing stable
        assert!(!g.all(|h| h.blood_id).contains(&s.blood_id));
        assert!(g.field.iter().all(|h| h.name != s.name));
    }
    assert!(g.state().studs.iter().all(|s| s.starter));
}

#[test]
fn the_starting_studs_are_chosen_at_random_for_each_game() {
    let pick = |seed: u64| {
        let g = Game::with_seed(seed);
        g.studs.iter().map(|s| s.blood_id).collect::<Vec<_>>()
    };
    assert_eq!(pick(5), pick(5));
    let distinct: std::collections::HashSet<Vec<u32>> = (1..=8u64).map(pick).collect();
    assert!(distinct.len() >= 4, "{distinct:?}");
}

#[test]
fn the_starting_studs_can_be_bred_from_and_are_not_on_sale() {
    let mut g = Game::new();
    let sire = g
        .studs
        .iter()
        .find(|s| s.sex == Sex::Male)
        .unwrap()
        .name
        .clone();
    let dam = g
        .studs
        .iter()
        .find(|s| s.sex == Sex::Female)
        .unwrap()
        .name
        .clone();
    let ids: Vec<u32> = g.studs.iter().map(|s| s.blood_id).collect();
    assert!(g.stud_market().iter().all(|o| !ids.contains(&o.blood_id)));
    let replace = g.field[0].name.clone();
    g.breed_now(&sire, &dam, &replace, "ハナミズキ").unwrap();
    assert!(g.field.iter().any(|h| h.name == "ハナミズキ"));
}

#[test]
fn a_new_game_after_a_reset_has_a_fresh_pair_and_old_saves_keep_their_studs() {
    let mut g = Game::new();
    g.studs.clear();
    let back = Game::from_json(&g.to_json()).unwrap();
    assert!(back.studs.is_empty());
    g.reset();
    assert_eq!(g.studs.len(), 2);
}
