use std::{process::{Command, Stdio}, io::BufRead};

use super::Camera;
use crate::fs;

const DEFAULT_CAMERA_PORT : u16 = 8081;

pub struct MotionCamera {
    fs: fs::Fs,
}

impl MotionCamera {
    pub fn new (fs: fs::Fs) -> Self {
        MotionCamera { fs }
    }

    pub fn is_available() -> bool {
        Command::new("motion")
            .arg("-h")
            .stdout(Stdio::null())
            .status()
            .is_ok()
    }

    fn get_port(&self) -> Option<u16> {
        let file_path = self.fs.camera_config_file().ok()?;
        let file = std::fs::File::open(file_path).ok()?;
        let reader = std::io::BufReader::new(file);
        for l in reader.lines() {
            let line = l.ok()?;
            if line.starts_with("stream_port")  {
                return line.split_whitespace()
                .nth(1)
                .map(|s| s.parse::<u16>().unwrap_or(DEFAULT_CAMERA_PORT));
            }
        }

        None
    }
}

impl Camera for MotionCamera {
    fn is_active(&self) -> bool {
        self.fs.camera_pid_file().map_or(false, |path| path.exists())
    }

    fn start(&self) -> std::io::Result<()> {
        let config_file = self.fs.camera_config_file()?;
        let pid_file = self.fs.camera_pid_file()?;

        Command::new("motion")
            .arg("-b")
            .arg("-c").arg(config_file.into_os_string())
            .arg("-p").arg(pid_file.into_os_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
    }

    fn stop(&self) -> std::io::Result<()> {
        let pid_file = self.fs.camera_pid_file()?;
        let pid = std::fs::read_to_string(pid_file)?;

        Command::new("kill")
            .arg(pid.trim())
            .spawn()
            .map(|_| ())
    }

    fn port(&self) -> u16 {
        self.get_port().unwrap_or(DEFAULT_CAMERA_PORT)
    }
}

impl Drop for MotionCamera {
    fn drop(&mut self) {
        self.stop();
        if let Ok(path) = self.fs.camera_pid_file() {
            std::fs::remove_file(path);
        }
    }
}