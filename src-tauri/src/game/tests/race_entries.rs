//! Entering races, exclusion, the draw and automatic entry (出走登録・除外・自動出走).

use super::*;

// ---- exclusion, the draw and priority (除外・抽選・優先出走権) ----

/// A game with the entry rules on, and the place in its stable of a horse a race takes.
fn with_rules_for(race: &CalendarRace) -> (Game, usize) {
    let mut g = Game::with_seed(1);
    g.entry_rules = true;
    g.week = race.week;
    let i = (0..g.field.len())
        .find(|&i| !race.eligibility.fillies_only || g.field[i].sex == Sex::Female)
        .unwrap();
    g.field[i].age = match race.eligibility.age {
        AgeRule::Two => 2,
        AgeRule::Three => 3,
        AgeRule::ThreeUp => g.field[i].age.max(3),
        AgeRule::FourUp => g.field[i].age.max(4),
    };
    g.field[i].record = RaceRecord::default();
    g.field[i].fatigue = 0;
    g.field[i].away = None;
    (g, i)
}

fn a_maiden_race() -> CalendarRace {
    calendar::shared()
        .races
        .iter()
        .find(|r| {
            r.grade == Grade::Win1
                && !r.eligibility.fillies_only
                && r.eligibility.age == AgeRule::ThreeUp
        })
        .unwrap()
        .clone()
}

#[test]
fn the_free_race_has_no_entry_rules() {
    let (g, i) = with_rules_for(&Game::free_race());
    assert!(!g.short_of_earnings(i, &Game::free_race()));
    assert!(!g.drawn_out(i, &Game::free_race()));
}

#[test]
fn some_but_not_all_entries_are_drawn_out_of_a_full_race() {
    let race = a_maiden_race();
    let (mut g, i) = with_rules_for(&race);
    let out = (1..=400)
        .filter(|&y| {
            g.year = y;
            g.drawn_out(i, &race)
        })
        .count();
    assert!((12..=60).contains(&out), "{out} of 400");
}

#[test]
fn a_horse_with_priority_cannot_be_drawn_out() {
    let race = a_maiden_race();
    let (mut g, i) = with_rules_for(&race);
    g.priority.push(g.field[i].name.clone());
    assert!((1..=200).all(|y| {
        g.year = y;
        !g.drawn_out(i, &race)
    }));
}

/// A year in which a horse is drawn out of a race, with the race's week set.
fn year_drawn_out(g: &mut Game, i: usize, race: &CalendarRace) -> u32 {
    g.week = race.week;
    (1..=2000)
        .find(|&y| {
            g.year = y;
            g.drawn_out(i, race)
        })
        .expect("a year it is drawn out")
}

#[test]
fn a_race_on_the_calendar_has_from_six_to_eighteen_runners_and_never_more_than_its_gate() {
    let mut g = Game::with_seed(1);
    g.entry_rules = true;
    let mut sizes = std::collections::HashSet::new();
    for year in 1..=3 {
        g.year = year;
        for race in calendar::shared().races.iter() {
            g.week = race.week;
            let spec = g.race_spec(Some(&key_of(race))).unwrap();
            let n = spec.entrants.len();
            assert!(n >= entries::MIN_FIELD, "{} {n}", race.name);
            assert!(n <= entries::full_gate(race.venue), "{} {n}", race.name);
            sizes.insert(n);
        }
    }
    assert!(
        sizes.contains(&18) && sizes.iter().any(|&n| n <= 8),
        "{sizes:?}"
    );
}

#[test]
fn the_graded_races_have_bigger_fields_than_the_others() {
    let mut g = Game::with_seed(1);
    g.entry_rules = true;
    let average = |g: &mut Game, graded: bool| {
        let (mut runners, mut races) = (0, 0);
        for year in 1..=3 {
            g.year = year;
            for race in calendar::shared().races.iter() {
                if matches!(race.grade, Grade::G1 | Grade::G2 | Grade::G3) == graded {
                    g.week = race.week;
                    runners += g.race_spec(Some(&key_of(race))).unwrap().entrants.len();
                    races += 1;
                }
            }
        }
        runners as f64 / races as f64
    };
    let (graded, others) = (average(&mut g, true), average(&mut g, false));
    assert!(
        graded >= 15.0 && graded > others + 2.0,
        "{graded} against {others}"
    );
}

