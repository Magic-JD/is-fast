use crate::config::load::Config;
use crate::errors::error::IsError;
use crate::errors::error::IsError::General;
use crate::search_engine::link::PageSource;
use crate::search_engine::scrape::format_url;
use crate::tui::display::Display;
use std::process::Command;

pub struct TuiApp {
    pub(crate) display: Display,
}

impl TuiApp {
    pub fn new() -> Self {
        let mut display = Display::new();
        display.loading();
        Self { display }
    }

    pub fn open_link(&mut self, index: usize, pages: &[PageSource]) -> Result<(), IsError> {
        let url = pages
            .get(index)
            .map(|page| format_url(&page.link.url))
            .ok_or(General(String::from("Page doesn't have a url")))?;
        if let Some(tool) = Config::get_open_command() {
            // If there is a user defined tool to open, use that
            Command::new(tool)
                .arg(&url)
                .status()
                .map_err(|e| General(format!("{e} - you are trying to open with '{tool}' - confirm running this tool with url {url} externally for more information")))?;
        } else {
            // Use system open tool
            open::that(url).map_err(|err| General(err.to_string()))?
        }
        self.display.refresh(); // Refresh display to protect against screen issues.
        Ok(())
    }
}
