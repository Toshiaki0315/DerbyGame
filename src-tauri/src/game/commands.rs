//! The commands the screen calls.

use super::*;

/// What the screen can call.
pub fn handler<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
{
    tauri::generate_handler![
        get_game_state,
        reset_game,
        train_horse,
        save_game,
        load_game,
        breed_horses,
        retire_horse,
        set_gear,
        geld_horse,
        rest_horse,
        set_auto_training,
        set_auto_entry,
        set_all_auto,
        set_trainer,
        sell_horse,
        pasture_horse,
        set_jockey,
        recall_horse,
        release_stud,
        get_stud_market,
        buy_stud,
        set_order,
        go_abroad,
        upgrade_farm,
        get_advice,
        accept_advice,
        get_auction,
        bid_at_auction,
        calendar::get_calendar,
        course::get_courses,
        get_course_fits,
        get_race_card,
        quote_bet,
        place_bets,
        watch_race,
        quote_tickets,
        expand_tickets,
        set_entries,
        next_week,
        bloodline::get_bloodline,
        breeding_outlook,
        track::get_track_effects
    ]
}

#[tauri::command]
pub fn get_game_state(store: tauri::State<GameStore>) -> GameState {
    store.0.lock().unwrap().state()
}

/// Chooses which of the stable's horses run in a race this week (`race` of `None` is the free race).
#[tauri::command]
pub fn set_entries(
    store: tauri::State<GameStore>,
    race: Option<RaceKey>,
    horses: Vec<String>,
) -> Result<(), String> {
    store
        .0
        .lock()
        .unwrap()
        .set_entries(race.as_ref(), &horses)
        .map_err(|e| e.to_string())
}

/// Buys several tickets in one race and runs it.
#[tauri::command]
pub fn place_bets(
    store: tauri::State<GameStore>,
    tickets: Vec<Ticket>,
    seed: u64,
    race: Option<RaceKey>,
) -> Result<RoundOutcome, String> {
    store
        .0
        .lock()
        .unwrap()
        .play_tickets(race.as_ref(), &tickets, seed)
        .map_err(|e| e.to_string())
}

/// Runs a race without buying anything.
#[tauri::command]
pub fn watch_race(
    store: tauri::State<GameStore>,
    seed: u64,
    race: Option<RaceKey>,
) -> Result<RoundOutcome, String> {
    store
        .0
        .lock()
        .unwrap()
        .watch_race(race.as_ref(), seed)
        .map_err(|e| e.to_string())
}

/// The odds each of these tickets would pay.
#[tauri::command]
pub fn quote_tickets(
    store: tauri::State<GameStore>,
    tickets: Vec<Ticket>,
    race: Option<RaceKey>,
) -> Result<Vec<f64>, String> {
    store
        .0
        .lock()
        .unwrap()
        .quote_tickets(race.as_ref(), &tickets)
        .map_err(|e| e.to_string())
}

/// The tickets of a ボックス or 流し on these horses.
#[tauri::command]
pub fn expand_tickets(
    store: tauri::State<GameStore>,
    style: BetStyle,
    kind: BetKind,
    axis: Option<String>,
    horses: Vec<String>,
    race: Option<RaceKey>,
) -> Result<Vec<Vec<String>>, String> {
    store
        .0
        .lock()
        .unwrap()
        .expand_selection(race.as_ref(), style, kind, axis.as_deref(), &horses)
        .map_err(|e| e.to_string())
}

/// The odds a ticket would pay, without buying it.
#[tauri::command]
pub fn quote_bet(
    store: tauri::State<GameStore>,
    horses: Vec<String>,
    kind: BetKind,
    race: Option<RaceKey>,
) -> Result<f64, String> {
    store
        .0
        .lock()
        .unwrap()
        .quote_bet(race.as_ref(), kind, &horses)
        .map_err(|e| e.to_string())
}

/// The field and odds of a race this week; `None` is the free race.
#[tauri::command]
pub fn get_race_card(
    store: tauri::State<GameStore>,
    race: Option<RaceKey>,
) -> Result<RaceCard, String> {
    store
        .0
        .lock()
        .unwrap()
        .race_card(race.as_ref())
        .map_err(|e| e.to_string())
}

/// Skips to the next week without racing.
#[tauri::command]
pub fn next_week(store: tauri::State<GameStore>) -> GameState {
    let mut g = store.0.lock().unwrap();
    g.advance_week();
    g.state()
}

#[tauri::command]
pub fn train_horse(
    store: tauri::State<GameStore>,
    horse: String,
    workout: Workout,
) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.workout(&horse, workout).map_err(|e| e.to_string())?;
    Ok(g.state())
}

fn save_path<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<std::path::PathBuf, String> {
    use tauri::Manager;
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("save.json"))
}

#[tauri::command]
pub fn save_game<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    store: tauri::State<GameStore>,
) -> Result<(), String> {
    store.0.lock().unwrap().save_to(&save_path(&app)?)
}

#[tauri::command]
pub fn load_game<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    store: tauri::State<GameStore>,
) -> Result<GameState, String> {
    let loaded = Game::load_from(&save_path(&app)?)?;
    let mut g = store.0.lock().unwrap();
    *g = loaded;
    Ok(g.state())
}

#[tauri::command]
pub fn breed_horses(
    store: tauri::State<GameStore>,
    sire: String,
    dam: String,
    replace: String,
    name: String,
) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.breed(&sire, &dam, &replace, &name)
        .map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Leaves a horse's weekly training to the trainer (お任せ), or takes it back.
