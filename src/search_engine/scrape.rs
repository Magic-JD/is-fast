use crate::errors::error::IsError;
use crate::errors::error::IsError::{General, Scrape};
use crate::search_engine::cache::{cached_pages_purge, cached_pages_read, cached_pages_write};
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use std::process::Command;
use std::time::Duration;

pub static REQWEST_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .http1_only()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("Failed to build reqwest client")
});

pub fn scrape(url: &str) -> Result<String, IsError> {
    let url = &format_url(url).ok_or(General(String::from("invalid url")))?;
    if let Some(html) = cached_pages_read(url) {
        return Ok(html);
    }
    reqwest_scrape(url)
        .or_else(|request_error| {
            curl_scrape(url).map_err(|curl_error| {
                Scrape(format!(
                    "\nReqwest error {request_error}, \nCurl error {curl_error}",
                ))
            })
        })
        .inspect(|html| cached_pages_write(url, html))
}

pub fn cache_purge(url: &str) {
    cached_pages_purge(url);
}

pub fn format_url(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }
    if url.starts_with("http") {
        return Some(url.to_string());
    }
    Some(format!("https://{url}"))
}

fn reqwest_scrape(url: &str) -> Result<String, IsError> {
    REQWEST_CLIENT
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Accept", "text/html,application/xhtml+xml,application/json")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .map_err(|_| {
            Scrape(format!(
                "Request failed for {url} - check your internet connection"
            ))
        })
        .and_then(|response| {
            // Check for HTTP errors and capture the response status
            if !response.status().is_success() {
                return Err(Scrape(format!(
                    "Request failed for {url}: HTTP Status {} - {}",
                    response.status().as_u16(),
                    response
                        .status()
                        .canonical_reason()
                        .unwrap_or("Unknown error")
                )));
            }
            response.text().map_err(|_| {
                Scrape(format!(
                    "Request failed for {url}, could not extract content."
                ))
            })
        })
}

// Some sites seem to be more comfortable serving curl rather than reqwest
fn curl_scrape(url: &str) -> Result<String, IsError> {
    let output = Command::new("curl")
        .args([
            "--max-time",
            "2",
            "-A",
            "Mozilla/5.0 (compatible; MSIE 7.01; Windows NT 5.0)",
            url,
        ])
        .output()
        .map_err(|e| Scrape(e.to_string()))?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned()).and_then(|out| {
        if out.is_empty() {
            Err(Scrape("Curl failed, no result".to_string()))
        } else {
            Ok(out)
        }
    })
}
