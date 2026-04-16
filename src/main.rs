pub mod cli;
pub mod config;
pub mod error;
pub mod log;
pub mod models;
pub mod ui;
pub mod utils;

use clap::Parser;
use cli::Cli;

use crate::config::Config;
use crate::error::Error;
use crate::models::Pomodoro;
use crate::ui::tui::TuiRunner;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let conf = Config::load()?;

    let mut model = create_model(&cli, &conf);
    model.start().unwrap();
    let mut runner = TuiRunner::new(conf, model).unwrap();
    runner.run().unwrap();

    Ok(())
}

fn create_model(cli: &Cli, conf: &Config) -> Pomodoro {
    let timer = conf.pomodoro.timer.clone();

    let focus = cli.focus.unwrap_or(timer.focus);
    let long_break = cli.long_break.unwrap_or(timer.long);
    let short_break = cli.short_break.unwrap_or(timer.short);
    let long_interval = cli.long_interval.unwrap_or(timer.long_interval);

    Pomodoro::new(focus, long_break, short_break, long_interval)
}
