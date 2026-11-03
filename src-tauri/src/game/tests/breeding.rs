//! The breeding season, conception, foals and mating (繁殖・配合).

use super::*;

// ---- the breeding season and the foal's coming (繁殖期と出産) ----

/// Retires the last racing mare to stud and gives her name.
fn second_mare(g: &mut Game) -> String {
    let i = (0..g.field.len())
        .rev()
        .find(|&i| g.field[i].sex == Sex::Female)
        .unwrap();
    let name = g.field[i].name.clone();
    g.retire(&name).unwrap();
    name
}

#[test]
fn the_season_is_about_three_months_in_spring() {
    assert_eq!(BREEDING_END_WEEK - BREEDING_START_WEEK + 1, 13);
}

#[test]
fn a_mating_outside_the_season_is_refused_and_costs_nothing() {
    for week in [1, BREEDING_START_WEEK - 1, BREEDING_END_WEEK + 1, 52] {
        let (mut g, s, d, _) = in_season();
        g.week = week;
        assert_eq!(
            g.cover(&s, &d, "", "ハナミズキ"),
            Err(GameError::OutOfSeason)
        );
        assert_eq!(g.money, START_MONEY);
        assert!(g.state().pregnancies.is_empty());
    }
}

#[test]
fn a_mating_may_be_made_on_the_first_and_last_week_of_the_season() {
    for week in [BREEDING_START_WEEK, BREEDING_END_WEEK] {
        let (mut g, s, d, _) = in_season();
        g.week = week;
        g.cover(&s, &d, "", "ハナミズキ").unwrap();
    }
}

#[test]
fn the_state_says_whether_the_season_is_open() {
    let (mut g, _, _, _) = in_season();
    let st = g.state();
    assert!(st.breeding_open);
    assert_eq!(st.breeding_start_week, BREEDING_START_WEEK);
    assert_eq!(st.breeding_end_week, BREEDING_END_WEEK);
    g.week = BREEDING_END_WEEK + 1;
    assert!(!g.state().breeding_open);
}

#[test]
fn a_mating_is_paid_for_at_once_but_no_foal_comes_yet() {
    let (mut g, s, d, r) = in_season();
    let before = g.field.len();
    g.cover(&s, &d, &r, "ハナミズキ").unwrap();
    assert_eq!(g.money, START_MONEY - BREED_COST);
    assert_eq!(g.field.len(), before);
    assert!(g.field.iter().any(|h| h.name == r));
    assert!(g.field.iter().all(|h| h.name != "ハナミズキ"));
    assert!(g.bloodline_all().iter().all(|h| h.name != "ハナミズキ"));
    let st = g.state();
    assert_eq!(st.pregnancies.len(), 1);
    let p = &st.pregnancies[0];
    assert_eq!(
        (
            p.foal_name.as_str(),
            p.sire.as_str(),
            p.dam.as_str(),
            p.replace.as_str()
        ),
        ("ハナミズキ", s.as_str(), d.as_str(), r.as_str())
    );
    assert_eq!(p.weeks_left, GESTATION_WEEKS);
}

#[test]
fn the_foal_comes_about_a_year_after_the_mating() {
    assert!((44..=52).contains(&GESTATION_WEEKS));
    let (mut g, s, d, _) = in_season();
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    weeks_pass(&mut g, GESTATION_WEEKS - 1);
    assert!(g.field.iter().all(|h| h.name != "ハナミズキ"));
    assert_eq!(g.state().pregnancies[0].weeks_left, 1);
    weeks_pass(&mut g, 1);
    let st = g.state();
    assert!(st.pregnancies.is_empty());
    assert!(st
        .news
        .iter()
        .any(|n| n.contains("ハナミズキ") && n.contains("誕生")));
    let foal = st.field.iter().find(|h| h.name == "ハナミズキ").unwrap();
    assert_eq!(foal.age, 0);
    let pedigree = foal.pedigree.as_ref().unwrap();
    assert_eq!(
        (pedigree.sire.as_str(), pedigree.dam.as_str()),
        (s.as_str(), d.as_str())
    );
    assert_eq!(g.year, 2);
    assert_eq!(
        g.week,
        BREEDING_START_WEEK + GESTATION_WEEKS - calendar::WEEKS_PER_YEAR
    );
    assert!(g.bloodline_all().iter().any(|h| h.name == "ハナミズキ"));
}

