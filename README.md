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

# 📜 Commands

## 🔍 Search the Web
Run a search query from the terminal:
```sh
is-fast "search query"
```
or
```sh
is-fast search query
```

## ⚙ Generate Default Configuration
Creates a `config.toml` in your system's configuration directory for customization:
```sh
is-fast --generate-config
```


# Configuration Guide

This project supports both built-in and user-provided configurations for styles and content selection rules. Configuration is handled using a TOML file, and a default configuration is embedded within the binary. Users can override this configuration by placing a custom config file in their system's configuration directory.

## Default Configuration

A built-in configuration is included with the binary and is loaded automatically. The default configuration defines styles for various elements and selectors for extracting content from different websites.

Users can override the default configuration by creating a TOML configuration file in their system’s configuration directory.

### Location of User Configuration File

If not generated the configuration file should be placed in:

- **Linux**: `~/.config/is-fast/config.toml`
- **macOS**: `~/Library/Application Support/is-fast/config.toml`
- **Windows**: `%APPDATA%\is-fast\config.toml`

If the `--generate-config` command is used a copy of the default configuration will be placed there automatically.

# Configuration

This guide explains how different elements in your `config.toml` file affect the parsing of HTML content for display.

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

---

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

---

## 🔍 Selectors
### Definition
Selectors allow you to **extract only relevant content** from different websites. This is useful for customizing certain sites for a better user experience.

### Example Configuration
```toml
[selectors]
"en.wikipedia.org" = "p"
"www.w3schools.com" = "#main"
```

### Effect
When processing content from Wikipedia, only `<p>` elements will be extracted. For w3schools, only elements inside `main` will be considered.

---

## 🎨 Text Styles
### Definition
This section defines **how different HTML tags should be styled** in the output.

### Example Configuration
```toml
[styles.h1]
bold = true

[styles.a]
fg = "Cyan"

[styles.code]
fg = "Red"
```
This means:
- `<h1>` will be **bold**.
- `<a>` (links) will be **cyan**.
- `<code>` will be **red**.

---

## 🌈 Syntax Highlighting

The `[syntax]` section defines syntax highlighting settings.

Example:

```toml
[syntax]
default_language = "plain"
theme = "base16-ocean.dark"
```

| **Key**            | **Description**                                        | **Example**           |
| ------------------ | ------------------------------------------------------ | --------------------- |
| `default_language` | The fallback language for highlighting                 | `"plain"`             |
| `theme`            | The theme for syntax highlighting                      | `"base16-ocean.dark"` |

The **default language** is used as a fallback when syntax highlighting is applied. The language is first determined from the CSS classes present in the HTML tags. If no valid language is detected, the default language specified in the configuration will be used instead.

Themes should be a valid theme from **syntect**.

---

This updated document includes **all configuration options** and a **clear table of values**. Let me know if you need more details! 🚀

## 📌 Summary

| Configuration           | Purpose                                                             |
|-------------------------|---------------------------------------------------------------------|
| **Block Elements**      | Ensure new lines before and after specified tags.                   |
| **Ignored Tags**        | Remove unnecessary elements like scripts, metadata, and navigation. |
| **Selectors**           | Extract only specific content from websites.                        |
| **Styles**              | Define how text should be formatted.                                |
| **Syntax Highlighting** | Defines how the syntax highlighting should be handled.              |

## Modifying the Configuration

To customize styles or add new site selectors, edit your user configuration file and restart the application for changes to take effect.

If you need to reset to the default configuration, delete the user configuration file and restart the application.

# Running the Project 🏃

This guide explains how to set up and install the project through Cargo after cloning the repository.

## Prerequisites

Before running the project, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version) 🦀
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust) 📦

## Installing the program (🐧 🍏 🪟)

Run the following command to clone the repository:

```sh
git clone https://github.com/Magic-JD/is-fast.git
cd is-fast
cargo install --path .
is-fast "your search term"
```

