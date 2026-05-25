use winreg::enums::*;
use winreg::RegKey;

const INTERNET_SETTINGS: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

pub fn set_system_proxy(enable: bool, addr: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu
        .open_subkey_with_flags(INTERNET_SETTINGS, KEY_SET_VALUE)
        .map_err(|e| format!("open registry: {}", e))?;

    if enable {
        settings
            .set_value("ProxyEnable", &1u32)
            .map_err(|e| format!("set ProxyEnable: {}", e))?;
        settings
            .set_value("ProxyServer", &addr)
            .map_err(|e| format!("set ProxyServer: {}", e))?;
    } else {
        settings
            .set_value("ProxyEnable", &0u32)
            .map_err(|e| format!("set ProxyEnable: {}", e))?;
        settings.delete_value("ProxyServer").ok();
    }

    Ok(())
}


/// Test if a TCP connection can be established to the proxy address
pub fn test_proxy_connection(addr: &str) -> Result<bool, String> {
    let addr = if addr.contains("://") {
        addr.to_string()
    } else {
        format!("http://{}", addr)
    };

    let url = url::Url::parse(&addr).map_err(|e| format!("invalid address: {}", e))?;
    let host = url.host_str().unwrap_or("127.0.0.1");
    let port = url.port().unwrap_or(7890);

    let socket = format!("{}:{}", host, port);
    match std::net::TcpStream::connect_timeout(
        &socket.parse().map_err(|e| format!("parse: {}", e))?,
        std::time::Duration::from_secs(3),
    ) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
