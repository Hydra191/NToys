use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use serde::Serialize;
use tauri::{Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::runner::icon;
use crate::runner::settings::{self, SettingsState};
use pinyin::ToPinyin;

const LAUNCHER_W: f64 = 600.0;
const LAUNCHER_H: f64 = 58.0;

static HAS_FOCUSED: AtomicBool = AtomicBool::new(false);
static PREVENT_HIDE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Serialize)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
    #[serde(skip)]
    pub pinyin: String,
    #[serde(skip)]
    pub pinyin_initials: String,
}

pub struct AppState {
    pub apps: Mutex<Vec<AppEntry>>,
}

fn launcher_position(app_handle: &tauri::AppHandle) -> PhysicalPosition<f64> {
    if let Ok(Some(monitor)) = app_handle.primary_monitor() {
        let size = monitor.size();
        let x = (size.width as f64 - LAUNCHER_W) / 2.0;
        let y = (size.height as f64 - LAUNCHER_H) / 4.0;
        PhysicalPosition::new(x.max(0.0), y.max(0.0))
    } else {
        PhysicalPosition::new(0.0, 0.0)
    }
}

fn offscreen_y(app_handle: &tauri::AppHandle) -> f64 {
    if let Ok(Some(monitor)) = app_handle.primary_monitor() {
        monitor.size().height as f64 + 100.0
    } else {
        10000.0
    }
}

fn make_pinyin(name: &str) -> (String, String) {
    let mut full = String::new();
    let mut initials = String::new();
    for py in name.to_pinyin() {
        if let Some(py) = py {
            full.push_str(py.plain());
            initials.push_str(py.first_letter());
        }
    }
    (full.to_lowercase(), initials.to_lowercase())
}

