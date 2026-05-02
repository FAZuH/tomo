use std::path::Path;

use clap::Parser;
use log::info;
use tomo::cli::Cli;
use tomo::config::Alarm;
use tomo::config::Config;
use tomo::error::AppError;
use tomo::log::setup_logging;
use tomo::model::Pomodoro;
use tomo::repo::Repos;
use tomo::service::SoundService;
use tomo::service::alarm::AlarmService;
use tomo::ui::Runner;

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    let conf = Config::load()?;
    setup_logging(&conf.logs_path)?;
    color_eyre::install().unwrap();
    info!("initializing {} v{}", tomo::APP_NAME, tomo::APP_VERSION);

    let repo = repo(&conf.db_path);
    let alarm = alarm();
    let pomo = pomodoro(&cli, &conf);

    repo.session().close_all_sessions().unwrap();

    let mut view = runner(conf, pomo, alarm, repo);
    info!("starting view");
    view.run().unwrap();

    Ok(())
}

fn runner<'b>(
    conf: Config,
    pomo: Pomodoro,
    alarm: Box<dyn SoundService<SoundType = Alarm> + 'static>,
    repo: Box<dyn Repos>,
) -> Box<dyn Runner + 'b> {
    use tomo::ui::tui::TuiRunner;
    use tomo::ui::tui::view::TuiView;

    let view = Box::new(TuiView::new());
    Box::new(TuiRunner::new(pomo, conf, view, alarm, repo).unwrap())
}

fn repo(path: &Path) -> Box<dyn Repos> {
    use tomo::repo::sqlite::SqliteDb;
    use tomo::repo::sqlite::SqliteRepos;

    let url = format!("sqlite://{}", path.display());
    let db = SqliteDb::new(url).unwrap();
    let repo = SqliteRepos::new(db);
    Box::new(repo)
}

fn pomodoro(cli: &Cli, conf: &Config) -> Pomodoro {
    let timer = conf.pomodoro.timer.clone();

    let focus = cli.focus.unwrap_or(timer.focus);
    let long_break = cli.long_break.unwrap_or(timer.long);
    let short_break = cli.short_break.unwrap_or(timer.short);
    let long_interval = cli.long_interval.unwrap_or(timer.long_interval);

    Pomodoro::new(focus, long_break, short_break, long_interval)
}

fn alarm() -> Box<dyn SoundService<SoundType = Alarm>> {
    let alarm = AlarmService::new();
    Box::new(alarm)
}
