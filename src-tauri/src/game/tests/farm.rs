//! The farm, the yearling sale and buying breeding stock (牧場・セリ・繁殖馬の購入).

use super::*;

// ---- running the farm (牧場経営) ----

#[test]
fn a_new_farm_has_no_improvements_and_each_costs_more_than_the_last() {
    let mut g = Game::with_seed(1);
    g.money = 100_000;
    let st = g.state();
    assert_eq!((st.farm.pasture, st.farm.nursery), (0, 0));
    assert_eq!(st.farm.pasture_cost, Some(FARM_COSTS[0]));
    assert!(FARM_COSTS.windows(2).all(|c| c[0] < c[1]));
    g.upgrade_farm(Facility::Pasture).unwrap();
    assert_eq!(g.money, 100_000 - FARM_COSTS[0]);
    let st = g.state();
    assert_eq!((st.farm.pasture, st.farm.nursery), (1, 0));
    assert_eq!(st.farm.pasture_cost, Some(FARM_COSTS[1]));
    assert_eq!(st.farm.nursery_cost, Some(FARM_COSTS[0]));
}

#[test]
fn a_facility_stops_at_its_top_level_and_cannot_be_bought_on_credit() {
    let mut g = Game::with_seed(1);
    g.money = FARM_COSTS[0];
    assert_eq!(
        g.upgrade_farm(Facility::Nursery),
        Err(GameError::InsufficientFunds)
    );
    g.money = 1_000_000;
    for _ in 0..FARM_COSTS.len() {
        g.upgrade_farm(Facility::Nursery).unwrap();
    }
    assert_eq!(
        g.upgrade_farm(Facility::Nursery),
        Err(GameError::FacilityFull)
    );
    assert_eq!(g.state().farm.nursery_cost, None);
}

#[test]
fn an_improved_farm_costs_more_to_keep() {
    let mut g = Game::with_seed(1);
    g.money = 100_000;
    let before = g.weekly_upkeep();
    g.upgrade_farm(Facility::Pasture).unwrap();
    g.upgrade_farm(Facility::Nursery).unwrap();
    assert_eq!(g.weekly_upkeep(), before + 2 * UPKEEP_PER_FACILITY_LEVEL);
}

/// A foal's stats and legs when it comes, with the facilities at these levels.
fn foal_from_farm(pasture: u8, nursery: u8) -> (Game, usize) {
    let (mut g, s, d, _) = in_season();
    g.farm = Farm { pasture, nursery };
    g.rng = 12345;
    g.mate(&s, &d, "", "ハナミズキ", 100).unwrap();
    g.pregnancies[0].weeks_left = 1;
    g.deliver_foals();
    let i = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    (g, i)
}

#[test]
fn good_pasture_gives_a_foal_stronger_legs_and_more_stamina() {
    let (poor, i) = foal_from_farm(0, 0);
    let (rich, j) = foal_from_farm(2, 0);
    assert!(rich.field[j].legs > poor.field[i].legs || rich.field[j].legs == LEGS_MAX);
    assert!(rich.field[j].stamina > poor.field[i].stamina);
    assert_eq!(rich.field[j].speed, poor.field[i].speed);
}

#[test]
fn a_good_nursery_brings_a_yearling_on_when_it_comes_in_from_the_farm() {
    let (mut plain, i) = foal_from_farm(0, 0);
    let (mut good, j) = foal_from_farm(0, 2);
    assert_eq!(plain.field[i].speed, good.field[j].speed);
    for g in [&mut plain, &mut good] {
        g.money = 1_000_000;
        for _ in 0..FOAL_FARM_WEEKS {
            g.advance_week();
        }
    }
    assert!(good.field[j].speed > plain.field[i].speed);
    assert!(good.field[j].power > plain.field[i].power);
}

#[test]
fn a_farm_survives_a_save_and_an_old_save_has_a_plain_one() {
    let mut g = Game::with_seed(1);
    g.money = 100_000;
    g.upgrade_farm(Facility::Pasture).unwrap();
    assert_eq!(
        Game::from_json(&g.to_json()).unwrap().state().farm.pasture,
        1
    );
    let mut json: serde_json::Value = old_save(&g);
    json.as_object_mut().unwrap().remove("farm");
    assert_eq!(
        Game::from_json(&json.to_string())
            .unwrap()
            .state()
            .farm
            .pasture,
        0
    );
}

// ---- the yearling sale (1歳馬のセリ) ----

