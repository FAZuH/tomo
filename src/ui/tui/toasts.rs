use std::ops::Deref;
use std::ops::DerefMut;
use std::time::Duration;

use ratatui::layout::Rect;
use ratatui_toaster::ToastEngine;
use ratatui_toaster::ToastEngineBuilder;
use ratatui_toaster::ToastMessage;

pub struct ToastHandler {
    engine: ToastEngine<ToastMessage>,
}

impl ToastHandler {
    pub fn new() -> Self {
        let engine = ToastEngineBuilder::new(Rect::default())
            .default_duration(Duration::from_secs(3))
            .build();

        // Self { engine, rx }
        Self { engine }
    }

    pub fn tick(&mut self) {
        self.engine.purge_expired();
    }
}

impl Default for ToastHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DerefMut for ToastHandler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.engine
    }
}

impl Deref for ToastHandler {
    type Target = ToastEngine<ToastMessage>;

    fn deref(&self) -> &Self::Target {
        &self.engine
    }
}
