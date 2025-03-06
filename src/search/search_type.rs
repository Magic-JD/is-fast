use crate::errors::error::IsError;
use crate::search::duckduckgo::DuckDuckGoSearch;
use crate::search::google::GoogleSearch;
use crate::search::link::Link;

#[derive(Debug, Clone)]
pub enum SearchEngine {
    DuckDuckGo,
    Google,
}

/// # Adding a New Search Engine
///
/// To add a new search engine, follow these steps:
///
/// 1. Implement the [`Search`] trait for your new engine:
///    ```rust
///    struct MySearchEngine;
///
///    impl Search for MySearchEngine {
///        fn search(query: &str) -> Vec<Link> {
///            // Custom search logic
///            vec![]
///        }
///    }
///    ```
///
/// 2. Add it to the [`SearchEngine`] enum:
///    ```rust
///    enum SearchEngine {
///        DuckDuckGo,
///        Google,
///        MySearch, // New search engine
///    }
///    ```
///
/// 3. Update [`config::load::to_search_engine`] to support it.
///
/// # Configuration
///
/// To allow this engine to be set through the configuration, update
/// [`config::load::to_search_engine`] to recognize the new engine type.
///
/// # See Also
/// - [`SearchEngine`] for the available engines.
/// - [`Search`] trait for implementing a new search.
impl Search for SearchEngine {
    fn search(&self, query: &str) -> Result<Vec<Link>, IsError> {
        match self {
            SearchEngine::DuckDuckGo => DuckDuckGoSearch.search(query),
            SearchEngine::Google => GoogleSearch.search(query),
        }
    }
}

pub trait Search {
    fn search(&self, query: &str) -> Result<Vec<Link>, IsError>;
}
