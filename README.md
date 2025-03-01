# 🌍 Internet Search Fast from the Terminal

`is-fast` is a TUI tool designed for quick and efficient internet searches directly from the terminal, ideal for environments where you don't have easy access to a browser. With simple commands, you can search the web, navigate results, and view content seamlessly in the terminal. It supports custom configurations for styling, content extraction, and syntax highlighting, and allows direct URL viewing, local HTML file rendering, and history tracking. is-fast is fast, lightweight, and perfect for developers and terminal enthusiasts.

This tool makes **searching from the terminal fast and simple!** 🚀

## ⚡ is-fast

```sh
is-fast "search query"
```
or
```sh
is-fast search query
```

### 🔧 Navigation Controls
- 🔍 **Next result:** `n` / `→`
- ⬅️ **Go back:** `b` / `←`
- ⬇️ **Scroll down:** `j` / `↓`
- ⬆️ **Scroll up:** `k` / `↑`
- 📜 **Page down:** `CTRL + d`
- 📜 **Page up:** `CTRL + u`
- 🖥️ **Open in browser** `o`
- ❌ **Quit:** `q`

No waiting - just internet search fast in your terminal.  
**It is fast!** ⚡

---

### Table Of Contents

- [🔧 Configuration Guide](#configuration-guide)
    - [Default Configuration](#default-configuration)
    - [🏷 Block Elements](#-block-elements)
    - [🚫 Ignored Tags](#-ignored-tags)
    - [🔍 Selectors](#-selectors)
    - [🎨 Text Styles](#-text-styles)
    - [🌈 Syntax Highlighting](#-syntax-highlighting)
    - [📌 Summary](#-summary)
- [🌐 Using `is-fast` to Open URLs Directly](#-using-is-fast-to-open-urls-directly)
    - [`--direct` / `-d`](#--direct---d)
- [📃 Using `is-fast` with Local HTML Files](#-using-is-fast-with-local-html-files)
    - [`--file` / `-f`](#--file---f)
    - [`--url` / `-u`](#--url---u)
- [📜 Viewing History in `is-fast`](#-viewing-history-in-is-fast)
    - [`--history`](#--history)
    - [`--select` / `-s`](#--select---s)
- [Running the Project 🏃](#running-the-project-)
    - [Prerequisites](#prerequisites)
    - [Installing the program (🐧 🍏 🪟)](#installing-the-program---)

---

# Configuration Guide

This project supports both built-in and user-provided configurations for styles and content selection rules. Configuration is handled using a TOML file, and a default configuration is embedded within the binary. Users can override this configuration by placing a custom config file in their system's configuration directory. Changes will only take effect once the program is run again.

## Default Configuration

A built-in configuration is included with the binary and is loaded automatically. The default configuration defines styles for various elements and selectors for extracting content from different websites.

Users can override the default configuration by creating a TOML configuration file in their system’s configuration directory.

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
Block elements are HTML tags that should have **a new line before and after** them when processed. This helps preserve readability and logical structure in the parsed content.

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
Ignored tags are HTML elements that **will be completely removed** from the processed content. These typically include **scripts, metadata, and interactive elements** that are irrelevant to text processing.

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
Selectors allow you to **extract only relevant content** from different websites. This is useful for customizing certain sites for a better user experience. If no selector is provided for a specific site then `body` will be used. Glob matching is used to match the site, or even certain urls within the site to extract the most relevant text. NOTE: If there are multiple globs that could match, the most restrictive should be placed higher in the config! Selectors are chosen from the first match only.

### Example Configuration
```toml
[selectors]
"*en.wikipedia.org*" = "p"
"*github.com/*/blob/*" = ".react-code-line-contents" # Selectors will apply in this order
"*github.com*" = ".markdown-body"
```

### Effect
When processing content from Wikipedia, only `<p>` elements will be extracted. For github, if the url contains the endpoint blob it will return only elements with the CSS class .react-code-line-contents. Otherwise it will return the .markdown-body.

## 🎨 Text Styles
### Definition
This section defines **how different HTML tags should be styled** in the output.

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

The `[syntax]` section defines syntax highlighting settings for code. Where possible the language type will be determined from the CSS classes present in the HTML.

### Default Language

This defines the language that is used if the language type cannot be determined from the CSS classes. This should be set to your primary development language.

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

## 📌 Summary

| Configuration           | Purpose                                                             |
|-------------------------|---------------------------------------------------------------------|
| **Block Elements**      | Ensure new lines before and after specified tags.                   |
| **Ignored Tags**        | Remove unnecessary elements like scripts, metadata, and navigation. |
| **Selectors**           | Extract only specific content from websites.                        |
| **Styles**              | Define how text should be formatted.                                |
| **Syntax Highlighting** | Defines how the syntax highlighting should be handled.              |

---

# 🌐 Using `is-fast` to Open URLs Directly

`is-fast` allows you to open a specific URL directly in its terminal viewer, bypassing the search functionality. This is done using the `--direct` option.

### `--direct` / `-d`
**Open a given URL directly in the TUI viewer.**

If this option is provided, `is-fast` will immediately load and render the contents of the given URL inside the terminal interface.

#### Example Usage:
```sh
is-fast --direct "https://example.com"
is-fast -d https://example.com
```

---

# 📃 Using `is-fast` with Local HTML Files

`is-fast` also supports rendering local HTML files inside its terminal viewer. This is done using the `--file` option. Additionally, you can associate the file with a reference URL using the `--url` option.

### `--file` / `-f`
**View a local HTML file instead of performing an internet search.**

If this option is provided, `is-fast` will render the given HTML file inside its terminal viewer instead of fetching search results from the internet.

#### Example Usage:
```sh
is-fast --file example.html
is-fast -f example.html
```

### `--url` / `-u`
**Associate the HTML file with a reference URL.**

This option is only valid when `--file` is used. It allows you to provide a URL that will be used for informing which selector should be used with this file.

#### Example Usage:
```sh
is-fast --file example.html --url example.com
is-fast -f example.html -u example.com
```

---

# 📜 Viewing History in `is-fast`

`is-fast` allows you to view and select previously visited pages using the `--history` option.

### `--history`

**Show previously viewed pages.**

If this option is provided, `is-fast` will display a list of previously visited webpages, numbered with the most recent entries at the bottom. You can scroll up and down and select to open.

#### Example Usage:

```sh
is-fast --history
```

### `--select` / `-s`

**Select a page from history to view.**

This option works in conjunction with `--history`. It allows you to choose a specific previously viewed webpage by its index in the history list. The selected page will be loaded directly in the TUI viewer.

#### Example Usage:

```sh
is-fast --history --select 2
is-fast -s 3 --history
```

This will open the selected entry in the terminal viewer.

---

# Running the Project 🏃

This guide explains how to set up and install the project through Cargo after cloning the repository.

## Prerequisites

Before running the project, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version) 🦀
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust) 📦

## Installing the program 🐧 🍏 🪟

Run the following command to clone the repository and install it on your system:

```sh
git clone https://github.com/Magic-JD/is-fast.git
cd is-fast
cargo install --path .
is-fast "your search term"
```

