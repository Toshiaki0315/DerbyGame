//! What the screen calls, called the way the screen calls it (画面からの呼び出し).

use super::*;
use serde_json::{json, Value};
use tauri::test::{
    get_ipc_response, mock_builder, mock_context, noop_assets, MockRuntime, INVOKE_KEY,
};
use tauri::webview::InvokeRequest;

fn screen() -> (tauri::App<MockRuntime>, tauri::WebviewWindow<MockRuntime>) {
    let app = mock_builder()
        .manage(GameStore(Mutex::new(Game::with_seed(1))))
        .invoke_handler(super::super::commands::handler())
        .build(mock_context(noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    (app, window)
}

fn call(
    window: &tauri::WebviewWindow<MockRuntime>,
    cmd: &str,
    args: Value,
) -> Result<Value, Value> {
    let request = InvokeRequest {
        cmd: cmd.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
        url: "tauri://localhost".parse().unwrap(),
        body: args.into(),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    };
    get_ipc_response(window, request).map(|r| r.deserialize::<Value>().unwrap())
}

/// Every command the screen calls, with arguments named as the screen names them.
/// Saving is left out: it would write to the real application data folder.
fn every_call(horse: &str, mare: &str, sire: &str) -> Vec<(&'static str, Value)> {
    let ticket = json!({"kind": "win", "horses": [horse], "amount": 100});
    vec![
        ("get_game_state", json!({})),
        ("get_calendar", json!({})),
        ("get_bloodline", json!({})),
        (
            "get_track_effects",
            json!({"speed": 50.0, "stamina": 50.0, "power": 50.0, "dirt": 50.0}),
        ),
        ("breeding_outlook", json!({"sire": sire, "dam": mare})),
        ("get_courses", json!({})),
        ("get_course_fits", json!({"horse": horse})),
        ("get_advice", json!({})),
        ("get_auction", json!({})),
        ("get_stud_market", json!({})),
        (
            "quote_bet",
            json!({"horses": [horse], "kind": "win", "race": null}),
        ),
        ("quote_tickets", json!({"tickets": [ticket], "race": null})),
        ("get_race_card", json!({"race": null})),
        (
            "expand_tickets",
            json!({"style": "box", "kind": "quinella", "axis": null, "horses": [horse, sire], "race": null}),
        ),
        ("set_entries", json!({"race": null, "horses": [horse]})),
        (
            "set_gear",
            json!({"horse": horse, "menko": true, "shadowRoll": true, "blinkers": true}),
        ),
        ("train_horse", json!({"horse": horse, "workout": "wood"})),
        ("set_auto_entry", json!({"horse": horse, "on": false})),
        ("set_jockey", json!({"horse": horse, "jockey": null})),
        ("set_trainer", json!({"horse": horse, "trainer": 0})),
        ("set_all_auto", json!({"on": false})),
        ("set_auto_training", json!({"horse": horse, "on": false})),
        ("set_order", json!({"horse": horse, "order": null})),
        ("rest_horse", json!({"horse": horse})),
        ("geld_horse", json!({"name": sire})),
        ("go_over_to_jumps", json!({"name": horse})),
        ("take_jump_exam", json!({"name": horse})),
        ("pasture_horse", json!({"name": horse})),
        ("recall_horse", json!({"name": horse})),
        ("accept_advice", json!({"horse": horse})),
        ("upgrade_farm", json!({"facility": "pasture"})),
        ("go_abroad", json!({"horse": horse, "race": 0})),
        ("buy_stud", json!({"bloodId": 1})),
        ("release_stud", json!({"name": "ダレモイナイ"})),
        (
            "bid_at_auction",
            json!({"lot": 1, "maxBid": 1, "name": "ハナミズキ"}),
        ),
        (
            "breed_horses",
            json!({"sire": sire, "dam": mare, "replace": "", "name": "ハナミズキ"}),
        ),
        ("watch_race", json!({"seed": 1, "race": null})),
        (
            "place_bets",
            json!({"tickets": [ticket], "seed": 1, "race": null}),
        ),
        ("next_week", json!({})),
        ("sell_horse", json!({"name": sire})),
        ("retire_horse", json!({"name": mare})),
        ("load_game", json!({})),
        ("reset_game", json!({})),
    ]
}

fn horses_of(state: &Value) -> Vec<(String, String)> {
    state["field"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| {
            (
                h["name"].as_str().unwrap().to_string(),
                h["sex"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn called_by_the_screen() -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    let mut dirs = vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src")];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let file = path.to_string_lossy().to_string();
            if path.is_dir() {
                dirs.push(path);
            } else if !file.contains(".test.") {
                let text = std::fs::read_to_string(&path).unwrap();
                for (at, _) in text.match_indices("invoke") {
                    let rest = &text[at + "invoke".len()..];
                    // invoke("name" or invoke<Type>("name"
                    if let Some(open) = rest.find("(\"") {
                        if rest[..open]
                            .chars()
                            .all(|c| !c.is_whitespace() || rest.starts_with('<'))
                            && (open == 0 || rest.starts_with('<'))
                        {
                            let name = &rest[open + 2..];
                            names.insert(name[..name.find('"').unwrap()].to_string());
                        }
                    }
                }
            }
        }
    }
    names
}

#[test]
fn everything_the_screen_calls_is_tried_here() {
    let tried: std::collections::BTreeSet<String> = every_call("", "", "")
        .iter()
        .map(|(cmd, _)| cmd.to_string())
        .chain(["save_game".to_string()])
        .collect();
    assert_eq!(called_by_the_screen(), tried);
}

#[test]
fn every_call_reaches_the_game_with_the_arguments_the_screen_sends() {
    let (_app, window) = screen();
    let state = call(&window, "get_game_state", json!({})).unwrap();
    let horses = horses_of(&state);
    let of = |sex: &str| horses.iter().find(|(_, s)| s == sex).unwrap().0.clone();
    let (sire, mare) = (of("male"), of("female"));
    let horse = horses
        .iter()
        .find(|(n, _)| *n != sire && *n != mare)
        .unwrap()
        .0
        .clone();
    for (cmd, args) in every_call(&horse, &mare, &sire) {
        // a rule of the game may refuse it, but it must have got as far as the game
        if let Err(refusal) = call(&window, cmd, args) {
            let text = refusal.as_str().unwrap_or_default();
            assert!(
                !text.is_empty()
                    && !text.contains("invalid args")
                    && !text.contains("missing required")
                    && !text.contains("not found"),
                "{cmd}: {refusal}"
            );
        }
    }
}

#[test]
fn a_call_with_an_argument_misnamed_never_reaches_the_game() {
    let (_app, window) = screen();
    let err = call(
        &window,
        "set_gear",
        json!({"horse": "x", "menko": true, "shadow_roll": true, "blinkers": true}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("shadowRoll"), "{err}");
}

#[test]
fn the_game_refusing_comes_back_in_words_the_screen_can_show() {
    let (_app, window) = screen();
    let err = call(
        &window,
        "train_horse",
        json!({"horse": "ダレモイナイ", "workout": "wood"}),
    )
    .unwrap_err();
    assert_eq!(err, json!(GameError::UnknownHorse.to_string()));
}

#[test]
fn the_calls_change_the_game_they_are_made_to() {
    let (_app, window) = screen();
    let state = call(&window, "get_game_state", json!({})).unwrap();
    let (horse, _) = horses_of(&state).remove(0);

    let after = call(
        &window,
        "set_auto_training",
        json!({"horse": horse, "on": true}),
    )
    .unwrap();
    assert_eq!(after["field"][0]["auto_training"], true);

    let after = call(&window, "set_all_auto", json!({"on": true})).unwrap();
    assert_eq!(after["field"][1]["auto_entry"], true);

    let week = call(&window, "next_week", json!({})).unwrap()["week"]
        .as_u64()
        .unwrap();
    assert_eq!(week, state["week"].as_u64().unwrap() + 1);

    let after = call(&window, "reset_game", json!({})).unwrap();
    assert_eq!(after["week"], 1);
    assert_eq!(after["money"], START_MONEY);
}

#[test]
fn a_race_is_run_and_paid_for_through_the_screens_call() {
    let (_app, window) = screen();
    let state = call(&window, "get_game_state", json!({})).unwrap();
    let card = call(&window, "get_race_card", json!({"race": null})).unwrap();
    let horse = card["entrants"][0]["name"].clone();
    let ticket = json!({"kind": "win", "horses": [horse], "amount": 100});
    let out = call(
        &window,
        "place_bets",
        json!({"tickets": [ticket], "seed": 7, "race": null}),
    )
    .unwrap();
    assert_eq!(
        out["results"].as_array().unwrap().len(),
        entries::FULL_FIELD
    );
    let after = call(&window, "get_game_state", json!({})).unwrap();
    assert_eq!(after["round"], state["round"].as_u64().unwrap() + 1);
}

#[test]
fn nothing_saved_means_nothing_to_load() {
    let (_app, window) = screen();
    // the mock application has its own data folder, where nothing is ever saved
    let err = call(&window, "load_game", json!({})).unwrap_err();
    assert_eq!(err, json!("セーブデータがありません"));
}

#[test]
fn the_game_can_still_be_reached_after_a_command_went_wrong_holding_it() {
    let store = std::sync::Arc::new(GameStore(std::sync::Mutex::new(Game::new())));
    let held = store.clone();
    let _ = std::thread::spawn(move || {
        let _g = held.game();
        panic!("a command went wrong");
    })
    .join();
    assert!(store.0.is_poisoned());
    assert_eq!(store.game().week, 1);
}
