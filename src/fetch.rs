use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::process::{Command, Stdio};
use std::io::Write;
use crate::scrape::scrape;
use crate::extract::extract_page_content;

pub fn fetch_url(link: Option<&crate::models::Link>) -> String {
    let first_url = match link {
        Some(link) => link.url.clone(),
        None => return "No links available".to_string(),
    };
    let res = scrape(&format!("https://{}", first_url));
    extract_page_content(&res)
}

