use std::{process::{Command, Stdio}, io::BufRead};
use std::io::{Result, Error, ErrorKind};

use super::Camera;
use crate::fs;

pub struct MotionCamera {
    fs: fs::Fs,
    port: u16,
}

impl MotionCamera {
    pub fn new (fs: fs::Fs) -> Result<Self> {
        let port = Self::get_port(&fs);
        port.and_then(|p| {
            println!("Camera port {}", p);
            Ok(MotionCamera {fs, port: p})
        })
    }

    pub fn is_available() -> bool {
        Command::new("motion")
            .arg("-h")
            .stdout(Stdio::null())
            .status()
            .is_ok()
    }

    fn get_port(fs: &fs::Fs) -> Result<u16> {
        let file_path = fs.camera_config_file()?;
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        for l in reader.lines() {
            let line = l?;
            if line.starts_with("stream_port")  {
                if let Some(port_string) = line.split_whitespace().nth(1) {
                    return port_string.parse::<u16>().map_err(|e|
                        Error::new(ErrorKind::InvalidData, format!("Failed to parse port number from the config file.\n{}", e))
                    )
                }
            }
        }

        Err(Error::new(ErrorKind::NotFound, "Failed to find camera port setting in config file"))
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
            .arg("-INT")
            .arg(pid.trim())
            .spawn()
            .map(|_| ())
    }

    fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for MotionCamera {
    fn drop(&mut self) {
        self.stop();
    }
}