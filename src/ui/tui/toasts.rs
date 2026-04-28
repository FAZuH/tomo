use std::ops::Deref;
use std::ops::DerefMut;
use std::time::Duration;

use ratatui::layout::Rect;
use ratatui_toaster::ToastEngine;
use ratatui_toaster::ToastEngineBuilder;
use ratatui_toaster::ToastMessage;
use tokio::sync::mpsc;

pub struct ToastHandler {
    engine: ToastEngine<ToastMessage>,
    rx: mpsc::Receiver<ToastMessage>,
}

impl ToastHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        let engine = ToastEngineBuilder::new(Rect::default())
            .default_duration(Duration::from_secs(3))
            .action_tx(tx)
            .build();

        Self { engine, rx }
    }

    pub fn tick(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            if matches!(msg, ToastMessage::Hide) {
                self.engine.hide_toast();
            }
        }
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
