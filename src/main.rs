use clap::Parser;
use tomo::cli::Cli;
use tomo::config::Config;
use tomo::error::AppError;
use tomo::log::setup_logging;
use tomo::model::Pomodoro;
use tomo::service::alarm::AlarmService;
use tomo::ui::Runner;

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    let conf = Config::load()?;
    setup_logging(&conf.logs_path)?;

    let pomo = pomodoro(&cli, &conf);

    let mut view = runner(conf, pomo);
    view.run().unwrap();

    Ok(())
}

fn runner<'b>(conf: Config, pomo: Pomodoro) -> impl Runner + 'b {
    use tomo::ui::tui::TuiRunner;
    use tomo::ui::tui::view::TuiView;

    let sound = Box::new(AlarmService::new(conf.pomodoro.alarm.clone()));
    let view = Box::new(TuiView::new());
    TuiRunner::new(pomo, conf, view, sound).unwrap()
}

fn pomodoro(cli: &Cli, conf: &Config) -> Pomodoro {
    let timer = conf.pomodoro.timer.clone();

    let focus = cli.focus.unwrap_or(timer.focus);
    let long_break = cli.long_break.unwrap_or(timer.long);
    let short_break = cli.short_break.unwrap_or(timer.short);
    let long_interval = cli.long_interval.unwrap_or(timer.long_interval);

    let mut ret = Pomodoro::new(focus, long_break, short_break, long_interval);

    if !timer.auto_start_on_launch {
        let _ = ret.pause();
    } else {
        let _ = ret.start();
    }

    ret
}
