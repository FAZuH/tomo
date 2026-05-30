use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use log::debug;
use log::info;
use serde::Deserialize;
use serde::Serialize;

use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub pomodoro: PomodoroConfig,
    #[serde(skip)]
    pub logs_path: PathBuf,
    #[serde(skip)]
    pub db_path: PathBuf,
    #[serde(skip)]
    pub conf_path: PathBuf,
    #[serde(skip)]
    pub conf_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(utils::conf_dir())
    }
}

impl Config {
    pub fn new(conf_dir: PathBuf) -> Self {
        let logs_path = conf_dir.join("logs");
        let db_path = conf_dir.join("data.db");
        let conf_path = conf_dir.join("config.yaml");

        Self {
            pomodoro: Default::default(),
            logs_path,
            db_path,
            conf_path,
            conf_dir,
        }
    }

    pub fn load(&mut self) -> Result<(), ConfigError> {
        let conf_dir = &self.conf_dir;
        let conf_path = &self.conf_path;

        debug!("config directory: {:?}", conf_dir);
        if !conf_dir.exists() {
            fs::create_dir_all(conf_dir)?;
            info!("created config directory at {conf_dir:?}");
        }

        if !conf_path.exists() {
            let file = fs::File::create(conf_path)?;
            serde_norway::to_writer(&file, &self)?;
            info!("written default config to {:?}", conf_path);
        } else {
            let file = fs::File::open(conf_path)?;
            let config: Config = serde_norway::from_reader(&file)?;
            info!("loaded configuration");
            self.pomodoro = config.pomodoro;
        };
        Ok(())
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let file = fs::File::create(&self.conf_path)?;
        serde_norway::to_writer(&file, self)?;
        info!("Configuration saved successfully");
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Volume(f32);

impl Volume {
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

    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::half()
    }
}

impl TryFrom<&str> for Volume {
    type Error = std::num::ParseIntError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let f: i32 = value.to_string().parse()?;
        Ok(Volume::new(f as f32 / 100.0))
    }
}

impl std::fmt::Display for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}%", self.0 * 100.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_norway::Error),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PomodoroConfig {
    pub timer: Timers,
    pub hook: Hooks,
    pub alarm: Alarms,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Timers {
    pub auto_start_on_launch: bool,
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
    use serde::Deserializer;
    use serde::Serializer;

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

impl Default for Timers {
    fn default() -> Self {
        Self {
            auto_start_on_launch: true,
            focus: Duration::from_mins(25),
            short: Duration::from_mins(5),
            long: Duration::from_mins(10),
            long_interval: 4,
            auto_focus: false,
            auto_short: false,
            auto_long: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Hooks {
    pub focus: String,
    pub short: String,
    pub long: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Alarms {
    pub focus: Alarm,
    pub short: Alarm,
    pub long: Alarm,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Alarm {
    pub path: Option<PathBuf>,
    pub volume: crate::config::Volume,
}

impl Alarm {
    pub fn volume(&self) -> String {
        self.volume.to_string()
    }

    pub fn path(&self) -> String {
        self.path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn percentage_clamps_above() {
        assert_eq!(Volume::new(1.5).value(), 1.0);
    }

    #[test]
    fn percentage_clamps_below() {
        assert_eq!(Volume::new(-0.5).value(), 0.0);
    }

    #[test]
    fn percentage_new_in_range() {
        assert_eq!(Volume::new(0.75).value(), 0.75);
    }

    #[test]
    fn percentage_try_from_str() {
        assert_eq!(Volume::try_from("50").unwrap().value(), 0.5);
        assert_eq!(Volume::try_from("100").unwrap().value(), 1.0);
    }

    #[test]
    fn percentage_try_from_str_invalid() {
        assert!(Volume::try_from("abc").is_err());
    }

    #[test]
    fn percentage_display() {
        assert_eq!(Volume::new(0.5).to_string(), "50%");
        assert_eq!(Volume::new(1.0).to_string(), "100%");
        assert_eq!(Volume::new(0.0).to_string(), "0%");
    }

    #[test]
    fn percentage_default() {
        let p = Volume::default();
        assert_eq!(p.value(), 0.5);
    }

    #[test]
    fn percentage_full() {
        assert_eq!(Volume::full().value(), 1.0);
    }

    #[test]
    fn percentage_muted() {
        assert_eq!(Volume::muted().value(), 0.0);
    }

    #[test]
    fn percentage_half() {
        assert_eq!(Volume::half().value(), 0.5);
    }

    #[test]
    fn duration_as_secs_roundtrip() {
        let dur = Duration::from_secs(300);
        let serialized = serde_norway::to_string(&duration_as_secs_wrapper::Wrapper(dur)).unwrap();
        let deserialized: duration_as_secs_wrapper::Wrapper =
            serde_norway::from_str(&serialized).unwrap();
        assert_eq!(deserialized.0, dur);
    }

    /// Minimal serde wrapper to test the `duration_as_secs` module in isolation.
    mod duration_as_secs_wrapper {
        use std::time::Duration;

        use serde::Deserialize;
        use serde::Serialize;

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub(super) struct Wrapper(
            #[serde(with = "super::super::duration_as_secs")] pub(super) Duration,
        );
    }
}
