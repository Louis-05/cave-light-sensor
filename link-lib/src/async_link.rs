use core::marker::PhantomData;

use postcard::{accumulator::CobsAccumulator, experimental::max_size::MaxSize};
use crate::error::MyError;


pub struct Link<S,REQ,RESP,const SIZE : usize> {
    serial: S,
    buffer : CobsAccumulator<SIZE>,
    phantom_req: PhantomData<REQ>,
    phantom_resp: PhantomData<RESP>
}

impl<'a, S,REQ,RESP,const SIZE : usize> Link<S,REQ,RESP,SIZE>
where
    S:  tokio::io::AsyncReadExt + tokio::io::AsyncWriteExt + std::marker::Unpin,
    REQ :  for<'de> serde::Deserialize<'de> + MaxSize,
    RESP : serde::Serialize
{
    pub fn new(serial: S) -> Self {
        Self {
            serial : serial,
            buffer : CobsAccumulator::new(),
            phantom_req : PhantomData,
            phantom_resp: PhantomData
        }
    }

    pub async fn send_response(&mut self,response : &RESP) -> Result<(), MyError<std::io::Error>> {
        let vec: heapless::Vec<u8, SIZE> = postcard::to_vec_cobs(&response).map_err(|_|MyError::Serialize)?;

        self.serial.write_all(&vec).await.map_err(|e|MyError::IO(e))?;
        self.serial.flush().await.map_err(|e|MyError::IO(e))?;
        Ok(())
    }

    pub async fn get_request(&mut self) -> Result<Option<REQ>, MyError<std::io::Error>> {
        let byte = self.serial.read_u8().await.map_err(|e|MyError::IO(e))?;

        match self.buffer.feed::<REQ>(&[byte]) {
            postcard::accumulator::FeedResult::Consumed => Ok(None),
            postcard::accumulator::FeedResult::OverFull(_) => Err(MyError::BufferFull),
            postcard::accumulator::FeedResult::DeserError(_) => Err(MyError::Deserialize),
            postcard::accumulator::FeedResult::Success { data, remaining: _ } => Ok(Some(data)),
        }
    }
}
