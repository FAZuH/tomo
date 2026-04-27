use tomo::config::Config;
use tomo::log::setup_logging;
use tomo::services::SoundService;
use tomo::services::sound::PomodoroNotificationService;

fn main() {
    let conf = Config::load().unwrap();
    setup_logging(&conf.logs_path).unwrap();

    let mut srv = PomodoroNotificationService::new(&conf.pomodoro.notification);

    srv.play().unwrap();
    srv.sleep_until_end();
}
