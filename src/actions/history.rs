use crate::actions::direct;
use crate::database::connect::{get_history, get_history_item, HistoryData};
use crate::tui::history::History;

pub fn run(){
    let history = get_history().unwrap_or_else(|_| vec![]);
    History::new().show_history(history);
}

pub fn run_open(index: usize) {
   get_history_item(index).map(|item| direct::run(Some(item.title), item.url)).unwrap_or_else(|_| println!("Item not found"));

}