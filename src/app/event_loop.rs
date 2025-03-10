use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn history_event_loop() -> HistoryAction {
    if let Ok(event::Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    })) = event::read()
    {
        return match code {
            KeyCode::Esc => HistoryAction::Exit,
            KeyCode::Up => HistoryAction::Up,
            KeyCode::Down => HistoryAction::Down,
            KeyCode::Enter => HistoryAction::Open,
            KeyCode::Delete => HistoryAction::Delete,
            KeyCode::Char(char) => HistoryAction::Text(char),
            KeyCode::Backspace => HistoryAction::Backspace,
            KeyCode::Tab => HistoryAction::ChangeSearch,
            _ => HistoryAction::Continue,
        };
    }
    HistoryAction::Continue
}

pub enum HistoryAction {
    Exit,
    Continue,
    Open,
    Up,
    Down,
    Delete,
    Text(char),
    Backspace,
    ChangeSearch,
}
pub fn page_event_loop() -> PageAction {
    if let Ok(event::Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        ..
    })) = event::read()
    {
        return match code {
            KeyCode::Char('q') | KeyCode::Esc => PageAction::Exit,
            KeyCode::Char('n') | KeyCode::Right => PageAction::Next,
            KeyCode::Char('b') | KeyCode::Left => PageAction::Previous,
            KeyCode::Down | KeyCode::Char('j') => PageAction::Down,
            KeyCode::Up | KeyCode::Char('k') => PageAction::Up,
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => PageAction::PageUp,
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => PageAction::PageDown,
            KeyCode::Char('o') => PageAction::Open,
            _ => PageAction::Continue,
        };
    }
    PageAction::Continue
}

pub(crate) enum PageAction {
    Exit,
    Open,
    Up,
    Down,
    PageUp,
    PageDown,
    Next,
    Previous,
    Continue,
}
