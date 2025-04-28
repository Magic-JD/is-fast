use clap::{ArgAction, Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ColorMode {
    Tui,
    Always,
    Never,
}

#[derive(Debug, PartialEq, Clone, ValueEnum, Default)]
pub enum CacheMode {
    #[default]
    Never,
    Read,
    Write,
    ReadWrite,
    Flash,
}

#[derive(Debug, PartialEq, Clone, ValueEnum, Default)]
pub enum LogLevel {
    #[default]
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Parser)]
pub struct OpenArgs {
    #[arg(help = "The search query to extract content from websites")]
    pub query: Option<Vec<String>>,

    #[arg(short = 'f', long = "file", help = "Path to the HTML file to render")]
    pub file: Option<String>,

    #[arg(
        short = 'u',
        long = "url",
        requires = "file",
        help = "Optional URL to associate with the file"
    )]
    pub url: Option<String>,

    #[arg(short = 'd', long = "direct", help = "Open the given URL/s directly")]
    pub direct: Vec<String>,

    #[arg(long, help = "Show last viewed page")]
    pub last: bool,

    #[arg(long = "site", help = "Show results only from a specific site.")]
    pub site: Option<String>,
}

#[derive(Debug, Parser)]
pub struct SelectionArgs {
    #[arg(
        short = 's',
        long = "selector",
        help = "Use the given CSS selector for this query."
    )]
    pub selector: Option<String>,

    #[arg(long = "ignore", help = "Ignore the given html element/s.")]
    pub ignore: Vec<String>,

    #[arg(
        long = "no-block",
        help = "Do not put block elements on separate lines."
    )]
    pub no_block: bool,

    #[arg(
        long = "nth-element",
        help = "Show only the nth element matching the selector"
    )]
    pub nth_element: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct CacheArgs {
    #[arg(
        long,
        help = "Apply caching for the given search (shorthand for --cache-mode=read-write)"
    )]
    pub cache: bool,

    #[arg(
        long,
        help = "Disable caching for the given search (shorthand for --cache-mode=never)"
    )]
    pub no_cache: bool,

    #[arg(long, value_enum, help = "Set cache mode")]
    pub cache_mode: Option<CacheMode>,

    #[arg(
        long,
        help = "Enable caching with an extremely short TTL (shorthand for --cache-mode=flash)"
    )]
    pub flash_cache: bool,
}

#[derive(Debug, Parser)]
pub struct HistoryArgs {
    #[arg(long, help = "Show previously viewed pages")]
    pub history: bool,

    #[arg(long, help = "Disable history for the given search")]
    pub no_history: bool,
}

#[derive(Debug, Parser)]
pub struct OutputArgs {
    #[arg(long, help = "Output the result to standard out")]
    pub piped: bool,

    #[arg(long, value_enum, help = "Set color mode")]
    pub color: Option<ColorMode>,

    #[arg(
        long,
        help = "Additional display configuration when printing to the terminal. Available options: wrap, margin, title"
    )]
    pub pretty_print: Vec<String>,

    #[arg(long, help = "Apply the given style to an element.")]
    pub style_element: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct TaskArgs {
    #[arg(long, action = ArgAction::SetTrue, help = "Generate a default configuration file")]
    pub generate_config: bool,

    #[arg(long, help = "Wipe all data")]
    pub clear_all: bool,

    #[arg(long, help = "Wipe history")]
    pub clear_history: bool,

    #[arg(long, help = "Wipe the cache")]
    pub clear_cache: bool,
}

#[derive(Debug, Parser)]
pub struct LogArgs {
    #[arg(long, help = "Activate logging")]
    pub log: bool,

    #[arg(long, value_enum, help = "Set log level")]
    pub log_level: Option<LogLevel>,
}

#[derive(Debug, Parser)]
#[command(name = "is-fast")]
#[command(about = "A fast content extractor for terminal-based internet searches")]
#[command(version = env!("CARGO_PKG_VERSION"), author = "Joseph Daunt")]
#[command(after_help = "For more details, visit https://github.com/Magic-JD/is-fast")]
pub struct Cli {
    #[command(flatten)]
    pub open: OpenArgs,

    #[command(flatten)]
    pub selection: SelectionArgs,

    #[command(flatten)]
    pub cache: CacheArgs,

    #[command(flatten)]
    pub history: HistoryArgs,

    #[command(flatten)]
    pub output: OutputArgs,

    #[command(flatten)]
    pub task: TaskArgs,

    #[command(flatten)]
    pub log: LogArgs,
}
