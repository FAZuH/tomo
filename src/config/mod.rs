pub mod pomodoro;

use std::fs;
use std::path::PathBuf;

use log::debug;
use log::info;
use serde::Deserialize;
use serde::Serialize;

use crate::config::pomodoro::PomodoroConfig;
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub pomodoro: PomodoroConfig,
    pub logs_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let conf_dir = utils::conf_dir();
        let logs_path = conf_dir.join("logs");
        Self {
            pomodoro: Default::default(),
            logs_path,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let conf_dir = utils::conf_dir();
        debug!("Config directory: {:?}", conf_dir);
        if !conf_dir.exists() {
            fs::create_dir_all(&conf_dir)?;
            info!("Created config directory at {conf_dir:?}");
        }
        let conf_path = conf_dir.join("config.yaml");
        if !conf_path.exists() {
            let config = Config::default();
            let file = fs::File::create(&conf_path)?;
            serde_yml::to_writer(&file, &config)?;
            info!("Default config written to {:?}", conf_path);
            Ok(config)
        } else {
            let file = fs::File::open(&conf_path)?;
            let config: Config = serde_yml::from_reader(&file)?;
            info!("Configuration loaded successfully");
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let conf_dir = utils::conf_dir();
        let conf_path = conf_dir.join("config.yaml");
        let file = fs::File::create(&conf_path)?;
        serde_yml::to_writer(&file, self)?;
        info!("Configuration saved successfully");
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Percentage(f32);

impl Percentage {
    pub fn new(perc: f32) -> Self {
        let mut _self = Self::muted();
        _self.set_clamp(perc);
        _self
    }

    pub fn set(&mut self, perc: f32) {
        self.0 = perc
    }

    pub fn set_clamp(&mut self, perc: f32) {
        self.0 = perc.clamp(0.0, 1.0)
    }

    pub fn muted() -> Self {
        Self(0.0)
    }

    pub fn half() -> Self {
        Self(0.5)
    }

    pub fn full() -> Self {
        Self(1.0)
    }

    pub fn volume(&self) -> f32 {
        self.0
    }
}

impl Default for Percentage {
    fn default() -> Self {
        Self::half()
    }
}

impl TryFrom<&str> for Percentage {
    type Error = std::num::ParseIntError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let f: i32 = value.to_string().parse()?;
        Ok(Percentage::new(f as f32 / 100.0))
    }
}

impl std::fmt::Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}%", self.0 * 100.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_yml::Error),
}