fn at_the_sale() -> Game {
    let mut g = Game::with_seed(1);
    g.week = AUCTION_START_WEEK;
    g.money = 1_000_000;
    g
}

/// The most the others will pay for a lot.
fn others_limit(g: &Game, lot: u32) -> u32 {
    g.auction_entry(lot).unwrap().rival_top
}

#[test]
fn the_sale_is_held_in_july() {
    assert_eq!((AUCTION_START_WEEK, AUCTION_END_WEEK), (27, 30));
    let mut g = at_the_sale();
    assert!(g.state().auction_open);
    for week in [AUCTION_START_WEEK - 1, AUCTION_END_WEEK + 1] {
        g.week = week;
        assert!(!g.state().auction_open);
        assert!(g.auction().is_empty());
        assert_eq!(
            g.bid_at_auction(1, 100_000, "ハナミズキ"),
            Err(GameError::SaleClosed)
        );
    }
}

#[test]
fn the_lots_stay_the_same_through_the_sale_and_change_each_year() {
    let mut g = at_the_sale();
    let lots = g.auction();
    assert_eq!(lots.len(), AUCTION_LOTS as usize);
    g.week = AUCTION_END_WEEK;
    assert_eq!(g.auction(), lots);
    g.year += 1;
    assert_ne!(g.auction(), lots);
}

#[test]
fn a_bid_above_what_the_others_will_pay_wins_at_just_over_their_limit() {
    let mut g = at_the_sale();
    let top = others_limit(&g, 1);
    let start = g.auction()[0].start_price;
    let price = start.max(top + AUCTION_STEP);
    let result = g.bid_at_auction(1, price + 500, "ハナミズキ").unwrap();
    assert_eq!((result.won, result.price), (true, price));
    assert_eq!(g.money, 1_000_000 - price);
}

#[test]
fn a_bought_yearling_joins_the_stable_ready_to_train_but_not_to_race() {
    let mut g = at_the_sale();
    let lot = g.auction()[0].clone();
    let before = g.field.len();
    g.bid_at_auction(1, 500_000, "ハナミズキ").unwrap();
    assert_eq!(g.field.len(), before + 1);
    let i = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    assert_eq!(g.field[i].age, 1);
    assert_eq!(g.field[i].sex, lot.sex);
    assert!(g.field[i].away.is_none());
    g.workout("ハナミズキ", Workout::Wood).unwrap();
    g.field[i].fatigue = 0;
    assert!(calendar::shared()
        .races
        .iter()
        .all(|r| !g.eligible_horses(r).contains(&i)));
    let st = g.state();
    let pedigree = st.field[i].pedigree.as_ref().unwrap();
    assert_eq!(
        (pedigree.sire.as_str(), pedigree.dam.as_str()),
        (lot.sire.as_str(), lot.dam.as_str())
    );
    assert!(g.bloodline_all().iter().any(|h| h.name == "ハナミズキ"));
}

#[test]
fn a_beaten_bid_costs_nothing_and_says_what_the_lot_went_for() {
    let mut g = at_the_sale();
    let top = others_limit(&g, 1);
    let start = g.auction()[0].start_price;
    assert!(top >= start, "the seed should give a contested lot");
    let result = g.bid_at_auction(1, top, "ハナミズキ").unwrap();
    assert_eq!((result.won, result.price), (false, top));
    assert_eq!(g.money, 1_000_000);
    assert!(g.field.iter().all(|h| h.name != "ハナミズキ"));
}

#[test]
fn a_lot_can_be_bid_on_only_once() {
    let mut g = at_the_sale();
    g.bid_at_auction(1, 500_000, "ハナミズキ").unwrap();
    assert!(g.auction()[0].closed);
    assert!(!g.auction()[1].closed);
    assert_eq!(
        g.bid_at_auction(1, 500_000, "サクラソウ"),
        Err(GameError::LotClosed)
    );
    let mut h = at_the_sale();
    let start = h.auction()[0].start_price;
    h.bid_at_auction(1, start.max(others_limit(&h, 1)), "ハナミズキ")
        .unwrap();
    assert_eq!(
        h.bid_at_auction(1, 500_000, "ハナミズキ"),
        Err(GameError::LotClosed)
    );
}

#[test]
fn next_years_sale_is_not_closed_by_this_years_bids() {
    let mut g = at_the_sale();
    g.bid_at_auction(1, 500_000, "ハナミズキ").unwrap();
    g.year += 1;
    assert!(g.auction().iter().all(|l| !l.closed));
}

