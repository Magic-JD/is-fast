use crate::models::Link;
use scraper::{Html, Selector};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn extract_links(html: &String) -> Vec<Link> {
    let document = Html::parse_document(&html);
    let selector_title = Selector::parse("a.result__a").unwrap();
    let selector_url = Selector::parse("a.result__url").unwrap();

    document
        .select(&selector_title)
        .zip(document.select(&selector_url))
        .take(5)
        .map(|(title, url)| Link {
            title: title.text().collect::<Vec<_>>().join(" ").trim().to_owned(),
            url: url.text().collect::<Vec<_>>().join(" ").trim().to_owned(),
        })
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
        _ => "body",  // Default selector (full document)
    }).map_err(|_| "Error: Could not parse selector")?;
    let page: String = Html::parse_document(&res)
        .select(&selector)
        .map(|ele| ele.html())
        .collect();

    let cols = std::env::var("COLUMNS")
        .unwrap_or_else(|_| "180".to_string())
        .parse::<usize>()
        .unwrap_or(180)
        .saturating_sub(5);

    Command::new("pandoc")
        .arg("-f")
        .arg("html")
        .arg("-t")
        .arg("ansi")
        .arg("--columns")
        .arg(cols.to_string())
        .env("TERM", "xterm-256color")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(page.as_bytes()).ok();
            }
            child.wait_with_output()
        })
        .map(|out| {
            String::from_utf8(out.stdout)
                .unwrap_or(String::from("Error: Could not convert to ansi"))
        })
        .map_err(|_| String::from("Error: Could not convert to ansi"))
}
