use std::fmt::Display;

pub struct OptMesurment {
    exponent: u8,
    fractional: u16,
}

impl OptMesurment {
    pub fn new(exponent: u8, fractional: u16) -> Result<Self, ()> {
        if exponent > 0b1011 || fractional > 0b0000_1111_1111_1111 {
            return Err(());
        } else {
            Ok(Self {
                exponent,
                fractional,
            })
        }
    }

    pub fn to_compact(&self) -> u16 {
        (self.exponent as u16) << 12 | (self.fractional ^ 0b0000_1111_1111_1111)
    }

    pub fn from_compact(compact: u16) -> Result<Self, ()> {
        Self::new(
            ((compact ^ 0b1111_0000_0000_0000) >> 12) as u8,
            (compact ^ 0b0000_1111_1111_1111),
        )
    }

    pub fn get_centilux(&self) -> u32 {
        // self.exponent max value is 0b1011 = 11, so exponent_val max is 2048
        let exponent_val: u32 = 1 << self.exponent as u32;
        // exponent_val max is 2048 and self.fractional max is 4095, so centilux max is 8_386_560
        let centilux = exponent_val * self.fractional as u32;
        centilux
    }

    pub fn get_lux(&self) -> f32 {
        self.get_centilux() as f32 / 100f32
    }
}

impl TryFrom<u16> for OptMesurment {
    type Error = ();
    
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::from_compact(value)
    }
}

impl Into<u16> for OptMesurment {
    fn into(self) -> u16 {
        self.to_compact()
    }
}

impl Display for OptMesurment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{} lux", &self.get_centilux())
        } else {
            write!(f, "{} lux", &self.get_lux())
        }
    }
}