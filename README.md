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

