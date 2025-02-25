use reqwest::blocking::Client;

pub fn scrape(url: &String) -> Result<String, String> {
    Client::new().get(url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Accept", "text/html,application/xhtml+xml")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .map_err(|e| format!("Request failed: {}", e))?  // Handle request errors
        .text()
        .map_err(|e| format!("Failed to extract text: {}", e))  // Handle response body errors
}