#[test]
fn a_race_keeps_the_same_field_size_however_often_it_is_looked_at() {
    let race = a_maiden_race();
    let (g, _) = with_rules_for(&race);
    let size = || g.race_spec(Some(&key_of(&race))).unwrap().entrants.len();
    assert_eq!(size(), size());
}

#[test]
fn the_free_race_always_has_a_full_field() {
    let mut g = Game::with_seed(1);
    g.entry_rules = true;
    for year in 1..=30 {
        g.year = year;
        assert_eq!(
            g.race_spec(None).unwrap().entrants.len(),
            entries::FULL_FIELD
        );
    }
}

#[test]
fn a_registered_horse_may_lose_the_draw_and_then_does_not_run() {
    let race = a_maiden_race();
    let (mut g, i) = with_rules_for(&race);
    year_drawn_out(&mut g, i, &race);
    let name = g.field[i].name.clone();
    g.set_entries(Some(&key_of(&race)), std::slice::from_ref(&name))
        .unwrap();
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    let mine = card.my_horses.iter().find(|h| h.name == name).unwrap();
    assert!(mine.eligible && mine.registered && mine.drawn_out && !mine.entered);
    assert!(card.entrants.iter().all(|e| e.name != name));
    assert!(card.nominated > card.gate);
    assert_eq!(card.entrants.len(), card.gate);
}

#[test]
fn the_card_says_how_many_are_put_forward_and_how_many_can_run() {
    let race = a_maiden_race();
    let (mut g, i) = with_rules_for(&race);
    let name = g.field[i].name.clone();
    let key = key_of(&race);
    g.set_entries(Some(&key), &[]).unwrap();
    let alone = g.race_card(Some(&key)).unwrap();
    assert_eq!(alone.gate, entries::full_gate(race.venue));
    g.priority.push(name.clone());
    g.set_entries(Some(&key), std::slice::from_ref(&name))
        .unwrap();
    let with = g.race_card(Some(&key)).unwrap();
    assert_eq!(with.nominated, alone.nominated + 1);
    assert_eq!(with.entrants.len(), with.nominated.min(with.gate));
    assert_eq!(with.place_slots, bets::place_slots(with.entrants.len()));
}

#[test]
fn a_horse_that_loses_the_draw_has_priority_next_time_and_is_told_so() {
    let race = a_maiden_race();
    let (mut g, i) = with_rules_for(&race);
    year_drawn_out(&mut g, i, &race);
    let name = g.field[i].name.clone();
    g.set_entries(Some(&key_of(&race)), std::slice::from_ref(&name))
        .unwrap();
    g.advance_week();
    assert!(g.priority.contains(&name));
    assert!(
        g.news
            .iter()
            .any(|n| n.contains(&name) && n.contains("除外")),
        "{:?}",
        g.news
    );
}

#[test]
fn a_horse_only_gets_priority_from_the_draw_if_it_stayed_registered() {
    let race = a_maiden_race();
    let (mut g, i) = with_rules_for(&race);
    year_drawn_out(&mut g, i, &race);
    let name = g.field[i].name.clone();
    let key = key_of(&race);
    g.set_entries(Some(&key), std::slice::from_ref(&name))
        .unwrap();
    g.set_entries(Some(&key), &[]).unwrap();
    g.advance_week();
    assert!(!g.priority.contains(&name));
}

#[test]
fn a_trainer_s_horse_can_lose_the_draw_too_and_then_the_race_is_left_alone() {
    let mut g = Game::with_seed(1);
    g.entry_rules = true;
    // a single horse left to its trainer, in a year the draw goes against it
    g.week = 10;
    let i = (0..g.field.len())
        .find(|&i| {
            g.field.iter_mut().for_each(|h| h.auto_entry = false);
            g.field[i].auto_entry = true;
            g.field[i].age = g.field[i].age.max(4);
            g.field[i].since_race = NEVER_RACED;
            g.field[i].fatigue = 0;
            !g.auto_entries(None).is_empty()
        })
        .unwrap();
    assert!(g.field[i].auto_entry);
    let found = (1..=3000).find(|&y| {
        g.year = y;
        let entries = g.auto_entries(None);
        !entries.is_empty()
            && entries.iter().all(|(race, names)| {
                names.iter().all(|n| {
                    let i = g.field.iter().position(|h| &h.name == n).unwrap();
                    g.drawn_out(i, race)
                })
            })
    });
    let year = found.expect("a week all are drawn out");
    g.year = year;
    let out: Vec<String> = g
        .auto_entries(None)
        .into_iter()
        .flat_map(|(_, names)| names)
        .collect();
    let (round, history) = (g.round, g.history.len());
    g.advance_week();
    assert_eq!((g.round, g.history.len()), (round, history));
    for name in &out {
        assert!(g.priority.contains(name));
        assert!(g
            .news
            .iter()
            .any(|n| n.contains(name.as_str()) && n.contains("除外")));
    }
}

