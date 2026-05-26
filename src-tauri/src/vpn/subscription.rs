use base64::Engine;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static SHARED_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .build()
        .expect("failed to build shared reqwest client")
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyNode {
    pub name: String,
    pub r#type: String,
    pub server: String,
    pub port: u16,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub url: String,
    pub name: String,
    pub nodes: Vec<ProxyNode>,
}

fn decode_name(raw: &str) -> String {
    urlencoding::decode(raw)
        .unwrap_or(std::borrow::Cow::Borrowed(raw))
        .to_string()
}

fn parse_node(line: &str) -> Option<ProxyNode> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    // vmess://
    if let Some(rest) = line.strip_prefix("vmess://") {
        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(rest) {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                return Some(ProxyNode {
                    name: json["ps"].as_str().unwrap_or("VMess").to_string(),
                    r#type: "vmess".into(),
                    server: json["add"].as_str().unwrap_or("").into(),
                    port: json["port"]
                        .as_str()
                        .and_then(|p| p.parse().ok())
                        .or_else(|| json["port"].as_u64().map(|v| v as u16))
                        .unwrap_or(0),
                    raw: line.to_string(),
                });
            }
        }
        return None;
    }

    // trojan://, ss://, vless:// — use URL parser
    if let Ok(url) = url::Url::parse(line) {
        let r#type = url.scheme().to_string();
        if !matches!(r#type.as_str(), "trojan" | "ss" | "vless" | "hysteria" | "hysteria2" | "tuic" | "hy2") {
            return None;
        }
        let name = url
            .fragment()
            .map(decode_name)
            .unwrap_or_else(|| format!("{} Node", r#type.to_uppercase()));
        let server = url.host_str().unwrap_or("").to_string();
        let port = url.port().unwrap_or(0);
        return Some(ProxyNode {
            name,
            r#type,
            server,
            port,
            raw: line.to_string(),
        });
    }

    None
}

pub fn parse_subscription_content(content: &str) -> Vec<ProxyNode> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(content.trim())
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_else(|_| content.to_string());

    decoded.lines().filter_map(parse_node).collect()
}

async fn try_fetch(url: &str, proxy: Option<&str>) -> Result<String, String> {
    let resp = if let Some(addr) = proxy {
        let proxy_url = if addr.starts_with("http") {
            addr.to_string()
        } else {
            format!("http://{}", addr)
        };
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .danger_accept_invalid_certs(true)
            .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| format!("代理配置错误: {}", e))?)
            .build()
            .map_err(|e| format!("创建客户端失败: {}", e))?;
        client
            .get(url)
            .header("User-Agent", "ClashforWindows/0.20.0")
            .header("Accept", "text/plain, */*")
            .send()
            .await
            .map_err(|e| {
                let mut msg = format!("请求失败: {}", e);
                if e.is_timeout() {
                    msg.push_str("\n连接超时");
                } else if e.is_connect() {
                    msg.push_str("\n无法连接到服务器");
                }
                msg
            })?
    } else {
        SHARED_CLIENT
            .get(url)
            .header("User-Agent", "ClashforWindows/0.20.0")
            .header("Accept", "text/plain, */*")
            .send()
            .await
            .map_err(|e| {
                let mut msg = format!("请求失败: {}", e);
                if e.is_timeout() {
                    msg.push_str("\n连接超时");
                } else if e.is_connect() {
                    msg.push_str("\n无法连接到服务器");
                }
                msg
            })?
    };

    if !resp.status().is_success() {
        return Err(format!(
            "服务器返回错误: {} {}",
            resp.status().as_u16(),
            resp.status().canonical_reason().unwrap_or("")
        ));
    }

    resp.text().await.map_err(|e| format!("读取响应失败: {}", e))
}

pub async fn fetch_subscription(url: &str, proxy: Option<&str>) -> Result<Vec<ProxyNode>, String> {
    // 1. Try direct
    let body = match try_fetch(url, None).await {
        Ok(b) => b,
        Err(direct_err) => {
            // 2. Fallback to proxy if configured
            if let Some(addr) = proxy {
                match try_fetch(url, Some(addr)).await {
                    Ok(b) => b,
                    Err(proxy_err) => {
                        return Err(format!(
                            "直连失败: {}\n代理 {} 也失败: {}\n\n提示: 使用「手动导入」——浏览器打开订阅链接，复制内容粘贴即可。",
                            direct_err, addr, proxy_err
                        ));
                    }
                }
            } else {
                return Err(format!("{}\n提示: 使用「手动导入」粘贴订阅内容。", direct_err));
            }
        }
    };

    if body.trim().is_empty() {
        return Err("订阅内容为空".into());
    }

    let nodes = parse_subscription_content(&body);
    if nodes.is_empty() {
        return Err("未解析到任何节点".into());
    }

    Ok(nodes)
}
