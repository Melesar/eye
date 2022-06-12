use prost::{bytes::{Buf, BufMut}, DecodeError};
use messages::{HelloRequest, HelloResponse};
use prost::Message;
use tokio::io::{AsyncReadExt, AsyncRead, AsyncWriteExt};
use tokio::sync::mpsc;

pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}

#[derive(Clone, Copy, Debug)]
enum MessageType {
    HelloRequest,
    HelloResponse
}

enum ReadState {
    MsgType,
    Length,
    Payload
}

struct ReceivedMessage {
    sender_id: u32,
    msg_type: MessageType,
    payload: Vec<u8>
}

unsafe impl Send for ReceivedMessage {}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6688").await.unwrap();
    let (stream, _sender) = listener.accept().await?;
    let (reader, mut writer) = stream.into_split();
    let (tx, mut rx) = mpsc::channel(10);
    tokio::spawn(async move { handle_connection(reader, tx.clone()).await });

    if let Some(message) = rx.recv().await {
        match message.msg_type {
            MessageType::HelloRequest => {
                let mut cursor = std::io::Cursor::new(message.payload);
                match HelloRequest::decode(&mut cursor) {
                    Ok(hello_request) => {
                        println!("Received hello request with code {}", hello_request.code);
                        writer.write_u32(42).await?;
                    },
                    Err(decode_error) => {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, decode_error));
                    }
                }
            },
            MessageType::HelloResponse => todo!(),
        }
    }

    Ok(())
}

async fn handle_connection<R>(reader: R, sender: mpsc::Sender<ReceivedMessage>) -> Result<(), std::io::Error>
    where R: AsyncRead + std::marker::Unpin {

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

                sender.send(ReceivedMessage {
                    sender_id: 1,
                    msg_type: current_message_type,
                    payload: buffer[0..message_length].to_owned() 
                })
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            },
        }
    }

    Ok(())
}