#[test]
fn nobody_who_has_a_place_in_a_g1_is_drawn_out_of_it() {
    let g1 = calendar_race("疾風賞");
    let (mut g, i) = with_rules_for(&g1);
    g.field[i].career.earnings = G1_MIN_EARNINGS;
    assert!((1..=500).all(|y| {
        g.year = y;
        !g.drawn_out(i, &g1)
    }));
}

#[test]
fn a_horse_in_the_first_five_has_priority_next_time_and_one_behind_them_does_not() {
    let race = a_maiden_race();
    let (mut g, i) = with_rules_for(&race);
    let name = g.field[i].name.clone();
    g.entry_rules = false;
    g.set_entries(Some(&key_of(&race)), std::slice::from_ref(&name))
        .unwrap();
    let spec = g.race_spec(Some(&key_of(&race))).unwrap();
    g.note_results(&spec, &with_horse_placed(&spec, &name, 4));
    assert!(g.priority.contains(&name));
    g.note_results(&spec, &with_horse_placed(&spec, &name, 5));
    assert!(!g.priority.contains(&name));
}

#[test]
fn a_g1_is_only_for_horses_with_enough_earnings_or_a_right_to_run() {
    let race = calendar_race("疾風賞");
    let (mut g, i) = with_rules_for(&race);
    g.field[i].career.earnings = 0;
    assert!(g.short_of_earnings(i, &race));
    assert!(!g.eligible_horses(&race).contains(&i));
    g.field[i].career.earnings = G1_MIN_EARNINGS;
    assert!(!g.short_of_earnings(i, &race));
    assert!(g.eligible_horses(&race).contains(&i));
    g.field[i].career.earnings = 0;
    g.trial_rights.push(TrialRight {
        horse: g.field[i].name.clone(),
        year: g.year,
        race: race.name.clone(),
    });
    assert!(!g.short_of_earnings(i, &race));
    g.year += 1;
    assert!(g.short_of_earnings(i, &race));
}

#[test]
fn a_two_year_old_needs_less_to_run_in_a_g1() {
    let race = calendar::shared()
        .races
        .iter()
        .find(|r| {
            r.grade == Grade::G1 && r.eligibility.age == AgeRule::Two && !r.eligibility.fillies_only
        })
        .unwrap()
        .clone();
    let (mut g, i) = with_rules_for(&race);
    g.field[i].career.earnings = G1_MIN_EARNINGS_TWO;
    assert!(!g.short_of_earnings(i, &race));
    g.field[i].career.earnings = G1_MIN_EARNINGS_TWO - 1;
    assert!(g.short_of_earnings(i, &race));
}

#[test]
fn a_classic_has_a_graded_trial_on_the_same_surface_shortly_before() {
    let g1 = calendar_race("疾風賞");
    let trial = trial_of(&g1).unwrap();
    assert!(matches!(trial.grade, Grade::G2 | Grade::G3));
    assert_eq!(trial.surface, g1.surface);
    assert_eq!(trial.eligibility.age, g1.eligibility.age);
    assert!(trial.week < g1.week && g1.week - trial.week <= TRIAL_WINDOW_WEEKS);
    assert!(trials_for(&trial).iter().any(|r| r == "疾風賞"));
    assert!(trials_for(&g1).is_empty());
}

#[test]
fn the_first_three_in_a_trial_earn_a_right_to_run_in_the_g1() {
    let g1 = calendar_race("疾風賞");
    let trial = trial_of(&g1).unwrap();
    let (mut g, i) = with_rules_for(&trial);
    let name = g.field[i].name.clone();
    g.entry_rules = false;
    g.set_entries(Some(&key_of(&trial)), std::slice::from_ref(&name))
        .unwrap();
    let spec = g.race_spec(Some(&key_of(&trial))).unwrap();
    g.entry_rules = true;
    g.note_results(&spec, &with_horse_placed(&spec, &name, 3));
    assert!(g.trial_rights.is_empty());
    g.note_results(&spec, &with_horse_placed(&spec, &name, 2));
    assert!(!g.short_of_earnings(i, &g1));
    assert!(g
        .news
        .iter()
        .any(|n| n.contains(&name) && n.contains("優先出走権")));
}

