pub mod proxy;
pub mod v2ray;
pub mod subscription;

use std::sync::Mutex;
use subscription::{fetch_subscription, ProxyNode, Subscription};
use tauri::Manager;

pub struct VpnState {
    pub subscriptions: Mutex<Vec<Subscription>>,
    pub active_node: Mutex<Option<ProxyNode>>,
    pub proxy_enabled: Mutex<bool>,
    pub proxy_addr: Mutex<String>,
}

fn vpn_data_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .expect("no app data dir")
        .join("vpn.json")
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VpnData {
    pub subscriptions: Vec<Subscription>,
    pub active_node: Option<ProxyNode>,
    pub proxy_addr: String,
}

impl Default for VpnData {
    fn default() -> Self {
        Self {
            subscriptions: vec![],
            active_node: None,
            proxy_addr: "127.0.0.1:7890".into(),
        }
    }
}

pub fn load_vpn_data(app: &tauri::AppHandle) -> VpnData {
    let path = vpn_data_path(app);
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        VpnData::default()
    }
}

fn save_vpn_data(app: &tauri::AppHandle, data: &VpnData) {
    let path = vpn_data_path(app);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(&path, json);
    }
}

#[tauri::command]
pub async fn add_subscription(
    app: tauri::AppHandle,
    state: tauri::State<'_, VpnState>,
    url: String,
) -> Result<Vec<Subscription>, String> {
    let url_trimmed = url.trim().to_string();
    if url_trimmed.is_empty() {
        return Err("URL is empty".into());
    }

    {
        let subs = state.subscriptions.lock().map_err(|e| e.to_string())?;
        if subs.iter().any(|s| s.url == url_trimmed) {
            return Err("Subscription already exists".into());
        }
    }

    let name = url_trimmed
        .split("://")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap_or(&url_trimmed)
        .to_string();

    let proxy_addr = state.proxy_addr.lock().map_err(|e| e.to_string())?.clone();
    let nodes = fetch_subscription(&url_trimmed, Some(&proxy_addr)).await?;
    let sub = Subscription {
        url: url_trimmed.clone(),
        name,
        nodes,
    };

    let subs_clone = {
        let mut subs = state.subscriptions.lock().map_err(|e| e.to_string())?;
        subs.push(sub);
        subs.clone()
    };

    let mut data = load_vpn_data(&app);
    data.subscriptions = subs_clone.clone();
    save_vpn_data(&app, &data);

    Ok(subs_clone)
}

#[tauri::command]
pub fn import_subscription_content(
    app: tauri::AppHandle,
    state: tauri::State<VpnState>,
    name: String,
    content: String,
) -> Result<Vec<Subscription>, String> {
    let content = content.trim().to_string();
    if content.is_empty() {
        return Err("内容为空".into());
    }

    let nodes = subscription::parse_subscription_content(&content);
    if nodes.is_empty() {
        return Err("未解析到任何节点".into());
    }

    let sub = Subscription {
        url: format!("manual:{}", name),
        name,
        nodes,
    };

    let subs_clone = {
        let mut subs = state.subscriptions.lock().map_err(|e| e.to_string())?;
        subs.push(sub);
        subs.clone()
    };

    let mut data = load_vpn_data(&app);
    data.subscriptions = subs_clone.clone();
    save_vpn_data(&app, &data);

    Ok(subs_clone)
}

#[tauri::command]
pub fn remove_subscription(
    app: tauri::AppHandle,
    state: tauri::State<VpnState>,
    index: usize,
) -> Result<Vec<Subscription>, String> {
    let subs_clone = {
        let mut subs = state.subscriptions.lock().map_err(|e| e.to_string())?;
        if index >= subs.len() {
            return Err("Invalid index".into());
        }
        subs.remove(index);
        subs.clone()
    };

    let mut active = state.active_node.lock().map_err(|e| e.to_string())?;
    *active = None;
    drop(active);

    let mut data = load_vpn_data(&app);
    data.subscriptions = subs_clone.clone();
    save_vpn_data(&app, &data);

    Ok(subs_clone)
}

