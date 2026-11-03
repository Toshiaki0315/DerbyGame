//! The stable's money, and leaving the whole stable to its trainers (預託料・売却).

use super::*;

// ---- leaving the whole stable to its trainers ----

#[test]
fn the_trainers_leave_more_in_the_bank_the_bigger_the_stable() {
    let mut g = Game::new();
    g.studs.clear();
    g.field.truncate(0);
    assert_eq!(g.auto_keep(), AUTO_KEEP_MONEY);
    let g = Game::new();
    assert_eq!(g.auto_keep(), g.weekly_upkeep() * AUTO_KEEP_WEEKS);
    assert!(g.auto_keep() > AUTO_KEEP_MONEY);
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["auto_keep_money"], g.auto_keep());
}

#[test]
fn a_stable_left_to_itself_gets_through_its_first_year_without_running_short() {
    for seed in 1..=10u64 {
        let mut g = Game::with_seed(seed);
        g.entry_rules = true;
        g.set_all_auto(true);
        let n = g.field.len();
        for week in 1..=calendar::WEEKS_PER_YEAR {
            g.advance_week();
            assert!(
                g.money >= 4 * g.weekly_upkeep(),
                "seed {seed} week {week}: {} against {} a week",
                g.money,
                g.weekly_upkeep()
            );
        }
        assert_eq!(g.field.len(), n, "seed {seed} had to sell");
    }
}

#[test]
fn a_stable_left_to_itself_for_four_years_may_shrink_but_does_not_go_under() {
    // 19 and 11 are the seeds that do worst: they have to sell horses to pay their way
    for seed in [1u64, 11, 19, 23] {
        let mut g = Game::with_seed(seed);
        g.entry_rules = true;
        g.set_all_auto(true);
        for _ in 0..4 * calendar::WEEKS_PER_YEAR as u32 {
            g.advance_week();
            assert!(
                !g.is_game_over(),
                "seed {seed} in year {} week {}",
                g.year,
                g.week
            );
        }
        assert!(
            g.field.len() >= 4,
            "seed {seed} is down to {}",
            g.field.len()
        );
    }
}

#[test]
fn the_whole_stable_can_be_left_to_its_trainers_and_taken_back() {
    let mut g = Game::new();
    g.field[1].auto_training = true;
    g.set_all_auto(true);
    assert!(g.all(|h| h.auto_training).iter().all(|a| *a));
    assert!(g.all(|h| h.auto_entry).iter().all(|a| *a));
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["field"][0]["auto_entry"], true);
    g.set_all_auto(false);
    assert!(g.all(|h| h.auto_training).iter().all(|a| !a));
    assert!(g.all(|h| h.auto_entry).iter().all(|a| !a));
}

// ---- the stable's money (預託料・売却・種付け料) ----

#[test]
fn every_horse_and_stud_costs_money_each_week() {
    let mut g = Game::new();
    g.money = 100_000;
    let bill = g.field.len() as u32 * UPKEEP_PER_HORSE + g.studs.len() as u32 * UPKEEP_PER_STUD;
    assert!(bill > 0);
    assert_eq!(g.weekly_upkeep(), bill);
    assert_eq!(g.state().weekly_upkeep, bill);
    g.advance_week();
    assert_eq!(g.money, 100_000 - bill);
}

#[test]
fn a_calendar_race_week_is_charged_too_but_a_free_race_is_not() {
    let mut g = Game::new();
    g.money = 100_000;
    let bill = g.weekly_upkeep();
    let week = g.week;
    let n = g.field[0].name.clone();
    g.set_entries(None, std::slice::from_ref(&n)).unwrap();
    let before = g.money;
    let out = g.play_combo(None, &[n], 10, 1, BetKind::Win).unwrap();
    assert_eq!(g.week, week);
    assert_eq!(g.money, before - 10 + out.payout as u32);
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let key = key_of(&race);
    let rival = a_rival_in(&g, &key);
    let before = g.money;
    let fees = race_fees(&g, &key);
    let out = g
        .play_combo(Some(&key), &[rival], 10, 1, BetKind::Win)
        .unwrap();
    assert_eq!(
        g.money,
        before - 10 + out.payout as u32 + out.prize_total - fees - bill
    );
    assert_eq!(out.money, g.money);
}

