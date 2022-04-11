mod networking;

use std::process::{Command, Stdio};
use std::convert::TryInto;
use std::io::{Result, ErrorKind};
use std::net::{Ipv4Addr, UdpSocket, SocketAddr};
use std::time::Duration;
use std::u128;

use pnet::ipnetwork::IpNetwork;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;

#[derive(Clone)]
pub struct ServerConfig {
   pub display_name: String,
   pub code: u128,
}

#[derive(Debug, Copy, Clone)]
enum Event {
    Connected,
    Disconnected
}

const CAMERA_PORT : u16 = 8081;

#[tokio::main]
async fn main() {
    //Testing code
    if cfg!(feature = "client") {
        start_client().await.expect("Client failed");
        return;
    }

    if !check_camera_availability() {
        eprintln!("Camera is not available. Shutting down");
        return;
    }
    
    let current_ip = get_current_ip_address().expect("Failed to get current ip address");

    //TODO optionally: check for gpio availability

    let config = ServerConfig { display_name: "My Raspberry".into(), code: 483921341};

    let multicast_config = config.clone();
    std::thread::spawn(move || {
        if let Err(e) = start_multicasting(&multicast_config) {
            eprintln!("Multicast failed. No server discovery anymore");
            eprintln!("{}", e);
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6688").await.unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mut currently_connected = 0_u32;

    loop {
        tokio::select! {
            accept_result = listener.accept() => if let Ok((mut stream, _)) = accept_result {
                let config = config.clone();
                let sender = tx.clone();
                tokio::spawn(async move {
                    let (reader, mut writer) = stream.split();
                    let mut reader = tokio::io::BufReader::new(reader);
                    let code = reader.read_u128_le().await.unwrap_or(0);
                    println!("Received code {}", code);
                    if code == 0 {
                        sender.send(Event::Disconnected).await.unwrap_or_default();
                    } else if config.code == code {
                        sender.send(Event::Connected).await.unwrap_or_default();

                        let mut buffer = [0u8; 6];
                        buffer[..4].copy_from_slice(&current_ip.octets());
                        buffer[4..].copy_from_slice(&CAMERA_PORT.to_le_bytes());

                        writer.write(&buffer).await.unwrap_or_default();
                    }
                });
            },

            receive_result = rx.recv() => match receive_result {
                Some(Event::Connected) => {
                    if currently_connected == 0 { enable_camera() } 

                    currently_connected += 1;
                },
                Some(Event::Disconnected) => {
                    if currently_connected == 1 {
                        disable_camera();
                        currently_connected = 0;
                    } else if currently_connected != 0 {
                        currently_connected -= 1;
                    }
                }
                _ => {}
            }
        }
    }
}

fn start_multicasting(config: &ServerConfig) -> Result<()> {
    let socket = networking::create_server_multicast_socket()?;

    loop {
        std::thread::sleep(Duration::from_secs(3));
        networking::send_multicast_packet(&socket, &config)?;
    }
}

fn enable_camera() {

}

fn disable_camera() {

}

fn get_current_ip_address() -> Result<Ipv4Addr> {
    let err = || std::io::Error::new(std::io::ErrorKind::NotFound, "No network interfaces found");

    let interface = pnet::datalink::interfaces()
        .into_iter()
        .filter(|i| i.is_up() && !i.is_loopback() && !i.ips.is_empty())
        .take(1)
        .next()
        .ok_or_else(err)?;

    interface.ips.first()
        .and_then(|addr| match addr {
            IpNetwork::V4(ipv4_addr) => Some(ipv4_addr.ip()),
            _ => None,
        })
        .ok_or_else(err)
}

fn check_camera_availability() -> bool {
    Command::new("motion")
        .arg("-h")
        .stdout(Stdio::null())
        .status()
        .is_ok()
}


async fn start_client() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:6688")?;
    let multi_address = Ipv4Addr::new(239, 255, 6, 6);
    assert!(multi_address.is_multicast());
    let interface = Ipv4Addr::new(0, 0, 0, 0);
    socket.join_multicast_v4(&multi_address, &interface)?;

    let mut buff = [0u8; 65000];
    println!("Client is waiting");
    let (amount, sender) = socket.recv_from(&mut buff)?;

    println!("Received {} bytes from {:?}", amount, sender);
    let (int_bytes, rest) = buff.split_at(std::mem::size_of::<u32>());
    let msg_length = u32::from_le_bytes(int_bytes.try_into().unwrap());
    println!("Message length: {}", msg_length);

    let (mut code, rest) = rest.split_at(16_usize);
    let (name_bytes, _) = rest.split_at((msg_length - 16) as usize);
    println!("Server name: {}", String::from_utf8(name_bytes.into()).unwrap());
    drop(socket);

    let mut tcp_stream = TcpSocket::new_v4()?.connect(sender).await?;
    tcp_stream.write_all_buf(&mut code).await?;

    let mut buffer = vec![];
    tcp_stream.read_to_end(&mut buffer).await?;

    let address = Ipv4Addr::new(buffer[0], buffer[1], buffer[2], buffer[3]);
    let mut port_bytes = [0u8; 2];
    port_bytes.copy_from_slice(&buffer[4..]);
    let port = u16::from_le_bytes(port_bytes);

    println!("Received address {}:{}", address, port);

    Ok(())
}