#[test]
fn the_foal_comes_though_the_weeks_end_with_a_race() {
    let (mut g, s, d, _) = in_season();
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    g.money = 1_000_000;
    g.pregnancies[0].weeks_left = 1;
    g.finish_week(None);
    assert!(g.field.iter().any(|h| h.name == "ハナミズキ"));
}

#[test]
fn the_horse_to_be_replaced_races_on_until_the_foal_comes() {
    let (mut g, s, d, r) = in_season();
    g.cover(&s, &d, &r, "ハナミズキ").unwrap();
    let places = g.field.len();
    weeks_pass(&mut g, GESTATION_WEEKS - 1);
    assert!(g.field.iter().any(|h| h.name == r));
    weeks_pass(&mut g, 1);
    assert!(g.field.iter().all(|h| h.name != r));
    assert!(g.field.iter().any(|h| h.name == "ハナミズキ"));
    assert_eq!(g.field.len(), places);
    assert!(g.news.iter().any(|n| n.contains(&r)));
}

#[test]
fn a_foal_takes_an_empty_place_if_the_horse_it_was_to_replace_has_gone() {
    let (mut g, s, d, r) = in_season();
    g.cover(&s, &d, &r, "ハナミズキ").unwrap();
    let places = g.field.len();
    g.retire(&r).unwrap();
    g.pregnancies[0].weeks_left = 1;
    g.money = 1_000_000;
    g.advance_week();
    assert_eq!(g.field.len(), places);
    assert!(g.field.iter().any(|h| h.name == "ハナミズキ"));
}

#[test]
fn a_mare_in_foal_cannot_be_mated_again_until_she_has_foaled() {
    let (mut g, s, d, _) = in_season();
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    let paid = g.money;
    assert_eq!(
        g.cover(&s, &d, "", "サクラソウ"),
        Err(GameError::MareInFoal)
    );
    assert_eq!(g.money, paid);
    // the next spring she has foaled and is free again
    weeks_pass(&mut g, GESTATION_WEEKS);
    g.week = BREEDING_START_WEEK;
    g.cover(&s, &d, "", "サクラソウ").unwrap();
}

#[test]
fn a_stallion_can_cover_several_mares_in_a_season() {
    let (mut g, s, d, _) = in_season();
    let other = second_mare(&mut g);
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    g.cover(&s, &other, "", "サクラソウ").unwrap();
    assert_eq!(g.state().pregnancies.len(), 2);
}

#[test]
fn a_foals_name_is_taken_from_the_mating_on() {
    let (mut g, s, d, _) = in_season();
    let other = second_mare(&mut g);
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    assert_eq!(
        g.cover(&s, &other, "", "ハナミズキ"),
        Err(GameError::DuplicateName)
    );
}

#[test]
fn foals_on_the_way_each_have_their_own_record() {
    let (mut g, s, d, _) = in_season();
    let other = second_mare(&mut g);
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    g.cover(&s, &other, "", "サクラソウ").unwrap();
    weeks_pass(&mut g, GESTATION_WEEKS);
    let id = |g: &Game, name: &str| {
        let i = g.field.iter().position(|h| h.name == name).unwrap();
        g.field[i].blood_id
    };
    assert_ne!(id(&g, "ハナミズキ"), id(&g, "サクラソウ"));
}

#[test]
fn a_foal_still_comes_if_its_dam_has_been_let_go() {
    let (mut g, s, d, _) = in_season();
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    g.release_stud(&d).unwrap();
    weeks_pass(&mut g, GESTATION_WEEKS);
    let st = g.state();
    let foal = st.field.iter().find(|h| h.name == "ハナミズキ").unwrap();
    assert_eq!(foal.pedigree.as_ref().unwrap().dam, d);
}

