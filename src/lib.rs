pub mod cli;
pub mod config;
pub mod error;
pub mod log;
pub mod models;
pub mod repo;
pub mod ui;
pub mod utils;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
