# 🌍 Internet Search Fast from the Terminal

`is-fast` is a TUI tool designed for quick and efficient internet searches directly from the terminal, ideal for
environments where you don't have easy access to a browser. With simple commands, you can search the web, navigate
results, and view content seamlessly in the terminal. It supports custom configurations for styling, content extraction,
and syntax highlighting, and allows direct URL viewing, local HTML file rendering, and history tracking. is-fast is
fast, lightweight, and perfect for developers and terminal enthusiasts.

This tool makes **searching from the terminal fast and simple!** 🚀

![demo](demos/main_demo.gif)
[See more demos here!](demos/DEMOS.md)

## ⚡ is-fast

```sh
is-fast "search query"
```

or

```sh
is-fast search query
```

No waiting - just internet search fast in your terminal.  
**It is fast!** ⚡

---

# Running the Project 🏃

## Prerequisites

Before running the project, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version) 🦀
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust) 📦

## Installing the program 🐧 🍏 🪟

### Install prebuilt binaries via shell script

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Magic-JD/is-fast/releases/download/v0.1.3/is-fast-installer.sh | sh
```

### Install prebuilt binaries via Homebrew

```sh
brew install magic-jd/tap/is-fast
```

Install latest from source with cargo:

```sh
cargo install --git https://github.com/Magic-JD/is-fast.git
```


---

### Table Of Contents

- [🔧 Configuration Guide](#configuration-guide)
    - [Default Configuration](#default-configuration)
    - [🏷 Block Elements](#-block-elements)
    - [🚫 Ignored Tags](#-ignored-tags)
    - [🔍 Selectors](#-selectors)
    - [🎨 Text Styles](#-text-styles)
    - [🌈 Syntax Highlighting](#-syntax-highlighting)
    - [🎨 Display Settings](#-display-settings)
    - [🕰️ History Settings](#-history-settings)
    - [🔍 Search Engine Configuration](#-search-engine-configuration)
    - [📌 Summary](#-summary)
- [🌐 Using `is-fast` to Open URLs Directly](#-using-is-fast-to-open-urls-directly)
    - [`--direct` / `-d`](#--direct---d)
- [📃 Using `is-fast` with Local HTML Files](#-using-is-fast-with-local-html-files)
    - [`--file` / `-f`](#--file---f)
    - [`--url` / `-u`](#--url---u)
- [🔄 Using `--piped` to Output to Standard Output](#-using---piped-to-output-to-standard-output)
- [📜 Viewing History in `is-fast`](#-viewing-history-in-is-fast)
    - [`--history`](#--history)

---

# Configuration Guide

This project supports both built-in and user-provided configurations for styles and content selection rules.
Configuration is handled using a TOML file, and a default configuration is embedded within the binary. Users can
override this configuration by placing a custom config file in their system's configuration directory. Changes will only
take effect once the program is run again.

## Default Configuration

A built-in configuration is included with the binary and is loaded automatically. The default configuration defines
styles for various elements and selectors for extracting content from different websites.

Users can override the default configuration by creating a TOML configuration file in their system’s configuration
directory.

### `--generate-config`

Creates a `config.toml` in your system's configuration directory for customization:

```sh
is-fast --generate-config
```

This will only run if the user does not already have a configuration file in that location.

### Location of User Configuration File

If not generated the configuration file should be placed in:

- **Linux**: `~/.config/is-fast/config.toml`
- **macOS**: `~/Library/Application Support/is-fast/config.toml`
- **Windows**: `%APPDATA%\is-fast\config.toml`

If the `--generate-config` command is used a copy of the default configuration will be placed there automatically.

## 🏷 Block Elements

### Definition

Block elements are HTML tags that should have **a new line before and after** them when processed. This helps preserve
readability and logical structure in the parsed content.

### Example Configuration

```toml
block_elements = [
    "p", "div", "article", "section", "pre", "blockquote", "ul", "ol", "dl", "dt", "dd", "li",
    "h1", "h2", "h3", "h4", "h5", "h6"
]
```

### Effect on Output

#### Input HTML:

```html
<p>This is a paragraph.</p><h2>Title</h2><ul><li>Item 1</li><li>Item 2</li></ul>
```

#### Output After Processing:

```
This is a paragraph.

