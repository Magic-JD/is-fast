use crate::errors::error::IsError;
use crate::errors::error::IsError::{General, Scrape};
use crate::search_engine::cache::{cached_pages_purge, cached_pages_read, cached_pages_write};
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use reqwest::tls::Version;
use std::time::Duration;

pub static REQWEST_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .use_rustls_tls()
        .min_tls_version(Version::TLS_1_2)
        .timeout(Duration::from_secs(4))
        .build()
        .expect("Failed to build reqwest client")
});

pub fn scrape(url: &str) -> Result<String, IsError> {
    let url = &format_url(url).ok_or(General(String::from("invalid url")))?;
    if let Some(html) = cached_pages_read(url) {
        return Ok(html);
    }
    reqwest_scrape(url)
        .inspect(|html| log::trace!("scraping page {html}"))
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
        .header("User-Agent", "Lynx/2.8.8dev.3 libwww-FM/2.14 SSL-MM/1.4.1")
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .map_err(|_| {
            Scrape(format!(
                "Request failed for {url} - check your internet connection"
            ))
        })
        .and_then(|response| {
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
