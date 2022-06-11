use prost::{bytes::{Buf, BufMut}, DecodeError};
use messages::{HelloRequest, HelloResponse};
use prost::Message;
use tokio::io::AsyncReadExt;

pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}

#[derive(Clone, Copy)]
enum MessageType {
    HelloRequest,
    HelloResponse
}

enum ReadState {
    MsgType,
    Length,
    Payload
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6688").await.unwrap();
    let (mut stream, _sender) = listener.accept().await?;
    let (reader, writer) = stream.split();
    let mut reader = tokio::io::BufReader::new(reader);

    let messages_lookup = vec![MessageType::HelloRequest, MessageType::HelloResponse];

    let mut current_state = ReadState::MsgType;
    let mut num_buffer = [0_u8; 4];
    let mut buffer : Vec<u8> = Vec::with_capacity(2048);
    let mut current_message_type = MessageType::HelloRequest;
    let mut message_length = 0;
    loop {
        match current_state {
            ReadState::MsgType => {
                let bytes_read = reader.read(&mut num_buffer).await?;
                if bytes_read < num_buffer.len() { continue; }

                let message_type_index = u32::from_be_bytes(num_buffer);
                match messages_lookup.get(message_type_index as usize).map(|t| *t) {
                    Some(tt) => { current_message_type = tt },
                    None => { eprintln!("Unrecognized message type index {}", message_type_index); break; },
                }

                current_state = ReadState::Length;
            },
            ReadState::Length => {
                let bytes_read = reader.read(&mut num_buffer).await?;
                if bytes_read < num_buffer.len() { continue; }

                message_length = u32::from_be_bytes(num_buffer) as usize;
                current_state = ReadState::Payload;
            },
            ReadState::Payload => {
                if buffer.len() < message_length {
                    buffer.resize(message_length, 0);
                }

                let bytes_read = reader.read(&mut buffer[0..message_length]).await?;
                if bytes_read < buffer.len() { continue; }

                match current_message_type {
                    MessageType::HelloRequest => {
                        let mut cursor = std::io::Cursor::new(buffer);
                        match HelloRequest::decode(&mut cursor) {
                            Ok(request) => {
                                println!("Received request {:?}", request);
                                current_state = ReadState::MsgType;
                                buffer = Vec::with_capacity(2048);
                            }
                            Err(e) => { eprintln!("Failed to decode a message {}", e); break },
                        }
                    },
                    MessageType::HelloResponse => todo!(),
                }
            },
        }
    }

    Ok(())
}
