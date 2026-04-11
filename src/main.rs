pub mod error;
pub mod log;

pub use error::Error;
pub use error::Result;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() -> Result<()> {
    tokio::signal::ctrl_c().await?;
    info!("Ctrl+C received, shutting down.");
    Ok(())
}
