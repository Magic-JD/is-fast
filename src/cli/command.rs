use clap::{ArgAction, Parser};

/// is-fast - Internet Search Fast from the Terminal
///
/// is-fast is a command-line tool that allows you to quickly search the internet
/// and view the results from within a terminal-only environment.
#[derive(Parser)]
#[command(name = "is-fast")]
#[command(about = "A fast content extractor for terminal-based internet searches")]
#[command(version = env!("CARGO_PKG_VERSION"), author = "Joseph Daunt")]
#[command(after_help = "For more details, visit https://github.com/Magic-JD/is-fast")]
pub struct Cli {
    /// The search query to extract content from websites
    #[arg(help = "The search query to extract content from websites")]
    pub(crate) query: Option<Vec<String>>,

    /// View a local HTML file instead of searching online.
    #[arg(short = 'f', long = "file", help = "Path to the HTML file to render")]
    pub(crate) file: Option<String>,

    /// Associate the HTML file with a reference URL. This option is only valid when --file is used.
    /// It is used for determining which selector should be used with this file.
    #[arg(
        short = 'u',
        long = "url",
        requires = "file",
        help = "Optional URL to associate with the file"
    )]
    pub(crate) url: Option<String>,

    /// Open the given URL directly in the TUI viewer.
    #[arg(short = 'd', long = "direct", help = "Open the given URL directly")]
    pub(crate) direct: Vec<String>,

    /// Output the result to standard out instead of rendering in the TUI. If | or > is detected this will be automatically applied.
    #[arg(long = "piped", help = "Output the result to standard out")]
    pub(crate) piped: bool,

    /// Use custom selector for 1 time use when viewing a page. This will override any existing configuration.
    #[arg(
        short = 's',
        long = "selector",
        help = "Use selector overriding existing configuration."
    )]
    pub(crate) selector: Option<String>,

    /// Show previously viewed pages.
    #[arg(long = "history", help = "Show previously viewed pages")]
    pub(crate) history: bool,

    /// Generate a default configuration file if one doesn't already exist.
    #[arg(long, action = ArgAction::SetTrue, help = "Generate a default configuration file")]
    pub(crate) generate_config: bool,
}
