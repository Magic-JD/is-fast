use crate::cli::command::ColorMode;
use crate::config::load::Config;
use crate::DisplayConfig;
use nu_ansi_term::Style;
use terminal_size::{terminal_size, Width};
use textwrap::{fill, Options, WrapAlgorithm};

pub fn conditional_formatting(mut content: String) -> String {
    content = content.trim().to_string(); // Trim content, should be no leading spaces.
    let display_configuration_list = Config::get_display_configuration();
    // No additional configuration, early return.
    if display_configuration_list.is_empty() {
        return format!("\n{content}\n"); //Ensure consistent blank first and last line
    }

    let mut width = if let Some((Width(w), _)) = terminal_size() {
        w
    } else {
        log::error!("Failed to get terminal size - defaulting to sane value");
        80
    };
    let mut margin = 0;
    let mut wrap = false;
    let mut title = None;
    for display_config in display_configuration_list {
        match display_config {
            DisplayConfig::Margin(amount) => {
                margin = *amount;
                wrap = true;
            }
            // As we always wrap if there is an object in the list, we don't need to do
            // anything further here.
            DisplayConfig::Wrap => wrap = true,
            DisplayConfig::Title(title_val) => title = Some(title_val),
        }
    }
    if let Some(title_val) = title {
        let mut title = title_val.to_string();
        if let ColorMode::Always = Config::get_color_mode() {
            let style = Style::new().bold();
            title = style.paint(title).to_string();
        }
        title = format!("{title}\n\n"); // Don't need leading newline as that is added later.
        content.insert_str(0, &title);
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
    format!("\n{content}\n") //Ensure consistent blank first and last line
}
