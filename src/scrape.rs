use reqwest::blocking::Client;

pub fn scrape(url: &String) -> String {
    let client = Client::new();
    let html = client.get(url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .expect("Failed to fetch search results")
        .text()
        .expect("Failed to read response");
    html
}
