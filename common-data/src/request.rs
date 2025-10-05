use serde::{Serialize,Deserialize};
use postcard::experimental::max_size::MaxSize;


pub const MAX_REQUEST_SIZE: usize = Request::POSTCARD_MAX_SIZE + 2;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq,Clone, Copy,MaxSize)]
pub enum Request{
    Ping,
    GetEmbeddedTime,
    SetEmbeddedTime(u32),
    GetSensorId,
    SetSensorId(u64),
    ClearMemory,
    GetNumberOfEvent,
    GetVersion,
    GetEvent(u32),
}