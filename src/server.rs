use prost::Message;
use std::collections::HashMap;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::Sender;

use crate::{camera, servo, networking};
use networking::MessageType;

use self::messages::{HelloRequest, ServoRotateRequest};

macro_rules! on_message {
    ($t:expr, $s:expr, {$($p:ident => $f:ident),+}) => {
        match $t.msg_type {
            $(
                MessageType::$p => {
                    let sender_id = $t.sender_id;
                    let mut cursor = std::io::Cursor::new($t.payload);
                    if let Ok(request) = $p::decode(&mut cursor) {
                        $f(sender_id, request, $s).await;
                    }
                },
            )+
            _ => {}
        }
    };
}

pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}

mod features {
    pub const CAMERA: u32 = 1 << 0;
    pub const SERVO: u32 = 1 << 1; 
}

#[derive(Debug, Clone)]
enum Event {
    Connected,
    Disconnected,
    MessageReceived(ReceivedMessage)
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
    camera: Box<dyn camera::Camera>,
    servo: Option<Box<dyn servo::Servo>>,
    client_connections: HashMap<u32, OwnedWriteHalf>
}

impl Server {

    pub fn new(camera: Box<dyn camera::Camera>, servo: Option<Box<dyn servo::Servo>>) -> Self {
        Server { camera, servo, client_connections: HashMap::new() }
    }
    
    pub async fn start(mut self) -> Result<(), std::io::Error> {
        println!("Features: {}", get_feature_set());
        let listener = tokio::net::TcpListener::bind("0.0.0.0:6688").await.unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let mut currently_connected = 0_u32;
        let mut current_client_id = 0_u32;

        loop {
            tokio::select! {
                accept_result = listener.accept() => if let Ok((stream, _)) = accept_result {
                    let (reader, writer) = stream.into_split();
                    self.client_connections.insert(current_client_id, writer);
                    
                    let sender = tx.clone();
                    tokio::spawn(async move { handle_client_connection(reader, sender, current_client_id).await });
                    current_client_id += 1;
                },

                receive_result = rx.recv() => match receive_result {
                    Some(Event::Connected) => {
                        println!("Client connected");
                        if currently_connected == 0 && !self.camera.is_active() {
                            println!("Enabling camera");
                            if let Err(e) = self.camera.start() {
                                eprintln!("Failed to start camera: {}", e);
                            };
                        } 

                        currently_connected += 1;
                    },
                    Some(Event::Disconnected) => {
                        println!("Client disconnected");
                        if currently_connected == 1 && self.camera.is_active() {
                            println!("Disabling camera");
                            if let Err(e) = self.camera.stop() {
                                eprintln!("Failed to stop camera: {}", e);
                            }
                            currently_connected = 0;
                        } else if currently_connected != 0 {
                            currently_connected -= 1;
                        }
                    },
                    Some(Event::MessageReceived(message_data)) => {
                        println!("Received {:?} from {}", message_data.msg_type, message_data.sender_id);
                        on_message!(message_data, &mut self, {
                            HelloRequest => on_hello_request,
                            ServoRotateRequest => on_servo_rotate_request
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn on_hello_request(sender_id: u32, _request: HelloRequest, server: &mut Server) {
    if let Some(connection) = server.client_connections.get_mut(&sender_id) {
        let camera_port = server.camera.port();
        let message = messages::HelloResponse { 
            stream_host: crate::get_current_ip_address().to_string(),
            stream_port: camera_port as i32,
            feature_set: get_feature_set(),
        };
        networking::send_message(connection, MessageType::HelloResponse, message).await.unwrap_or_default();
    }
}

async fn on_servo_rotate_request(_sender_id: u32, request: ServoRotateRequest, server: &mut Server) {
    if let Some(s) = server.servo.as_mut() {
        s.rotate(request.dx as i8, request.dy as i8);
    }
}

async fn handle_client_connection<R>(reader: R, sender: Sender<Event>, client_id: u32) -> Result<(), std::io::Error>
    where R: AsyncRead + std::marker::Unpin  {

    let mut reader = tokio::io::BufReader::new(reader);
    let mut current_state = ReadState::MsgType;
    let mut num_buffer = [0_u8; 4];
    let mut buffer : Vec<u8> = Vec::with_capacity(2048);
    let mut current_message_type = MessageType::HelloRequest;
    let mut message_length = 0;

    sender.send(Event::Connected).await.unwrap_or_default();

    loop {
        match current_state {
            ReadState::MsgType => {
                let bytes_read = reader.read(&mut num_buffer).await?;
                if disconnect_if_none_is_read(bytes_read, &sender).await { break; }
                else if bytes_read < num_buffer.len() { continue; }

                let message_type_index = u32::from_le_bytes(num_buffer);

                match networking::msg_type_from_id(message_type_index as u32) {
                    Some(tt) => { current_message_type = tt },
                    None => { eprintln!("Unrecognized message type index {}", message_type_index); break; },
                }

                current_state = ReadState::Length;
            },
            ReadState::Length => {
                let bytes_read = reader.read(&mut num_buffer).await?;
                if disconnect_if_none_is_read(bytes_read, &sender).await { break; }
                else if bytes_read < num_buffer.len() { continue; }

                message_length = u32::from_le_bytes(num_buffer) as usize;
                current_state = ReadState::Payload;
            },
            ReadState::Payload => {
                if message_length == 0 {
                    sender.send(Event::MessageReceived(ReceivedMessage {
                        sender_id: client_id,
                        msg_type: current_message_type,
                        payload: buffer.clone()
                    }))
                    .await
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
                    current_state = ReadState::MsgType;
                    continue;
                }

                if buffer.len() < message_length {
                    buffer.resize(message_length, 0);
                }

                let bytes_read = reader.read(&mut buffer[0..message_length]).await?;
                if disconnect_if_none_is_read(bytes_read, &sender).await { break; }
                else if bytes_read < buffer.len() { continue; }

                sender.send(Event::MessageReceived(ReceivedMessage {
                    sender_id: client_id,
                    msg_type: current_message_type,
                    payload: buffer[0..message_length].to_owned() 
                }))
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

                current_state = ReadState::MsgType;
                buffer.clear();
            },
        }

        async fn disconnect_if_none_is_read(bytes_read: usize, sender: &Sender<Event>) -> bool {
            if bytes_read == 0 {
                sender.send(Event::Disconnected).await.unwrap_or_default();
                return true;
            }

            false
        }

    }

    Ok(())
}

fn get_feature_set() -> u32 {
    let mut features = features::CAMERA;

    #[cfg(feature="servo")]
    {
        features |= features::SERVO;
    }

    features
}
