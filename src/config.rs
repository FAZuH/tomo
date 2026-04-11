use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

use crate::debug;
use crate::info;
use crate::utils;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub pomodoro: PomodoroConfig,
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
        let file = fs::File::open(&conf_path)?;
        serde_yml::to_writer(&file, self)?;
        info!("Configuration saved successfully");
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PomodoroConfig {
    pub timer: PomodoroTimerConfig,
    pub hook: PomodoroHookConfig,
    pub sound: PomodoroSoundConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PomodoroTimerConfig {
    pub focus: Duration,
    pub short: Duration,
    pub long: Duration,

    pub long_interval: u32,

    pub auto_focus: bool,
    pub auto_short: bool,
    pub auto_long: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PomodoroHookConfig {
    pub focus: Vec<String>,
    pub short: Vec<String>,
    pub long: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PomodoroSoundConfig {
    pub focus: Option<PathBuf>,
    pub short: Option<PathBuf>,
    pub long: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_yml::Error),
}
