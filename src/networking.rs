use tokio::io::{AsyncWrite, AsyncWriteExt};
use std::io::Cursor;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MessageType {
    HelloRequest,
    HelloResponse,
    ServoRotateRequest
}

thread_local! {
    static MESSAGES_LOOKUP: Vec<MessageType> = vec![
        MessageType::HelloRequest,
        MessageType::HelloResponse,
        MessageType::ServoRotateRequest
    ];
}

pub async fn send_message<Msg, W>(writer: &mut W, msg_type: MessageType, message: Msg) -> Result<(), std::io::Error> 
    where Msg: prost::Message, W: AsyncWrite + std::marker::Unpin {

    if let Some(id) = msg_id_from_type(msg_type) {
        let mut cursor = Cursor::new(vec![]);
        cursor.write(&u32::to_le_bytes(id)).await?;
        cursor.write(&u32::to_le_bytes(message.encoded_len() as u32)).await?;
        cursor.write_all(&mut message.encode_to_vec()).await?;

        writer.write_all(&mut cursor.into_inner()).await?;
    }

    Ok(())
}

pub fn msg_type_from_id(message_type_id: u32) -> Option<MessageType> {
    MESSAGES_LOOKUP.with(|types| types.get(message_type_id as usize).map(|t| *t))
}

fn msg_id_from_type(message_type: MessageType) -> Option<u32> {
    MESSAGES_LOOKUP.with(|types| types
        .iter()
        .enumerate()
        .find(|(_, msg_type)| **msg_type == message_type)
        .map(|(ind, _)| ind as u32)
    )
}