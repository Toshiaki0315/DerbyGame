#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bets;
mod bloodline;
mod calendar;
mod commentary;
mod course;
mod entries;
mod form;
mod game;
mod horse;
mod jockey;
mod race;
mod rivals;
mod track;
mod trainer;
mod weight;

fn main() {
    tauri::Builder::default()
        .manage(game::GameStore(std::sync::Mutex::new(
            game::Game::with_seed(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(1),
            ),
        )))
        .invoke_handler(tauri::generate_handler![
            game::commands::get_game_state,
            game::commands::reset_game,
            game::commands::train_horse,
            game::commands::save_game,
            game::commands::load_game,
            game::commands::breed_horses,
            game::commands::retire_horse,
            game::commands::set_gear,
            game::commands::rest_horse,
            game::commands::set_auto_training,
            game::commands::set_auto_entry,
            game::commands::set_all_auto,
            game::commands::set_trainer,
            game::commands::sell_horse,
            game::commands::pasture_horse,
            game::commands::set_jockey,
            game::commands::recall_horse,
            game::commands::release_stud,
            game::commands::get_stud_market,
            game::commands::buy_stud,
            game::commands::set_order,
            game::commands::go_abroad,
            game::commands::upgrade_farm,
            game::commands::get_advice,
            game::commands::accept_advice,
            game::commands::get_auction,
            game::commands::bid_at_auction,
            calendar::get_calendar,
            course::get_courses,
            game::commands::get_course_fits,
            game::commands::get_race_card,
            game::commands::quote_bet,
            game::commands::place_bets,
            game::commands::watch_race,
            game::commands::quote_tickets,
            game::commands::expand_tickets,
            game::commands::set_entries,
            game::commands::next_week,
            bloodline::get_bloodline,
            game::commands::breeding_outlook,
            track::get_track_effects
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
