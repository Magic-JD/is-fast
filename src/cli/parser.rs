use crate::cli::command::{CacheArgs, CacheMode};
use crate::errors::error::IsError::General;
use crate::DisplayConfig;

pub fn determine_nth_element(nth_element: Vec<String>) -> Vec<usize> {
    nth_element
        .into_iter()
        .flat_map(|s| s.split(',').map(|s| s.trim()).map(String::from).collect::<Vec<String>>())
        .filter_map(|n| n.parse::<usize>().map_err(|_| {
            log::error!("Invalid index to pass for --nth-element. {}", n);
            General("Error parsing args".to_string())
        }).and_then(|index| {
            if index == 0 {
                log::error!("Invalid index to pass --nth-element - 0. Nth element uses 1 based indexing");
                return Err(General("Invalid index to pass --nth-element - 0.".to_string()))
            }
            Ok(index)
        }).ok())
        .collect()
}

pub fn determine_ignored(ignored: Vec<String>) -> Vec<String> {
    ignored
        .into_iter()
        .flat_map(|s| {
            s.split(',')
                .filter_map(|s| {
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                })
                .collect::<Vec<String>>()
        })
        .collect()
}

pub fn determine_cache_mode(cache: &CacheArgs) -> Option<CacheMode> {
    match (
        cache.cache_mode.clone(),
        cache.cache,
        cache.no_cache,
        cache.flash_cache,
    ) {
        (Some(cache_mode), false, false, false) => Some(cache_mode),
        (None, true, false, false) => Some(CacheMode::ReadWrite),
        (None, false, true, false) => Some(CacheMode::Never),
        (None, false, false, true) => Some(CacheMode::Flash),
        (None, false, false, false) => None, // None applied, do not apply any difference.
        (_, _, _, _) => Some(CacheMode::Never), // Failsafe disable cache for inconsistent modes
    }
}

