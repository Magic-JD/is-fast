pub struct TextApp {}

impl TextApp {
    pub fn new() -> Self {
        Self {}
    }

    pub fn terminating_error(error_message: &str) -> ! {
        eprintln!("{error_message}");
        std::process::exit(1)
    }
}
