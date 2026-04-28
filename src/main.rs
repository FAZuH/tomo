use clap::Parser;
use tomo::cli::Cli;
use tomo::config::Config;
use tomo::error::AppError;
use tomo::log::setup_logging;
use tomo::models::Pomodoro;
use tomo::services::alarm::AlarmService;
use tomo::ui::AppModel;
use tomo::ui::View;
use tomo::ui::tui::TuiView;

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    let conf = Config::load()?;
    setup_logging(&conf.logs_path)?;

    let mut pomodoro = pomodoro(&cli, &conf);
    pomodoro.start().unwrap();

    let model = AppModel {
        timer: pomodoro,
        settings: conf,
    };

    let mut view = view(&model);
    view.run(model).unwrap();

    Ok(())
}

fn view<'b>(model: &AppModel) -> impl View<Model = AppModel> + 'b {
    let sound = Box::new(AlarmService::new(model.settings.pomodoro.alarm.clone()));
    TuiView::new(sound).unwrap()
}

fn pomodoro(cli: &Cli, conf: &Config) -> Pomodoro {
    let timer = conf.pomodoro.timer.clone();

    let focus = cli.focus.unwrap_or(timer.focus);
    let long_break = cli.long_break.unwrap_or(timer.long);
    let short_break = cli.short_break.unwrap_or(timer.short);
    let long_interval = cli.long_interval.unwrap_or(timer.long_interval);

    Pomodoro::new(focus, long_break, short_break, long_interval)
}
