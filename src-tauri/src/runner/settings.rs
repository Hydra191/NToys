use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{Emitter, Manager};

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
    #[serde(default = "default_false")]
    pub autostart: bool,
    #[serde(default = "default_show_window_shortcut")]
    pub show_window_shortcut: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: default_shortcut(),
            max_visible: default_max_visible(),
            prevent_hide_on_text: true,
            save_search_history: false,
            autostart: false,
            show_window_shortcut: default_show_window_shortcut(),
        }
    }
}

fn default_shortcut() -> String { "Alt+Space".into() }
fn default_max_visible() -> usize { 8 }
fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_show_window_shortcut() -> String { "Alt+Shift+N".into() }

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
pub fn set_autostart(
    app_handle: tauri::AppHandle,
    state: tauri::State<SettingsState>,
    enable: bool,
) -> Result<bool, String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_str = exe_path.to_str().ok_or("invalid exe path")?;

    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            winreg::enums::KEY_SET_VALUE,
        )
        .map_err(|e| format!("open registry: {}", e))?;

    if enable {
        run_key
            .set_value("NToys", &exe_str)
            .map_err(|e| format!("set autostart: {}", e))?;
    } else {
        run_key
            .delete_value("NToys")
            .map_err(|e| format!("delete autostart: {}", e))?;
    }

    let mut s = state.settings.lock().map_err(|e| e.to_string())?;
    s.autostart = enable;
    save_settings(&app_handle, &s);
    Ok(enable)
}

#[tauri::command]
pub fn get_autostart(state: tauri::State<SettingsState>) -> bool {
    state.settings.lock().unwrap().autostart
}

use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_global_shortcut::ShortcutState;

pub fn register_show_window_shortcut(
    app_handle: &tauri::AppHandle,
    shortcut: &str,
) {
    let _ = app_handle.global_shortcut().on_shortcut(
        shortcut,
        |app_handle, _sc, event| {
            if event.state == ShortcutState::Pressed {
                if let Some(win) = app_handle.get_webview_window("main") {
                    if win.is_visible().unwrap_or(false) {
                        let _ = win.hide();
                        let _ = app_handle.emit("main-window-hidden", ());
                    } else {
                        let _ = win.show();
                        let _ = win.set_focus();
                        let _ = app_handle.emit("main-window-shown", ());
                    }
                }
            }
        },
    );
}

#[tauri::command]
pub fn set_show_window_shortcut(
    app_handle: tauri::AppHandle,
    state: tauri::State<SettingsState>,
    shortcut: String,
) -> Result<String, String> {
    let mut s = state.settings.lock().map_err(|e| e.to_string())?;

    if shortcut == s.shortcut {
        return Err("快捷键与 Launcher 冲突".into());
    }

    let old = s.show_window_shortcut.clone();

    // Unregister old shortcut
    app_handle.global_shortcut().unregister(old.as_str()).ok();

    // Register new
    register_show_window_shortcut(&app_handle, &shortcut);

    s.show_window_shortcut = shortcut.clone();
    save_settings(&app_handle, &s);
    Ok(shortcut)
}
