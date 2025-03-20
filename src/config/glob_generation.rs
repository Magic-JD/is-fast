use globset::{Glob, GlobSet, GlobSetBuilder};

pub fn generate_globs(urls: Vec<&String>) -> (GlobSet, Vec<Glob>) {
    let mut builder = GlobSetBuilder::new();
    let mut globs = Vec::new();
    urls.iter().for_each(|pattern| {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob.clone());
            globs.push(glob);
        }
    });
    let matcher = builder
        .build()
        .inspect_err(|err| log::error!("{err} : cannot build glob matcher."))
        .unwrap_or_default(); // Should be safe as only valid globs added
    (matcher, globs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_generate_globs() {
        let selectors = {
            let mut map = HashMap::new();
            map.insert("example.com/*".to_string(), "body".to_string());
            map.insert("*.org".to_string(), "div".to_string());
            map
        };
        let (matcher, globs) = generate_globs(selectors.keys().collect());

        assert_eq!(globs.len(), 2);
        assert!(matcher.is_match("example.com/index.html"));
        assert!(matcher.is_match("test.org"));
        assert!(!matcher.is_match("random.net"));
    }
}
