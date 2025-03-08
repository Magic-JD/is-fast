use crate::database::connect::{get_history, HistoryData};
use crate::errors::error::IsError;
use crate::errors::error::IsError::Csv;
use crate::tui::history::History;
use csv::Writer;

pub fn run(is_piped: bool) {
    let history = get_history().unwrap_or_else(|_| vec![]);
    match is_piped {
        true => pipe_history(history).unwrap_or_else(|_| eprintln!("Pipe broken!")),
        false => History::new().show_history(history),
    }
}

fn pipe_history(history_entries: Vec<HistoryData>) -> Result<(), IsError> {
    let mut wtr = Writer::from_writer(std::io::stdout());

    wtr.write_record(["title", "url", "timestamp"])
        .map_err(|_| Csv("Header could not be written.".to_string()))?;

    let history: Vec<(String, String, String)> = history_entries
        .into_iter()
        .map(|hd| (hd.title.replace('"', "\"\""), hd.url, hd.time.to_string()))
        .collect();

    for (title, url, timestamp) in history {
        wtr.write_record(&[title, url, timestamp])
            .map_err(|_| Csv("Record could not be written.".to_string()))?;
    }

    wtr.flush().map_err(|_| Csv("Failed to flush.".to_string()))
}
