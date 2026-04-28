use std::fs::File;
use std::thread::JoinHandle;

use log::info;
use rodio::Decoder;
use rodio::DeviceSinkBuilder;
use rodio::Player;

use crate::config::pomodoro::Alarm;
use crate::config::pomodoro::Alarms;
use crate::models::pomodoro::State;
use crate::services::SoundError;
use crate::services::SoundService;

pub struct AlarmService {
    focus: Alarm,
    long: Alarm,
    short: Alarm,
    state: Option<State>,

    sound_thread: Option<JoinHandle<()>>,
}

impl AlarmService {
    pub fn new(conf: Alarms) -> Self {
        Self {
            focus: conf.focus,
            long: conf.long,
            short: conf.short,
            state: None,
            sound_thread: None,
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = Some(state);
    }

    pub fn set_sounds(&mut self, conf: &Alarms) {
        self.focus = conf.focus.clone();
        self.long = conf.long.clone();
        self.short = conf.short.clone();
    }
}

impl SoundService for AlarmService {
    type SoundType = State;

    fn play(&mut self) -> Result<(), SoundError> {
        let state = match self.state {
            Some(s) => s,
            None => return Err(SoundError::ConfigError("state is empty".to_string())),
        };

        let alarm = match state {
            State::Focus => &self.focus,
            State::LongBreak => &self.long,
            State::ShortBreak => &self.short,
        };
        if let Some(path) = &alarm.path
            && let Ok(file) = File::open(path)
        {
            info!("Playing sound file {}", path.display());
            let decoder = Decoder::try_from(file).unwrap();
            let handle = DeviceSinkBuilder::open_default_sink()?;
            let player = Player::connect_new(handle.mixer());
            player.set_volume(alarm.volume.volume());
            player.append(decoder);

            self.sound_thread = Some(std::thread::spawn(move || {
                player.sleep_until_end();
                drop(handle);
            }));
        }

        Ok(())
    }

    fn set_sound(&mut self, sound: Self::SoundType) {
        self.state = Some(sound);
    }

    fn is_playing(&self) -> bool {
        !self.sound_thread.is_none()
    }

    fn sleep_until_end(&mut self) {
        if let Some(thread) = self.sound_thread.take() {
            thread.join().ok();
        }
    }
}