#[test]
fn a_bad_bid_is_refused_and_leaves_the_lot_open() {
    let mut g = at_the_sale();
    let start = g.auction()[0].start_price;
    assert_eq!(
        g.bid_at_auction(1, start - 10, "ハナミズキ"),
        Err(GameError::BidTooLow)
    );
    assert_eq!(
        g.bid_at_auction(99, 500_000, "ハナミズキ"),
        Err(GameError::UnknownLot)
    );
    assert_eq!(
        g.bid_at_auction(1, 500_000, "bad"),
        Err(GameError::InvalidName)
    );
    let taken = g.field[0].name.clone();
    assert_eq!(
        g.bid_at_auction(1, 500_000, &taken),
        Err(GameError::DuplicateName)
    );
    g.money = start;
    assert_eq!(
        g.bid_at_auction(1, start, "ハナミズキ"),
        Err(GameError::InsufficientFunds)
    );
    assert!(!g.auction()[0].closed);
    assert_eq!(g.money, start);
}

#[test]
fn a_full_stable_cannot_buy() {
    let mut g = at_the_sale();
    while g.field.len() < STABLE_SIZE {
        let name = format!("カリ{}", g.field.len());
        g.field.push(StableHorse::starter(&name, 50, 50, 0));
    }
    assert_eq!(
        g.bid_at_auction(1, 500_000, "ハナミズキ"),
        Err(GameError::StableFull)
    );
}

