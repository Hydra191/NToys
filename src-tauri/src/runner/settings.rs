use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_shortcut")]
    pub shortcut: String,
    #[serde(default = "default_max_visible")]
    pub max_visible: usize,
    #[serde(default = "default_true")]
    pub prevent_hide_on_text: bool,
    #[serde(default = "default_false")]
    pub save_search_history: bool,
    #[serde(default = "default_music_api_url")]
    pub music_api_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: default_shortcut(),
            max_visible: default_max_visible(),
            prevent_hide_on_text: true,
            save_search_history: false,
            music_api_url: default_music_api_url(),
        }
    }
}

fn default_shortcut() -> String {
    "Alt+Space".into()
}
fn default_max_visible() -> usize {
    8
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_music_api_url() -> String {
    "http://localhost:3000".into()
}

pub struct SettingsState {
    pub settings: std::sync::Mutex<Settings>,
}

fn settings_path(app_handle: &tauri::AppHandle) -> std::path::PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join("settings.json")
}

pub fn load_settings(app_handle: &tauri::AppHandle) -> Settings {
    let path = settings_path(app_handle);
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        let defaults = Settings::default();
        save_settings(app_handle, &defaults);
        defaults
    }
}

pub fn save_settings(app_handle: &tauri::AppHandle, settings: &Settings) {
    let path = settings_path(app_handle);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(&path, json);
    }
}

#[tauri::command]
pub fn get_settings(state: tauri::State<SettingsState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_max_visible(state: tauri::State<SettingsState>) -> usize {
    state.settings.lock().unwrap().max_visible
}

#[tauri::command]
pub fn get_music_api_url(state: tauri::State<SettingsState>) -> String {
    state.settings.lock().unwrap().music_api_url.clone()
}

#[tauri::command]
pub fn set_music_api_url(
    app_handle: tauri::AppHandle,
    state: tauri::State<SettingsState>,
    url: String,
) -> Result<String, String> {
    let mut s = state.settings.lock().map_err(|e| e.to_string())?;
    s.music_api_url = url.trim().to_string();
    save_settings(&app_handle, &s);
    Ok(s.music_api_url.clone())
}
