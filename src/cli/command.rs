use clap::{ArgAction, Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ColorMode {
    Tui,
    Always,
    Never,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum CacheMode {
    Read,
    Write,
    ReadWrite,
    Never,
    Flash,
}

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

    /// Open the given URL/s directly in the TUI viewer. If multiple are given they will all open in the tui.
    #[arg(short = 'd', long = "direct", help = "Open the given URL/s directly")]
    pub(crate) direct: Vec<String>,

    /// Output the result to standard out instead of rendering in the TUI. If | or > is detected this will be automatically applied.
    #[arg(long = "piped", help = "Output the result to standard out")]
    pub(crate) piped: bool,

    /// Use custom selector for 1 time use when viewing a page. This will override any existing configuration.
    #[arg(
        short = 's',
        long = "selector",
        help = "Use the given css selector for this query."
    )]
    pub(crate) selector: Option<String>,

    /// Show only the nth elements. Can be specified multiple times.
    #[arg(
        long = "nth-element",
        help = "Show only the nth element with content that matches the provided selector"
    )]
    pub(crate) nth_element: Vec<usize>,

    /// Search only a specific site.
    #[arg(long = "site", help = "Show results only from site.")]
    pub(crate) site: Option<String>,

    /// Show previously viewed pages.
    #[arg(long = "history", help = "Show previously viewed pages")]
    pub(crate) history: bool,

    /// Show last page.
    #[arg(long = "last", help = "Show last viewed page")]
    pub(crate) last: bool,

    /// Generate a default configuration file if one doesn't already exist.
    #[arg(long, action = ArgAction::SetTrue, help = "Generate a default configuration file")]
    pub(crate) generate_config: bool,

    #[arg(long, value_enum, help = "Set color mode (tui, always, never)")]
    pub color: Option<ColorMode>,

    #[arg(long, help = "Wipe the cache")]
    pub(crate) clear_cache: bool,

    #[arg(long, help = "Wipe history")]
    pub(crate) clear_history: bool,

    #[arg(long, help = "Wipe all data")]
    pub(crate) clear_all: bool,

    #[arg(
        long,
        help = "Apply caching for the given search, shorthand for --cache-mode=readwrite"
    )]
    pub(crate) cache: bool,

    #[arg(
        long,
        help = "Disable caching for the given search, shorthand for --cache-mode=never"
    )]
    pub(crate) no_cache: bool,

    #[arg(long, help = "Disable history for the given search")]
    pub(crate) no_history: bool,

    #[arg(
        long,
        value_enum,
        help = "Set cache mode (never, read, write, readwrite, flash)"
    )]
    pub cache_mode: Option<CacheMode>,

    #[arg(
        long,
        help = "Enable caching with an extremely short ttl, and maximal cache size, useful for scripting, shorthand for --cache-mode=flash"
    )]
    pub(crate) flash_cache: bool,

    #[arg(
        long,
        help = "Additional display configuration when printing to the terminal. Available options are wrap, margin and title"
    )]
    pub(crate) pretty_print: Vec<String>,
}
