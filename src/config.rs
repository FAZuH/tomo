use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PomodoroTimerConfig {
    #[serde(with = "duration_as_secs")]
    pub focus: Duration,
    #[serde(with = "duration_as_secs")]
    pub short: Duration,
    #[serde(with = "duration_as_secs")]
    pub long: Duration,

    pub long_interval: u32,

    pub auto_focus: bool,
    pub auto_short: bool,
    pub auto_long: bool,
}

mod duration_as_secs {
    use super::*;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl Default for PomodoroTimerConfig {
    fn default() -> Self {
        Self {
            focus: Duration::from_mins(25),
            short: Duration::from_mins(5),
            long: Duration::from_mins(10),
            long_interval: 3,
            auto_focus: false,
            auto_short: false,
            auto_long: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PomodoroHookConfig {
    pub focus: String,
    pub short: String,
    pub long: String,
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
