use std::path::Path;

use super::Camera;


pub struct FakeCamera;

impl FakeCamera {
    fn get_lock_path() -> &'static Path {
        Path::new("camera_active")
    }
}

impl Camera for FakeCamera {
    fn is_active(&self) -> bool {
        Self::get_lock_path().exists()
    }

    fn port(&self) -> u16 {
        8981
    }

    fn start(&self) -> std::io::Result<()> {
        let path = Self::get_lock_path();
        if !path.exists() {
            std::fs::File::create(path)?;
        }

        Ok(())
    }

    fn stop(&self) -> std::io::Result<()> {
        let path = Self::get_lock_path();
        if path.exists() {
            std::fs::remove_file(path)
        } else {
            Ok(())
        }
    }
}