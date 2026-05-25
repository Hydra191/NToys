use std::os::windows::process::CommandExt;
use std::sync::Mutex;
use tauri::Manager;

pub struct V2rayState {
    pub running: Mutex<bool>,
    pub child: Mutex<Option<std::process::Child>>,
}

fn core_binary_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    let bases = [
        app.path().resource_dir().unwrap_or_default(),
        std::env::current_dir().unwrap_or_default(),
        std::env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .to_path_buf(),
    ];

    let mut search_dirs: Vec<std::path::PathBuf> = vec![];
    for base in &bases {
        search_dirs.push(base.join("binaries"));
        search_dirs.push(base.join("src-tauri/binaries"));
    }

    // Find v2ray*.exe or xray*.exe in place — don't copy
    for dir in &search_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name_str = entry.file_name().to_string_lossy().to_lowercase();
                if (name_str.starts_with("v2ray") || name_str.starts_with("xray"))
                    && name_str.ends_with(".exe")
                {
                    return entry.path();
                }
            }
        }
    }

    // Fallback: legacy copy at app data dir
    app.path()
        .app_data_dir()
        .expect("no app data dir")
        .join("v2ray.exe")
}

fn config_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .expect("no app data dir")
        .join("v2ray-config.json")
}

/// Parse ss:// URI. Two formats:
///   Legacy: ss://base64(method:password@host:port)
///   SIP002: ss://base64(method:password)@host:port#name
fn parse_ss(line: &str) -> Option<serde_json::Value> {
    let rest = line.strip_prefix("ss://")?;
    // The part before '@' may be base64 or plain userinfo, the part after '@' is host:port
    let (userinfo, host_port) = if rest.contains('@') {
        let at = rest.rfind('@')?;
        (&rest[..at], &rest[at + 1..])
    } else {
        // Legacy: everything is base64 encoded
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            rest,
        )
        .ok()?;
        let s = String::from_utf8_lossy(&decoded);
        let at = s.rfind('@')?;
        // userinfo = method:password, host_port = host:port
        return parse_ss_parts(&s[..at], &s[at + 1..]);
    };

    // Decode userinfo (may be base64)
    let userinfo = if !userinfo.contains(':') {
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            userinfo,
        )
        .ok()?;
        String::from_utf8_lossy(&decoded).to_string()
    } else {
        userinfo.to_string()
    };

    parse_ss_parts(&userinfo, host_port)
}

fn parse_ss_parts(userinfo: &str, host_port: &str) -> Option<serde_json::Value> {
    let colon = userinfo.find(':')?;
    let method = &userinfo[..colon];
    let password = &userinfo[colon + 1..];

    let (host, port) = if let Some(col) = host_port.rfind(':') {
        (&host_port[..col], host_port[col + 1..].parse::<u16>().ok()?)
    } else {
        (host_port, 8388u16)
    };

    Some(serde_json::json!({
        "protocol": "shadowsocks",
        "tag": "proxy",
        "settings": {
            "servers": [{
                "address": host,
                "port": port,
                "method": method,
                "password": password,
            }]
        },
    }))
}

