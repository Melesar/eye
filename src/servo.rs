use std::{fmt::Display, time::Duration};
use tokio::sync::mpsc::{self, Sender, Receiver};

#[cfg(feature = "servo")]
mod pca_servo;
mod test_servo;

#[cfg(feature = "servo")]
use pca_servo::Pca9685Servo;

const SERVO_ROTATION_INTERVAL: u64 = 500;

#[derive(Debug)]
pub enum Error {
    ServoNotEnabled,
    DeviceNotAvailable,
    CommunicationFailure,
}

trait ServoImpl {
    fn rotate(&mut self, dx: i8, dy: i8);
}

struct ServoControl {
    pub dx: i8,
    pub dy: i8,
}

pub struct Servo {
    sender: Sender<ServoControl>,
}

#[cfg(feature = "servo")]
pub fn init() -> Result<Servo, Error> {
    // let r#impl = test_servo::TestServo;
    let r#impl = Pca9685Servo::new()?;
    Servo::new(r#impl)
}

#[cfg(not(feature = "servo"))]
pub fn init() -> Result<Servo, Error> {
    Err(Error::ServoNotEnabled)
}

impl Servo {
    fn new<T>(servo_impl: T) -> Result<Self, Error> where T: ServoImpl + Send + 'static {
        let (sender, receiver) = mpsc::channel(1);
        tokio::spawn(async move {
            if let Err(e) = servo_control_routine(servo_impl, receiver).await {
                eprintln!("Servo has failed: {}", e);
            }
        });
        Ok(Servo { sender })
    }

    pub fn rotate(&mut self, dx: i8, dy: i8) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            sender.send(ServoControl { dx, dy }).await.unwrap_or_default();
        });
    }
}

async fn servo_control_routine<T>(mut servo_impl: T, mut receiver: Receiver<ServoControl>) -> Result<(), Error> where T: ServoImpl + Send{
    while let Some(mut ctl) = receiver.recv().await {
        while !ctl.should_stop() {
            servo_impl.rotate(ctl.dx, ctl.dy);

            tokio::time::sleep(Duration::from_millis(SERVO_ROTATION_INTERVAL)).await;

            ctl = receiver.try_recv().unwrap_or(ctl);
        }
    }

    Ok(())
}

impl ServoControl {
    fn should_stop(&self) -> bool {
        self.dx == 0 && self.dy == 0
    }
}

impl Display for Error {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        let message = match self {
            Error::ServoNotEnabled => "Servo feature is not enabled",
            Error::DeviceNotAvailable => "Failed to open servo control device",
            Error::CommunicationFailure => "Error during communication with the servo device",
        };
        write!(formatter, "{}", message)
    }
}
