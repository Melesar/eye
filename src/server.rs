use std::collections::HashMap;
use std::net::Ipv4Addr;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::Sender;

use crate::ServerConfig;
use crate::{camera, networking};

thread_local! {
    static MESSAGES_LOOKUP: Vec<MessageType> = vec![
        MessageType::HelloRequest,
        MessageType::HelloResponse
    ];
}

pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}

#[derive(Debug, Clone)]
enum Event {
    Connected,
    Disconnected,
    MessageReceived(ReceivedMessage)
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

#[derive(Clone, Debug)]
struct ReceivedMessage {
    sender_id: u32,
    msg_type: MessageType,
    payload: Vec<u8>
}

pub struct Server {
    config: ServerConfig,
    current_ip: Ipv4Addr,
    client_connections: HashMap<u32, OwnedWriteHalf>
}

impl Server {

    pub fn new(config: ServerConfig, current_ip: Ipv4Addr) -> Self {
        Server { config, current_ip, client_connections: HashMap::new() }
    }
    
    pub async fn start(mut self) -> Result<(), std::io::Error> {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:6688").await.unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let mut currently_connected = 0_u32;
        let mut current_client_id = 0_u32;

        loop {
            tokio::select! {
                accept_result = listener.accept() => if let Ok((stream, _)) = accept_result {
                    let (reader, mut writer) = stream.into_split();
                    let config = self.config.clone();
                    let sender = tx.clone();
                    tokio::spawn(async move { handle_client_connection(reader, config, sender).await });

                    let message = messages::HelloResponse { 
                        stream_host: self.current_ip.clone().to_string(),
                        stream_port: camera::CAMERA_PORT as i32 
                    };
                    networking::send_message(&mut writer, message).await?;

                    self.client_connections.insert(current_client_id, writer);
                    current_client_id += 1;
                },

                receive_result = rx.recv() => match receive_result {
                    Some(Event::Connected) => {
                        println!("Client connected");
                        if currently_connected == 0 && !camera::is_active() {
                            println!("Enabling camera");
                            camera::start();
                        } 

                        currently_connected += 1;
                    },
                    Some(Event::Disconnected) => {
                        println!("Client disconnected");
                        if currently_connected == 1 && camera::is_active() {
                            println!("Disabling camera");
                            camera::stop();
                            currently_connected = 0;
                        } else if currently_connected != 0 {
                            currently_connected -= 1;
                        }
                    },
                    Some(Event::MessageReceived(message_data)) => {
                        println!("Received {:?} from {}", message_data.msg_type, message_data.sender_id);
                    },
                    _ => {}
                }
            }
        }
    }
}

async fn handle_client_connection<R>(reader: R, config: ServerConfig, sender: Sender<Event>) -> Result<(), std::io::Error>
    where R: AsyncRead + std::marker::Unpin  {

    let mut reader = tokio::io::BufReader::new(reader);
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
                match MESSAGES_LOOKUP.with(|l| l.get(message_type_index as usize).map(|t| *t)) {
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

                sender.send(Event::MessageReceived(ReceivedMessage {
                    sender_id: 1,
                    msg_type: current_message_type,
                    payload: buffer[0..message_length].to_owned() 
                }))
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            },
        }
    }

    Ok(())
}
