use std::io::Result;
use std::net::{Ipv4Addr, UdpSocket};

fn main() {
    //TODO check for motion availability

    //TODO optionally: check for gpio availability

    let handle = std::thread::spawn(|| {
        if cfg!(feature = "client") {
            start_client().expect("Client failed");
        } else {
            start_server().expect("Server failed");
        }
    });

    handle.join().expect("Failed to join the handle");
}

fn start_client() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:6688")?;
    let multi_address = Ipv4Addr::new(239, 255, 6, 6);
    assert!(multi_address.is_multicast());
    let interface = Ipv4Addr::new(0, 0, 0, 0);
    socket.join_multicast_v4(&multi_address, &interface)?;

    let mut buff = [0u8; 65000];
    println!("Client is waiting");
    let (amount, sender) = socket.recv_from(&mut buff)?;

    println!("Received {} bytes from {:?}", amount, sender);
    Ok(())
}

fn start_server() -> Result<()> {
    println!("Server starts and falls asleep");
    std::thread::sleep(std::time::Duration::from_secs(1));
    let socket = UdpSocket::bind("0.0.0.0:6685")?;
    let buff = [1u8; 100];
    if let Err(e) = socket.send_to(&buff, "239.255.6.6:6688") {
        println!("Failed to send multicast.");
        eprintln!("{}", e);
    };

    Ok(())
}
