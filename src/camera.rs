use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Result;

pub const CAMERA_PORT : u16 = 8081;

#[cfg(feature = "camera")]
pub fn is_available() -> bool {
    Command::new("motion")
        .arg("-h")
        .stdout(Stdio::null())
        .status()
        .is_ok()
}

#[cfg(feature = "camera")]
pub fn is_active() -> bool {
    false
}

#[cfg(feature = "camera")]
pub fn start() -> Result<()> {
    Command::new("motion")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(feature = "camera")]
pub fn stop() {

}

#[cfg(not(feature = "camera"))]
pub fn is_available() -> bool {
    true
}

#[cfg(not(feature = "camera"))]
pub fn is_active() -> bool {
    get_lock_path().exists()
}

#[cfg(not(feature = "camera"))]
pub fn start() -> Result<()> {
    let path = get_lock_path();
    if !path.exists() {
        std::fs::File::create(path)?;
    }

    Ok(())
}

#[cfg(not(feature = "camera"))]
pub fn stop() {
    let path = get_lock_path();
    if path.exists() {
        std::fs::remove_file(path);
    }
}

pub fn get_lock_path() -> &'static Path {
    Path::new("camera_active")
}
