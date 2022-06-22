mod networking;
mod camera;
mod server;

use std::net::Ipv4Addr;

use pnet::ipnetwork::IpNetwork;
use server::Server;


#[tokio::main]
async fn main() {

    if !camera::is_available() {
        eprintln!("Camera is not available. Shutting down");
        return;
    }
    
    let server = Server::new();
    if let Err(e) = server.start().await {
        eprintln!("Server failed: {}", e);
    }
}


fn get_current_ip_address() -> Ipv4Addr {
    let interface = pnet::datalink::interfaces()
        .into_iter()
        .filter(|i| i.is_up() && !i.is_loopback() && !i.ips.is_empty())
        .take(1)
        .next();

    if let Some(i) = interface {
        i.ips.first()
            .and_then(|addr| match addr {
                IpNetwork::V4(ipv4_addr) => Some(ipv4_addr.ip()),
                _ => None,
            })
            .unwrap_or(Ipv4Addr::LOCALHOST)
    } else {
        Ipv4Addr::LOCALHOST
    }

}

