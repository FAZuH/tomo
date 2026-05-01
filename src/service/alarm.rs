use std::fs::File;

use log::info;
use rodio::Decoder;
use rodio::DeviceSinkBuilder;
use rodio::MixerDeviceSink;
use rodio::Player;

use crate::config::pomodoro::Alarm;
use crate::service::SoundError;
use crate::service::SoundService;

pub struct AlarmService {
    conf: Option<Alarm>,

    player: Option<Player>,
    // Player does not work if mixer is dropped.
    mixer: Option<MixerDeviceSink>,
}

impl AlarmService {
    pub fn new() -> Self {
        Self {
            conf: None,
            player: None,
            mixer: None,
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
            let decoder = Decoder::try_from(file)?;

            info!("Playing sound file {}", path.display());

            let mixer = DeviceSinkBuilder::open_default_sink()?;
            let player = Player::connect_new(mixer.mixer());
            player.set_volume(state.volume.volume());
            player.append(decoder);
            player.play();

            self.player = Some(player);
            self.mixer = Some(mixer);
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<(), SoundError> {
        if let Some(player) = &self.player {
            player.stop()
        }
        Ok(())
    }

    fn set_sound(&mut self, sound: Self::SoundType) {
        self.conf = Some(sound);
    }

    fn is_playing(&self) -> bool {
        if let Some(player) = &self.player {
            player.is_paused()
        } else {
            false
        }
    }

    fn sleep_until_end(&mut self) {
        if let Some(player) = &self.player {
            player.sleep_until_end()
        }
    }
}
