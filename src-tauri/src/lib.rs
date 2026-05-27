mod state;

use serde::Serialize;
use state::{CatState, Mood};
use tauri::AppHandle;

#[derive(Serialize)]
struct CatSnapshot {
    #[serde(flatten)]
    state: CatState,
    mood: Mood,
}

impl CatSnapshot {
    fn of(state: CatState) -> Self {
        let mood = state.mood();
        CatSnapshot { state, mood }
    }
}

#[tauri::command]
fn get_cat_state(app: AppHandle) -> Result<CatSnapshot, String> {
    let mut cat = state::load(&app);
    cat.apply_decay(state::now());
    state::save(&app, &cat)?;
    Ok(CatSnapshot::of(cat))
}

#[tauri::command]
fn feed_cat(app: AppHandle) -> Result<CatSnapshot, String> {
    let mut cat = state::load(&app);
    cat.apply_decay(state::now());
    cat.satiety = (cat.satiety + 30).min(100);
    state::save(&app, &cat)?;
    Ok(CatSnapshot::of(cat))
}

#[tauri::command]
fn play_with_cat(app: AppHandle) -> Result<CatSnapshot, String> {
    let mut cat = state::load(&app);
    cat.apply_decay(state::now());
    cat.happiness = (cat.happiness + 25).min(100);
    state::save(&app, &cat)?;
    Ok(CatSnapshot::of(cat))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_cat_state,
            feed_cat,
            play_with_cat
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
