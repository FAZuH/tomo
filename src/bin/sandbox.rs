use tomo::config::Config;
use tomo::log::setup_logging;
use tomo::services::SoundService;
use tomo::services::alarm::AlarmService;

fn main() {
    let conf = Config::load().unwrap();
    setup_logging(&conf.logs_path).unwrap();

    let mut srv = AlarmService::new(&conf.pomodoro.alarm);

    srv.play().unwrap();
    srv.sleep_until_end();
}
