use crate::models::Link;
use scraper::{Html, Selector};
use std::io::Write;
use std::process::{Command, Stdio};
use strip_ansi_escapes::strip_str;

pub fn extract_links(html: &String) -> Vec<Link> {
    let document = Html::parse_document(&html);
    let selector_title = Selector::parse("a.result__a").unwrap();
    let selector_url = Selector::parse("a.result__url").unwrap();

    document
        .select(&selector_title)
        .zip(document.select(&selector_url))
        .take(5)
        .map(|(title, url)| Link::new(
            title.text().collect::<Vec<_>>().join(" ").trim().to_owned(),
            url.text().collect::<Vec<_>>().join(" ").trim().to_owned()
        ))
        .collect()
}

pub fn extract_page_content(url: &String, res: &String) -> Result<String, String> {
    let selector = Selector::parse(match url {
        u if u.contains("en.wikipedia.org") => "p",
        u if u.contains("www.baeldung.com") => ".post-content",
        u if u.contains("www.w3schools.com") => "#main",
        u if u.contains("linuxhandbook.com") => "article",
        u if u.contains("docs.spring.io") => "article",
        u if u.contains("stackoverflow.com") => ".js-post-body, .user-details, .comment-body",
        u if u.contains("github.com") => ".markdown-body",
        _ => "body",
    }).map_err(|_| "Error: Could not parse selector")?;
    let page: String = Html::parse_document(&res)
        .select(&selector)
        .map(|ele| ele.html())
        .collect();

    Command::new("pandoc")
        .arg("-f")
        .arg("html")
        .arg("-t")
        .arg("ansi")
        .arg("--columns")
        .arg("10000") // Handle wrap later
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(page.as_bytes()).ok();
                stdin.flush().ok();
            }
            child.wait_with_output()
        })
        .map(|out| strip_non_color_ansi(String::from_utf8(out.stdout).unwrap_or(String::from("Error: Could not parse page content")).as_str()))
        .map_err(|e| String::from(e.to_string()))
}

fn strip_non_color_ansi(input: &str) -> String {
    strip_str(input)
}