#[test]
fn foals_on_the_way_count_against_the_room_in_the_stable() {
    let (mut g, s, d, _) = in_season();
    let other = second_mare(&mut g);
    let third = second_mare(&mut g);
    while g.field.len() < STABLE_SIZE - 1 {
        let name = format!("カリ{}", g.field.len());
        let mut horse = StableHorse::starter(&name, 50, 50, 0);
        horse.age = 3;
        horse.sex = Sex::Male;
        horse.dirt = 0;
        horse.power = 50;
        horse.temper = 50;
        horse.record = RaceRecord::default();
        horse.legs = 60;
        g.field.push(horse);
    }
    g.money = 100_000;
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    assert_eq!(
        g.cover(&s, &other, "", "サクラソウ"),
        Err(GameError::StableFull)
    );
    // a horse named to make way leaves room, but it can make way only once
    let r = g.field[2].name.clone();
    assert_eq!(g.cover(&s, &other, &r, "サクラソウ"), Ok(()));
    assert_eq!(
        g.cover(&s, &third, &r, "ナノハナ"),
        Err(GameError::StableFull)
    );
}

#[test]
fn a_foal_joins_a_stable_left_entirely_to_itself_on_the_same_terms() {
    let (mut g, s, d, _) = in_season();
    g.set_all_auto(true);
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    weeks_pass(&mut g, GESTATION_WEEKS);
    let i = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    assert!(g.field[i].auto_training && g.field[i].auto_entry);
}

#[test]
fn a_foal_is_left_to_the_player_when_the_stable_is_not_all_automatic() {
    let (mut g, s, d, _) = in_season();
    let racing = g.field[0].name.clone();
    g.set_auto_training(&racing, true).unwrap();
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    weeks_pass(&mut g, GESTATION_WEEKS);
    let i = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    assert!(!g.field[i].auto_training && !g.field[i].auto_entry);
}

#[test]
fn a_pregnancy_survives_save_and_load() {
    let (mut g, s, d, _) = in_season();
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    weeks_pass(&mut g, 5);
    let mut restored = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(restored.state(), g.state());
    assert_eq!(
        restored.state().pregnancies[0].weeks_left,
        GESTATION_WEEKS - 5
    );
    weeks_pass(&mut restored, GESTATION_WEEKS - 5);
    assert!(restored.field.iter().any(|h| h.name == "ハナミズキ"));
}

#[test]
fn a_save_from_before_the_breeding_season_still_loads() {
    let g = Game::with_seed(1);
    let mut json: serde_json::Value = old_save(&g);
    json.as_object_mut().unwrap().remove("pregnancies");
    let restored = Game::from_json(&json.to_string()).unwrap();
    assert!(restored.state().pregnancies.is_empty());
}

// ---- who may be bred from, and how a foal grows up (繁殖牝馬と仔馬の成長) ----

fn a_race_for_debutants() -> crate::calendar::CalendarRace {
    calendar::shared()
        .races
        .iter()
        .find(|r| {
            r.eligibility.age == AgeRule::Two && !r.eligibility.fillies_only && r.grade.admits(0, 0)
        })
        .unwrap()
        .clone()
}

#[test]
fn only_a_retired_mare_can_be_bred_from() {
    let (mut g, s, _, _) = in_season();
    let racing = g.field[4].name.clone();
    assert_eq!(g.field[4].sex, Sex::Female);
    assert_eq!(
        g.cover(&s, &racing, "", "ハナミズキ"),
        Err(GameError::DamNotRetired)
    );
    assert_eq!(g.money, START_MONEY);
    assert!(g.pregnancies.is_empty());
    g.retire(&racing).unwrap();
    g.cover(&s, &racing, "", "ハナミズキ").unwrap();
}

#[test]
fn only_a_retired_stallion_can_stand_at_stud() {
    let (mut g, _, d, _) = in_season();
    let racing = g.field[0].name.clone();
    assert_eq!(g.field[0].sex, Sex::Male);
    assert_eq!(
        g.breed(&racing, &d, "", "ハナミズキ"),
        Err(GameError::SireNotRetired)
    );
    assert_eq!(g.money, START_MONEY);
    g.retire(&racing).unwrap();
    g.cover(&racing, &d, "", "ハナミズキ").unwrap();
}

#[test]
fn a_stallion_on_the_market_can_be_used_for_a_fee() {
    let (mut g, _, d, _) = in_season();
    g.money = 100_000;
    let offer = g
        .stud_market()
        .into_iter()
        .find(|o| o.sex == Sex::Male)
        .unwrap();
    g.cover(&offer.name, &d, "", "ハナミズキ").unwrap();
}

// ---- conception (受胎) ----