#[test]
fn the_card_says_why_a_horse_is_barred_and_what_a_trial_leads_to() {
    let g1 = calendar_race("疾風賞");
    let (mut g, i) = with_rules_for(&g1);
    g.field[i].career.earnings = 0;
    let name = g.field[i].name.clone();
    let card = g.race_card(Some(&key_of(&g1))).unwrap();
    let mine = card.my_horses.iter().find(|h| h.name == name).unwrap();
    assert!(!mine.eligible);
    assert_eq!(mine.barred.as_deref(), Some("賞金不足"));
    let trial = trial_of(&g1).unwrap();
    g.week = trial.week;
    assert!(g
        .race_card(Some(&key_of(&trial)))
        .unwrap()
        .trial_for
        .iter()
        .any(|r| r == "疾風賞"));
}

#[test]
fn the_trainers_do_not_enter_a_horse_in_a_race_it_is_barred_from() {
    let g1 = calendar_race("疾風賞");
    let (mut g, i) = with_rules_for(&g1);
    g.field[i].career.earnings = 0;
    assert!(g.auto_choices(i, None).iter().all(|r| r.name != "疾風賞"));
    g.field[i].career.earnings = G1_MIN_EARNINGS;
    assert!(
        g.choices_in(i, g1.week, None)
            .iter()
            .all(|r| r.name != "疾風賞")
            || !g.short_of_earnings(i, &g1)
    );
}

#[test]
fn rights_and_priority_survive_a_save_and_an_old_save_gets_the_rules() {
    let mut g = Game::with_seed(1);
    g.priority.push("アサギリライズ".to_string());
    g.trial_rights.push(TrialRight {
        horse: "アサギリライズ".into(),
        year: 1,
        race: "疾風賞".into(),
    });
    let r = Game::from_json(&g.to_json()).unwrap();
    assert_eq!((r.priority.len(), r.trial_rights.len()), (1, 1));
    let mut json: serde_json::Value = old_save(&g);
    let o = json.as_object_mut().unwrap();
    o.remove("entry_rules");
    o.remove("priority");
    o.remove("trial_rights");
    assert!(Game::from_json(&json.to_string()).unwrap().entry_rules);
}

// ---- which of the player's horses run (出走登録) ----

fn mine_in(card: &RaceCard) -> Vec<String> {
    card.entrants
        .iter()
        .filter(|e| e.mine)
        .map(|e| e.name.clone())
        .collect()
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

#[test]
fn only_a_few_of_the_players_horses_run_and_rivals_fill_the_rest() {
    let mut g = Game::new();
    // the free race
    let free = g.race_card(None).unwrap();
    assert_eq!(free.entrants.len(), entries::FULL_FIELD);
    assert_eq!(mine_in(&free).len(), MAX_OWN_ENTRIES);
    // a calendar race that all eight of the stable could enter
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    assert_eq!(card.entrants.len(), entries::FULL_FIELD);
    assert_eq!(mine_in(&card).len(), MAX_OWN_ENTRIES);
    assert_eq!(
        card.entrants.iter().filter(|e| !e.mine).count(),
        entries::FULL_FIELD - MAX_OWN_ENTRIES
    );
}

#[test]
fn the_limit_is_three_horses() {
    assert_eq!(MAX_OWN_ENTRIES, 3);
}

#[test]
fn by_default_the_strongest_eligible_horses_are_entered() {
    let g = Game::new();
    let mut ranked: Vec<(u32, String)> = g
        .field
        .iter()
        .map(|h| (h.speed + h.stamina, h.name.clone()))
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0));
    let expected = sorted(
        ranked
            .iter()
            .take(MAX_OWN_ENTRIES)
            .map(|r| r.1.clone())
            .collect(),
    );
    assert_eq!(sorted(mine_in(&g.race_card(None).unwrap())), expected);
}

#[test]
fn a_race_with_fewer_eligible_horses_than_the_limit_enters_only_those() {
    let mut g = Game::new();
    g.week = 20;
    // 4yo+ fillies and mares: only the 4-year-old mare qualifies
    let race = calendar_race("薫風ヴィーナスマイル");
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    assert_eq!(mine_in(&card), vec!["ヘキクウブレイブ"]);
    assert_eq!(card.entrants.len(), entries::FULL_FIELD);
}

