use crate::database::connect::show_history;

pub fn run(){
    let history = show_history().unwrap_or_else(|_| String::from("No history found"));
    println!("{}", history);
}