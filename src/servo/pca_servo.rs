use super::{Servo, Error};
use tokio::{self, sync::mpsc};

type I2c = i2c_linux::I2c<std::fs::File>;

const I2C_DEVICE : &str = "/dev/i2c-1";
const I2C_ADDRESS : u16 = 0x80 >> 1;

const PCA9685_MODE1 : u8 = 0x0;
const PCA9685_PRESCALE : u8 = 0xFE;
const LED0_ON_L : u8 = 0x6;

const DEFAULT_PWM_FREQUENCY : f32 = 60_f32;

const SERVO_UP_CHANNEL : u8 = 0_u8;
const SERVO_DOWN_CHANNEL : u8 = 1_u8;

struct ServoControl {
    pub x: i8,
    pub y: i8,
}

pub struct Pca9685Servo {
    sender: mpsc::Sender<ServoControl>
}

impl Servo for Pca9685Servo {
    fn rotate(&mut self, dx: i8, dy: i8) {
        let sender = self.sender.clone();
        tokio::spawn(async move { sender.send(ServoControl{x: dx, y: dy}).await });
    }
}

impl Pca9685Servo {
    pub fn new () -> Result<Self, Error> {
        let mut i2c_bus = I2c::from_path(I2C_DEVICE).device_unavailable()?;
        let result = i2c_bus.smbus_set_slave_address(I2C_ADDRESS, false);
        println!("Setting slave address: {:?}", result);
        result.device_unavailable()?;

        let (sender, receiver) = mpsc::channel(15);

        tokio::spawn(async move {
             if let Err(e) = run_servo(i2c_bus, receiver).await {
                eprintln!("Servo failed: {}", e);
             } 
        });


        Ok(Pca9685Servo{ sender })
    }
}

async fn run_servo(mut i2c_bus: I2c, mut receiver: mpsc::Receiver<ServoControl>) -> Result<(), Error> {
    let reset_result  = reset(&mut i2c_bus).await;
    println!("Reset: {:?}", reset_result);

    reset_result.communication_failure()?;

    let frequency_result = set_pwm_frequency(&mut i2c_bus, DEFAULT_PWM_FREQUENCY).await;
    println!("Frequency {:?}", frequency_result);

    frequency_result.communication_failure()?;

    let degree_result = set_servo_degree(&mut i2c_bus, SERVO_UP_CHANNEL, 90).and(set_servo_degree(&mut i2c_bus, SERVO_DOWN_CHANNEL, 90));
    println!("Degree: {:?}", degree_result);

    degree_result.communication_failure()?;

    while let Some(_control) = receiver.recv().await {
        //Rotate the servo
    }

    Ok(())
}

async fn reset(i2c_bus: &mut I2c) -> std::io::Result<()> {
    i2c_bus.smbus_write_byte_data(PCA9685_MODE1, 0x80)?;

    delay(10).await;

    Ok(())
}

async fn set_pwm_frequency(i2c_bus: &mut I2c, mut frequency: f32) -> std::io::Result<()> {
    frequency *= 0.9;  // Correct for overshoot in the frequency setting.

    let mut prescale_value = 25000000_f32;
    prescale_value /= 4096_f32;
    prescale_value /= frequency;
    prescale_value -= 1_f32;

    let prescale = (prescale_value + 0.5).floor() as u8;
    let old_mode = i2c_bus.smbus_read_byte_data(PCA9685_MODE1)?;
    let new_mode = (old_mode & 0x7F) | 0x10; // sleep
    i2c_bus.smbus_write_byte_data(PCA9685_MODE1, new_mode)?;
    i2c_bus.smbus_write_byte_data(PCA9685_PRESCALE, prescale)?;
    i2c_bus.smbus_write_byte_data(PCA9685_MODE1, old_mode)?;

    delay(5).await;

    i2c_bus.smbus_write_byte_data(PCA9685_MODE1, old_mode | 0xA0)?;

    Ok(())
}

fn set_servo_degree(i2c_bus: &mut I2c, channel: u8, mut degree: u8) -> std::io::Result<()> {
    degree = degree.max(0).min(180);

    const PULSE_LENGTH : f64 = 1000.0 / 60.0 / 4096.0;
    
    let pulse = (degree as f64 + 45.0) / (90.0 * 1000.0);
    let pulse = (pulse * 1000.0 / PULSE_LENGTH) as u16;

    i2c_bus.smbus_write_byte_data(LED0_ON_L + 4*channel, 0)?;
    i2c_bus.smbus_write_byte_data(LED0_ON_L + 4*channel+1, 0)?;
    i2c_bus.smbus_write_byte_data(LED0_ON_L + 4*channel+2, pulse as u8)?;
    i2c_bus.smbus_write_byte_data(LED0_ON_L + 4*channel+3, (pulse >> 8) as u8)?;

    Ok(())
}

async fn delay(millis: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(millis)).await
}

trait ServoResult<T> {
    fn communication_failure(self) -> Result<T, Error>;
    fn device_unavailable(self) -> Result<T, Error>;
}

impl<T> ServoResult<T> for std::io::Result<T> {
    fn communication_failure(self) -> Result<T, Error> {
        self.map_err(|_| Error::CommunicationFailure)
    }

    fn device_unavailable(self) -> Result<T, Error> {
        self.map_err(|_| Error::DeviceNotAvailable)
    }
}