#[test]
fn the_player_can_choose_which_horses_run() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let pick = strs(&["サクラフブキ", "ミナモムーン"]);
    g.set_entries(Some(&key), &pick).unwrap();
    let card = g.race_card(Some(&key)).unwrap();
    assert_eq!(sorted(mine_in(&card)), sorted(pick));
    assert_eq!(card.entrants.len(), entries::FULL_FIELD);
}

#[test]
fn the_free_race_entries_can_be_chosen_too() {
    let mut g = Game::new();
    g.set_entries(None, &strs(&["サクラフブキ"])).unwrap();
    assert_eq!(mine_in(&g.race_card(None).unwrap()), vec!["サクラフブキ"]);
}

#[test]
fn no_entries_at_all_makes_the_field_all_rivals_and_stays_that_way() {
    let mut g = Game::new();
    g.set_entries(None, &[]).unwrap();
    for _ in 0..2 {
        let card = g.race_card(None).unwrap();
        assert!(mine_in(&card).is_empty());
        assert_eq!(card.entrants.len(), entries::FULL_FIELD);
    }
}

#[test]
fn at_most_three_horses_may_be_entered() {
    let mut g = Game::new();
    let four: Vec<String> = g.field.iter().take(4).map(|h| h.name.clone()).collect();
    assert_eq!(g.set_entries(None, &four), Err(GameError::TooManyEntries));
    let three: Vec<String> = g.field.iter().take(3).map(|h| h.name.clone()).collect();
    assert!(g.set_entries(None, &three).is_ok());
}

#[test]
fn only_horses_that_meet_the_conditions_may_be_entered() {
    let mut g = Game::new();
    g.week = 20;
    let race = calendar_race("薫風ヴィーナスマイル"); // 4yo+ fillies and mares
    let key = key_of(&race);
    // a 3-year-old mare, a 5-year-old stallion
    assert_eq!(
        g.set_entries(Some(&key), &strs(&["サクラフブキ"])),
        Err(GameError::NotEligible)
    );
    assert_eq!(
        g.set_entries(Some(&key), &strs(&["ギンセイストーム"])),
        Err(GameError::NotEligible)
    );
    assert!(g
        .set_entries(Some(&key), &strs(&["ヘキクウブレイブ"]))
        .is_ok());
}

#[test]
fn unknown_and_repeated_horses_are_refused() {
    let mut g = Game::new();
    let a = g.field[0].name.clone();
    assert_eq!(
        g.set_entries(None, &strs(&["Nobody"])),
        Err(GameError::UnknownHorse)
    );
    assert_eq!(
        g.set_entries(None, &[a.clone(), a]),
        Err(GameError::DuplicateHorse)
    );
}

#[test]
fn a_refused_entry_leaves_the_earlier_choice_alone() {
    let mut g = Game::new();
    g.set_entries(None, &strs(&["サクラフブキ", "ミナモムーン"]))
        .unwrap();
    let before = g.race_card(None).unwrap();
    assert!(g.set_entries(None, &strs(&["Nobody"])).is_err());
    let four: Vec<String> = g.field.iter().take(4).map(|h| h.name.clone()).collect();
    assert!(g.set_entries(None, &four).is_err());
    assert_eq!(g.race_card(None).unwrap(), before);
}

#[test]
fn entering_for_a_race_that_is_not_on_this_week_is_refused() {
    let mut g = Game::new();
    let race = calendar_race("薫風ヴィーナスマイル"); // week 20
    assert_eq!(
        g.set_entries(Some(&key_of(&race)), &strs(&["ヘキクウブレイブ"])),
        Err(GameError::NoSuchRace)
    );
}

#[test]
fn entries_belong_to_one_race_only() {
    let mut g = Game::new();
    g.week = 20;
    let a = calendar_race("初夏スプリントカップ");
    let b = calendar_race("若葉の森ステークス");
    let default_b = mine_in(&g.race_card(Some(&key_of(&b))).unwrap());
    g.set_entries(Some(&key_of(&a)), &strs(&["サクラフブキ"]))
        .unwrap();
    assert_eq!(
        mine_in(&g.race_card(Some(&key_of(&a))).unwrap()),
        vec!["サクラフブキ"]
    );
    assert_eq!(mine_in(&g.race_card(Some(&key_of(&b))).unwrap()), default_b);
    assert_eq!(
        sorted(mine_in(&g.race_card(None).unwrap())).len(),
        MAX_OWN_ENTRIES
    );
}