/// Generate v2ray config from a proxy node URI
pub fn generate_config(raw: &str, proxy_addr: &str) -> Result<String, String> {
    let line = raw.trim();
    let mut outbound = serde_json::json!({});

    if let Some(rest) = line.strip_prefix("vmess://") {
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            rest,
        )
        .map_err(|e| format!("base64 decode: {}", e))?;
        let v: serde_json::Value =
            serde_json::from_slice(&decoded).map_err(|e| format!("json parse: {}", e))?;

        let net = v["net"].as_str().unwrap_or("tcp");
        let host = v["host"].as_str().unwrap_or("");
        let path = v["path"].as_str().unwrap_or("/");
        let sni = v["sni"].as_str().unwrap_or("");
        let tls = v["tls"].as_str() == Some("tls");

        let mut stream = serde_json::json!({"network": net});

        match net {
            "ws" => {
                let mut ws = serde_json::json!({"path": path});
                if !host.is_empty() {
                    ws["headers"] = serde_json::json!({"Host": host});
                }
                stream["wsSettings"] = ws;
            }
            "tcp" => {
                if !host.is_empty() {
                    stream["tcpSettings"] = serde_json::json!({
                        "header": {
                            "type": "http",
                            "request": {
                                "headers": {"Host": [host]},
                                "path": [path],
                            }
                        }
                    });
                }
            }
            _ => {
            }
        }

        if tls {
            stream["security"] = serde_json::json!("tls");
            stream["tlsSettings"] = serde_json::json!({
                "serverName": if sni.is_empty() { v["add"].as_str().unwrap_or("") } else { sni },
            });
        }

        outbound = serde_json::json!({
            "protocol": "vmess",
            "tag": "proxy",
            "settings": {
                "vnext": [{
                    "address": v["add"].as_str().unwrap_or(""),
                    "port": v["port"].as_str().and_then(|p| p.parse::<u16>().ok()).unwrap_or(0),
                    "users": [{
                        "id": v["id"].as_str().unwrap_or(""),
                        "alterId": v["aid"].as_i64().unwrap_or(0),
                        "security": "auto",
                    }]
                }]
            },
            "streamSettings": stream,
        });
    } else if let Some(_rest) = line.strip_prefix("ss://") {
        outbound = parse_ss(line).ok_or_else(|| "无法解析 ss:// 节点".to_string())?;
    } else if let Some(rest) = line.strip_prefix("trojan://") {
        let url = url::Url::parse(&format!("trojan://{}", rest))
            .map_err(|e| format!("解析 trojan URL 失败: {}", e))?;
        let sni = url
            .query_pairs()
            .find(|(k, _)| k == "sni" || k == "peer")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| url.host_str().unwrap_or("").to_string());
        let net = url
            .query_pairs()
            .find(|(k, _)| k == "type")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "tcp".to_string());
        let sec = url
            .query_pairs()
            .find(|(k, _)| k == "security")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "tls".to_string());
        let path = url
            .query_pairs()
            .find(|(k, _)| k == "path")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "/".to_string());
        let ws_host = url
            .query_pairs()
            .find(|(k, _)| k == "host")
            .map(|(_, v)| v.to_string())
            .unwrap_or_default();

        let mut stream = serde_json::json!({"network": net});
        if net == "ws" {
            let mut ws = serde_json::json!({"path": path});
            if !ws_host.is_empty() {
                ws["headers"] = serde_json::json!({"Host": ws_host});
            }
            stream["wsSettings"] = ws;
        }
        if sec == "tls" {
            stream["security"] = serde_json::json!("tls");
            stream["tlsSettings"] = serde_json::json!({"serverName": sni});
        }

        outbound = serde_json::json!({
            "protocol": "trojan",
            "tag": "proxy",
            "settings": {
                "servers": [{
                    "address": url.host_str().unwrap_or(""),
                    "port": url.port().unwrap_or(443),
                    "password": url.username(),
                }]
            },
            "streamSettings": stream,
        });
    } else if let Some(rest) = line.strip_prefix("vless://") {
        let url = url::Url::parse(&format!("vless://{}", rest))
            .map_err(|e| format!("解析 vless URL 失败: {}", e))?;
        let network = url
            .query_pairs()
            .find(|(k, _)| k == "type")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "tcp".to_string());
        let security = url
            .query_pairs()
            .find(|(k, _)| k == "security")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "none".to_string());
        let sni = url
            .query_pairs()
            .find(|(k, _)| k == "sni" || k == "peer")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| url.host_str().unwrap_or("").to_string());
        let flow = url
            .query_pairs()
            .find(|(k, _)| k == "flow")
            .map(|(_, v)| v.to_string())
            .unwrap_or_default();
        let path = url
            .query_pairs()
            .find(|(k, _)| k == "path")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "/".to_string());
        let ws_host = url
            .query_pairs()
            .find(|(k, _)| k == "host")
            .map(|(_, v)| v.to_string())
            .unwrap_or_default();

        let mut stream = serde_json::json!({"network": network});
        if network == "ws" {
            let mut ws = serde_json::json!({"path": path});
            if !ws_host.is_empty() {
                ws["headers"] = serde_json::json!({"Host": ws_host});
            }
            stream["wsSettings"] = ws;
        }
        if security == "tls" || security == "reality" {
            stream["security"] = serde_json::json!(security);
            stream["tlsSettings"] = serde_json::json!({"serverName": sni});
        }

        let mut user = serde_json::json!({
            "id": url.username(),
            "encryption": "none",
        });
        if !flow.is_empty() {
            user["flow"] = serde_json::json!(flow);
        }

        outbound = serde_json::json!({
            "protocol": "vless",
            "tag": "proxy",
            "settings": {
                "vnext": [{
                    "address": url.host_str().unwrap_or(""),
                    "port": url.port().unwrap_or(443),
                    "users": [user],
                }]
            },
            "streamSettings": stream,
        });
    } else {
        return Err(format!(
            "不支持的协议: {}。v2ray 支持 vmess, vless, trojan, ss",
            line.chars().take(20).collect::<String>()
        ));
    }

    let (host, port) = {
        let parts: Vec<&str> = proxy_addr.split(':').collect();
        let h = parts.first().copied().unwrap_or("127.0.0.1");
        let p = parts.get(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(7890);
        (h, p)
    };
    let socks_port = port + 1;

    let config = serde_json::json!({
        "log": {"loglevel": "warning"},
        "inbounds": [
            {
                "tag": "http-in",
                "listen": host,
                "port": port,
                "protocol": "http",
                "settings": {},
            },
            {
                "tag": "socks-in",
                "listen": host,
                "port": socks_port,
                "protocol": "socks",
                "settings": {"auth": "noauth", "udp": true},
            },
        ],
        "outbounds": [
            {
                "tag": "direct",
                "protocol": "freedom",
                "settings": {},
            },
            outbound,
        ],
        "routing": {
            "rules": [
                {
                    "type": "field",
                    "inboundTag": ["http-in", "socks-in"],
                    "outboundTag": "proxy",
                }
            ]
        },
    });

    serde_json::to_string_pretty(&config).map_err(|e| format!("json: {}", e))
}

