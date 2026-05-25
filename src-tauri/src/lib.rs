mod runner;
mod vpn;
use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use runner::launcher::{AppState, setup_launcher};
use runner::settings::{SettingsState, load_settings};

#[tauri::command]
fn exit_app(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

#[tauri::command]
fn hide_main(app_handle: tauri::AppHandle) {
    if let Some(win) = app_handle.get_webview_window("main") {
        let _ = win.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let apps = runner::launcher::scan_start_menu();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            apps: std::sync::Mutex::new(apps),
        })
        .setup(|app| {
            let settings = load_settings(app.handle());
            let shortcut = settings.shortcut.clone();
            app.manage(SettingsState {
                settings: std::sync::Mutex::new(settings),
            });

            // VPN state
            let vpn_data = vpn::load_vpn_data(app.handle());
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
            runner::launcher::update_settings,
            runner::settings::get_settings,
            runner::settings::get_max_visible,
            runner::settings::get_music_api_url,
            runner::settings::set_music_api_url,
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
