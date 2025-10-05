use core::marker::PhantomData;

use heapless::Vec;
use postcard::{accumulator::CobsAccumulator, experimental::max_size::MaxSize};
use crate::error::MyError;

pub struct Link<S,INPUT,OUTPUT,const SIZE : usize> {
    serial: S,
    buffer : CobsAccumulator<SIZE>,
    phantom_req: PhantomData<INPUT>,
    phantom_resp: PhantomData<OUTPUT>
}

impl<'a, S,INPUT,OUTPUT,const SIZE : usize> Link<S,INPUT,OUTPUT,SIZE>
where
    S: embedded_hal::serial::Read<u8> + embedded_hal::blocking::serial::Write<u8>,
    INPUT :  for<'de> serde::Deserialize<'de> + MaxSize,
    OUTPUT : serde::Serialize
{
    pub fn new(serial: S) -> Self {
        Self {
            serial : serial,
            buffer : CobsAccumulator::new(),
            phantom_req : PhantomData,
            phantom_resp: PhantomData
        }
    }

    pub fn send_response(&mut self,response : &OUTPUT) -> Result<(), MyError<<S as embedded_hal::blocking::serial::Write<u8>>::Error>> {
        let vec: Vec<u8, SIZE> = postcard::to_vec_cobs(&response).map_err(|_|MyError::Serialize)?;

        self.serial.bwrite_all(&vec).map_err(|e|MyError::IO(e))?;
        self.serial.bflush().map_err(|e|MyError::IO(e))?;
        Ok(())
    }

    pub fn get_request(&mut self) -> Result<Option<INPUT>, MyError<<S as embedded_hal::serial::Read<u8>>::Error>> {
        let byte = nb::block!(self.serial.read()).map_err(|e|MyError::IO(e))?;

       match self.buffer.feed::<INPUT>(&[byte]) {
            postcard::accumulator::FeedResult::Consumed => Ok(None),
            postcard::accumulator::FeedResult::OverFull(_) => Err(MyError::BufferFull),
            postcard::accumulator::FeedResult::DeserError(_) => Err(MyError::Deserialize),
            postcard::accumulator::FeedResult::Success { data, remaining: _ } => Ok(Some(data)),
        }
    }
}
