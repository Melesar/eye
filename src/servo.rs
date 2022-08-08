use std::fmt::Display;

#[cfg(feature = "servo")]
mod pca_servo;

#[cfg(feature = "servo")]
use pca_servo::Pca9685Servo;

#[derive(Debug)]
pub enum Error {
    ServoNotEnabled,
    DeviceNotAvailable,
    CommunicationFailure
}

pub trait Servo {
    fn rotate(&mut self, dx: i8, dy: i8);
}

#[cfg(feature = "servo")]
pub fn init() -> Result<Box<dyn Servo>, Error> {
    let servo = Pca9685Servo::new()?;
    Ok(Box::new(servo))
}  

#[cfg(not(feature = "servo"))]
pub fn init() -> Result<Box<dyn Servo>, Error> {
    Err(Error::ServoNotEnabled)
}

impl Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let message = match self {
            Error::ServoNotEnabled => "Servo feature is not enabled",
            Error::DeviceNotAvailable => "Failed to open servo control device",
            Error::CommunicationFailure => "Error during communication with the servo device"
        };
        write!(formatter, "{}", message)
    }
}