Title

- Item 1
- Item 2
```

Each **block element** is **separated by a new line** for better readability.

## 🚫 Ignored Tags

### Definition

Ignored tags are HTML elements that **will be completely removed** from the processed content. These typically include *
*scripts, metadata, and interactive elements** that are irrelevant to text processing.

### Example Configuration

```toml
ignored_tags = [
    "script", "style", "noscript", "head", "title", "meta", "input", "button", "svg", "nav",
    "footer", "header", "aside"
]
```

### Effect on Output

#### Input HTML:

```html
<head><title>My Page</title></head>
<body>
  <p>Hello, world!</p>
  <script>alert("Hello");</script>
  <footer>© 2025 My Website</footer>
</body>
```

#### Output After Processing:

```
Hello, world!
```

- **`<script>` and `<footer>` are removed**.
- **Only meaningful content remains**.

## 🔍 Selectors

### Definition

Selectors allow you to **extract only relevant content** from different websites. This is useful for customizing certain
sites for a better user experience. If no selector is provided for a specific site then `body` will be used. Glob
matching is used to match the site, or even certain urls within the site to extract the most relevant text. NOTE: If
there are multiple globs that could match, the most restrictive should be placed higher in the config! Selectors are
chosen from the first match only.

### Example Configuration

```toml
[selectors]
"*en.wikipedia.org*" = "p"
"*github.com/*/blob/*" = ".react-code-line-contents" # Selectors will apply in this order
"*github.com*" = ".markdown-body"
```

### Effect

When processing content from Wikipedia, only `<p>` elements will be extracted. For github, if the url contains the
endpoint blob it will return only elements with the CSS class .react-code-line-contents. Otherwise it will return the
.markdown-body.

## 🎨 Text Styles

### Definition

This section defines **how different HTML tags should be styled** in the output. Colors can be specified using standard color names (e.g., red, blue),
hex values (e.g., #ff5733), or RGB notation (e.g., rgb(255, 87, 51)).

### Example Configuration

```toml
[styles.h1]
bold = true

[styles.a]
fg = "Cyan"