#[tauri::command]
pub fn get_subscriptions(state: tauri::State<VpnState>) -> Result<Vec<Subscription>, String> {
    state
        .subscriptions
        .lock()
        .map(|s| s.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn select_node(
    app: tauri::AppHandle,
    state: tauri::State<VpnState>,
    node_raw: String,
) -> Result<(), String> {
    let subs = state.subscriptions.lock().map_err(|e| e.to_string())?;
    let found = subs
        .iter()
        .flat_map(|s| s.nodes.iter())
        .find(|n| n.raw == node_raw)
        .cloned();
    drop(subs);

    if let Some(node) = found {
        let mut active = state.active_node.lock().map_err(|e| e.to_string())?;
        *active = Some(node.clone());

        let mut data = load_vpn_data(&app);
        data.active_node = Some(node);
        save_vpn_data(&app, &data);
    }
    Ok(())
}

#[tauri::command]
pub fn get_active_node(
    state: tauri::State<VpnState>,
) -> Result<Option<ProxyNode>, String> {
    state
        .active_node
        .lock()
        .map(|n| n.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_proxy(
    state: tauri::State<VpnState>,
    app: tauri::AppHandle,
    enable: bool,
) -> Result<bool, String> {
    let addr = {
        let lock = state.proxy_addr.lock().map_err(|e| e.to_string())?;
        lock.clone()
    };

    proxy::set_system_proxy(enable, &addr)?;

    let mut enabled = state.proxy_enabled.lock().map_err(|e| e.to_string())?;
    *enabled = enable;
    drop(enabled);

    let mut data = load_vpn_data(&app);
    data.proxy_addr = addr;
    save_vpn_data(&app, &data);

    Ok(enable)
}

#[tauri::command]
pub fn get_proxy_state(
    state: tauri::State<VpnState>,
) -> Result<(bool, String), String> {
    let enabled = state.proxy_enabled.lock().map_err(|e| e.to_string())?;
    let addr = state.proxy_addr.lock().map_err(|e| e.to_string())?;
    Ok((*enabled, addr.clone()))
}

#[tauri::command]
pub fn update_proxy_addr(
    state: tauri::State<VpnState>,
    app: tauri::AppHandle,
    addr: String,
) -> Result<(), String> {
    {
        let mut proxy_addr = state.proxy_addr.lock().map_err(|e| e.to_string())?;
        *proxy_addr = addr.clone();
    }

    let mut data = load_vpn_data(&app);
    data.proxy_addr = addr;
    save_vpn_data(&app, &data);

    Ok(())
}

#[tauri::command]
pub fn test_proxy(
    state: tauri::State<VpnState>,
) -> Result<bool, String> {
    let addr = state.proxy_addr.lock().map_err(|e| e.to_string())?.clone();
    proxy::test_proxy_connection(&addr)
}

#[tauri::command]
pub fn start_core(
    app: tauri::AppHandle,
    state: tauri::State<VpnState>,
) -> Result<bool, String> {
    let node = {
        let active = state.active_node.lock().map_err(|e| e.to_string())?;
        active.clone()
    };
    let node = node.ok_or("请先选择一个节点")?;
    let addr = state.proxy_addr.lock().map_err(|e| e.to_string())?.clone();

    v2ray::start_core(&app, &node.raw, &addr)?;

    // Automatically enable system proxy
    proxy::set_system_proxy(true, &addr)?;
    let mut enabled = state.proxy_enabled.lock().map_err(|e| e.to_string())?;
    *enabled = true;

    Ok(true)
}

#[tauri::command]
pub fn stop_core(
    app: tauri::AppHandle,
    state: tauri::State<VpnState>,
) -> Result<bool, String> {
    v2ray::stop_core(&app)?;

    // Disable system proxy
    proxy::set_system_proxy(false, "")?;
    let mut enabled = state.proxy_enabled.lock().map_err(|e| e.to_string())?;
    *enabled = false;

    Ok(false)
}

#[tauri::command]
pub fn core_status(app: tauri::AppHandle) -> Result<bool, String> {
    v2ray::core_running(&app)
}
