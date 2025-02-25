# 🌍 Internet Search Fast from the Terminal

Ever been stuck in a **terminal-only environment** and needed to look something up? Maybe you're:
- Using a **Raspberry Pi** with no desktop 🍓
- Struggling with **copy-pasting** between a **local browser and sshed terminal** 📝
- Tired of waiting for an **LLM** to generate paragraphs when you just need a quick answer ⏳

This tool makes **searching from the terminal fast and simple!** 🚀

---

## ⚡ is-fast

This tool fetches the **first search result** from Google and presents only the key information.  

### 🔧 Navigation Controls
- 🔍 **Next result:** `n` / `→`
- ⬅️ **Go back:** `b` / `←`
- ⬇️ **Scroll down:** `j` / `↓`
- ⬆️ **Scroll up:** `k` / `↑`
- 📜 **Page down:** `CTRL + d`
- 📜 **Page up:** `CTRL + u`
- ❌ **Quit:** `q`

No waiting - just internet search fast in your terminal.  
**It is fast!** ⚡


# Configuration Guide

This project supports both built-in and user-provided configurations for styles and content selection rules. Configuration is handled using a TOML file, and a default configuration is embedded within the binary. Users can override this configuration by placing a custom config file in their system's configuration directory.

## Default Configuration

A built-in configuration is included with the binary and is loaded automatically. The default configuration defines styles for various elements and selectors for extracting content from different websites.

### Full Default Configuration

```toml
[styles.h1]
bold = true

[styles.h2]
bold = true

[styles.h3]
bold = true

[styles.a]
fg = "Cyan"

[styles.code]
fg = "Red"

[styles.em]
italic = true

[styles.i]
italic = true

[styles.strong]
bold = true

[styles.b]
bold = true

[styles.blockquote]
fg = "Gray"
italic = true

[styles.del]
crossed_out = true

[styles.ins]
underlined = true

[styles.mark]
fg = "Black"
bg = "Yellow"

[styles.small]
fg = "Gray"

[styles.sub]
fg = "Gray"
dim = true

[styles.sup]
fg = "Gray"
dim = true

[styles.pre]
fg = "White"
bg = "Black"

[styles.kbd]
fg = "White"
bg = "DarkGray"

[styles.var]
fg = "Cyan"

[styles.samp]
fg = "Magenta"

[styles.u]
underlined = true

[styles.li]
bold = true

[styles.dt]
bold = true

[styles.dd]
fg = "Gray"

[selectors]
"en.wikipedia.org" = "p"
"www.baeldung.com" = ".post-content"
"www.w3schools.com" = "#main"
"linuxhandbook.com" = "article"
"docs.spring.io" = "article"
"stackoverflow.com" = ".js-post-body, .user-details, .comment-body"
"github.com" = ".markdown-body"
```

## User Configuration

Users can override the default configuration by creating a TOML configuration file in their system’s configuration directory.

### Location of User Configuration File

The configuration file should be placed in:

- **Linux**: `~/.config/is-fast/config.toml`
- **macOS**: `~/Library/Application Support/is-fast/config.toml`
- **Windows**: `%APPDATA%\is-fast\config.toml`

### Example User Configuration File

```toml
[styles.h1]
fg = "Blue"
bold = true

[styles.code]
fg = "Green"

[selectors]
"example.com" = "article"
"website.gov" = ".main-section"
```

## Configuration Loading Behavior

1. The program first loads the built-in configuration.
2. If a user configuration file exists, it is loaded and overrides the corresponding values from the default configuration.
3. Any missing values in the user configuration will fall back to the default values.

## Selecting Elements from Websites

The application extracts content from different websites based on the `selectors` mapping. When processing a URL, it checks against the keys in the `selectors` table and applies the corresponding CSS selector to extract relevant content.

### Example Usage

When processing a wikipedia page, the program looks up `en.wikipedia.org` in the `selectors` table and applies the selector `p` to extract the article content.

If no matching selector is found, it defaults to extracting content from the `<body>` tag.

## Modifying the Configuration

To customize styles or add new site selectors, edit your user configuration file and restart the application for changes to take effect.

If you need to reset to the default configuration, delete the user configuration file and restart the application.

# Running the Project 🏃

This guide explains how to set up and run the project after cloning the repository.

## Prerequisites

Before running the project, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version) 🦀
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust) 📦

## Cloning the Repository

Run the following command to clone the repository:

```sh
git clone https://github.com/Magic-JD/is-fast.git
cd is-fast
```

## Running on Linux 🐧

1. Ensure Rust and Cargo are installed.
2. Run the following commands:

```sh
cargo build --release
cargo run "search query"
```

## Running on macOS 🍏

1. Install Rust and Cargo.
2. Run the following:

```sh
cargo build --release
cargo run "search query"
```

## Running on Windows 🪟

1. Install Rust and Cargo using [rustup](https://rustup.rs/).
2. Open a command prompt or PowerShell and run:

```sh
cargo build --release
cargo run "search query"
```