[styles.blockquote]
fg = "Gray"
italic = true
```

This means:

- `<h1>` will be **bold**.
- `<a>` (links) will be **cyan**.
- `<blockquote>` will be **gray** and **italicised**.

## 🌈 Syntax Highlighting

The `[syntax]` section defines syntax highlighting settings for code. Where possible the language type will be
determined from the CSS classes present in the HTML.

### Default Language

This defines the language that is used if the language type cannot be determined from the CSS classes. This should be
set to your primary development language.

### Theme

This sets the theme that should be used. Valid themes are:

```
InspiredGitHub
Solarized (dark)
Solarized (light)
base16-eighties.dark
base16-mocha.dark
base16-ocean.dark
base16-ocean.light
```

### Example:

```toml
[syntax]
default_language = "rust"
theme = "base16-ocean.dark"
```


## 🎨 Display Settings

The `[display]` section defines visual aspects of the output.

### Border Color

This sets the border color used in the UI.

### Page Margin

A percentage of the page width that should be empty on either side.

### Example:

```toml
[display]
border_color = "#74c7ec"
page_margin = 10
```

## 🕰️ History Settings

The `[history]` section defines how historical entries should be displayed.

### Title Color

Sets the color for titles in the history list.

### URL Color

Defines the color for URLs in the history list.

### Time Color

Sets the color for the time field in the history list.

### Text Color

Defines the text color of the search bar for history entries.

### Search Type

Determines the type of search used for history entries. Available options:
- `fuzzy` (default) - Uses a fuzzy search algorithm.
- `substring` - Matches substrings exactly.
- `exact` - Requires an exact match.

### Example:

```toml
[history]
title_color = "rgb(137, 180, 250)"
url_color = "rgb(186, 194, 222)"
time_color = "rgb(242, 205, 205)"
text_color = "rgb(116, 199, 236)"
search_type = "fuzzy"
```

## 🔍 Search Engine Configuration

Determines which search engine is used when performing searches. Available options:

- `duckduckgo` (default) - Uses DuckDuckGo for search queries.
- `google` - Uses Google Custom Search. **Requires API configuration** (see below).

### 📌 API Configuration for Google Search

If you choose `google` as your search engine, you must set up a Google Custom Search API. Follow these steps:

1. Visit the [Google Custom Search API](https://developers.google.com/custom-search/v1/overview) page.
2. Click **Get Started** and enable the API in your Google Cloud Console.
3. Generate an **API Key** from the credentials section.
4. Create a **Custom Search Engine** and obtain the **Search Engine ID**.
5. Set the following environment variables:

```sh
export IS_FAST_GOOGLE_API_KEY="your_api_key_here"
export IS_FAST_GOOGLE_SEARCH_ENGINE_ID="your_search_engine_id_here"
```

These values must be provided for Google Search to function properly.

### Custom search engine

If you want to add your own custom search engine, please fork the repository and follow the instructions on [this file](src/search/search_type.rs).


### 🛠 Example Configuration

```toml
[search]
engine = "google"
```

## 📌 Summary

| Configuration           | Purpose                                                             |
|-------------------------|---------------------------------------------------------------------|
| **Block Elements**      | Ensure new lines before and after specified tags.                   |
| **Ignored Tags**        | Remove unnecessary elements like scripts, metadata, and navigation. |
| **Selectors**           | Extract only specific content from websites.                        |
| **Styles**              | Define how text should be formatted.                                |
| **Syntax Highlighting** | Defines how the syntax highlighting should be handled.              |
| **Display Settings**    | Controls visual aspects like borders and margins.                   |
| **History Settings**    | Configures history display colors and search behavior.              |
| **Search Engine**       | Determines whether to use DuckDuckGo or Google.                     |
---

# 🌐 Using `is-fast` to Open URLs Directly

`is-fast` allows you to open a specific URL directly in its terminal viewer, bypassing the search functionality. This is
done using the `--direct` option.

### `--direct` / `-d`

**Open a given URL directly in the TUI viewer.**

If this option is provided, `is-fast` will immediately load and render the contents of the given URL inside the terminal
interface.

#### Example Usage:

```sh
is-fast --direct "https://example.com"
is-fast -d https://example.com
```

---

# 📃 Using `is-fast` with Local HTML Files

`is-fast` also supports rendering local HTML files inside its terminal viewer. This is done using the `--file` option.
Additionally, you can associate the file with a reference URL using the `--url` option.

### `--file` / `-f`

**View a local HTML file instead of performing an internet search.**

If this option is provided, `is-fast` will render the given HTML file inside its terminal viewer instead of fetching
search results from the internet.

#### Example Usage:

```sh
is-fast --file example.html
is-fast -f example.html
```

### `--url` / `-u`

**Associate the HTML file with a reference URL.**

This option is only valid when `--file` is used. It allows you to provide a URL that will be used for informing which
selector should be used with this file.

#### Example Usage:

```sh
is-fast --file example.html --url example.com
is-fast -f example.html -u example.com
```

---

# 🔄 Using `--piped` to Output to Standard Output

Instead of rendering the content inside the TUI viewer, `is-fast` provides an option to output the processed result
directly to **standard output (stdout)**. This allows you to **pipe the output** to other commands or **write it to a
file**.

### `--piped`

**Output the result to standard output instead of rendering in the TUI.**

This option requires either `--file` or `--direct` to specify the input source.

#### Example Usage:

```sh
# Output the contents of a local file to stdout
is-fast --file example.html --piped

# Fetch and output the contents of a URL to stdout
is-fast --direct "https://example.com" --piped

# Save the output to a file
is-fast --file example.html --piped > output.txt

# Pipe the output into another command
is-fast --direct "https://example.com" --piped | grep "keyword"
```

Using `--piped` makes `is-fast` behave more like a **command-line utility** for extracting and processing content,
rather than an interactive TUI viewer.

---

# 📜 Viewing History in `is-fast`

`is-fast` allows you to view and select previously visited pages using the `--history` option.

### `--history`

**Show previously viewed pages.**

If this option is provided, `is-fast` will display a list of previously visited webpages, numbered with the most recent
entries at the bottom. You can scroll up and down and select to open. The entries are stored locally in a SQLlite
database.

#### Example Usage:

```sh
is-fast --history
```

