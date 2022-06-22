mod motion_camera;
mod fake_camera;

use std::io::Result;

use motion_camera::MotionCamera;
use fake_camera::FakeCamera;

use crate::fs::Fs;

pub trait Camera {
    fn is_active(&self) -> bool;
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn port(&self) -> u16;
}

pub fn init_camera(fs: Fs) -> Box<dyn Camera> {
    if MotionCamera::is_available() {
        Box::new(MotionCamera::new(fs))
    } else {
        Box::new(FakeCamera{})
    }
}