use crate::tui::display::Display;

pub struct TuiApp {
    pub(crate) display: Display,
}

impl TuiApp {
    pub fn new() -> Self {
        let mut display = Display::new();
        display.loading();
        Self { display }
    }
}
