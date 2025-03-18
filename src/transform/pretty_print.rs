use crate::cli::command::ColorMode;
use crate::config::load::Config;
use crate::DisplayConfig;
use nu_ansi_term::Style;
use textwrap::{fill, Options, WrapAlgorithm};

pub fn conditional_formatting(
    mut content: String,
    display_configuration_list: &[DisplayConfig],
) -> String {
    // Trim content and remove unnecessary leading spaces.
    content = content.trim().to_string();

    // Early return if no additional configuration is provided.
    if display_configuration_list.is_empty() {
        return format!("\n{content}\n"); // Ensure consistent blank first and last line.
    }

    let mut width = terminal_width();

    let (mut margin, mut wrap, mut title) = (0, false, None);
    for display_config in display_configuration_list {
        match display_config {
            DisplayConfig::Margin(amount) => {
                margin = *amount;
                wrap = true;
            }
            DisplayConfig::Wrap => wrap = true,
            DisplayConfig::Title(title_val) => title = Some(title_val),
        }
    }

    if let Some(title_val) = title {
        let title = if let ColorMode::Always = Config::get_extractor_config().color_mode() {
            Style::new().bold().paint(title_val.trim()).to_string()
        } else {
            title_val.trim().to_string()
        };
        content.insert_str(0, &format!("{title}\n\n"));
    }

    if wrap {
        let total_margin = margin * 2;
        if width > total_margin {
            width -= total_margin;
        }
        content = fill(
            &content,
            Options::new(width as usize).wrap_algorithm(WrapAlgorithm::FirstFit),
        );
        if margin > 0 {
            content = textwrap::indent(&content, &" ".repeat(margin as usize));
        }
    }

    // Ensure consistent blank first and last lines.
    format!("\n{content}\n")
}

#[cfg(not(test))]
fn terminal_width() -> u16 {
    use terminal_size::{terminal_size, Width};
    if let Some((Width(w), _)) = terminal_size() {
        w
    } else {
        log::error!("Failed to get terminal size - defaulting to sane value");
        80
    }
}

#[cfg(test)]
fn terminal_width() -> u16 {
    80
}

#[cfg(test)]
mod tests {
    use crate::transform::pretty_print::conditional_formatting;
    use crate::DisplayConfig;
    use std::fs;
    use std::path::Path;

    fn read_file_content(file_name: &str) -> String {
        let path = Path::new("tests/data").join(file_name);
        fs::read_to_string(path).expect("Failed to read file")
    }

    #[test]
    fn test_conditional_formatting_case_none() {
        let input = read_file_content("unformatted.txt");
        let expected_output = read_file_content("unformatted_result.txt");

        let display_config = vec![]; // No display config for this case
        let result = conditional_formatting(input, &display_config);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_conditional_formatting_case_title() {
        let input = read_file_content("unformatted.txt");
        let expected_output = read_file_content("test_output_title.txt");

        let display_config = vec![DisplayConfig::Title("Test Title".to_string())];
        let result = conditional_formatting(input, &display_config);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_conditional_formatting_with_wrap() {
        let input = read_file_content("unformatted.txt");
        let expected_output = read_file_content("test_output_margin_wrap.txt");

        let display_config = vec![DisplayConfig::Margin(4), DisplayConfig::Wrap];
        let result = conditional_formatting(input, &display_config);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_conditional_formatting_with_title_and_margin_and_wrap() {
        let input = read_file_content("unformatted.txt");
        let expected_output = read_file_content("test_output_title_margin_wrap.txt");

        let display_config = vec![
            DisplayConfig::Title("Custom Title".to_string()),
            DisplayConfig::Margin(5),
            DisplayConfig::Wrap,
        ];
        let result = conditional_formatting(input, &display_config);
        assert_eq!(result, expected_output);
    }
}
