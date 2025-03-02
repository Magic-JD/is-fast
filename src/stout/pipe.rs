use ratatui::text::Text;

pub fn out_to_std(text: Text) {
    text.lines.into_iter().for_each(|line| {
        println!("{}", line);
    })
}
