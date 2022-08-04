use std::io::SeekFrom;
use std::io::Seek;
use messages::{HelloRequest, HelloResponse, ServoRotateRequest};
use prost::Message;
use tokio::{net::TcpSocket, io::{AsyncWriteExt, AsyncReadExt}};
use std::io::Cursor;

pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let sender = "192.168.1.251:6688".parse().unwrap();
    let mut tcp_stream = TcpSocket::new_v4()?.connect(sender).await?;
    tcp_stream.write_u32_le(0).await?;

    let mut message = HelloRequest::default();
    message.code = 123;
    tcp_stream.write_u32_le(message.encoded_len() as u32).await?;

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

    let mut cursor = Cursor::new(receive_buffer);
    cursor.seek(SeekFrom::Current(8));
    match HelloResponse::decode(cursor) {
        Ok(response) => println!("Response: {}:{}", response.stream_host, response.stream_port),
        Err(e) => eprintln!("Failed to decode response: {}", e),
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut rotate_request = ServoRotateRequest::default();
    rotate_request.dx = 1;
    rotate_request.dy = 0;
    buffer.clear();
    match rotate_request.encode(&mut buffer) {
        Ok(_) => {
            for i in 0..3 {
                println!("Rotating {}...", i + 1);
                tcp_stream.write_all(&mut buffer).await?;
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        },
        
        Err(e) => eprintln!("Failed to encode rotate message {}", e),
    }


    Ok(())
}
