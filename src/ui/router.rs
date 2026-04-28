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

impl From<Page> for Navigation {
    fn from(value: Page) -> Self {
        Self::GoTo(value)
    }
}

#[derive(Debug)]
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

    pub fn quit(&mut self) {
        self.active_page = None
    }

    pub fn active_page(&self) -> Option<Page> {
        self.active_page
    }

    pub fn is_quit(&self) -> bool {
        self.active_page.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit() {
        let mut router = Router::new(Page::Timer);
        router.quit();

        assert!(router.is_quit())
    }

    #[test]
    fn navigate() {
        let mut router = Router::new(Page::Timer);
        assert_eq!(router.active_page(), Some(Page::Timer));

        router.navigate(Page::Settings);
        assert_eq!(router.active_page(), Some(Page::Settings));
    }
}
