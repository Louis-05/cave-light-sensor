use serde::{Serialize,Deserialize};
use postcard::experimental::max_size::MaxSize;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq,Clone, Copy,MaxSize)]
pub enum Event {
    High(i64),
    Low(i64),
    Error(i64),
    PwrOn(i64),
}
// Compact u32 format :
// Timestamps have 1s resolution
// Timestamps start date are 1st of january , 2024
// Timestamps are encoded in the last 30 bits and will not overflow for 34 years (January 9th ,
// 2058) The first two bits encode the type of event
const EVENT_MASK: u32 = 0xC000_0000;

const LOW_EVENT: u32 = 0x0000_0000;
const HIGH_EVENT: u32 = 0x4000_0000;
const ERR_EVENT: u32 = 0x8000_0000;
const PWR_ON_EVENT: u32 = 0xC000_0000;

const TS_OFFSET_FROM_UNIX: i64 = 1704067200;

impl Event {
    pub fn from_compact_u32(val: u32) -> Result<Event, ()> {
        match EVENT_MASK & val {
            LOW_EVENT => Ok(Event::Low(
                (val & (!EVENT_MASK)) as i64 + TS_OFFSET_FROM_UNIX,
            )),
            HIGH_EVENT => Ok(Event::High(
                (val & (!EVENT_MASK)) as i64 + TS_OFFSET_FROM_UNIX,
            )),
            ERR_EVENT => Ok(Event::Error(
                (val & (!EVENT_MASK)) as i64 + TS_OFFSET_FROM_UNIX,
            )),
            PWR_ON_EVENT => Ok(Event::PwrOn(
                (val & (!EVENT_MASK)) as i64 + TS_OFFSET_FROM_UNIX,
            )),
            _ => Err(()),
        }
    }

    pub fn to_compact_u32(&self) -> u32 {
        match &self {
            Event::Low(t) => LOW_EVENT | (((*t - TS_OFFSET_FROM_UNIX) as u32) & (!EVENT_MASK)),
            Event::High(t) => HIGH_EVENT | (((*t - TS_OFFSET_FROM_UNIX) as u32) & (!EVENT_MASK)),
            Event::Error(t) => ERR_EVENT | (((*t - TS_OFFSET_FROM_UNIX) as u32) & (!EVENT_MASK)),
            Event::PwrOn(t) => PWR_ON_EVENT | (((*t - TS_OFFSET_FROM_UNIX) as u32) & (!EVENT_MASK)),
        }
    }
}
