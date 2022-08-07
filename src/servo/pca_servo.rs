use i2c_linux;

const I2C_DEVICE : &str = "/dev/i2c-1";

use super::{Servo, Error};

pub struct Pca9685Servo;

impl Pca9685Servo {
    pub fn new () -> Result<Self, Error> {
        let mut i2c = i2c_linux::I2c::from_path(I2C_DEVICE)
            .map_err(|_| Error::DeviceNotAvailable)?;
        
        println!("Servo initialized successfully");
        
        Ok(Pca9685Servo {})
    }
}

impl Servo for Pca9685Servo {
    fn rotate(&mut self, dx: i8, dy: i8) {

    }
}