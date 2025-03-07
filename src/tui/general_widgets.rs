use crate::config::load::Config;
use once_cell::sync::Lazy;
use ratatui::prelude::{Modifier, Span, Style};
use ratatui::widgets::{Block, Borders};
pub static TUI_BORDER_COLOR: Lazy<Style> = Lazy::new(Config::get_border_color);

pub fn default_block(title: &str, instructions: &str) -> Block<'static> {
    Block::default()
        .title(tui_border_span(title))
        .title_bottom(tui_border_span(instructions))
        .borders(Borders::TOP)
        .style(*TUI_BORDER_COLOR)
}

fn tui_border_span(text: &str) -> Span<'static> {
    Span::styled(
        text.to_string(),
        (*TUI_BORDER_COLOR).add_modifier(Modifier::BOLD),
    )
}
