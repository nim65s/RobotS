/// Sensor: AHT20
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, MaxSize, Debug, Eq, PartialEq, Copy, Clone)]
pub struct SensorOk {
    pub h: u32,
    pub t: u32,
}

impl SensorOk {
    #[must_use]
    pub fn rh(self) -> f64 {
        100.0 * f64::from(self.h) / (f64::from(1 << 20))
    }
    #[must_use]
    pub fn celsius(self) -> f64 {
        (200.0 * f64::from(self.t) / (f64::from(1 << 20))) - 50.0
    }
}

#[repr(u8)]
#[derive(Deserialize, Serialize, MaxSize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum SensorErr {
    Uncalibrated,
    Bus,
    CheckSum,
    Uninitialized,
}

#[derive(Deserialize, Serialize, MaxSize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum SensorResult {
    Err(SensorErr),
    Ok(SensorOk),
}