pub fn scan_start_menu() -> Vec<AppEntry> {
    let mut apps = Vec::new();

    let dirs = [
        std::env::var("PROGRAMDATA")
            .map(|p| format!(r"{}\Microsoft\Windows\Start Menu\Programs", p)),
        std::env::var("APPDATA")
            .map(|p| format!(r"{}\Microsoft\Windows\Start Menu\Programs", p)),
    ];

    for dir in dirs.iter().flatten() {
        if let Ok(walker) = walkdir::WalkDir::new(dir)
            .max_depth(4)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
        {
            for entry in walker {
                let path = entry.path();
                if path.extension().map(|e| e == "lnk").unwrap_or(false) {
                    let (name, _icon) = icon::parse_lnk_info(path);
                    if !name.is_empty() {
                        let (pinyin, pinyin_initials) = make_pinyin(&name);
                        apps.push(AppEntry {
                            name,
                            path: path.to_string_lossy().to_string(),
                            pinyin,
                            pinyin_initials,
                        });
                    }
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));
    apps
}

#[tauri::command]
pub fn search_apps(
    state: tauri::State<AppState>,
    query: String,
) -> Vec<AppEntry> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let apps = state.apps.lock().unwrap();
    apps.iter()
        .filter(|a| {
            a.name.to_lowercase().contains(&query)
                || a.pinyin.contains(&query)
                || a.pinyin_initials.contains(&query)
        })
        .take(50)
        .cloned()
        .collect()
}

#[tauri::command]
pub fn get_icon(path: String) -> String {
    icon::get_icon_base64(&path)
}

#[tauri::command]
pub fn launch_app(app_handle: tauri::AppHandle, path: String) {
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", "", &path])
        .spawn();
    if let Some(launcher) = app_handle.get_webview_window("launcher") {
        let _ = launcher.set_position(PhysicalPosition::new(0.0, offscreen_y(&app_handle)));
    }
}

#[tauri::command]
pub fn hide_launcher(app_handle: tauri::AppHandle) {
    if let Some(launcher) = app_handle.get_webview_window("launcher") {
        let _ = launcher.set_position(PhysicalPosition::new(0.0, offscreen_y(&app_handle)));
    }
}

#[tauri::command]
pub fn set_prevent_hide(prevent: bool) {
    PREVENT_HIDE.store(prevent, Ordering::Relaxed);
}

#[tauri::command]
pub fn set_launcher_size(app_handle: tauri::AppHandle, height: f64) {
    if let Some(launcher) = app_handle.get_webview_window("launcher") {
        let _ = launcher.set_size(tauri::Size::Logical(LogicalSize::new(LAUNCHER_W, height)));
    }
}

pub fn setup_launcher(app: &mut tauri::App, shortcut: &str) -> Result<(), Box<dyn std::error::Error>> {
    let off_y = offscreen_y(app.handle());

    let launcher = WebviewWindowBuilder::new(
        app,
        "launcher",
        WebviewUrl::App("/launcher.html".into()),
    )
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .position(0.0, off_y)
    .inner_size(LAUNCHER_W, LAUNCHER_H)
    .visible(true)
    .build()
    .expect("failed to create launcher window");

    let app_handle = app.handle().clone();
    launcher.on_window_event(move |event| {
        use tauri::WindowEvent;
        if let WindowEvent::Focused(false) = event {
            if !PREVENT_HIDE.load(Ordering::Relaxed) {
                if let Some(win) = app_handle.get_webview_window("launcher") {
                    let _ = win.set_position(PhysicalPosition::new(
                        0.0,
                        offscreen_y(&app_handle),
                    ));
                }
            }
        }
    });

    register_shortcut(app.handle(), shortcut);
    Ok(())
}

fn toggle_launcher(app_handle: &tauri::AppHandle) {
    if let Some(launcher) = app_handle.get_webview_window("launcher") {
        let screen_h = app_handle
            .primary_monitor()
            .ok()
            .flatten()
            .map(|m| m.size().height as f64)
            .unwrap_or(1080.0);
        let is_offscreen = launcher
            .outer_position()
            .map(|p| (p.y as f64) >= screen_h - LAUNCHER_H)
            .unwrap_or(false);
        if is_offscreen {
            let _ = launcher.set_position(launcher_position(app_handle));
            if HAS_FOCUSED.swap(true, Ordering::Relaxed) {
                let _ = launcher.set_focus();
            }
        } else {
            let _ = launcher.set_position(PhysicalPosition::new(0.0, offscreen_y(app_handle)));
        }
    }
}

pub fn register_shortcut(app_handle: &tauri::AppHandle, shortcut: &str) {
    if app_handle
        .global_shortcut()
        .on_shortcut(shortcut, |app_handle, _sc, event| {
            if event.state == ShortcutState::Pressed {
                toggle_launcher(app_handle);
            }
        })
        .is_err()
    {
        // Fallback to default if saved shortcut is invalid
        let _ = app_handle.global_shortcut().on_shortcut(
            "Alt+Space",
            |app_handle, _sc, event| {
                if event.state == ShortcutState::Pressed {
                    toggle_launcher(app_handle);
                }
            },
        );
    }
}

#[tauri::command]
pub fn update_settings(
    app_handle: tauri::AppHandle,
    settings_state: tauri::State<SettingsState>,
    shortcut: String,
    max_visible: usize,
    prevent_hide_on_text: bool,
    save_search_history: bool,
) -> Result<settings::Settings, String> {
    let mut s = settings_state.settings.lock().map_err(|e| e.to_string())?;
    let old = s.shortcut.clone();
    s.shortcut = shortcut.clone();
    s.max_visible = max_visible;
    s.prevent_hide_on_text = prevent_hide_on_text;
    s.save_search_history = save_search_history;
    settings::save_settings(&app_handle, &s);
    if shortcut != old {
        app_handle.global_shortcut().unregister(old.as_str()).ok();
        register_shortcut(&app_handle, &shortcut);
    }
    let _ = app_handle.emit("settings-changed", &*s);
    Ok(s.clone())
}
