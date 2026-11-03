//! Orders, weight and jockeys (作戦・斤量・騎手).

use super::*;

// ---- orders to the jockey (作戦指示) ----

#[test]
fn a_runner_goes_by_its_own_way_unless_told_otherwise() {
    let mut g = Game::with_seed(1);
    let name = g.field[0].name.clone();
    let own = g.stable_entrant(0).style;
    let other = if own == Some(Style::Nige) {
        Style::Oikomi
    } else {
        Style::Nige
    };
    g.set_order(&name, Some(other)).unwrap();
    assert_eq!(g.stable_entrant(0).style, Some(other));
    assert_eq!(g.state().field[0].order, Some(other));
    g.set_order(&name, None).unwrap();
    assert_eq!(g.stable_entrant(0).style, own);
    assert_eq!(g.state().field[0].order, None);
}

#[test]
fn a_horse_made_to_run_against_its_way_is_harder_to_settle() {
    let mut g = Game::with_seed(1);
    let name = g.field[0].name.clone();
    let own = g.stable_entrant(0).style.unwrap();
    let temper = g.stable_entrant(0).temper;
    g.set_order(&name, Some(own)).unwrap();
    assert_eq!(g.stable_entrant(0).temper, temper);
    let other = if own == Style::Nige {
        Style::Oikomi
    } else {
        Style::Nige
    };
    g.set_order(&name, Some(other)).unwrap();
    assert_eq!(
        g.stable_entrant(0).temper,
        temper.saturating_sub(ORDER_TEMPER_COST)
    );
}

#[test]
fn orders_go_for_an_unknown_horse_are_refused_and_they_survive_a_save() {
    let mut g = Game::with_seed(1);
    assert_eq!(
        g.set_order("Nobody", Some(Style::Nige)),
        Err(GameError::UnknownHorse)
    );
    let name = g.field[0].name.clone();
    g.set_order(&name, Some(Style::Sashi)).unwrap();
    let restored = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(restored.state().field[0].order, Some(Style::Sashi));
}

#[test]
fn a_foal_taking_a_horses_place_does_not_inherit_its_orders() {
    let mut g = Game::with_seed(1);
    let name = g.field[5].name.clone();
    g.set_order(&name, Some(Style::Sashi)).unwrap();
    breed_default(&mut g).unwrap();
    assert_eq!(g.state().field[5].order, None);
}

// ---- the weight a horse carries (斤量) ----

#[test]
fn every_runner_in_a_calendar_race_carries_a_weight_and_the_free_race_has_none() {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let spec = g.race_spec(Some(&key_of(&race))).unwrap();
    assert!(spec.entrants.iter().all(|e| e.weight.is_some()));
    let free = g.race_spec(None).unwrap();
    assert!(free.entrants.iter().all(|e| e.weight.is_none()));
}

#[test]
fn a_runner_carries_what_the_race_rule_says() {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let spec = g.race_spec(Some(&key_of(&race))).unwrap();
    let mine = spec.entrants.iter().find(|e| e.mine).unwrap();
    let i = g.field.iter().position(|h| h.name == mine.name).unwrap();
    assert_eq!(
        mine.weight,
        Some(weight::carried(
            &race.eligibility,
            race.week,
            mine.age,
            mine.sex == Sex::Female,
            g.field[i].record.wins,
            mine.speed,
            rivals::speed_centre(race.grade),
        ))
    );
}

#[test]
fn a_heavier_weight_takes_something_out_of_a_horse() {
    let plain = fit_runner(4, 4, (1000, 3600));
    let mut heavy = plain.clone();
    heavy.weight = Some(60);
    let mut light = plain.clone();
    light.weight = Some(50);
    let stats =
        |e: &crate::rivals::Entrant| Game::run_stats(e, Surface::Turf, 1800, TrackCondition::Good);
    let (s0, t0) = stats(&plain);
    assert_eq!(
        stats(&heavy),
        (
            (s0 as f64 * weight::multiplier(60)).round() as u32,
            (t0 as f64 * weight::multiplier(60)).round() as u32
        )
    );
    assert!(stats(&light).0 > s0);
}

