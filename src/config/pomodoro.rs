use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

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
    pub volume: crate::config::Percentage,
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
