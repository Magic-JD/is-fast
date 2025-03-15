use crate::errors::error::IsError;
use crate::errors::error::IsError::{General, Scrape};
use crate::search_engine::cache::{cached_pages_read, cached_pages_write};
use once_cell::sync::Lazy;
use reqwest::blocking::{Client, Response};
use std::process::Command;

pub static REQWEST_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .http1_only()
        .build()
        .expect("Failed to build reqwest client")
});

pub fn scrape(url: &str) -> Result<String, IsError> {
    let url = &format_url(url).ok_or(General(String::from("invalid url")))?;
    if let Some(html) = cached_pages_read(url) {
        log::debug!("Cache hit for: {}", url);
        return Ok(html);
    }
    log::debug!("Cache miss for: {}", url);
    reqwest_scrape(url)
        .or_else(|_| curl_scrape(url))
        .inspect(|html| cached_pages_write(url, html))
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
        .and_then(Response::error_for_status) // Ensure HTTP errors are caught
        .and_then(Response::text)
        .map_err(|e| Scrape(format!("Request failed for {url}: {e}")))
}

// Some sites seem to be more comfortable serving curl rather than reqwest
fn curl_scrape(url: &str) -> Result<String, IsError> {
    let output = Command::new("curl")
        .args([
            "-A",
            "Mozilla/5.0 (compatible; MSIE 7.01; Windows NT 5.0)",
            url,
        ])
        .output()
        .map_err(|e| Scrape(e.to_string()))?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
