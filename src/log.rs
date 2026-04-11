use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_LEVEL: OnceLock<u8> = OnceLock::new();

const RESET:  &str = "\x1b[0m";
const BOLD:   &str = "\x1b[1m";
const DIM:    &str = "\x1b[2m";

const RED:    &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN:   &str = "\x1b[36m";
const WHITE:  &str = "\x1b[37m";

const BG_RED: &str = "\x1b[41m";

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        if $crate::log::log_level() >= 1 {
            eprintln!(
                "{dim}{ts}{reset}  {bg}{bold} ERROR {reset}  {red}{msg}{reset}",
                dim   = $crate::log::DIM,
                ts    = $crate::log::timestamp(),
                reset = $crate::log::RESET,
                bg    = concat!($crate::log::BG_RED, $crate::log::WHITE, $crate::log::BOLD),
                bold  = $crate::log::BOLD,
                red   = $crate::log::RED,
                msg   = format_args!($($arg)*),
            );
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        if $crate::log::log_level() >= 2 {
            eprintln!(
                "{dim}{ts}{reset}  {bold}{yellow} WARN {reset}  {yellow}{msg}{reset}",
                dim    = $crate::log::DIM,
                ts     = $crate::log::timestamp(),
                reset  = $crate::log::RESET,
                bold   = $crate::log::BOLD,
                yellow = $crate::log::YELLOW,
                msg    = format_args!($($arg)*),
            );
        }
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if $crate::log::log_level() >= 3 {
            eprintln!(
                "{dim}{ts}{reset}  {bold}{cyan} INFO {reset}  {msg}",
                dim   = $crate::log::DIM,
                ts    = $crate::log::timestamp(),
                reset = $crate::log::RESET,
                bold  = $crate::log::BOLD,
                cyan  = $crate::log::CYAN,
                msg   = format_args!($($arg)*),
            );
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::log::log_level() >= 4 {
            eprintln!(
                "{dim}{ts}{reset}  {dim}DEBUG{reset}  {dim}{msg}{reset}",
                dim   = $crate::log::DIM,
                ts    = $crate::log::timestamp(),
                reset = $crate::log::RESET,
                msg   = format_args!($($arg)*),
            );
        }
    };
}

pub fn log_level() -> u8 {
    *LOG_LEVEL.get_or_init(|| {
        match std::env::var("FORGOR_LOG")
            .map(|v| v.to_ascii_lowercase())
            .as_deref()
        {
            Ok("error") => 1,
            Ok("warn")  => 2,
            Ok("info")  => 3,
            Ok("debug") => 4,
            _           => 0,
        }
    })
}

fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let ms = now.subsec_millis();
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
}

