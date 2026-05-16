use rand::{thread_rng, Rng};
use base64::{engine::general_purpose, Engine as _};
use std::net::IpAddr;

pub fn gen_session_id() -> String {
    let mut rng = thread_rng();
    let mut buf = [0u8; 16];
    rng.fill(&mut buf);
    general_purpose::STANDARD.encode(buf).replace("=", "")
}

pub fn is_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip4) => {
            let octets = ip4.octets();
            octets[0] == 10 ||
            (octets[0] == 172 && (octets[1] & 0xf0) == 16) ||
            (octets[0] == 192 && octets[1] == 168) ||
            (octets[0] == 100 && (octets[1] & 0xc0) == 64) ||
            (octets[0] == 169 && octets[1] == 254)
        }
        IpAddr::V6(ip6) => {
            let segments = ip6.segments();
            (segments[0] & 0xfe00) == 0xfc00
        }
    }
}

pub fn strip_local_addresses(sdp: &str) -> String {
    let lines: Vec<&str> = sdp.lines().collect();
    let mut new_lines = Vec::new();

    for line in lines {
        if line.starts_with("a=candidate:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 8 {
                let address = parts[4];
                let typ = parts[7];
                if typ == "host" {
                    if let Ok(ip) = address.parse::<IpAddr>() {
                        if is_local(ip) || ip.is_unspecified() || ip.is_loopback() {
                            continue;
                        }
                    }
                }
            }
        }
        new_lines.push(line);
    }
    new_lines.join("\r\n") + "\r\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_is_local() {
        assert!(is_local(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(is_local(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(!is_local(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }

    #[test]
    fn test_strip_local_addresses() {
        let sdp = "v=0\r\na=candidate:1 1 UDP 2130706431 192.168.1.1 12345 typ host\r\na=candidate:2 1 UDP 2130706431 8.8.8.8 12345 typ host\r\n";
        let stripped = strip_local_addresses(sdp);
        assert!(!stripped.contains("192.168.1.1"));
        assert!(stripped.contains("8.8.8.8"));
    }
}
