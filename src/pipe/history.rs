use crate::database::connect::HistoryData;
use crate::errors::error::IsError;
use crate::errors::error::IsError::Csv;
use csv::Writer;

pub fn pipe_history(history_entries: Vec<HistoryData>) -> Result<(), IsError> {
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