#[test]
fn in_a_handicap_the_stronger_horse_carries_more() {
    let race = calendar::shared()
        .races
        .iter()
        .find(|r| r.eligibility.weight == WeightRule::Handicap)
        .unwrap()
        .clone();
    let mut g = Game::new();
    g.money = 100_000;
    g.week = race.week;
    let key = key_of(&race);
    let carried_by = |g: &Game, i: usize| {
        let name = g.field[i].name.clone();
        g.race_spec(Some(&key))
            .unwrap()
            .entrants
            .iter()
            .find(|e| e.name == name)
            .and_then(|e| e.weight)
    };
    let i = (0..g.field.len())
        .find(|&i| {
            race.eligibility
                .allows(g.field[i].age, g.field[i].sex == Sex::Female)
        })
        .unwrap();
    let name = g.field[i].name.clone();
    g.field[i].record = RaceRecord::default();
    g.set_entries(Some(&key), std::slice::from_ref(&name))
        .unwrap();
    g.field[i].speed = 90;
    let strong = carried_by(&g, i).unwrap();
    g.field[i].speed = 50;
    let weak = carried_by(&g, i).unwrap();
    assert!(strong > weak, "{strong} vs {weak}");
}

#[test]
fn the_race_card_shows_the_weights() {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let key = key_of(&calendar_race("国際友好ステークス"));
    let spec = g.race_spec(Some(&key)).unwrap();
    let card = g.race_card(Some(&key)).unwrap();
    for (c, e) in card.entrants.iter().zip(&spec.entrants) {
        assert_eq!(c.weight, e.weight);
    }
    assert!(g
        .race_card(None)
        .unwrap()
        .entrants
        .iter()
        .all(|e| e.weight.is_none()));
}

// ---- jockeys (騎手) ----

/// A calendar race with one of the stable's horses in it.
fn ridden() -> (Game, CalendarRace, RaceKey, usize) {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let mine = my_runner(&g, &key);
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    (g, race, key, i)
}

fn ride_of(g: &Game, key: &RaceKey, name: &str) -> Ride {
    g.race_spec(Some(key))
        .unwrap()
        .entrants
        .iter()
        .find(|e| e.name == name)
        .unwrap()
        .jockey
        .clone()
        .unwrap()
}

#[test]
fn a_jockey_can_be_chosen_for_a_horse_or_left_to_the_trainer() {
    let (mut g, _, key, i) = ridden();
    let name = g.field[i].name.clone();
    assert_eq!(g.field[i].jockey, None);
    g.set_jockey(&name, Some(3)).unwrap();
    assert_eq!(g.field[i].jockey, Some(3));
    assert_eq!(ride_of(&g, &key, &name).id, 3);
    g.set_jockey(&name, None).unwrap();
    assert_eq!(g.field[i].jockey, None);
    assert_eq!(g.set_jockey(&name, Some(10)), Err(GameError::UnknownJockey));
    assert_eq!(g.set_jockey("いない馬", None), Err(GameError::UnknownHorse));
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["jockeys"].as_array().unwrap().len(), 10);
    assert!(v["field"][i]["jockey"].is_null());
}

#[test]
fn a_trainer_books_a_jockey_to_suit_what_they_are_aiming_for() {
    let (mut g, _, key, i) = ridden();
    let name = g.field[i].name.clone();
    let fee_with = |g: &mut Game, aim: trainer::Aim| {
        g.field[i].trainer = TRAINERS.iter().find(|t| t.aim == aim).unwrap().id;
        ride_of(g, &key, &name).fee
    };
    let glory = fee_with(&mut g, trainer::Aim::Glory);
    let balanced = fee_with(&mut g, trainer::Aim::Balanced);
    let steady = fee_with(&mut g, trainer::Aim::Steady);
    assert!(
        glory > balanced && balanced > steady,
        "{glory} {balanced} {steady}"
    );
    assert!(balanced <= jockey::BUDGET_BALANCED && steady <= jockey::BUDGET_STEADY);
}

