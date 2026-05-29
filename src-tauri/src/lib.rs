mod runner;
mod settings;
mod vpn;
use std::fs;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use std::time::Instant;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, Pid, ProcessesToUpdate, System};
use runner::launcher::{AppState, setup_launcher};
use settings::settings::{SettingsState, load_settings, register_show_window_shortcut};

#[tauri::command]
fn exit_app(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

#[tauri::command]
fn hide_main(app_handle: tauri::AppHandle) {
    if let Some(win) = app_handle.get_webview_window("main") {
        let _ = win.hide();
        let _ = app_handle.emit("main-window-hidden", ());
    }
}

#[tauri::command]
fn get_ncm_cookie(app_handle: tauri::AppHandle) -> String {
    let path = app_handle
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join("ncm_cookie");
    fs::read_to_string(&path).unwrap_or_default()
}

#[tauri::command]
fn set_ncm_cookie(app_handle: tauri::AppHandle, cookie: String) {
    let path = app_handle
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join("ncm_cookie");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, cookie);
}

#[tauri::command]
fn get_playback_state(app_handle: tauri::AppHandle) -> String {
    let path = app_handle
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join("playback_state");
    fs::read_to_string(&path).unwrap_or_default()
}

#[tauri::command]
fn set_playback_state(app_handle: tauri::AppHandle, state: String) {
    let path = app_handle
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join("playback_state");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, state);
}

#[derive(serde::Serialize)]
struct SystemStats {
    cpu: f64,
    memory_total: u64,
    memory_used: u64,
    self_memory_mb: f64,
    download_speed: f64,
    upload_speed: f64,
}

struct NetState {
    networks: Networks,
    prev_rx: u64,
    prev_tx: u64,
    prev_time: Instant,
}

#[tauri::command]
fn get_system_stats(sys_state: tauri::State<SystemState>) -> SystemStats {
    let mut sys = sys_state.sys.lock().unwrap();
    sys.refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage());
    sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());

    let cpu = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;

    let pid = Pid::from_u32(std::process::id());
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        sysinfo::ProcessRefreshKind::nothing().with_memory(),
    );
    let self_memory_mb = sys
        .process(pid)
        .map(|p| p.memory() as f64 / 1024.0 / 1024.0)
        .unwrap_or(0.0);

    // ── network speed ──
    let (dl, ul) = {
        let mut net = sys_state.net.lock().unwrap();
        net.networks.refresh(true);
        let now = Instant::now();
        let total_rx: u64 = net.networks.iter().map(|(_, d)| d.received()).sum();
        let total_tx: u64 = net.networks.iter().map(|(_, d)| d.transmitted()).sum();
        let dt = (now - net.prev_time).as_secs_f64();
        let (dl, ul) = if dt > 0.0 && net.prev_rx > 0 {
            let rx_diff = total_rx.saturating_sub(net.prev_rx) as f64;
            let tx_diff = total_tx.saturating_sub(net.prev_tx) as f64;
            (rx_diff / dt, tx_diff / dt)
        } else {
            (0.0, 0.0)
        };
        net.prev_rx = total_rx;
        net.prev_tx = total_tx;
        net.prev_time = now;
        (dl, ul)
    };

    SystemStats {
        cpu: (cpu as f64).round(),
        memory_total: sys.total_memory(),
        memory_used: sys.used_memory(),
        self_memory_mb: (self_memory_mb * 10.0).round() / 10.0,
        download_speed: (dl * 10.0).round() / 10.0,
        upload_speed: (ul * 10.0).round() / 10.0,
    }
}

struct SystemState {
    sys: Mutex<System>,
    net: Mutex<NetState>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let apps = runner::launcher::scan_start_menu();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            apps: std::sync::Mutex::new(apps),
        })
        .setup(|app| {
            let settings = load_settings(app.handle());
            let shortcut = settings.runner["shortcut"].as_str().unwrap_or("Alt+Space").to_string();
            let show_shortcut = settings.global["show_window_shortcut"].as_str().unwrap_or("Alt+Shift+N").to_string();
            app.manage(SettingsState {
                settings: std::sync::Mutex::new(settings),
            });

            // VPN state
            let vpn_data = vpn::load_vpn_data(app.handle());
            app.manage(SystemState {
                sys: Mutex::new(System::new_all()),
                net: Mutex::new(NetState {
                    networks: Networks::new_with_refreshed_list(),
                    prev_rx: 0,
                    prev_tx: 0,
                    prev_time: Instant::now(),
                }),
            });

            app.manage(vpn::VpnState {
                subscriptions: std::sync::Mutex::new(vpn_data.subscriptions),
                active_node: std::sync::Mutex::new(vpn_data.active_node),
                proxy_enabled: std::sync::Mutex::new(false),
                proxy_addr: std::sync::Mutex::new(vpn_data.proxy_addr),
            });
            app.manage(vpn::v2ray::V2rayState {
                running: std::sync::Mutex::new(false),
                child: std::sync::Mutex::new(None),
            });

            setup_launcher(app, &shortcut)?;

            register_show_window_shortcut(app.handle(), &show_shortcut);

            // ── tray ──────────────────────────────────
            let show = MenuItemBuilder::with_id("show", "主界面").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show)
                .item(&quit)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .tooltip("NToys")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                            let _ = app.emit("main-window-shown", ());
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // ── close button → hide to tray ──────────
            let handle = app.handle().clone();
            if let Some(main_win) = app.get_webview_window("main") {
                main_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(win) = handle.get_webview_window("main") {
                            let _ = win.hide();
                            let _ = handle.emit("main-window-hidden", ());
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runner::launcher::search_apps,
            runner::launcher::get_icon,
            runner::launcher::launch_app,
            runner::launcher::hide_launcher,
            runner::launcher::set_prevent_hide,
            runner::launcher::set_launcher_size,
            settings::settings::get_settings,
            settings::settings::update_settings_section,
            get_ncm_cookie,
            set_ncm_cookie,
            get_playback_state,
            set_playback_state,
            get_system_stats,
            exit_app,
            hide_main,
            vpn::add_subscription,
            vpn::import_subscription_content,
            vpn::remove_subscription,
            vpn::get_subscriptions,
            vpn::select_node,
            vpn::get_active_node,
            vpn::set_proxy,
            vpn::get_proxy_state,
            vpn::test_proxy,
            vpn::update_proxy_addr,
            vpn::start_core,
            vpn::stop_core,
            vpn::core_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
