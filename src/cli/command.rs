use clap::{Parser, ArgAction};

/// is-fast - Internet Search Fast from the Terminal
///
/// is-fast is a command-line tool that allows you to quickly search the internet
/// from a terminal-only environment. Instead of loading a full web browser,
/// it fetches the first search result and presents only the key information.
///
/// Navigation Controls default keybindings:
///
/// - Next result: n / →
///
/// - Go back: b / ←
///
/// - Scroll down: j / ↓
///
/// - Scroll up: k / ↑
///
/// - Page down: CTRL + d
///
/// - Page up: CTRL + u
///
/// - Open in browser: o
///
/// - Quit: q
#[derive(Parser)]
#[command(name = "is-fast")]
#[command(about = "A fast content extractor for terminal-based internet searches")]
#[command(version = "1.0.0", author = "Joseph Daunt")]
#[command(after_help = "For more details, visit https://github.com/Magic-JD/is-fast")]
pub struct Cli {
    /// Generate a default configuration file
    ///
    /// Running this option will create a config.toml file inside the default configuration
    /// directory if one doesn't already exist.
    ///
    /// Example Usage:
    ///
    /// is-fast --generate-config
    #[arg(long, action = ArgAction::SetTrue, help = "Generate a default configuration file")]
    pub(crate) generate_config: bool,

    /// The search query to extract content from websites
    ///
    /// The provided string will be used as a search query. If multiple words
    /// are given, they will be combined into a single query.
    ///
    /// Example Usage:
    ///
    /// is-fast "how to install Rust"
    ///
    /// is-fast Rust tutorial
    #[arg(help = "The search query to extract content from websites")]
    pub(crate) query: Option<Vec<String>>,

    /// View a local HTML file instead of performing an internet search.
    ///
    /// If this option is provided, is-fast will render the given HTML file inside
    /// its terminal viewer instead of fetching search results from the internet.
    ///
    /// Example Usage:
    ///
    ///   is-fast --file example.html
    ///
    ///   is-fast -f example.html
    #[arg(short = 'f', long = "file", help = "Path to the HTML file to render")]
    pub(crate) file: Option<String>,

    /// Associate the HTML file with a reference URL.
    ///
    /// This option is only valid when --file is used. It allows you to provide
    /// a URL that will be used for informing which selector should be used with this file.
    ///
    /// Example Usage:
    ///
    ///   is-fast --file example.html --url "example.com"
    ///
    ///   is-fast -f example.html -u "example.com"
    #[arg(short = 'u', long = "url", requires = "file", help = "Optional URL to associate with the file")]
    pub(crate) url: Option<String>,
}
