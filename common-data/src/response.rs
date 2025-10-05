use serde::{Serialize,Deserialize};
use postcard::experimental::max_size::MaxSize;

use crate::event::Event;

pub const MAX_RESPONSE_SIZE: usize = Response::POSTCARD_MAX_SIZE + 2;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq,Clone, Copy,MaxSize)]
pub enum Response{
    Ok,
    Error,
    NumberOfEvent(u32),
    EventInfo(Event),
    SensorId(u64),
    EmbeddedTime(u32),
    Version(u64)
}