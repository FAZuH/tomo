#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Page {
    Timer,
    Settings,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Navigation {
    Quit,
    Stay,
    GoTo(Page),
}

pub struct Router {
    active_page: Option<Page>,
}

impl Router {
    pub fn new(page: Page) -> Self {
        Self {
            active_page: Some(page),
        }
    }

    pub fn navigate(&mut self, nav: impl Into<Navigation>) {
        match nav.into() {
            Navigation::Quit => self.active_page = None,
            Navigation::Stay => {}
            Navigation::GoTo(page) => self.active_page = Some(page),
        }
    }

    pub fn active_page(&self) -> Option<Page> {
        self.active_page
    }
}
