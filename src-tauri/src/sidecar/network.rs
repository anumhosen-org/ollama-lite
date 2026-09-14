use std::net::UdpSocket;

#[tauri::command]
pub fn get_local_ip_addresses() -> Vec<String> {
    let mut ips = Vec::new();
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                ips.push(addr.ip().to_string());
            }
        }
    }
    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }
    ips
}

#[tauri::command]
pub fn get_local_ip() -> String {
    get_local_ip_addresses().first().cloned().unwrap_or_else(|| "127.0.0.1".to_string())
}