#[test]
fn entries_are_forgotten_when_the_week_moves_on() {
    let mut g = Game::new();
    g.set_entries(None, &[]).unwrap();
    assert!(mine_in(&g.race_card(None).unwrap()).is_empty());
    g.advance_week();
    assert_eq!(mine_in(&g.race_card(None).unwrap()).len(), MAX_OWN_ENTRIES);
}

#[test]
fn entries_survive_save_and_load_and_old_saves_without_them_still_load() {
    let mut g = Game::new();
    g.set_entries(None, &strs(&["サクラフブキ"])).unwrap();
    let restored = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(
        mine_in(&restored.race_card(None).unwrap()),
        vec!["サクラフブキ"]
    );
    let mut v: serde_json::Value = old_save(&g);
    v.as_object_mut().unwrap().remove("entries");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert_eq!(
        mine_in(&old.race_card(None).unwrap()).len(),
        MAX_OWN_ENTRIES
    );
}

#[test]
fn the_card_lists_every_stable_horse_with_whether_it_can_and_does_run() {
    let mut g = Game::new();
    g.week = 20;
    let race = calendar_race("薫風ヴィーナスマイル");
    let card = g.race_card(Some(&key_of(&race))).unwrap();
    assert_eq!(card.max_entries, MAX_OWN_ENTRIES);
    assert_eq!(card.my_horses.len(), g.field.len());
    let by = |n: &str| card.my_horses.iter().find(|h| h.name == n).unwrap();
    assert!(by("ヘキクウブレイブ").eligible && by("ヘキクウブレイブ").entered);
    assert!(!by("サクラフブキ").eligible && !by("サクラフブキ").entered);
    assert!(!by("ギンセイストーム").eligible);
    // in the free race everyone may run, but only the entered ones do
    let free = g.race_card(None).unwrap();
    assert!(free.my_horses.iter().all(|h| h.eligible));
    assert_eq!(
        free.my_horses.iter().filter(|h| h.entered).count(),
        MAX_OWN_ENTRIES
    );
    let entered: Vec<String> = free
        .my_horses
        .iter()
        .filter(|h| h.entered)
        .map(|h| h.name.clone())
        .collect();
    assert_eq!(sorted(entered), sorted(mine_in(&free)));
}

#[test]
fn a_stable_horse_that_is_not_entered_does_not_run_and_cannot_be_bet_on() {
    let mut g = Game::new();
    g.set_entries(None, &strs(&["サクラフブキ"])).unwrap();
    let spec = g.race_spec(None).unwrap();
    assert!(spec.entrants.iter().any(|e| e.name == "サクラフブキ"));
    assert!(!spec.entrants.iter().any(|e| e.name == "ミナモムーン"));
    assert_eq!(
        g.play_combo(None, &strs(&["ミナモムーン"]), 10, 1, BetKind::Win),
        Err(GameError::UnknownHorse)
    );
    let out = g
        .play_combo(None, &strs(&["サクラフブキ"]), 10, 1, BetKind::Win)
        .unwrap();
    assert!(out.runners.iter().any(|r| r == "サクラフブキ"));
    assert!(!out.runners.iter().any(|r| r == "ミナモムーン"));
    assert_eq!(out.runners.len(), entries::FULL_FIELD);
}

#[test]
fn the_free_race_has_rivals_that_are_stable_for_a_week_and_new_the_next() {
    let mut g = Game::new();
    let rivals = |g: &Game| -> Vec<String> {
        let c = g.race_card(None).unwrap();
        c.entrants
            .iter()
            .filter(|e| !e.mine)
            .map(|e| e.name.clone())
            .collect()
    };
    let first = rivals(&g);
    assert_eq!(first, rivals(&g));
    assert_eq!(first.len(), entries::FULL_FIELD - MAX_OWN_ENTRIES);
    g.advance_week();
    assert_ne!(first, rivals(&g));
}

#[test]
fn rivals_never_share_a_name_with_the_stable_horses() {
    let mut g = Game::new();
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let stable: std::collections::HashSet<String> =
        g.field.iter().map(|h| h.name.clone()).collect();
    for card in [
        g.race_card(None).unwrap(),
        g.race_card(Some(&key_of(&race))).unwrap(),
    ] {
        for e in card.entrants.iter().filter(|e| !e.mine) {
            assert!(!stable.contains(&e.name), "{}", e.name);
        }
        let names: std::collections::HashSet<_> = card.entrants.iter().map(|e| &e.name).collect();
        assert_eq!(names.len(), card.entrants.len());
    }
}