pub fn parse_pretty_print(line: &str) -> Vec<DisplayConfig> {
    if line.is_empty() {
        return vec![];
    }
    line.split(',')
        .filter_map(|s| {
            let mut parts = s.split(':');
            match (parts.next(), parts.next()) {
                (Some("wrap"), None) => Some(DisplayConfig::Wrap),
                (Some("margin"), Some(value)) => {
                    value.parse::<u16>().ok().map(DisplayConfig::Margin)
                }
                (Some("title"), some_or_none) => {
                    Some(DisplayConfig::Title(some_or_none.map(ToString::to_string)))
                }
                _ => {
                    log::error!("Invalid display configuration: {}", s);
                    None
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nth_element_valid_nth_elements() {
        let input = vec!["1,3,5".to_string()];
        let result = determine_nth_element(input);
        assert_eq!(result, vec![1, 3, 5]);
    }

    #[test]
    fn test_nth_element_single_valid_nth_element() {
        let input = vec!["2".to_string()];
        let result = determine_nth_element(input);
        assert_eq!(result, vec![2]);
    }

    #[test]
    fn test_nth_element_mixed_valid_and_invalid_inputs() {
        let input = vec!["1,a,3".to_string()];
        let result = determine_nth_element(input);
        assert_eq!(result, vec![1, 3]);
    }

    #[test]
    fn test_nth_element_invalid_zero_index() {
        let input = vec!["0,2".to_string()];
        let result = determine_nth_element(input);
        assert_eq!(result, vec![2]);
    }

    #[test]
    fn test_nth_element_empty_input() {
        let input = vec!["".to_string()];
        let result = determine_nth_element(input);
        assert!(result.is_empty());
    }

    #[test]
    fn test_nth_element_multiple_inputs() {
        let input = vec!["1,2".to_string(), "3,4".to_string()];
        let result = determine_nth_element(input);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_nth_element_all_invalid_inputs() {
        let input = vec!["a,b,c".to_string()];
        let result = determine_nth_element(input);
        assert!(result.is_empty());
    }

    #[test]
    fn test_nth_element_leading_and_trailing_commas() {
        let input = vec![",1,2,,".to_string()];
        let result = determine_nth_element(input);
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_nth_element_spaces_in_input() {
        let input = vec![" 1 , 3 , 5 ".to_string()];
        let result = determine_nth_element(input);
        assert_eq!(result, vec![1, 3, 5]);
    }

    #[test]
    fn test_nth_element_large_numbers() {
        let input = vec!["100,200,300".to_string()];
        let result = determine_nth_element(input);
        assert_eq!(result, vec![100, 200, 300]);
    }

    #[test]
    fn test_nth_element_negative_numbers() {
        let input = vec!["-1,-3,2".to_string()];
        let result = determine_nth_element(input);
        assert_eq!(result, vec![2]);
    }
    #[test]
    fn test_ignored_single_values() {
        let input = vec!["foo".to_string(), "bar".to_string()];
        let result = determine_ignored(input);
        assert_eq!(result, vec!["foo", "bar"]);
    }

    #[test]
    fn test_ignored_comma_separated_values() {
        let input = vec!["foo,bar,baz".to_string()];
        let result = determine_ignored(input);
        assert_eq!(result, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_ignored_mixed_separate_and_comma_separated_values() {
        let input = vec!["foo".to_string(), "bar,baz".to_string()];
        let result = determine_ignored(input);
        assert_eq!(result, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_ignored_empty_input() {
        let input: Vec<String> = vec![];
        let result = determine_ignored(input);
        assert!(result.is_empty());
    }

    #[test]
    fn test_ignored_empty_strings() {
        let input = vec!["".to_string()];
        let result = determine_ignored(input);
        assert!(result.is_empty());
    }

    #[test]
    fn test_ignored_leading_and_trailing_commas() {
        let input = vec![",foo,bar,".to_string()];
        let result = determine_ignored(input);
        assert_eq!(result, vec!["foo", "bar"]);
    }

    #[test]
    fn test_ignored_spaces_in_input() {
        let input = vec![" foo , bar ".to_string()];
        let result = determine_ignored(input);
        assert_eq!(result, vec!["foo", "bar"]);
    }

    #[test]
    fn test_ignored_multiple_comma_separated_strings() {
        let input = vec!["a,b".to_string(), "c,d".to_string()];
        let result = determine_ignored(input);
        assert_eq!(result, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_cache_mode_explicit_cache_mode() {
        let args = CacheArgs {
            cache_mode: Some(CacheMode::ReadWrite),
            cache: false,
            no_cache: false,
            flash_cache: false,
        };
        assert_eq!(determine_cache_mode(&args), Some(CacheMode::ReadWrite));
    }

    #[test]
    fn test_cache_mode_cache_flag_enabled() {
        let args = CacheArgs {
            cache_mode: None,
            cache: true,
            no_cache: false,
            flash_cache: false,
        };
        assert_eq!(determine_cache_mode(&args), Some(CacheMode::ReadWrite));
    }

    #[test]
    fn test_cache_mode_no_cache_flag_enabled() {
        let args = CacheArgs {
            cache_mode: None,
            cache: false,
            no_cache: true,
            flash_cache: false,
        };
        assert_eq!(determine_cache_mode(&args), Some(CacheMode::Never));
    }

    #[test]
    fn test_cache_mode_flash_cache_flag_enabled() {
        let args = CacheArgs {
            cache_mode: None,
            cache: false,
            no_cache: false,
            flash_cache: true,
        };
        assert_eq!(determine_cache_mode(&args), Some(CacheMode::Flash));
    }

    #[test]
    fn test_cache_mode_no_flags_enabled() {
        let args = CacheArgs {
            cache_mode: None,
            cache: false,
            no_cache: false,
            flash_cache: false,
        };
        assert_eq!(determine_cache_mode(&args), None);
    }

    #[test]
    fn test_cache_mode_conflicting_flags_default_to_never() {
        let args = CacheArgs {
            cache_mode: None,
            cache: true,
            no_cache: true,
            flash_cache: false,
        };
        assert_eq!(determine_cache_mode(&args), Some(CacheMode::Never));
    }

    #[test]
    fn test_cache_mode_all_flags_enabled_defaults_to_never() {
        let args = CacheArgs {
            cache_mode: None,
            cache: true,
            no_cache: true,
            flash_cache: true,
        };
        assert_eq!(determine_cache_mode(&args), Some(CacheMode::Never));
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let result = parse_pretty_print(input);
        assert!(result.is_empty());
    }

    #[test]
    fn test_display_config_single_wrap() {
        let input = "wrap";
        let result = parse_pretty_print(input);
        assert_eq!(result, vec![DisplayConfig::Wrap]);
    }

    #[test]
    fn test_display_config_margin_with_valid_number() {
        let input = "margin:10";
        let result = parse_pretty_print(input);
        assert_eq!(result, vec![DisplayConfig::Margin(10)]);
    }

    #[test]
    fn test_display_config_title_with_value() {
        let input = "title:MyTitle";
        let result = parse_pretty_print(input);
        assert_eq!(
            result,
            vec![DisplayConfig::Title(Some("MyTitle".to_string()))]
        );
    }

    #[test]
    fn test_display_config_title_without_value() {
        let input = "title";
        let result = parse_pretty_print(input);
        assert_eq!(result, vec![DisplayConfig::Title(None)]);
    }

    #[test]
    fn test_display_config_multiple_valid_configs() {
        let input = "wrap,margin:15,title:Document";
        let result = parse_pretty_print(input);
        assert_eq!(
            result,
            vec![
                DisplayConfig::Wrap,
                DisplayConfig::Margin(15),
                DisplayConfig::Title(Some("Document".to_string()))
            ]
        );
    }

    #[test]
    fn test_display_config_invalid_format_ignored() {
        let input = "invalid,margin:not_a_number";
        let result = parse_pretty_print(input);
        assert!(result.is_empty()); // Since both are invalid, result should be empty
    }

    #[test]
    fn test_display_config_mixed_valid_and_invalid_configs() {
        let input = "wrap,invalid,margin:20,title";
        let result = parse_pretty_print(input);
        assert_eq!(
            result,
            vec![
                DisplayConfig::Wrap,
                DisplayConfig::Margin(20),
                DisplayConfig::Title(None),
            ]
        );
    }
}