#[test]
fn a_better_estimate_tends_to_mean_a_better_horse_and_a_higher_price() {
    let mut g = at_the_sale();
    let (mut best, mut worst): (Vec<f64>, Vec<f64>) = (vec![], vec![]);
    let (mut best_price, mut worst_price) = (vec![], vec![]);
    for year in 1..60 {
        g.year = year;
        for lot in 1..=AUCTION_LOTS {
            let entry = g.auction_entry(lot).unwrap();
            match entry.lot.estimate.as_str() {
                "A" => {
                    best.push(stat_sum(&entry.foal));
                    best_price.push(entry.lot.start_price as f64);
                }
                "C" => {
                    worst.push(stat_sum(&entry.foal));
                    worst_price.push(entry.lot.start_price as f64);
                }
                _ => {}
            }
        }
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    assert!(
        best.len() > 20 && worst.len() > 20,
        "{} {}",
        best.len(),
        worst.len()
    );
    assert!(mean(&best) > mean(&worst) + 10.0);
    assert!(mean(&best_price) > mean(&worst_price));
}

#[test]
fn the_estimate_does_not_always_tell_the_truth() {
    let mut g = at_the_sale();
    let mut sums = vec![];
    for year in 1..60 {
        g.year = year;
        for lot in 1..=AUCTION_LOTS {
            let entry = g.auction_entry(lot).unwrap();
            sums.push((entry.lot.estimate.clone(), stat_sum(&entry.foal)));
        }
    }
    let best_a = sums
        .iter()
        .filter(|s| s.0 == "B")
        .map(|s| s.1)
        .fold(0.0, f64::max);
    let worst_a = sums
        .iter()
        .filter(|s| s.0 == "A")
        .map(|s| s.1)
        .fold(f64::MAX, f64::min);
    assert!(best_a > worst_a, "{best_a} {worst_a}");
}

#[test]
fn a_game_saved_mid_sale_keeps_the_lots_closed() {
    let mut g = at_the_sale();
    g.bid_at_auction(1, 500_000, "ハナミズキ").unwrap();
    let restored = Game::from_json(&g.to_json()).unwrap();
    assert!(restored.auction()[0].closed);
    assert_eq!(restored.auction(), g.auction());
}

#[test]
fn a_save_from_before_the_sale_still_loads() {
    let g = Game::with_seed(1);
    let mut json: serde_json::Value = old_save(&g);
    json.as_object_mut().unwrap().remove("auction_closed");
    assert!(Game::from_json(&json.to_string()).is_ok());
}

// ---- buying stallions and mares (繁殖馬の購入) ----

#[test]
fn the_market_offers_four_stallions_and_four_mares_nobody_owns() {
    let g = Game::new();
    let m = g.stud_market();
    assert_eq!(m.len(), 2 * MARKET_EACH);
    assert_eq!(m.iter().filter(|o| o.sex == Sex::Male).count(), MARKET_EACH);
    assert_eq!(
        m.iter().filter(|o| o.sex == Sex::Female).count(),
        MARKET_EACH
    );
    for o in &m {
        let rec = g.blood_record(o.blood_id).unwrap();
        assert!(!rec.owned && rec.gen >= MARKET_MIN_GEN);
        assert_eq!((o.name.clone(), o.sex), (rec.name.clone(), rec.gender));
        assert!(o.price >= 200 && o.price % 10 == 0);
    }
}

#[test]
fn stronger_horses_cost_more() {
    let m = Game::new().stud_market();
    let strongest = m
        .iter()
        .max_by_key(|o| o.speed + o.stamina + o.power)
        .unwrap();
    let weakest = m
        .iter()
        .min_by_key(|o| o.speed + o.stamina + o.power)
        .unwrap();
    assert!(strongest.price >= weakest.price);
}

#[test]
fn the_market_is_the_same_all_year_and_changes_with_the_year() {
    let mut g = Game::new();
    let ids = |g: &Game| {
        g.stud_market()
            .iter()
            .map(|o| o.blood_id)
            .collect::<Vec<_>>()
    };
    let first = ids(&g);
    g.week = 30;
    assert_eq!(ids(&g), first);
    g.year += 1;
    assert_ne!(ids(&g), first);
}

#[test]
fn buying_a_mare_puts_her_in_the_stud_book_for_her_price() {
    let mut g = rich();
    let offer = g
        .stud_market()
        .into_iter()
        .find(|o| o.sex == Sex::Female)
        .unwrap();
    g.buy_stud(offer.blood_id).unwrap();
    assert_eq!(g.money, 100_000 - offer.price);
    let s = g.studs.last().unwrap();
    assert_eq!(
        (
            s.name.clone(),
            s.sex,
            s.speed,
            s.stamina,
            s.power,
            s.temper,
            s.bought,
            s.retired_year
        ),
        (
            offer.name.clone(),
            Sex::Female,
            offer.speed,
            offer.stamina,
            offer.power,
            offer.temper,
            true,
            g.year
        )
    );
    assert_eq!(s.record, RaceRecord::default());
    // she is the stable's now, and off the market
    assert!(g.blood_record(offer.blood_id).unwrap().owned);
    assert!(g.stud_market().iter().all(|o| o.blood_id != offer.blood_id));
    assert!(g.state().studs.last().unwrap().bought);
}

#[test]
fn a_bought_stallion_can_sire_a_foal() {
    let mut g = rich();
    let offer = g
        .stud_market()
        .into_iter()
        .find(|o| o.sex == Sex::Male)
        .unwrap();
    g.buy_stud(offer.blood_id).unwrap();
    let (_, dam) = stallion_and_mare(&g);
    let name = g.field[0].name.clone();
    g.retire(&name).unwrap(); // make room
    g.breed_now(&offer.name, &dam, "", "ハナミズキ").unwrap();
    let i = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    assert_eq!(
        g.blood_record(g.field[i].blood_id).unwrap().sire_id,
        Some(offer.blood_id)
    );
}

#[test]
fn only_horses_on_offer_can_be_bought_with_money_and_room() {
    let mut g = Game::new();
    assert_eq!(g.buy_stud(1), Err(GameError::NotForSale));
    let offer = g.stud_market().into_iter().max_by_key(|o| o.price).unwrap();
    g.money = offer.price - 1;
    assert_eq!(
        g.buy_stud(offer.blood_id),
        Err(GameError::InsufficientFunds)
    );
    let mut g = rich();
    g.studs.clear(); // without the starting pair
    for _ in 0..MAX_STUDS {
        let name = g.field[0].name.clone();
        g.retire(&name).unwrap();
    }
    let offer = g.stud_market()[0].clone();
    assert_eq!(g.buy_stud(offer.blood_id), Err(GameError::StudFull));
    assert_eq!(g.money, 100_000);
}

#[test]
fn a_bought_stud_survives_a_save() {
    let mut g = rich();
    let offer = g.stud_market()[0].clone();
    g.buy_stud(offer.blood_id).unwrap();
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.studs, g.studs);
}

#[test]
fn the_market_json_carries_what_the_ui_shows() {
    let v = serde_json::to_value(Game::new().stud_market()).unwrap();
    for k in [
        "blood_id", "name", "sex", "speed", "stamina", "power", "dirt", "temper", "price",
    ] {
        assert!(!v[0][k].is_null(), "{k}");
    }
    assert!(v[0]["blood"]["lineage"].is_string());
}