#[test]
fn the_rivals_barely_change_when_more_of_my_horses_are_entered() {
    let mut g = Game::new();
    g.set_entries(None, &[]).unwrap();
    let none: Vec<String> = g
        .race_card(None)
        .unwrap()
        .entrants
        .iter()
        .map(|e| e.name.clone())
        .collect();
    g.set_entries(
        None,
        &strs(&["サクラフブキ", "ミナモムーン", "グレンキング"]),
    )
    .unwrap();
    let three = g.race_card(None).unwrap();
    let rivals: Vec<&String> = three
        .entrants
        .iter()
        .filter(|e| !e.mine)
        .map(|e| &e.name)
        .collect();
    // the nine rivals are the first nine of the twelve, so the field is steady
    assert!(rivals.iter().all(|r| none.contains(r)));
}

#[test]
fn the_card_json_carries_the_entry_information() {
    let g = Game::new();
    let v = serde_json::to_value(g.race_card(None).unwrap()).unwrap();
    assert_eq!(v["max_entries"], MAX_OWN_ENTRIES);
    let h = &v["my_horses"][0];
    for field in ["name", "eligible", "entered"] {
        assert!(h.get(field).is_some(), "missing {field}");
    }
}

// ---- entering a horse automatically (自動出走) ----

#[test]
fn a_horse_is_only_offered_races_it_can_run_and_that_suit_it() {
    let mut g = Game::new();
    g.money = 100_000;
    let mut offered = 0;
    for week in 1..=calendar::WEEKS_PER_YEAR {
        g.week = week;
        for i in 0..g.field.len() {
            let e = g.stable_entrant(i);
            for r in g.auto_choices(i, None) {
                offered += 1;
                assert_eq!(r.week, week);
                assert!(r
                    .eligibility
                    .allows(g.field[i].age, g.field[i].sex == Sex::Female));
                assert!(r
                    .grade
                    .admits(g.field[i].record.starts, g.field[i].record.wins));
                assert_eq!(
                    form::distance_effect(r.distance, e.apt_min, e.apt_max).fit,
                    form::DistanceFit::Fit
                );
                assert!(r.surface == Surface::Turf || g.field[i].dirt >= 0);
            }
        }
    }
    assert!(offered > 0);
}

#[test]
fn a_strong_horse_goes_for_the_richest_race_on_offer() {
    let (mut g, _) = auto_runner();
    g.field[0].speed = MAX_STAT;
    let richest = g.auto_choices(0, None).iter().map(|r| r.prize).max();
    assert_eq!(g.auto_race(0, &[], None).map(|r| r.prize), richest);
}

#[test]
fn a_weak_horse_goes_for_the_easiest_race_on_offer() {
    let (mut g, _) = auto_runner();
    g.field[0].speed = 1;
    let easiest = g
        .auto_choices(0, None)
        .iter()
        .map(|r| rivals::speed_centre(r.grade))
        .min();
    let picked = g.auto_race(0, &[], None).unwrap();
    assert_eq!(Some(rivals::speed_centre(picked.grade)), easiest);
}

#[test]
fn a_horse_does_not_aim_above_the_class_it_can_compete_in() {
    let (mut g, _) = auto_runner();
    let classes: Vec<i64> = g
        .auto_choices(0, None)
        .iter()
        .map(|r| rivals::speed_centre(r.grade))
        .collect();
    let lowest = *classes.iter().min().unwrap();
    g.field[0].speed = lowest as u32; // exactly up to the lowest class on offer
    let picked = g.auto_race(0, &[], None).unwrap();
    assert_eq!(rivals::speed_centre(picked.grade), lowest);
}

#[test]
fn the_week_ending_runs_an_entered_horse_without_a_bet() {
    let (mut g, name) = auto_runner();
    let race = g.auto_race(0, &[], None).unwrap();
    g.field[0].jockey = Some(7);
    let (money, round) = (g.money - g.weekly_upkeep() - JOCKEYS[7].fee, g.round);
    let other = g.field[1].record;
    g.advance_week();
    assert_eq!(g.field[0].record.starts, 1);
    assert_eq!(g.field[1].record, other);
    assert_eq!(
        g.field[0].fatigue,
        race_fatigue(race.distance) - WEEKLY_RECOVERY
    );
    assert_eq!(g.round, round + 1);
    let h = g.history.last().unwrap();
    assert_eq!(h.race.as_deref(), Some(race.name.as_str()));
    assert_eq!((h.ticket_count, h.stake, h.payout), (0, 0, 0));
    assert!(h.horse.starts_with(&name));
    assert_eq!(g.money, money + h.prize);
    assert!(
        g.news
            .iter()
            .any(|n| n.contains(&name) && n.contains(&race.name)),
        "{:?}",
        g.news
    );
}

