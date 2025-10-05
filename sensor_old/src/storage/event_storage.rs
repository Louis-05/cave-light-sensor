use common_data::event::Event;
use embedded_hal::blocking::i2c::{Write, WriteRead};
use rtt_target::debug_rprintln;

use super::eeprom::MyEeprom;

pub struct EventStorage<I2C> {
    storage: MyEeprom<I2C>,
}

const DEVICE_ID_ADDR: u32 = 0u32;
const NUM_OF_EVENT_ADDR: u32 = 8u32;
const EVENT_ADDR: u32 = 12u32;

const EVENT_SIZE: u32 = 4u32;

impl<I2C, E> EventStorage<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        let storage = MyEeprom::new(i2c);
        Self { storage }
    }

    pub fn get_event(&self, event_number: u32) -> Result<Option<Event>, ()> {
        if self.get_number_of_event()? < event_number {
            return Ok(None);
        }
        let mut data: [u8; 4] = [0u8; 4];
        self.storage
            .read_data_from_eeprom(EVENT_ADDR + EVENT_SIZE * event_number, &mut data)?;
        Ok(Some(Event::from_compact_u32(u32::from_be_bytes(data))?))
    }

    pub fn get_last_event(&self) -> Result<Option<Event>, ()> {
        self.get_event_from_end(0)
    }

    pub fn get_event_from_end(&self, event_number: u32) -> Result<Option<Event>, ()> {
        let number_of_event = self.get_number_of_event()?;
        if number_of_event < event_number {
            return Ok(None);
        }
        let mut data: [u8; 4] = [0u8; 4];
        self.storage.read_data_from_eeprom(
            EVENT_ADDR + EVENT_SIZE * ((number_of_event - 1) - event_number),
            &mut data,
        )?;
        Ok(Some(Event::from_compact_u32(u32::from_be_bytes(data))?))
    }

    fn set_event(&self, event: Event, number: u32) -> Result<(), ()> {
        debug_rprintln!("Setting event n°{}", number);
        let data: [u8; 4] = event.to_compact_u32().to_be_bytes();
        self.storage
            .write_data_to_eeprom_blocking(EVENT_ADDR + EVENT_SIZE * number, &data)
    }

    /*pub fn set_last_event(&self, event: Event) -> Result<(), ()> {
        let number = self.get_number_of_event()?;
        self.set_event(event, number - 1)
    }*/

    pub fn add_new_event(&self, event: Event) -> Result<(), ()> {
        let number = self.get_number_of_event()?;
        self.set_event(event, number)?;
        self.increment_number_of_event()
    }

    pub fn add_or_overwrite_event(&self, event: Event, overwrite_delay: u32) -> Result<(), ()> {
        let Some(Event::High(high_event_ts)) = self.get_last_event()? else {
            return self.add_new_event(event)
        };
        let Some(Event::Low(last_low_event_ts)) = self.get_last_event()? else {
            return self.add_new_event(event)
        };
        if high_event_ts - last_low_event_ts > overwrite_delay as i64 {
            return self.add_new_event(event);
        }

        // To fuse two events into one , we just need to delete the low event at the end of the
        // first one and the high event at the start of the second one. Since here we didn't event
        // store the second one , we only need to remove the first one
        self.remove_last_event()
    }

    pub fn remove_last_event(&self) -> Result<(), ()> {
        let number = self.get_number_of_event()?;
        self.set_number_of_event(number - 1)
    }

    pub fn clear_event_storage(&self) -> Result<(), ()> {
        debug_rprintln!("Clearing storage");
        self.set_number_of_event(0)
    }

    pub fn get_number_of_event(&self) -> Result<u32, ()> {
        let mut data = [0u8; 4];
        self.storage
            .read_data_from_eeprom(NUM_OF_EVENT_ADDR, &mut data)?;
        Ok(u32::from_be_bytes(data))
    }

    fn set_number_of_event(&self, number: u32) -> Result<(), ()> {
        debug_rprintln!("New number of event : {}", number);
        let data: [u8; 4] = number.to_be_bytes();
        self.storage
            .write_data_to_eeprom_blocking(NUM_OF_EVENT_ADDR, &data)
    }

    fn increment_number_of_event(&mut self) -> Result<(), ()> {
        let number = self.get_number_of_event()?;
        self.set_number_of_event(number + 1)
    }

    pub fn set_device_id(&self, id: u64) -> Result<(), ()> {
        let data: [u8; 8] = id.to_be_bytes();
        self.storage
            .write_data_to_eeprom_blocking(DEVICE_ID_ADDR, &data)
    }

    pub fn get_device_id(&self) -> Result<u64, ()> {
        let mut data = [0u8; 8];
        self.storage
            .read_data_from_eeprom(DEVICE_ID_ADDR, &mut data)?;
        Ok(u64::from_be_bytes(data))
    }
}
