use crate::database::connect::HistoryData;
use crate::tui::history::SearchOn;
use crate::tui::history::SearchOn::{Title, Url};
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use once_cell::sync::Lazy;
use std::cmp::Ordering;

static SEARCH_TYPE: Lazy<&AtomKind> = Lazy::new(crate::config::load::Config::get_search_type);
pub fn order_by_match(
    history: &mut [HistoryData],
    user_search: &mut String,
    search_on: &SearchOn,
) -> Vec<HistoryData> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::new(
        &*user_search,
        CaseMatching::Ignore,
        Normalization::Smart,
        **SEARCH_TYPE,
    );
    let mut data_2_score = history
        .iter()
        .map(|h| {
            let match_on = search_on_history(h, search_on);
            (
                h,
                pattern.score(Utf32Str::new(match_on, &mut vec![]), &mut matcher),
            )
        })
        .filter(|(_, score)| score.is_some())
        .collect::<Vec<(&HistoryData, Option<u32>)>>();
    data_2_score.sort_by(|(h1, a), (h2, b)| {
        match a.unwrap_or_else(|| 0).cmp(&b.unwrap_or_else(|| 0)) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => h1.time.cmp(&h2.time),
            Ordering::Greater => Ordering::Greater,
        }
    });
    data_2_score.into_iter().map(|(a, _)| a.clone()).collect()
}

fn search_on_history<'a>(history: &'a HistoryData, search_on: &'a SearchOn) -> &'a str {
    match search_on {
        Title => &history.title,
        Url => &history.url,
    }
}
