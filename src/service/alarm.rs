use std::fs::File;
use std::thread::JoinHandle;

use log::info;
use rodio::Decoder;
use rodio::DeviceSinkBuilder;
use rodio::Player;

use crate::config::pomodoro::Alarm;
use crate::service::SoundError;
use crate::service::SoundService;

pub struct AlarmService {
    conf: Option<Alarm>,
    sound_thread: Option<JoinHandle<()>>,
}

impl AlarmService {
    pub fn new() -> Self {
        Self {
            conf: None,
            sound_thread: None,
        }
    }
}

impl SoundService for AlarmService {
    type SoundType = Alarm;

    fn play(&mut self) -> Result<(), SoundError> {
        let state = match &self.conf {
            Some(s) => s,
            None => return Err(SoundError::ConfigError("state is empty".to_string())),
        };

        if let Some(path) = &state.path
            && let Ok(file) = File::open(path)
        {
            info!("Playing sound file {}", path.display());
            let decoder = Decoder::try_from(file).unwrap();
            let handle = DeviceSinkBuilder::open_default_sink()?;
            let player = Player::connect_new(handle.mixer());
            player.set_volume(state.volume.volume());
            player.append(decoder);

            self.sound_thread = Some(std::thread::spawn(move || {
                player.sleep_until_end();
                drop(handle);
            }));
        }

        Ok(())
    }

    fn set_sound(&mut self, sound: Self::SoundType) {
        self.conf = Some(sound);
    }

    fn is_playing(&self) -> bool {
        self.sound_thread.is_some()
    }

    fn sleep_until_end(&mut self) {
        if let Some(thread) = self.sound_thread.take() {
            thread.join().ok();
        }
    }
}
