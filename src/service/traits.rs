use crate::service::SoundError;

pub trait SoundService {
    type SoundType;

    /// Plays sound in another thread.
    fn play(&mut self) -> Result<(), SoundError>;

    /// Stops currently playing song if any.
    fn stop(&mut self) -> Result<(), SoundError>;

    /// Is there any sound playing.
    fn is_playing(&self) -> bool;

    /// Set sound to play.
    fn set_sound(&mut self, sound: Self::SoundType);

    /// Sleeps the thread until the currently playing sound ends.
    fn sleep_until_end(&mut self);
}