#[test]
fn a_young_mare_takes_about_four_times_in_five_and_an_old_one_less() {
    assert_eq!(conception_percent(4), 80);
    assert_eq!(conception_percent(CONCEPTION_FALLS_FROM), 80);
    assert_eq!(conception_percent(CONCEPTION_FALLS_FROM + 5), 60);
    assert_eq!(conception_percent(40), CONCEPTION_FLOOR);
}

#[test]
fn the_stud_book_shows_how_likely_a_mare_is_to_take() {
    let (mut g, _, d, _) = in_season();
    let i = g.studs.iter().position(|h| h.name == d).unwrap();
    g.studs[i].retired_age = 12;
    g.year = 5;
    g.studs[i].retired_year = 2;
    let st = g.state();
    let mare = st.studs.iter().find(|h| h.name == d).unwrap();
    assert_eq!(mare.conception, conception_percent(15));
}

#[test]
fn matings_often_but_not_always_take() {
    let (base, s, d, _) = in_season();
    let mut took = 0;
    for seed in 0..200 {
        let mut g = with_rng(&base, seed);
        g.breed(&s, &d, "", "ハナミズキ").unwrap();
        took += g.pregnancies.len();
    }
    assert!((130..=185).contains(&took), "{took}");
}

#[test]
fn a_mating_that_does_not_take_is_still_paid_for_and_leaves_her_free() {
    let (base, s, d, _) = in_season();
    let mut g = (0..200)
        .map(|seed| {
            let mut g = with_rng(&base, seed);
            g.breed(&s, &d, "", "ハナミズキ").unwrap();
            g
        })
        .find(|g| g.pregnancies.is_empty())
        .unwrap();
    assert_eq!(g.money, START_MONEY - BREED_COST);
    // the name is not used up, and she can be tried again in the season
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    assert_eq!(g.pregnancies.len(), 1);
}

#[test]
fn a_refused_mating_does_not_disturb_the_dice() {
    let (mut g, s, _, _) = in_season();
    let rng = g.rng;
    let _ = g.breed(&s, "Nobody", "", "ハナミズキ");
    assert_eq!(g.rng, rng);
}

#[test]
fn a_foal_is_born_at_nought_and_stays_at_the_farm_for_a_year() {
    let (g, i) = with_a_newborn();
    assert_eq!(g.field[i].age, 0);
    let away = g.field[i].away.as_ref().unwrap();
    assert_eq!(away.kind, AwayKind::Growing);
    assert_eq!(away.weeks, FOAL_FARM_WEEKS);
    assert_eq!(FOAL_FARM_WEEKS, calendar::WEEKS_PER_YEAR as u32);
}

#[test]
fn a_foal_at_the_farm_cannot_be_worked_rested_or_sent_away() {
    let (mut g, i) = with_a_newborn();
    let name = g.field[i].name.clone();
    assert_eq!(g.workout(&name, Workout::Wood), Err(GameError::Away));
    assert_eq!(g.rest(&name), Err(GameError::Away));
    assert_eq!(g.send_to_pasture(&name), Err(GameError::Away));
    assert_eq!(g.recall(&name), Err(GameError::NotOnPasture));
}

#[test]
fn a_foal_starts_training_a_year_after_it_is_born() {
    let (mut g, i) = with_a_newborn();
    let name = g.field[i].name.clone();
    weeks_pass(&mut g, (FOAL_FARM_WEEKS - 1) as u8);
    assert_eq!(g.field[i].away.as_ref().unwrap().weeks, 1);
    weeks_pass(&mut g, 1);
    assert!(g.field[i].away.is_none());
    assert_eq!(g.field[i].age, 1);
    assert!(g
        .news
        .iter()
        .any(|n| n.contains(&name) && n.contains("調教")));
    g.workout(&name, Workout::Wood).unwrap();
}

#[test]
fn a_foal_left_to_its_trainer_is_not_trained_until_it_comes_from_the_farm() {
    let (mut g, s, d, _) = in_season();
    g.set_all_auto(true);
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    weeks_pass(&mut g, GESTATION_WEEKS);
    let i = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    let speed = g.field[i].speed;
    weeks_pass(&mut g, (FOAL_FARM_WEEKS - 1) as u8);
    assert_eq!(g.field[i].speed, speed);
    assert_eq!(g.field[i].trainings, 0);
}

