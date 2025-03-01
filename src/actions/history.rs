use crate::database::connect::get_history;
use crate::tui::history::History;

pub fn run(){
    let history = get_history().unwrap_or_else(|_| vec![]);
    History::new().show_history(history);
}