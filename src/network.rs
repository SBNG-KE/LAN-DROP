use local_ip_address::local_ip;
use std::net::IpAddr;

pub fn get_local_ip() -> Option<IpAddr> {
    // Fetches the primary network interface IP (e.g., 192.168.x.x)
    local_ip().ok()
}