#[test]
fn no_race_takes_a_horse_under_two_and_a_debut_race_takes_a_two_year_old() {
    let (mut g, i) = with_a_newborn();
    g.field[i].away = None;
    g.field[i].fatigue = 0;
    for age in [0, 1] {
        g.field[i].age = age;
        for race in &calendar::shared().races {
            assert!(!g.eligible_horses(race).contains(&i), "{}", race.name);
        }
        assert!(!g.eligible_horses(&Game::free_race()).contains(&i));
    }
    g.field[i].age = DEBUT_AGE;
    assert!(g.eligible_horses(&a_race_for_debutants()).contains(&i));
}

#[test]
fn a_yearling_is_not_entered_automatically() {
    let (mut g, i) = with_a_newborn();
    g.set_all_auto(true);
    g.field[i].away = None;
    g.field[i].age = 1;
    g.week = 1;
    weeks_pass(&mut g, 40);
    assert_eq!(g.field[i].record.starts, 0);
}

#[test]
fn a_foal_cannot_retire_until_it_is_old_enough_to_race() {
    let (mut g, i) = with_a_newborn();
    let name = g.field[i].name.clone();
    assert_eq!(g.retire(&name), Err(GameError::TooYoung));
    g.field[i].age = DEBUT_AGE;
    g.retire(&name).unwrap();
}

#[test]
fn a_foal_at_the_farm_costs_half_the_upkeep() {
    let (mut g, i) = with_a_newborn();
    let growing = g.weekly_upkeep();
    g.field[i].away = None;
    assert_eq!(g.weekly_upkeep(), growing + UPKEEP_PER_HORSE / 2);
}

#[test]
fn a_foal_at_the_farm_survives_save_and_load() {
    let (g, _) = with_a_newborn();
    assert_eq!(Game::from_json(&g.to_json()).unwrap().state(), g.state());
}

#[test]
fn a_full_field_is_eighteen_runners() {
    assert_eq!(entries::FULL_FIELD, 18);
    let mut g = Game::with_seed(1);
    g.money = 100_000;
    g.week = 47;
    let key = key_of(&calendar_race("国際友好ステークス"));
    let card = g.race_card(Some(&key)).unwrap();
    assert_eq!(card.entrants.len(), 18);
}

#[test]
fn a_place_ticket_never_pays_the_long_odds_of_a_win_ticket() {
    let g = Game::with_seed(1);
    let card = g.race_card(None).unwrap();
    assert!(card.entrants.iter().all(|e| e.place_odds <= MAX_PLACE_ODDS));
    assert!(card.entrants.iter().any(|e| e.odds > MAX_PLACE_ODDS));
}

// ---- mating theories (配合理論) ----

#[test]
fn a_good_rating_puts_something_on_a_foal() {
    let (base, s, d, _) = in_season();
    let mut with = base.clone();
    with.mate(&s, &d, "", "ハナミズキ", 100).unwrap();
    let foal = with.pregnancies[0].foal.clone();
    let outlook = base.breeding_outlook(&s, &d).unwrap();
    let points: u32 = outlook.theories.iter().map(|t| t.points).sum();
    assert!(points > 0);
    // the same mating without the theories, from the same dice
    let mut plain = base.clone();
    let db = plain.blood_map();
    let (sire, _) = plain.parent(&s).unwrap();
    let (dam, _) = plain.parent(&d).unwrap();
    let id = plain.next_blood_id();
    let mut rng = rivals::Rng::new(plain.next_u64());
    let gen = sire.gen.max(dam.gen) + 1;
    let without = bloodline::breed(&sire, &dam, gen, id, "ハナミズキ".into(), &db, &mut rng);
    let expect = points as f64 * THEORY_STAT_PER_POINT;
    assert!((foal.stats.speed - without.stats.speed - expect).abs() < 1e-9);
    assert!((foal.stats.stamina - without.stats.stamina - expect).abs() < 1e-9);
}

// ---- breeding (配合) ----
// default stable: stallions at 0,1,2,3,6 and mares at 4,5,7

#[test]
fn stable_has_stallions_and_mares() {
    let st = Game::new().state();
    let male = st.field.iter().filter(|h| h.sex == Sex::Male).count();
    let female = st.field.iter().filter(|h| h.sex == Sex::Female).count();
    assert!(male >= 2 && female >= 2);
    assert_eq!(male + female, 8);
    assert_eq!(st.breed_cost, BREED_COST);
}

