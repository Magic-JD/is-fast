use crate::actions::direct;
use crate::database::connect::{get_history_item, show_history};

pub fn run(){
    let history = show_history().unwrap_or_else(|_| String::from("No history found"));
    println!("{}", history);
}

pub fn run_open(index: usize) {
   get_history_item(index).map(|item| direct::run(Some(item.title), item.url)).unwrap_or_else(|_| println!("Item not found"));

}