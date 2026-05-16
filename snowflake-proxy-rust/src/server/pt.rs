use std::env;
use std::net::SocketAddr;

#[derive(Debug)]
#[allow(dead_code)]
pub struct BindAddr {
    pub method_name: String,
    pub addr: SocketAddr,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ServerInfo {
    pub bindaddrs: Vec<BindAddr>,
    pub or_port: Option<SocketAddr>,
}

#[allow(dead_code)]
pub fn server_setup() -> Result<ServerInfo, String> {
    let _ver = env::var("TOR_PT_MANAGED_TRANSPORT_VER")
        .map_err(|_| "TOR_PT_MANAGED_TRANSPORT_VER not set")?;

    let transports = env::var("TOR_PT_SERVER_TRANSPORTS")
        .map_err(|_| "TOR_PT_SERVER_TRANSPORTS not set")?;

    let bindaddrs_str = env::var("TOR_PT_SERVER_BINDADDRS")
        .map_err(|_| "TOR_PT_SERVER_BINDADDRS not set")?;

    let or_port = env::var("TOR_PT_ORPORT")
        .ok()
        .and_then(|s| s.parse().ok());

    let mut bindaddrs = Vec::new();
    for part in bindaddrs_str.split(',') {
        let kv: Vec<&str> = part.split('-').collect();
        if kv.len() == 2 {
            let method = kv[0].to_string();
            if transports.contains(&method) {
                if let Ok(addr) = kv[1].parse::<SocketAddr>() {
                    bindaddrs.push(BindAddr { method_name: method, addr });
                }
            }
        }
    }

    Ok(ServerInfo { bindaddrs, or_port })
}

#[allow(dead_code)]
pub fn smethod(method: &str, addr: SocketAddr) {
    println!("SMETHOD {} {}", method, addr);
}

#[allow(dead_code)]
pub fn smethods_done() {
    println!("SMETHODS DONE");
}