#[test]
fn nothing_runs_until_a_horse_is_entered_automatically() {
    let (mut g, name) = auto_runner();
    g.set_auto_entry(&name, false).unwrap();
    let (money, round) = (g.money - g.weekly_upkeep(), g.round);
    g.advance_week();
    assert_eq!(
        (g.field[0].record.starts, g.money, g.round),
        (0, money, round)
    );
    assert!(g.news.is_empty());
    assert_eq!(
        g.set_auto_entry("いない馬", true),
        Err(GameError::UnknownHorse)
    );
}

#[test]
fn a_tired_horse_or_one_at_rest_stays_at_home() {
    let (mut g, _) = auto_runner();
    g.field[0].fatigue = FATIGUE_FREE + 1;
    assert!(g.auto_entries(None).is_empty());
    g.field[0].fatigue = FATIGUE_FREE;
    assert!(!g.auto_entries(None).is_empty());
    g.field[0].fatigue = 0;
    g.field[0].resting = true;
    assert!(g.auto_entries(None).is_empty());
}

#[test]
fn no_races_are_run_once_the_game_is_over() {
    let (mut g, _) = auto_runner();
    g.money = 0;
    g.advance_week();
    assert_eq!((g.field[0].record.starts, g.money), (0, 0));
}

#[test]
fn a_horse_that_ran_in_the_players_race_does_not_run_again_that_week() {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    for i in 0..g.field.len() {
        g.field[i].auto_entry = true;
    }
    let spec = g.race_spec(Some(&key)).unwrap();
    for (r, names) in g.auto_entries(Some(&spec)) {
        assert!(!names.contains(&mine));
        assert!(
            (r.day, r.venue, r.race_no) != (race.day, race.venue, race.race_no),
            "the race already run is not on offer"
        );
    }
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    let starts = g.field[i].record.starts;
    let rival = a_rival_in(&g, &key);
    g.play_combo(Some(&key), &[rival], 10, 1, BetKind::Win)
        .unwrap();
    assert_eq!(g.field[i].record.starts, starts + 1);
}

#[test]
fn no_more_of_the_stable_run_in_a_race_than_the_limit_allows() {
    let mut g = Game::new();
    for i in 0..g.field.len() {
        g.field[i].auto_entry = true;
        g.field[i].age = 4;
    }
    let mut most = 0;
    for week in 1..=calendar::WEEKS_PER_YEAR {
        g.week = week;
        let entries = g.auto_entries(None);
        let mut all: Vec<&String> = entries.iter().flat_map(|(_, n)| n).collect();
        let total = all.len();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), total, "a horse runs once a week");
        for (_, names) in &entries {
            assert!(names.len() <= MAX_OWN_ENTRIES);
            most = most.max(names.len());
        }
    }
    assert!(most > 0);
}

#[test]
fn a_stable_left_to_itself_races_for_years_and_stays_solvent() {
    let play = || {
        let mut g = Game::new();
        for i in 0..g.field.len() {
            g.field[i].auto_entry = true;
            g.field[i].auto_training = true;
        }
        for _ in 0..(2 * calendar::WEEKS_PER_YEAR as usize) {
            g.advance_week();
            assert!(!g.is_game_over());
        }
        g
    };
    let g = play();
    assert!(g.all(|h| h.record).iter().map(|r| r.starts).sum::<u32>() > 0);
    assert!(g.all(|h| h.fatigue).iter().all(|&f| f <= MAX_FATIGUE));
    assert!(g.history.len() <= HISTORY_LIMIT);
    assert_eq!(g.money, play().money); // the same seed plays out the same way
}

#[test]
fn auto_entry_is_kept_in_a_save_and_old_saves_have_none() {
    let (g, _) = auto_runner();
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.all(|h| h.auto_entry), g.all(|h| h.auto_entry));
    let v = serde_json::to_value(back.state()).unwrap();
    assert_eq!(v["field"][0]["auto_entry"], true);
    let mut old: serde_json::Value = serde_json::to_value(Game::new()).unwrap();
    old.as_object_mut().unwrap().remove("auto_entry");
    let old = Game::from_json(&old.to_string()).unwrap();
    assert!(old.all(|h| h.auto_entry).iter().all(|a| !a));
}
