use std::path::PathBuf;

use xdg::{BaseDirectories, BaseDirectoriesError};

pub struct Fs {
    xdg: BaseDirectories,
}

impl Fs {
    pub fn new() -> Result<Self, BaseDirectoriesError> {
        let xdg = BaseDirectories::with_prefix("eye")?;
        Ok(Fs { xdg })
    }

    pub fn camera_pid_file(&self) -> std::io::Result<PathBuf> {
        self.xdg.place_runtime_file("motion.pid")
    }

    pub fn camera_config_file(&self) ->std::io::Result<PathBuf> {
        self.xdg.place_config_file("motion.conf")
    }
}