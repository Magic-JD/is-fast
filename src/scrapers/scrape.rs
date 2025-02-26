use reqwest::blocking::Client;
use std::process::Command;

pub fn curl_scrape(url: &String) -> Result<String, String> {
    let output = Command::new("curl")
        .args(&[
            "-A",
            "Mozilla/5.0 (compatible; MSIE 7.01; Windows NT 5.0)",
            url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string().into())
}

pub fn reqwest_scrape(url: &String) -> Result<String, String> {
    Client::new()
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Accept", "text/html,application/xhtml+xml")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .map_err(|e| format!("Request failed: {}", e))? // Handle request errors
        .text()
        .map_err(|e| format!("Failed to extract text: {}", e)) // Handle response body errors
}
