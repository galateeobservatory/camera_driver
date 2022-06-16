use anyhow::{anyhow, Result};
use rppal::i2c::I2c;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to create I2C object")]
    I2CCreationError,
    #[error("Failed to set slave I2C address {i2c_address:?}")]
    I2CSetSlaveAddressError { i2c_address: u16 },
    #[error("Failed to read I2C device {i2c_address:?}")]
    I2CReadingError { i2c_address: u16 },
    #[error("Failed to write I2C device {i2c_address:?}")]
    I2CWritingError { i2c_address: u16 },
}

#[derive(Debug)]
pub struct Hyt221 {
    i2c_device: I2c,
    i2c_address: u16,
}

impl Hyt221 {
    pub fn new(i2c_address: u16) -> Result<Self, anyhow::Error> {
        let mut i2c = I2c::new().map_err(|_| Error::I2CCreationError)?;
        i2c.set_slave_address(i2c_address)
            .map_err(|_| anyhow!("Failed to set slave I2C address {}", i2c_address))?;
        Ok(Hyt221 {
            i2c_device: i2c,
            i2c_address,
        })
    }

    pub fn read(&mut self) -> Result<(f32, f32), anyhow::Error> {
        let mut buf = [0u8; 4];
        self.i2c_device
            .write(&[0x00])
            .map_err(|_| anyhow!("Failed to write I2C device {}", self.i2c_address))?; // Write 0 to trigger refresh
        std::thread::sleep(std::time::Duration::from_millis(60)); // Wait until temperature and humidity are updated
        self.i2c_device
            .read(&mut buf)
            .map_err(|_| anyhow!("Failed to read I2C device {}", self.i2c_address))?;
        let humidity: u16 = ((buf[0] as u16 & 0x3f) << 8) | buf[1] as u16;
        let temperature: u16 = ((buf[2] as u16) << 8) | buf[3] as u16 & 0xfc;
        let humidity: f32 = (humidity as f32) * (100.0 / 0x3fff as f32);
        let temperature: f32 = (temperature as f32) * (165.0 / 0xfffc as f32) - 40.0;
        Ok((humidity, temperature))
    }
}
