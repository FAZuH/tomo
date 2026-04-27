//! Logging setup and configuration.

use std::path::Path;

use tracing_appender::rolling::RollingFileAppender;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::APP_NAME;
use crate::error::AppError;

/// Sets up logging with both console and file output.
pub fn setup_logging(logs_dir: &Path) -> Result<(), AppError> {
    if !logs_dir.exists() {
        std::fs::create_dir_all(logs_dir)
            .map_err(|e| AppError::ConfigError(format!("Failed creating {logs_dir:?}: {e}")))?;
    }
    let writer = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(APP_NAME)
        .filename_suffix("log")
        .max_log_files(7)
        .build(logs_dir)
        .map_err(|e| {
            AppError::ConfigError(format!(
                "Failed to initialize rolling file appender at '{logs_dir:?}': {e}"
            ))
        })?;

    let (non_blocking, guard) = tracing_appender::non_blocking(writer);

    // Leak the guard to prevent it from being dropped
    std::mem::forget(guard);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    Ok(())
}
