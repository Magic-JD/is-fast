use crate::config::site::SiteConfig;
use crate::errors::error::IsError;
use crate::errors::error::IsError::Scrape;
use crate::search_engine::cache::{cached_pages_purge, cached_pages_read, cached_pages_write};
use crate::search_engine::link::HtmlSource;
use encoding_rs::{Encoding, UTF_8};
use encoding_rs_io::DecodeReaderBytesBuilder;
use once_cell::sync::Lazy;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::tls::Version;
use std::io::Read;
use std::time::Duration;

pub static REQWEST_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .use_rustls_tls()
        .min_tls_version(Version::TLS_1_2)
        .timeout(Duration::from_secs(4))
        .gzip(true)
        .build()
        .expect("Failed to build reqwest client")
});

pub fn scrape(html_source: &HtmlSource) -> Result<String, IsError> {
    if let Some(html) = cached_pages_read(html_source) {
        return Ok(html);
    }
    reqwest_scrape(html_source)
        .inspect(|html| log::trace!("scraping page {html}"))
        .inspect(|html| cached_pages_write(html_source, html))
}

pub fn cache_purge(url: &HtmlSource) {
    cached_pages_purge(url);
}

fn reqwest_scrape(html_source: &HtmlSource) -> Result<String, IsError> {
    let url = html_source.get_url();
    let builder = REQWEST_CLIENT.get(url);
    let builder = add_url_based_headers(&html_source.get_config(), builder);
    builder
        .send()
        .map_err(|_| {
            Scrape(format!(
                "Request failed for {url} - check your internet connection"
            ))
        })
        .and_then(|res| {
            if !res.status().is_success() {
                return Err(error_for_fail_response_code(url, &res));
            }
            Ok(res)
        })
        .and_then(|response| decode_text(url, response))
}

fn add_url_based_headers(url: &SiteConfig, builder: RequestBuilder) -> RequestBuilder {
    let mut builder = builder;
    for (key, value) in url.get_call().get_headers() {
        builder = builder.header(key, value);
    }
    builder
}

fn error_for_fail_response_code(url: &str, response: &Response) -> IsError {
    Scrape(format!(
        "Request failed for {url}: HTTP Status {} - {}",
        response.status().as_u16(),
        response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown error")
    ))
}

fn decode_text(url: &str, response: Response) -> Result<String, IsError> {
    let encoding_from_headers = response
        .headers()
        .get("Content-Type")
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| {
            if let Some(start) = ct.find("charset=") {
                let charset = &ct[start + 8..].trim();
                Encoding::for_label(charset.as_bytes())
            } else {
                None
            }
        });

    let bytes = response.bytes().map_err(|_| {
        Scrape(format!(
            "Request failed for {url}, could not extract content."
        ))
    })?;

    let encoding = encoding_from_headers
        .or_else(|| Encoding::for_bom(&bytes).map(|(enc, _)| enc))
        .unwrap_or(UTF_8);

    let mut decoder = DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding))
        .build(&*bytes);

    let mut text = String::new();
    decoder.read_to_string(&mut text).map_err(|_| {
        Scrape(format!(
            "Request failed for {url}, could not decode content."
        ))
    })?;
    Ok(text)
}
