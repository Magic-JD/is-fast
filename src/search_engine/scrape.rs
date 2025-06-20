use crate::config::load::Config;
use crate::config::site::SiteConfig;
use crate::errors::error::IsError;
use crate::errors::error::IsError::Scrape;
use crate::search_engine::cache::{cached_pages_purge, cached_pages_read, cached_pages_write};
use crate::search_engine::link::HtmlSource;
use brotli::Decompressor;
use encoding_rs::{Encoding, UTF_8};
use encoding_rs_io::DecodeReaderBytesBuilder;
use once_cell::sync::Lazy;
use std::convert::Into;
use std::io::Read;
use std::time::Duration;
use ureq::http::Response;
use ureq::typestate::WithoutBody;
use ureq::{Agent, Body};

pub static TIMEOUT: Lazy<Duration> = Lazy::new(|| Duration::from_secs(Config::get_timeout()));

pub static UREQ_AGENT: Lazy<Agent> = Lazy::new(|| {
    Agent::config_builder()
        .timeout_global(Some(*TIMEOUT))
        .build()
        .into()
});

pub static HEADER_ORDERING: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "User-Agent",
        "Accept",
        "Accept-Encoding",
        "Accept-Language",
        "Referer",
        "Cookie",
        "Sec-Fetch-Site",
        "Sec-Fetch-Mode",
        "Sec-Fetch-User",
        "Sec-Fetch-Dest",
        "Upgrade-Insecure-Requests",
        "Cache-Control",
    ]
});

pub fn scrape(html_source: &HtmlSource) -> Result<String, IsError> {
    if let Some(html) = cached_pages_read(html_source) {
        return Ok(html);
    }
    ureq_scrape(html_source)
        .inspect(|html| log::trace!("scraping page {html}"))
        .inspect(|html| cached_pages_write(html_source, html))
}

pub fn cache_purge(url: &HtmlSource) {
    cached_pages_purge(url);
}

fn ureq_scrape(html_source: &HtmlSource) -> Result<String, IsError> {
    let url = html_source.get_url();
    let mut request = UREQ_AGENT.get(url);
    request = add_url_based_headers(&html_source.get_config(), request);

    let response = request.call().map_err(|e| {
        Scrape(format!(
            "Request failed for {url} - check your internet connection (internal server error): {e}"
        ))
    })?;

    if !response.status().is_success() {
        return Err(error_for_fail_response_code(url, &response));
    }

    decode_text(url, response)
}

fn add_url_based_headers(
    url: &SiteConfig,
    request: ureq::RequestBuilder<WithoutBody>,
) -> ureq::RequestBuilder<WithoutBody> {
    let mut request = request;
    let headers = url.get_call().get_headers();
    let mut sorted_headers: Vec<(&String, &String)> = headers.iter().collect();

    sorted_headers.sort_by_key(|(key, _)| {
        HEADER_ORDERING
            .iter()
            .position(|&name| name.eq_ignore_ascii_case(key))
            .unwrap_or(HEADER_ORDERING.len())
    });

    for (key, value) in sorted_headers {
        request = request.header(key, value);
    }

    request
}

fn error_for_fail_response_code(url: &str, response: &Response<Body>) -> IsError {
    Scrape(format!(
        "Request failed for {url}: HTTP Status {}",
        response.status()
    ))
}

fn decode_text(url: &str, response: Response<Body>) -> Result<String, IsError> {
    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|c| c.to_str().ok())
        .unwrap_or_default();
    let encoding_from_headers = if let Some(start) = content_type.find("charset=") {
        let charset = content_type[start + 8..].trim();
        Encoding::for_label(charset.as_bytes())
    } else {
        None
    };

    let is_brotli = response
        .headers()
        .get("Content-Encoding")
        .and_then(|e| e.to_str().ok())
        .is_some_and(|e| e.eq_ignore_ascii_case("br"));

    let mut bytes = Vec::new();
    let mut reader = response.into_body().into_reader();
    reader.read_to_end(&mut bytes).map_err(|_| {
        Scrape(format!(
            "Request failed for {url}, could not extract content."
        ))
    })?;

    if is_brotli {
        let mut decompressed = Vec::new();
        let mut brotli_decoder = Decompressor::new(&*bytes, 4096);
        brotli_decoder.read_to_end(&mut decompressed).map_err(|_| {
            Scrape(format!(
                "Request failed for {url}, Brotli decompression failed."
            ))
        })?;
        bytes = decompressed;
    }

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