#[test]
fn breeding_replaces_a_horse_with_a_named_foal_and_costs_money() {
    let mut g = Game::with_seed(1);
    let retired = g.field[5].name.clone();
    breed_default(&mut g).unwrap();
    let st = g.state();
    assert_eq!(st.money, START_MONEY - BREED_COST);
    assert_eq!(st.field.len(), 8);
    assert_eq!(st.field[5].name, "ニューホープ");
    assert!(st.field.iter().all(|h| h.name != retired));
}

#[test]
fn foal_inherits_pedigree_from_its_parents() {
    let mut g = Game::with_seed(1);
    let before = g.state();
    breed_default(&mut g).unwrap();
    let foal = &g.state().field[5];
    let p = foal.pedigree.as_ref().unwrap();
    let sire = &before.field[0];
    let dam = &before.field[4];
    assert_eq!(p.sire, sire.name);
    assert_eq!(p.dam, dam.name);
    assert_eq!(p.sire_sire, sire.pedigree.as_ref().unwrap().sire);
    assert_eq!(p.sire_dam, sire.pedigree.as_ref().unwrap().dam);
    assert_eq!(p.dam_sire, dam.pedigree.as_ref().unwrap().sire);
    assert_eq!(p.dam_dam, dam.pedigree.as_ref().unwrap().dam);
}

#[test]
fn foal_stats_stay_in_bounds_and_follow_the_parents() {
    let (mut strong_total, mut weak_total) = (0u32, 0u32);
    for seed in 0..40u64 {
        for (parent_stat, total) in [(92u32, &mut strong_total), (50u32, &mut weak_total)] {
            let mut g = with_rng(&shared_base(), seed);
            for i in [0usize, 4] {
                g.field[i].speed = parent_stat;
                g.field[i].stamina = parent_stat;
            }
            breed_default(&mut g).unwrap();
            let foal = &g.field[5];
            for v in [foal.speed, foal.stamina] {
                assert!(
                    (FOAL_MIN_STAT..=FOAL_MAX_STAT).contains(&v),
                    "stat {v} out of bounds"
                );
            }
            *total += foal.speed + foal.stamina;
        }
    }
    assert!(
        strong_total > weak_total + 40 * 2 * 20,
        "{strong_total} vs {weak_total}"
    );
}

#[test]
fn foals_are_not_all_identical_and_both_sexes_appear() {
    let mut stats = std::collections::HashSet::new();
    let mut sexes = std::collections::HashSet::new();
    for seed in 0..30u64 {
        let mut g = with_rng(&shared_base(), seed);
        breed_default(&mut g).unwrap();
        stats.insert((g.field[5].speed, g.field[5].stamina));
        sexes.insert(format!("{:?}", g.state().field[5].sex));
    }
    assert!(stats.len() > 5, "too little variety: {}", stats.len());
    assert_eq!(sexes.len(), 2);
}

#[test]
fn breeding_is_deterministic_for_a_seed() {
    let mut a = Game::with_seed(7);
    let mut b = Game::with_seed(7);
    breed_default(&mut a).unwrap();
    breed_default(&mut b).unwrap();
    assert_eq!(a.state(), b.state());
}

#[test]
fn foal_starts_with_fresh_training_cost() {
    let mut g = Game::with_seed(2);
    let slot_name = g.field[5].name.clone();
    g.workout(&slot_name, Workout::Wood).unwrap();
    g.workout(&slot_name, Workout::Wood).unwrap();
    breed_default(&mut g).unwrap();
    assert_eq!(g.train_cost("ニューホープ"), Some(TRAIN_BASE_COST));
    assert_eq!(g.train_cost(&slot_name), None);
}

#[test]
fn parents_must_be_a_stallion_and_a_mare() {
    let mut g = Game::new();
    let n = |i: usize, g: &Game| g.field[i].name.clone();
    // mare as sire
    assert_eq!(
        g.breed_now(&n(4, &g), &n(5, &g), &n(7, &g), "ニューホープ"),
        Err(GameError::WrongSex)
    );
    // stallion as dam
    assert_eq!(
        g.breed_now(&n(0, &g), &n(1, &g), &n(7, &g), "ニューホープ"),
        Err(GameError::WrongSex)
    );
    // swapped
    assert_eq!(
        g.breed_now(&n(4, &g), &n(0, &g), &n(7, &g), "ニューホープ"),
        Err(GameError::WrongSex)
    );
    assert_eq!(g.money, START_MONEY);
}