#[test]
fn the_rivals_and_the_free_race_have_no_jockey_to_speak_of() {
    let (g, _, key, _) = ridden();
    let spec = g.race_spec(Some(&key)).unwrap();
    assert!(spec
        .entrants
        .iter()
        .filter(|e| !e.mine)
        .all(|e| e.jockey.is_none()));
    assert!(spec
        .entrants
        .iter()
        .filter(|e| e.mine)
        .all(|e| e.jockey.is_some()));
    let free = g.race_spec(None).unwrap();
    assert!(free.entrants.iter().all(|e| e.jockey.is_none()));
}

#[test]
fn two_of_the_stable_never_share_a_jockey_in_a_race() {
    let (mut g, _, key, _) = ridden();
    for i in 0..g.field.len() {
        g.field[i].jockey = Some(0);
    }
    let spec = g.race_spec(Some(&key)).unwrap();
    let ids: Vec<u8> = spec
        .entrants
        .iter()
        .filter_map(|e| e.jockey.as_ref().map(|j| j.id))
        .collect();
    assert!(ids.len() >= 2, "needs two of the stable in this race");
    let unique: std::collections::HashSet<u8> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len());
    assert!(ids.contains(&0));
}

#[test]
fn a_good_jockey_brings_more_out_of_a_horse() {
    let plain = fit_runner(4, 4, (1000, 3600));
    let mut up = plain.clone();
    up.jockey = Some(Ride {
        id: 0,
        name: "A".into(),
        percent: 2.0,
        tact: 0,
        fee: 1,
    });
    let stats =
        |e: &crate::rivals::Entrant| Game::run_stats(e, Surface::Turf, 1800, TrackCondition::Good);
    let (s0, t0) = stats(&plain);
    assert_eq!(
        stats(&up),
        (
            (s0 as f64 * 1.02).round() as u32,
            (t0 as f64 * 1.02).round() as u32
        )
    );
}

#[test]
fn a_tactful_jockey_settles_a_hot_horse() {
    let (g, _, key, i) = ridden();
    let mut spec = g.race_spec(Some(&key)).unwrap();
    let name = g.field[i].name.clone();
    let at = spec.entrants.iter().position(|e| e.name == name).unwrap();
    spec.entrants[at].temper = 5;
    let pulls = |spec: &RaceSpec| {
        (0..400u64)
            .filter(|&seed| Game::pulled(spec, seed).contains(&name))
            .count()
    };
    spec.entrants[at].jockey = None;
    let alone = pulls(&spec);
    spec.entrants[at].jockey = Some(Ride {
        id: 0,
        name: "A".into(),
        percent: 0.0,
        tact: 40,
        fee: 1,
    });
    let settled = pulls(&spec);
    assert!(settled < alone, "{settled} vs {alone}");
}

#[test]
fn each_ride_costs_its_fee_win_or_lose() {
    let (mut g, _, key, i) = ridden();
    let name = g.field[i].name.clone();
    g.set_jockey(&name, Some(0)).unwrap();
    let spec = g.race_spec(Some(&key)).unwrap();
    let fees: u32 = spec
        .entrants
        .iter()
        .filter_map(|e| e.jockey.as_ref().map(|j| j.fee))
        .sum();
    assert!(fees >= 20);
    let bill = g.weekly_upkeep();
    let rival = a_rival_in(&g, &key);
    let before = g.money;
    let out = g
        .play_combo(Some(&key), &[rival], 10, 1, BetKind::Win)
        .unwrap();
    assert_eq!(
        g.money,
        before - 10 + out.payout as u32 + out.prize_total - fees - bill
    );
}

#[test]
fn a_fee_never_finishes_a_stable_off() {
    let mut g = mid_horse();
    let spec = {
        g.money = 100_000;
        g.week = 47;
        let race = calendar_race("国際友好ステークス");
        g.race_spec(Some(&key_of(&race))).unwrap()
    };
    g.money = 1;
    g.pay_jockeys(&spec);
    assert_eq!(g.money, 1);
}

