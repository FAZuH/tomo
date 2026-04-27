pub mod cmd_runner;
pub mod notify;
pub mod sound;
pub mod traits;

use rodio::DeviceSinkError;
pub use traits::*;

#[derive(thiserror::Error, Debug)]
pub enum SoundError {
    #[error(transparent)]
    DeviceSinkError(#[from] DeviceSinkError),

    #[error("sound configuration error: {0}")]
    ConfigError(String),

    #[error("failed to decode file: {0}")]
    Decode(#[from] rodio::decoder::DecoderError),
}
