// The window settings in tauri.conf.json that the game relies on.

#[test]
fn a_click_on_the_window_from_another_app_also_presses_the_button() {
    let conf: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    let window = &conf["app"]["windows"][0];
    assert_eq!(window["acceptFirstMouse"], serde_json::Value::Bool(true));
}
