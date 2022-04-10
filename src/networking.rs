use std::net::{UdpSocket, Ipv4Addr};
use std::io::Result;
use std::u128;
use crate::ServerConfig;

static ANY_ADDR : Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
static MULTICAST_ADDRESS : Ipv4Addr = Ipv4Addr::new(239, 255, 6, 6);
static MULTICAST_PORT : u16 = 6688;

pub fn create_server_multicast_socket() -> Result<UdpSocket> {
    UdpSocket::bind((ANY_ADDR, MULTICAST_PORT))
}

pub fn send_multicast_packet(socket: &UdpSocket, config: &ServerConfig) -> Result<()> {
    let code_length = std::mem::size_of::<u128>();
    let name_length = config.display_name.len();

    let mut buf = [0u8; 1000];
    let mut ptr = 0;

    buf[ptr..ptr+4].copy_from_slice(&(code_length + name_length).to_le_bytes());
    ptr += 4;

    buf[ptr..ptr+code_length].copy_from_slice(&config.code.to_le_bytes());
    ptr += code_length;

    buf[ptr..ptr+name_length].copy_from_slice(config.display_name.as_bytes());
    ptr += name_length;

    socket.send_to(&buf[0..ptr], (MULTICAST_ADDRESS, MULTICAST_PORT))?;

    Ok(())
}

