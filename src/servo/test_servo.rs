use super::ServoImpl;

pub struct TestServo;

impl ServoImpl for TestServo {
    fn rotate(&mut self, dx: i8, dy: i8) {
        println!("Rotating servo ({}, {})", dx, dy);
    }
}

unsafe impl Send for TestServo {}
