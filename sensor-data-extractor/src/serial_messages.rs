use anyhow::{anyhow, Result};
use link_lib::{BusMessage, Request, Response};
use link_lib::{MessageBuffer, MAX_RESPONSE_SIZE};
use std::mem;
use std::result::Result::Ok;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialStream;

pub async fn get_reponse(port: &mut SerialStream, request: Request, correct_response: Response) -> Result<Response> {
    write_request(port, request).await?;
    let response = read_response(port).await?;

    if mem::discriminant(&response) == mem::discriminant(&correct_response) {
        return Ok(response);
    } else {
        return Err(anyhow!("Expected response : {:?}, got : {:?}", correct_response, response));
    }
}

async fn write_request(port: &mut SerialStream, request: Request) -> Result<()> {
    let request_bytes = request.encode_message().map_err(|_| anyhow!("Could not encode request"))?;

    port.read_u8().await?;

    port.write_all(&request_bytes)
        .await
        .map_err(|_| anyhow!("Could not send request"))?;

    Ok(())
}

async fn read_response(port: &mut SerialStream) -> Result<Response> {
    let mut buff = MessageBuffer::<MAX_RESPONSE_SIZE>::new();
    let mut response: Option<Response> = None;

    for _i in 0..MAX_RESPONSE_SIZE {
        let byte = match port.read_u8().await {
            Ok(b) => b,
            Err(_) => return Err(anyhow!("Error when reading serial port")),
        };
        match buff.add_byte_and_check_for_response(&byte) {
            Ok(option) => match option {
                Some(_) => {
                    response = option;
                    break;
                }
                None => (),
            },
            Err(_) => return Err(anyhow!("Error when decoding response")),
        }
    }

    match response {
        Some(r) => Ok(r),
        None => Err(anyhow!("No response received")),
    }
}
