mod lib {
    include!(concat!(env!("OUT_DIR"), "/servo.rs"));
}

pub trait Servo {
    fn rotate(&self, dx: i8, dy: i8);
}

pub fn init() -> Box<dyn Servo> {
    Box::new(PcaServo::new())
}

pub struct PcaServo;

impl PcaServo {
    pub fn new() -> Self {
        unsafe {
            lib::PCA9685_init(lib::I2C_ADDR as u8);
            lib::PCA9685_setPWMFreq(60_f32);  // Analog servos run at ~60 Hz updates
        }
        PcaServo{}
    }
}

impl Servo for PcaServo {

    fn rotate(&self, dx: i8, dy: i8) { 
        if dx >= 0 {
            unsafe { lib::ServoDegreeIncrease(lib::SERVO_DOWN_CH as u8, dx as u8); }
        } else if dx < 0 {
            unsafe { lib::ServoDegreeDecrease(lib::SERVO_DOWN_CH as u8, (-dx) as u8); }
        } else if dy >= 0 {
            unsafe { lib::ServoDegreeIncrease(lib::SERVO_UP_CH as u8, dy as u8); }
        } else {
            unsafe { lib::ServoDegreeDecrease(lib::SERVO_UP_CH as u8, (-dy) as u8); }
        }
        
    }
}


