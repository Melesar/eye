use messages::HelloRequest;
use prost::Message;
use tokio::{net::TcpSocket, io::{AsyncWriteExt, AsyncReadExt}};

pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let sender = "0.0.0.0:6688".parse().unwrap();
    let mut tcp_stream = TcpSocket::new_v4()?.connect(sender).await?;
    tcp_stream.write_u32(0).await?;

    let mut message = HelloRequest::default();
    message.code = 123;
    tcp_stream.write_u32(message.encoded_len() as u32).await?;

    let mut buffer = Vec::new();
    match message.encode(&mut buffer) {
        Ok(_) => {
            tcp_stream.write_all(&mut buffer).await?;
        },
        Err(e) => eprintln!("Failed to encode a message {}", e),
    }

    println!("Getting ready to receive a message");
    let mut receive_buffer = [0_u8; 100];
    let bytes_read = tcp_stream.read(&mut receive_buffer).await?;
    println!("Received {} bytes back from the server", bytes_read);

    Ok(())
}
