use crate::config::load::Config;
use crate::errors::error::IsError;
use crate::errors::error::IsError::General;
use crate::search_engine::link::HtmlSource;
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

    pub fn run_link(&mut self, index: usize, pages: &[HtmlSource]) -> Result<(), IsError> {
        let url = Self::extract_url(index, pages)?;
        if let Some(tool) = Config::get_run_command() {
            match tool {
                Err(error) => return Err(General(error.to_string())),
                Ok(command) => {
                    let command = Self::insert_url(url, command);
                    let command_str = command.join(" ");
                    Command::new("sh")
                        .arg("-c")
                        .arg(&command_str)
                        .status()
                        .map_err(|e| General(format!("{e} - you are trying to open with '{command_str}' - confirm running this tool with url {url} externally for more information")))?;
                }
            }
            self.display.refresh(); // Refresh display to protect against screen issues.
        }
        Ok(())
    }

    fn insert_url(url: &str, command: &Vec<String>) -> Vec<String> {
        let mut command = command.clone();
        if let Some(index) = command.iter().position(|s| s == &"{}".to_owned()) {
            command[index] = url.to_owned();
        } else {
            command.push(url.to_owned());
        }
        command
    }

    pub fn open_link(&mut self, index: usize, pages: &[HtmlSource]) -> Result<(), IsError> {
        let url = Self::extract_url(index, pages)?;
        if let Some(tool) = Config::get_open_command() {
            match tool {
                Err(error) => return Err(General(error.to_string())),
                Ok(command) => {
                    let command = Self::insert_url(url, command);
                    let bin = command[0].as_str();
                    let args = &command[1..];
                    Command::new(bin)
                        .args(args)
                        .arg(url)
                        .status()
                        .map_err(|e| General(format!("{e} - you are trying to open with '{bin}' - confirm running this tool with url {url} externally for more information")))?;
                }
            }
            // If there is a user defined tool to open, use that
        } else {
            // Use system open tool
            open::that(url).map_err(|err| General(err.to_string()))?;
        }
        self.display.refresh(); // Refresh display to protect against screen issues.
        Ok(())
    }

    fn extract_url(index: usize, pages: &[HtmlSource]) -> Result<&str, IsError> {
        pages
            .get(index)
            .and_then(|page| Some(page.get_url()).filter(|s| !s.is_empty()))
            .ok_or(General(String::from("Page doesn't have a url")))
    }
}
