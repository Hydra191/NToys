use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_global_shortcut::ShortcutState;

// ── Settings ────────────────────────────────
// All sections are raw JSON — frontend owns key names.
// Side effects extract values by well-known keys from the JSON.

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub runner: serde_json::Value,
    #[serde(default)]
    pub global: serde_json::Value,
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

// ── State ──────────────────────────────────

pub struct SettingsState {
    pub settings: std::sync::Mutex<Settings>,
}

// ── Persistence ────────────────────────────

fn settings_path(app_handle: &tauri::AppHandle) -> std::path::PathBuf {
    app_handle.path().app_data_dir().expect("failed to resolve app data dir").join("settings.json")
}

pub fn load_settings(app_handle: &tauri::AppHandle) -> Settings {
    let path = settings_path(app_handle);
    if path.exists() {
        fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    } else {
        let defaults = Settings::default();
        save_settings(app_handle, &defaults);
        defaults
    }
}

pub fn save_settings(app_handle: &tauri::AppHandle, settings: &Settings) {
    let path = settings_path(app_handle);
    if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
    if let Ok(json) = serde_json::to_string_pretty(settings) { let _ = fs::write(&path, json); }
}

// ── IPC ─────────────────────────────────────

#[tauri::command]
pub fn get_settings(state: tauri::State<SettingsState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

/// Single entry point for all section updates.
/// Runner → re-registers shortcut if `shortcut` key changed.
/// Global → writes autostart registry + re-registers `show_window_shortcut`.
/// Others → stored as-is.
#[tauri::command]
pub fn update_settings_section(
    app_handle: tauri::AppHandle,
    state: tauri::State<SettingsState>,
    section: String,
    data: serde_json::Value,
) -> Result<(), String> {
    let mut s = state.settings.lock().map_err(|e| e.to_string())?;

    match section.as_str() {
        "runner" => {
            let new_shortcut = data["shortcut"].as_str().unwrap_or("").to_string();
            let old_shortcut = s.runner["shortcut"].as_str().unwrap_or("").to_string();
            s.runner = data;
            save_settings(&app_handle, &s);
            if new_shortcut != old_shortcut && !new_shortcut.is_empty() {
                app_handle.global_shortcut().unregister(old_shortcut.as_str()).ok();
                crate::runner::launcher::register_shortcut(&app_handle, &new_shortcut);
            }
            let _ = app_handle.emit("settings-changed", &*s);
        }
        "global" => {
            // autostart → Windows Registry
            let autostart = data["autostart"].as_bool().unwrap_or(false);
            let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = exe_path.to_str().ok_or("invalid exe path")?;
            let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
            let run_key = hkcu
                .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", winreg::enums::KEY_SET_VALUE)
                .map_err(|e| format!("open registry: {}", e))?;
            if autostart {
                run_key.set_value("NToys", &exe_str).map_err(|e| format!("set autostart: {}", e))?;
            } else {
                run_key.delete_value("NToys").map_err(|e| format!("delete autostart: {}", e))?;
            }

            // show-window shortcut
            let new_sc = data["show_window_shortcut"].as_str().unwrap_or("").to_string();
            let old_sc = s.global["show_window_shortcut"].as_str().unwrap_or("").to_string();
            let runner_sc = s.runner["shortcut"].as_str().unwrap_or("").to_string();
            if new_sc == runner_sc {
                return Err("快捷键与 Launcher 冲突".into());
            }
            if !new_sc.is_empty() && new_sc != old_sc {
                app_handle.global_shortcut().unregister(old_sc.as_str()).ok();
                register_show_window_shortcut(&app_handle, &new_sc);
            }

            s.global = data;
            save_settings(&app_handle, &s);
        }
        _ => {
            s.extensions.insert(section, data);
            save_settings(&app_handle, &s);
        }
    }
    Ok(())
}

// ── Show-window shortcut helper ────────────

pub fn register_show_window_shortcut(app_handle: &tauri::AppHandle, shortcut: &str) {
    let _ = app_handle.global_shortcut().on_shortcut(shortcut, |app_handle, _sc, event| {
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
    });
}