pub fn start_core(app: &tauri::AppHandle, raw: &str, proxy_addr: &str) -> Result<(), String> {
    let state = app.state::<V2rayState>();

    stop_core(app)?;

    let bin = core_binary_path(app);
    if !bin.exists() {
        let dir = bin.parent().unwrap_or(&bin).display().to_string();
        return Err(format!(
            "v2ray.exe / xray.exe 未找到。\n请将 v2ray �? xray 内核放到:\n{}\n\n请从 v2fly.org �? xtls.github.io 下载 Windows 版本，解压后放入上述目录�?",
            dir
        ));
    }

    let config_json = generate_config(raw, proxy_addr)?;
    let cfg_path = config_path(app);
    if let Some(parent) = cfg_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&cfg_path, config_json)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    let cfg_str = cfg_path.to_str().unwrap_or("");

    // Try multiple CLI syntaxes: v5/xray variants, then v4
    let cmd_sets: [(&[&str], &str); 4] = [
        (&["run", "-c"], "v5 run -c"),
        (&["run", "-config"], "v5 run -config"),
        (&["-c"], "v4 -c"),
        (&["run"], "v5 run positional"),
    ];

    let mut errors: Vec<String> = vec![];

    for (args, version) in &cmd_sets {
        let mut child = std::process::Command::new(&bin)
            .creation_flags(0x08000000)
            .args(*args)
            .arg(cfg_str)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("v2ray spawn error: {}", e))?;

        std::thread::sleep(std::time::Duration::from_millis(600));

        match child.try_wait() {
            Ok(None) => {
                let mut running = state.running.lock().map_err(|e| e.to_string())?;
                *running = true;
                let mut child_store = state.child.lock().map_err(|e| e.to_string())?;
                *child_store = Some(child);
                return Ok(());
            }
            Ok(Some(status)) => {
                use std::io::Read;
                let stderr = child.stderr.take().and_then(|mut pipe| {
                    let mut buf = String::new();
                    std::io::BufReader::new(&mut pipe).read_to_string(&mut buf).ok().map(|_| buf)
                }).unwrap_or_default();
                errors.push(format!(
                    "[{}] exit code {:?}: {}",
                    version, status.code(), stderr.trim()
                ));
            }
            Err(e) => {
                errors.push(format!("[{}] process error: {}", version, e));
            }
        }
    }

    Err(format!(
        "v2ray failed to start:\n{}\n\nConfig: {}",
        errors.join("\n"), cfg_str
    ))
}

pub fn stop_core(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<V2rayState>();
    let mut running = state.running.lock().map_err(|e| e.to_string())?;
    let mut child_store = state.child.lock().map_err(|e| e.to_string())?;

    if !*running {
        return Ok(());
    }

    if let Some(mut child) = child_store.take() {
        let _ = child.kill();
        let _ = child.wait();
    }

    *running = false;
    Ok(())
}

pub fn core_running(app: &tauri::AppHandle) -> Result<bool, String> {
    let state = app.state::<V2rayState>();
    let mut running = state.running.lock().map_err(|e| e.to_string())?;
    let mut child_store = state.child.lock().map_err(|e| e.to_string())?;

    // 检查子进程是否还活着
    if *running {
        if let Some(ref mut child) = *child_store {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // 进程已退�?
                    *running = false;
                    *child_store = None;
                }
                Ok(None) => {
                    // 进程仍在运行
                }
                Err(_) => {
                    // 无法检查，假定已退�?
                    *running = false;
                    *child_store = None;
                }
            }
        } else {
            *running = false;
        }
    }

    Ok(*running)
}
