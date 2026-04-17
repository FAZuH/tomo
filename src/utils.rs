use std::env;
use std::path::Path;
use std::path::PathBuf;

use figlet_rs::Toilet;

use crate::APP_NAME;

pub fn conf_dir() -> PathBuf {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| env::var("HOME").unwrap());

    #[cfg(target_os = "windows")]
    let home = env::var("APPDATA").unwrap();

    Path::new(&home).join(APP_NAME)
}

pub fn string_width(text: impl AsRef<str>) -> usize {
    text.as_ref()
        .lines()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0)
}

pub fn string_height(text: impl AsRef<str>) -> usize {
    text.as_ref().lines().count()
}

pub fn ascii_mono12(text: impl AsRef<str>) -> String {
    Toilet::mono12()
        .unwrap()
        .convert(text.as_ref())
        .unwrap()
        .to_string()
}

pub fn ascii_future(text: impl AsRef<str>) -> String {
    Toilet::future()
        .unwrap()
        .convert(text.as_ref())
        .unwrap()
        .to_string()
}