#[test]
fn a_stable_short_of_money_lets_its_cheapest_horse_go() {
    let mut g = Game::new();
    for i in 0..g.field.len() {
        g.field[i].speed = 90;
        g.field[i].stamina = 90;
        g.field[i].power = 90;
    }
    g.field[3].speed = 40;
    g.field[3].stamina = 40;
    g.field[3].power = 40;
    let (cheap, price) = (g.field[3].name.clone(), g.sale_price(3));
    let n = g.field.len();
    let bill = g.weekly_upkeep();
    g.money = bill; // paying it all would leave nothing
    g.advance_week();
    assert_eq!(g.field.len(), n - 1);
    assert!(g.field.iter().all(|h| h.name != cheap));
    assert!(
        g.news
            .iter()
            .any(|t| t.contains(&cheap) && t.contains("預託料")),
        "{:?}",
        g.news
    );
    assert_eq!(g.money, bill + price - g.weekly_upkeep());
    assert!(!g.is_game_over());
}

#[test]
fn a_stable_goes_under_only_when_it_has_nothing_left_to_sell() {
    let mut g = Game::new();
    g.studs.clear();
    while g.field.len() > 1 {
        let name = g.field[0].name.clone();
        g.sell_horse(&name).unwrap();
    }
    g.money = UPKEEP_PER_HORSE;
    g.advance_week();
    assert_eq!(g.field.len(), 1);
    assert!(g.is_game_over());
    assert!(
        g.news.iter().any(|t| t.contains("資金が尽き")),
        "{:?}",
        g.news
    );
    g.advance_week(); // nothing more to take
    assert_eq!(g.money, 0);
}

#[test]
fn a_session_costs_more_each_time_up_to_a_limit() {
    let mut g = mid_horse();
    let name = g.field[0].name.clone();
    let cost = |g: &Game| g.train_cost(&name).unwrap();
    g.field[0].trainings = 0;
    assert_eq!(cost(&g), TRAIN_BASE_COST);
    g.field[0].trainings = 2;
    assert_eq!(cost(&g), TRAIN_BASE_COST + 2 * TRAIN_COST_STEP);
    g.field[0].trainings = 10_000;
    assert_eq!(cost(&g), TRAIN_MAX_COST);
    let money = g.money;
    g.workout(&name, Workout::Wood).unwrap();
    assert_eq!(g.money, money - TRAIN_MAX_COST);
}

#[test]
fn a_win_in_a_low_class_pays_for_a_few_weeks_of_keep_and_a_g1_for_years() {
    let maiden = (550.0 * PRIZE_RATE).round() as u32;
    assert!(maiden >= 2 * UPKEEP_PER_HORSE, "{maiden}");
    let g1 = (20000.0 * PRIZE_RATE).round() as u32;
    assert!(g1 >= 52 * UPKEEP_PER_HORSE, "{g1}");
}

#[test]
fn a_horse_is_worth_more_the_better_it_is_and_the_more_it_has_won() {
    assert!(priced(80, 4, 0) > priced(60, 4, 0));
    assert!(priced(60, 4, 3) > priced(60, 4, 0));
    assert!(priced(60, 4, 0) > priced(60, 8, 0));
    assert_eq!(priced(60, 4, 0), priced(60, 2, 0));
    assert_eq!(priced(1, 9, 0), SALE_MIN);
    assert_eq!(priced(60, 4, 0) % 10, 0);
    assert_eq!(priced(90, 4, 0), SALE_MIN + (270 - 150) * SALE_PER_STAT);
}