#[test]
fn unknown_horses_are_rejected() {
    let mut g = Game::new();
    let (s, d) = (g.field[0].name.clone(), g.field[4].name.clone());
    assert_eq!(
        g.breed_now("Nobody", &d, &s, "ニューホープ"),
        Err(GameError::UnknownHorse)
    );
    assert_eq!(
        g.breed_now(&s, "Nobody", &s, "ニューホープ"),
        Err(GameError::UnknownHorse)
    );
    assert_eq!(
        g.breed_now(&s, &d, "Nobody", "ニューホープ"),
        Err(GameError::UnknownHorse)
    );
}

#[test]
fn foal_name_must_follow_the_naming_rule_and_be_unique() {
    let mut g = Game::new();
    let (s, d, r) = (
        g.field[0].name.clone(),
        g.field[4].name.clone(),
        g.field[5].name.clone(),
    );
    for bad in [
        "",
        "ア",
        "新星",
        "Newstar",
        "しんせい",
        "オウゴンオーシャンズ",
    ] {
        assert_eq!(
            g.breed_now(&s, &d, &r, bad),
            Err(GameError::InvalidName),
            "{bad}"
        );
    }
    let existing = g.field[2].name.clone();
    assert_eq!(
        g.breed_now(&s, &d, &r, &existing),
        Err(GameError::DuplicateName)
    );
    assert_eq!(g.money, START_MONEY);
}

#[test]
fn breeding_needs_money_and_a_running_game() {
    let mut g = Game::new();
    g.money = BREED_COST - 1;
    assert_eq!(breed_default(&mut g), Err(GameError::InsufficientFunds));
    assert_eq!(g.money, BREED_COST - 1);
    g.money = 0;
    assert_eq!(breed_default(&mut g), Err(GameError::GameOver));
}

#[test]
fn failed_breeding_changes_nothing() {
    let mut g = Game::new();
    let before = g.state();
    let (s, d) = (g.field[0].name.clone(), g.field[1].name.clone());
    let _ = g.breed_now(&s, &d, &s, "ニューホープ");
    assert_eq!(g.state(), before);
}

#[test]
fn a_parent_can_be_the_one_retired() {
    let mut g = Game::with_seed(3);
    let (s, d) = (g.field[0].name.clone(), g.field[4].name.clone());
    g.breed_now(&s, &d, &s, "ニューホープ").unwrap();
    assert_eq!(g.field[0].name, "ニューホープ");
    assert_eq!(g.state().field[0].pedigree.as_ref().unwrap().sire, s);
}

#[test]
fn breeding_survives_save_and_load() {
    let mut g = Game::with_seed(4);
    breed_default(&mut g).unwrap();
    let restored = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(restored.state(), g.state());
    assert_eq!(restored.state().field[5].name, "ニューホープ");
}

#[test]
fn old_saves_without_sexes_or_pedigrees_still_load() {
    let g = Game::with_seed(4);
    let mut v: serde_json::Value = old_save(&g);
    v.as_object_mut().unwrap().remove("sexes");
    v.as_object_mut().unwrap().remove("blood_ids");
    let old = Game::from_json(&v.to_string()).unwrap();
    assert_eq!(old.state().field[0].sex, g.state().field[0].sex);
    assert_eq!(old.state().field[0].pedigree, g.state().field[0].pedigree);
}

#[test]
fn reset_restores_the_original_stable() {
    let mut g = Game::with_seed(5);
    let original: Vec<String> = g.field.iter().map(|h| h.name.clone()).collect();
    breed_default(&mut g).unwrap();
    g.reset();
    let names: Vec<String> = g.field.iter().map(|h| h.name.clone()).collect();
    assert_eq!(names, original);
}

#[test]
fn bred_horse_can_race_and_be_bet_on() {
    let mut g = Game::with_seed(6);
    breed_default(&mut g).unwrap();
    let out = g.play_round("ニューホープ", 10, 1).unwrap();
    assert_eq!(out.results.len(), 8);
    assert!(g.odds("ニューホープ").is_some());
}
