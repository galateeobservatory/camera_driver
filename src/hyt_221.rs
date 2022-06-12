use rppal::i2c::I2c;

#[derive(Debug)]
pub struct Hyt221 {
    i2c_device: I2c,
}

impl Hyt221 {
    pub fn new(i2c_address: u16) -> Result<Self, &'static str> {
        let mut i2c = I2c::new().map_err(|_| "Failed to create i2c")?;
        i2c.set_slave_address(i2c_address).map_err(|_| "Failed to set i2c slave address")?;
        Ok(Hyt221 {
            i2c_device: i2c,
        })
    }

    pub fn read(&mut self) -> Result<(f32, f32), &'static str> {
        let mut buf = [0u8; 4];
        self.i2c_device.write(&[0x00]).map_err(|_| "Failed to write to i2c")?; // Write 0 to trigger refresh
        std::thread::sleep(std::time::Duration::from_millis(60));
        self.i2c_device.read(&mut buf).map_err(|_| "Failed to read i2c device")?;
        let humidity : u16 = ((buf[0] as u16  & 0x3f) << 8) | buf[1] as u16;
        let temperature : u16 = ((buf[2] as u16) << 8) | buf[3] as u16  & 0xfc;
        let humidity : f32 = (humidity as f32) * (100.0 / 0x3fff as f32);
        let temperature : f32 = (temperature as f32) * (165.0 / 0xfffc as f32) - 40.0;
        Ok((humidity, temperature))
    }
}