#[test]
fn a_horse_goes_better_for_a_jockey_it_keeps_and_starts_again_with_a_new_one() {
    let (mut g, race, key, i) = ridden();
    let name = g.field[i].name.clone();
    g.set_jockey(&name, Some(3)).unwrap();
    let first = ride_of(&g, &key, &name).percent;
    let spec = g.race_spec(Some(&key)).unwrap();
    let results = Game::finish_order(&spec, 1);
    g.add_to_records(&spec, &results);
    assert_eq!(g.field[i].partner, Some((3, 1)));
    let second = ride_of(&g, &key, &name).percent;
    assert!(second > first);
    g.add_to_records(&spec, &results);
    assert_eq!(g.field[i].partner, Some((3, 2)));
    let _ = race;
    // one not already booked by a stablemate in the race
    let busy: Vec<u8> = spec
        .entrants
        .iter()
        .filter_map(|e| e.jockey.as_ref().map(|j| j.id))
        .collect();
    let other = (0..10u8).find(|j| !busy.contains(j)).unwrap();
    g.set_jockey(&name, Some(other)).unwrap();
    let spec = g.race_spec(Some(&key)).unwrap();
    g.add_to_records(&spec, &results);
    assert_eq!(g.field[i].partner, Some((other, 1)));
}

#[test]
fn the_race_card_names_the_jockeys() {
    let (mut g, _, key, i) = ridden();
    let name = g.field[i].name.clone();
    g.set_jockey(&name, Some(2)).unwrap();
    let card = g.race_card(Some(&key)).unwrap();
    let mine = card.entrants.iter().find(|e| e.name == name).unwrap();
    assert_eq!(mine.jockey.as_deref(), Some("水無月 凛"));
    assert!(card
        .entrants
        .iter()
        .filter(|e| !e.mine)
        .all(|e| e.jockey.is_none()));
}

#[test]
fn an_automatic_run_pays_its_jockey_too() {
    let (mut g, _) = auto_runner();
    g.field[0].jockey = Some(0);
    let money = g.money - g.weekly_upkeep() - JOCKEYS[0].fee;
    g.advance_week();
    let h = g.history.last().unwrap();
    assert_eq!(g.money, money + h.prize);
    assert_eq!(g.field[0].partner, Some((0, 1)));
}

#[test]
fn jockeys_are_kept_in_a_save_and_an_old_save_leaves_them_to_the_trainer() {
    let (mut g, _, key, i) = ridden();
    let name = g.field[i].name.clone();
    g.set_jockey(&name, Some(2)).unwrap();
    let spec = g.race_spec(Some(&key)).unwrap();
    let results = Game::finish_order(&spec, 1);
    g.add_to_records(&spec, &results);
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.all(|h| h.jockey), g.all(|h| h.jockey));
    assert_eq!(back.all(|h| h.partner), g.all(|h| h.partner));
    let mut old = old_save(&Game::new());
    let o = old.as_object_mut().unwrap();
    o.remove("jockey_of");
    o.remove("partner");
    let old = Game::from_json(&old.to_string()).unwrap();
    assert!(old.all(|h| h.jockey).iter().all(|j| j.is_none()));
}

#[test]
fn a_foal_starts_with_no_jockey_of_its_own() {
    let mut g = Game::new();
    g.money = 100_000;
    let (sire, dam) = stallion_and_mare(&g);
    let slot = g
        .field
        .iter()
        .position(|h| h.name != sire && h.name != dam)
        .unwrap();
    g.field[slot].jockey = Some(1);
    g.field[slot].partner = Some((1, 3));
    let replace = g.field[slot].name.clone();
    g.breed_now(&sire, &dam, &replace, "ハナミズキ").unwrap();
    assert_eq!((g.field[slot].jockey, g.field[slot].partner), (None, None));
    g.breed_now(&sire, &dam, "", "サクラソウ").unwrap();
}
