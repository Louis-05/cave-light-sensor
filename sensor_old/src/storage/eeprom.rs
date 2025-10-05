use eeprom24x::{addr_size::TwoBytes, page_size::B64, Eeprom24x};
use embedded_hal::blocking::i2c::{Write, WriteRead};
use nb::block;
//use rtt_target::rprintln;

pub struct MyEeprom<I2C> {
    eeprom: Eeprom24x<I2C, B64, TwoBytes, eeprom24x::unique_serial::No>,
}

impl<I2C, E> MyEeprom<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        let eeprom = Eeprom24x::new_24x256(i2c, eeprom24x::SlaveAddr::default());
        Self { eeprom }
    }

    pub fn read_data_from_eeprom(&self, address: u32, data: &mut [u8]) -> Result<(), ()> {
        self.eeprom.read_data(address, data).map_err(|_| ())?;
        Ok(())
    }

    pub fn write_data_to_eeprom_blocking(&self, address: u32, data: &[u8]) -> Result<(), ()> {
        for (offset, byte) in data.iter().enumerate() {
            match self.write_byte_to_eeprom_blocking(address + offset as u32, *byte) {
                Ok(_) => (),
                Err(_) => return Err(()),
            }
        }
        return Ok(());
    }

    pub fn write_byte_to_eeprom_blocking(&self, address: u32, byte: u8) -> Result<(), ()> {
        block!(match self.eeprom.write_byte(address, byte) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                eeprom24x::Error::I2C(_) => Err(nb::Error::WouldBlock),
                eeprom24x::Error::TooMuchData => Err(nb::Error::Other(())),
                eeprom24x::Error::InvalidAddr => Err(nb::Error::Other(())),
            },
        })
    }
}
