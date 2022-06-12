use tokio::io::{AsyncWrite, AsyncWriteExt};
use std::io::Cursor;


pub async fn send_message<Msg, W>(writer: &mut W, message: Msg) -> Result<(), std::io::Error> 
    where Msg: prost::Message, W: AsyncWrite + std::marker::Unpin {

    let mut encoded_data = Cursor::new(message.encode_to_vec());
    writer.write_all_buf(&mut encoded_data).await?;

    Ok(())
}
