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
        .invoke_handler(game::commands::handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