#[tauri::command]
pub fn set_auto_training(
    store: tauri::State<GameStore>,
    horse: String,
    on: bool,
) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.set_auto_training(&horse, on).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Leaves the whole stable to its trainers, or takes it back.
#[tauri::command]
pub fn set_all_auto(store: tauri::State<GameStore>, on: bool) -> GameState {
    let mut g = store.0.lock().unwrap();
    g.set_all_auto(on);
    g.state()
}

/// Enters a horse in a suitable race each week (自動出走), or takes it back.
#[tauri::command]
pub fn set_auto_entry(
    store: tauri::State<GameStore>,
    horse: String,
    on: bool,
) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.set_auto_entry(&horse, on).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// How every course suits a horse.
#[tauri::command]
pub fn get_course_fits(
    store: tauri::State<GameStore>,
    horse: String,
) -> Result<Vec<VenueFit>, String> {
    let g = store.0.lock().unwrap();
    g.course_fits(&horse).map_err(|e| e.to_string())
}

/// Chooses a horse's jockey, or leaves it to the trainer.
#[tauri::command]
pub fn set_jockey(
    store: tauri::State<GameStore>,
    horse: String,
    jockey: Option<u8>,
) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.set_jockey(&horse, jockey).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Sends a horse to pasture.
#[tauri::command]
pub fn pasture_horse(store: tauri::State<GameStore>, name: String) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.send_to_pasture(&name).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Brings a horse back from pasture early.
#[tauri::command]
pub fn recall_horse(store: tauri::State<GameStore>, name: String) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.recall(&name).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Sells a horse out of the stable.
#[tauri::command]
pub fn sell_horse(store: tauri::State<GameStore>, name: String) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.sell_horse(&name).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Sends a horse to another trainer's stable.
#[tauri::command]
pub fn set_trainer(
    store: tauri::State<GameStore>,
    horse: String,
    trainer: u8,
) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.set_trainer(&horse, trainer).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Rests a horse for the week (休養).
#[tauri::command]
pub fn rest_horse(store: tauri::State<GameStore>, horse: String) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.rest(&horse).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Sets a horse's racing equipment.
#[tauri::command]
pub fn set_gear(
    store: tauri::State<GameStore>,
    horse: String,
    menko: bool,
    shadow_roll: bool,
    blinkers: bool,
) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.set_gear(
        &horse,
        Gear {
            menko,
            shadow_roll,
            blinkers,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Gelds a colt (去勢).
#[tauri::command]
pub fn geld_horse(store: tauri::State<GameStore>, name: String) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.geld(&name).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Retires a racehorse to stud.
#[tauri::command]
pub fn retire_horse(store: tauri::State<GameStore>, name: String) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.retire(&name).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// The stallions and mares on sale this year.
#[tauri::command]
pub fn get_stud_market(store: tauri::State<GameStore>) -> Vec<StudOffer> {
    store.0.lock().unwrap().stud_market()
}

/// Buys a stallion or mare for the stud book.
#[tauri::command]
pub fn buy_stud(store: tauri::State<GameStore>, blood_id: u32) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.buy_stud(blood_id).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Sends a horse abroad for a race.
#[tauri::command]
pub fn go_abroad(
    store: tauri::State<GameStore>,
    horse: String,
    race: usize,
) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.go_abroad(&horse, race).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Improves a facility of the farm.
#[tauri::command]
pub fn upgrade_farm(
    store: tauri::State<GameStore>,
    facility: Facility,
) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.upgrade_farm(facility).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// What each horse's trainer has to say.
#[tauri::command]
pub fn get_advice(store: tauri::State<GameStore>) -> Vec<Advice> {
    store.0.lock().unwrap().advice()
}

/// Does what the trainer proposes for a horse.
#[tauri::command]
pub fn accept_advice(store: tauri::State<GameStore>, horse: String) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.accept_advice(&horse).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Gives a horse's jockey an order, or leaves it to the jockey (none).
#[tauri::command]
pub fn set_order(
    store: tauri::State<GameStore>,
    horse: String,
    order: Option<Style>,
) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.set_order(&horse, order).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// Lets a stud go from the stud book.
#[tauri::command]
pub fn release_stud(store: tauri::State<GameStore>, name: String) -> Result<GameState, String> {
    let mut g = store.0.lock().unwrap();
    g.release_stud(&name).map_err(|e| e.to_string())?;
    Ok(g.state())
}

/// This year's lots at the yearling sale, while it is on.
#[tauri::command]
pub fn get_auction(store: tauri::State<GameStore>) -> Vec<AuctionLot> {
    store.0.lock().unwrap().auction()
}

/// Bids for a lot at the yearling sale.
#[tauri::command]
pub fn bid_at_auction(
    store: tauri::State<GameStore>,
    lot: u32,
    max_bid: u32,
    name: String,
) -> Result<BidResult, String> {
    let mut g = store.0.lock().unwrap();
    let outcome = g
        .bid_at_auction(lot, max_bid, &name)
        .map_err(|e| e.to_string())?;
    Ok(BidResult {
        won: outcome.won,
        price: outcome.price,
        state: g.state(),
    })
}

/// What mating a stallion and a mare would bring (inbreeding and nick), without doing it.
#[tauri::command]
pub fn breeding_outlook(
    store: tauri::State<GameStore>,
    sire: String,
    dam: String,
) -> Result<bloodline::Outlook, String> {
    store
        .0
        .lock()
        .unwrap()
        .breeding_outlook(&sire, &dam)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_game(store: tauri::State<GameStore>) -> GameState {
    let mut g = store.0.lock().unwrap();
    g.reset();
    g.state()
}