#[test]
fn breeding_foals_to_sell_them_does_not_make_money_on_average() {
    // an ordinary foal is worth less than it costs to breed
    assert!(priced(70, 2, 0) <= BREED_COST + 100);
    assert!(priced(60, 2, 0) < BREED_COST);
}

#[test]
fn selling_a_horse_brings_in_its_price_and_it_leaves_the_stable() {
    let mut g = Game::new();
    let name = g.field[2].name.clone();
    let (price, money, n) = (g.sale_price(2), g.money, g.field.len());
    g.sell_horse(&name).unwrap();
    assert_eq!(g.money, money + price);
    assert_eq!(g.field.len(), n - 1);
    assert!(g.field.iter().all(|h| h.name != name));
    assert!(g.studs.iter().all(|s| s.name != name));
    assert_eq!(g.sell_horse("いない馬"), Err(GameError::UnknownHorse));
}

#[test]
fn the_last_horse_cannot_be_sold_nor_any_once_the_game_is_over() {
    let mut g = Game::new();
    while g.field.len() > 1 {
        let name = g.field[0].name.clone();
        g.sell_horse(&name).unwrap();
    }
    let last = g.field[0].name.clone();
    assert_eq!(g.sell_horse(&last), Err(GameError::LastHorse));
    let mut g = Game::new();
    g.money = 0;
    let name = g.field[0].name.clone();
    assert_eq!(g.sell_horse(&name), Err(GameError::GameOver));
}

#[test]
fn the_screen_is_told_each_horses_price() {
    let g = Game::new();
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["field"][1]["sale_price"], g.sale_price(1));
}

/// A stallion on the market, and a mare of the stable.
fn outside_sire(g: &Game) -> (StudOffer, String) {
    let offer = g
        .stud_market()
        .into_iter()
        .find(|o| o.sex == Sex::Male)
        .unwrap();
    let (_, dam) = stallion_and_mare(g);
    (offer, dam)
}

#[test]
fn a_market_stallion_can_cover_a_mare_for_a_fee() {
    let mut g = Game::new();
    g.money = 100_000;
    let (offer, dam) = outside_sire(&g);
    assert!(offer.stud_fee > 0 && offer.stud_fee < offer.price);
    let studs = g.studs.len();
    g.breeding_outlook(&offer.name, &dam).unwrap();
    g.breed_now(&offer.name, &dam, &g.field[7].name.clone(), "ハナミズキ")
        .unwrap();
    assert_eq!(g.money, 100_000 - BREED_COST - offer.stud_fee);
    assert_eq!(g.studs.len(), studs); // the stallion stays where it is
    let foal = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    let rec = g.stable_blood(foal).unwrap();
    assert_eq!(rec.sire_id, Some(offer.blood_id));
}

#[test]
fn the_stables_own_stallion_comes_without_a_fee() {
    let mut g = Game::new();
    g.money = 100_000;
    let (sire, dam) = stallion_and_mare(&g);
    g.breed_now(&sire, &dam, &g.field[7].name.clone(), "ハナミズキ")
        .unwrap();
    assert_eq!(g.money, 100_000 - BREED_COST);
}

#[test]
fn the_fee_must_be_affordable_and_a_market_mare_does_not_count() {
    let mut g = Game::new();
    let (offer, dam) = outside_sire(&g);
    g.money = BREED_COST + offer.stud_fee - 1;
    let slot = g.field[7].name.clone();
    assert_eq!(
        g.breed_now(&offer.name, &dam, &slot, "ハナミズキ"),
        Err(GameError::InsufficientFunds)
    );
    let mare = g
        .stud_market()
        .into_iter()
        .find(|o| o.sex == Sex::Female)
        .unwrap();
    assert_eq!(mare.stud_fee, 0);
    g.money = 100_000;
    let (sire, _) = stallion_and_mare(&g);
    assert_eq!(
        g.breed_now(&sire, &mare.name, &slot, "ハナミズキ"),
        Err(GameError::UnknownHorse)
    );
}
