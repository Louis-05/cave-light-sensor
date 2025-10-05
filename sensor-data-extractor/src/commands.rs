use anyhow::{anyhow, Result};
use link_lib::{Request, Response};
use std::path::PathBuf;
use std::result::Result::Ok;
use tokio::sync::mpsc::Sender;
use tokio_serial::SerialStream;

use crate::serial_messages::get_reponse;

#[derive(PartialEq,Clone)]
pub enum Command {
    Ping,
    SetSensorId(u64),
    GetEverything,
    GetEverythingAndSave(PathBuf),
    ResetSensor,
    GetTime,
    SyncTime,
}

pub async fn ping(port: &mut SerialStream, log: &Sender<String>) -> Result<()> {
    log.send("Ping...\n".to_string()).await?;
    match get_reponse(port, Request::Ping, Response::Ok).await {
        Ok(_) => log.send("Pong !\n".to_string()).await?,
        Err(e) => {
            log.send(format!("Error : {}", e));
            return Err(anyhow!(""));
        }
    }

    Ok(())
}


