use std::process::{Command, Stdio};
use std::io::Result;

pub const CAMERA_PORT : u16 = 8081;

pub fn is_available() -> bool {
    Command::new("motion")
        .arg("-h")
        .stdout(Stdio::null())
        .status()
        .is_ok()
}

pub fn is_active() -> bool {
    false
}

pub fn start() -> Result<()> {
    Command::new("motion")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

pub fn stop() {

}
