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

## Installing the program ![Latest Release](https://img.shields.io/github/v/release/Magic-JD/is-fast?include_prereleases) 🐧 🍏 🪟


### Install prebuilt binaries via shell script

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Magic-JD/is-fast/releases/latest/download/is-fast-installer.sh | sh
```

### Install prebuilt binaries via Homebrew

```sh
brew install is-fast
```

### Install latest from source with cargo:

```sh
cargo install --git https://github.com/Magic-JD/is-fast.git
```


---

### Table Of Contents

- [🔧 Configuration Guide](#configuration-guide)
  - [Default Configuration](#default-configuration)
- [Tool Configuration](#tool-configuration)
  - [🎨 Display Settings](#-display-settings)
  - [🕰️ History Settings](#-history-settings)
  - [🔍 Search Configuration](#-search-configuration)
  - [🔍 Selectors](#-selectors)
  - [❓ Miscellaneous Configuration](#-miscellaneous-settings)
  - [📝 Custom Site Configuration](#-custom-site-configuration)
- [Site Configuration](#site-configuration)
  - [🏷 Block Elements](#-block-elements)
  - [🚫 Ignored Tags](#-ignored-tags)
  - [➡️ Indent Elements](#-indent-elements)
  - [🎨 Text Styles](#-text-styles)
  - [🌈 Syntax Highlighting](#-syntax-highlighting)
  - [🗄️ Cache Settings](#-cache-settings)
  - [🛂 Headers](#-headers)
- [🌍 Environment Variables](#-environment-variables)
  - [Directory Configuration](#directory-configuration)
  - [Search Api Configuration](#search-api-configuration)
- [🌐 Using `is-fast` to Open URLs Directly](#-using-is-fast-to-open-urls-directly)
  - [`--direct` / `-d`](#--direct---d)
- [📃 Using `is-fast` with Local HTML Files](#-using-is-fast-with-local-html-files)
  - [`--file` / `-f`](#--file---f)
  - [`--url` / `-u`](#--url---u)
- [ 🔄 Using `--piped`, `|` or `>` to Output to Standard Output](#-using---piped--or--to-output-to-standard-output)
- [📜 Viewing History in `is-fast`](#-viewing-history-in-is-fast)
  - [`--history`](#--history)
  - [`--no-history`](#--no-history)
- [⚡ Caching in `is-fast`](#-caching-in-is-fast)
  - [`--cache`](#--cache)
  - [`--no-cache`](#--no-cache)
  - [`--flash-cache`](#--flash-cache)
  - [`--cache-mode`](#--cache-mode)
- [🐞 Logging in `is-fast`](#-logging-in-is-fast)
  - [`--log`](#--log)
  - [`--log-level`](#--log-level)
- [🔑 Customizing your results](#-customizing-your-results)
  - [`--selector`](#--selector-s)
  - [`--nth-element`](#--nth-element)
  - [`--site`](#--site)
  - [`--color`](#--color)
  - [`--last`](#--last)
  - [`--ignore`](#--ignore)
  - [`--style-element`](#--style-element)
  - [`--no-block`](#--no-block)
  - [`--pretty-print`](#--pretty-print)
- [🧹 Clearing Data](#-clearing-data) 
- [Example scripts](#example-scripts)
- [Contributors](#contributors)
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

If you don't want to use the default config location, setting the environment variable `IS_FAST_CONFIG_PATH` will enable you to
place it wherever you like.

```sh
export IS_FAST_CONFIG_PATH="/full/path/to/config.toml"
```

# Tool Configuration

These configuration values are set for the entire tool, and will be in effect for every site.

## 🎨 Display Settings

The `[display]` section defines visual aspects of the output.

### Border Color

This sets the border color used in the UI.

### Page Margin

A percentage of the page width that should be empty on either side.

### Scroll Amount

The amount that page down/page up should scroll you. Default to full page. Valid values are `full`, `half` and a numerical value, which will scroll that number of lines.

### Color Mode

This sets when color should be shown. The default behavior is for it to show in the TUI but not in the `--piped` or redirected output. Possible values are `tui` `never` and `always`. This can be overriden by applying the `--color` flag when running `is-fast`

```toml
[display]
border_color = "#74c7ec"
page_margin = 10
scroll = "10"
color_mode = "always"
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

### Enabled

Set this to `false` to stop tracking the sites you have visited.

```toml
[history]
title_color = "rgb(137, 180, 250)"
url_color = "rgb(186, 194, 222)"
time_color = "rgb(242, 205, 205)"
text_color = "rgb(116, 199, 236)"
search_type = "fuzzy"
enabled = false # Not currently tracking your history.
```

## 🔍 Search Configuration

### Engine

Determines which search engine is used when performing searches. Available options:

- `duckduckgo` (default) - Uses DuckDuckGo for search queries.
- `google` - Uses Google Custom Search. **Requires API configuration** (see below).
- `kagi` - Uses Kagi Search. **Requires API configuration** (see below).

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

### 📌 API Configuration for Kagi Search

If you choose `kagi` as your search engine, you must have access to the Kagi search API. Relevant documentation is [here](https://help.kagi.com/kagi/api/search.html).

After obtaining access and your API key, set the following environment variable:
```sh
export IS_FAST_KAGI_API_KEY="your_api_key_here"
```

### Custom search engine

If you want to add your own custom search engine, please fork the repository and follow the instructions on [this file](src/search_engine/search_type.rs).


```toml
[search]
engine = "google"
```

### Site

If you want to restrict your search only to a certain domain, setting this value will only show you search results from
that domain. This can be overridden by the `--site` argument.

```toml
[search]
site = "en.wikipedia.org"
```

## 🔍 Selectors

### Definition

Selectors allow you to **extract only relevant content** from different websites. This is useful for customizing certain
sites for a better user experience. If no selector is provided for a specific site then `body` will be used. Glob
matching is used to match the site, or even certain urls within the site to extract the most relevant text. NOTE: If
there are multiple globs that could match, the most restrictive should be placed higher in the config! Selectors are
chosen from the first match only. Note that the CSS selectors defined here apply the full standard CSS selector logic,
and are not limited to #id and .class only. This is in the tool configuration rather than the site configuration, because it is the most common reason that you would want to have a site specific configuration. This saves the user having to create a separate file for every case.

```toml
[selectors]
"*en.wikipedia.org*" = "p"
"*github.com/*/blob/*" = ".react-code-line-contents" # Selectors will apply in this order
"*github.com*" = ".markdown-body"
```

### Effect

When processing content from Wikipedia, only `<p>` elements will be extracted. For GitHub, if the url contains the
endpoint blob it will return only elements with the CSS class .react-code-line-contents. Otherwise, it will return the
.markdown-body.

## ❓ Miscellaneous Settings

### Open tool

This setting is unset by default, and controls the program that is used to open the page if you choose to open in browser. If unset this will be your default open tool. If you set this value it will execute the tool given to it. The tool must be available in your system to be able to run.

### Text size supported

Enabling this will allow text size to be shown when available. This is currently only supported for direct output to kitty terminal. If you are not running this code in kitty v0.40.0+ terminal, or you are using tmux, screen, zellij or another alternate screen then this will not work, and you should not turn it on. By default, this is false. If this is switched off, the text size configuration will have no impact on the output.


```toml
[misc]
open_tool = "w3m"
text_size_supported = false
```

## 📝 Custom Site Configuration

This allows you to add or change the site configuration, based on the site url. Using glob matches, it allows you to specify any number of additional configurations, which are then applied in order. In any case the configurations are conflicting, the last one in the list will be applied. The configurations are given as file names, which must be located in the same config directory as your config.toml.

```toml
[custom_config]
"*.example.com/*" = ["alternate_headers.toml", "alternate_color_scheme.toml"]
```

---

# Site Configuration

Site configurations are set in the config.toml file to define the default behaviour for `is-fast`, but they can be updated with additional configurations in the custom config section of the tool configuration. This allows all of these configurations to be specified on a site by site basis.

## 🏷 Block Elements

### Definition

Block elements are HTML tags that should have **a new line before and after** them when processed. This helps preserve
readability and logical structure in the parsed content.

Block Elements support limited CSS selector features.

div#center        will newline only divs that are marked center. 
div.this.that     will newline divs with the class this or the class that.
.this.that        will newline any element with the class this OR the class that.
#that             will newline any element with the id that.
div#center.this   will newline any div with the id center OR the class this.
div.this#center   is INVALID and will not work.
.this#center      is INVALID and will not work.

You can specify if you want your configuration to override the existing tags, or just append to them.

```toml
block_elements = [
    "p", "div", "article", "section", "pre", "blockquote", "ul", "ol", "dl", "dt", "dd", "li",
    "h1", "h2", "h3", "h4", "h5", "h6"
]
clear_existing_block_tags = true
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

## 🚫 Ignored Tags

### Definition

Ignored tags are HTML elements that **will be completely removed** from the processed content. These typically include *
*scripts, metadata, and interactive elements** that are irrelevant to text processing.

Ignored tags support the same limited CSS selector logic as block elements. See above for more information.

You can specify if you want your new config to replace or append to the default list. By default this will append.

```toml
ignored_tags = [
    "script", "style", "noscript", "head", "title", "meta", "input", "button", "svg", "nav", "footer", "header", "aside"
]
clear_existing_ignored_tags = false
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

## ➡️ Indent Elements

All lines within these elements will be indented by 2 spaces (Note: on wrap the wrapped line will not be indented). Nested elements will be indented further.

Indented tags support the same limited CSS selector logic as block elements. See above for more information.

As above, you can specify if you want this to replace the default tags, or just append to the existing tags.

```toml
indent_tags = [
    "li"
]
# clear_existing_intent_tags = false -> Defaults to false if not included.
```

### Effect on Output

#### Input HTML:

```html
    <html>
        <body>
            <span> Here is a list: </span>
            <ol>
                <li value="1"><span>The Condition is evaluated:</span>
                    <ol>
                        <li value="1"><span>If true, the control moves to Step 4.</span></li>
                        <li value="2"><span>If false, the control jumps to Step 7.</span></li>
                    </ol>
                </li>
                <li value="2"><span>The body of the loop is executed.</span></li>
            </ol>
        </body>
    </html>
```

#### Output After Processing:

```
Here is a list:
  1. The condition is evaluated:
    1. If true, the control moves to Step 4.
    2. If false, the control jumps to Step 7.
  2 The body of the loop is executed.
```

## 🎨 Text Styles

### Definition

This section defines **how different HTML tags should be styled** in the output. Colors can be specified using standard color names (e.g., red, blue), hex values (e.g., #ff5733), or RGB notation (e.g., rgb(255, 87, 51)). Css selectors will be applied as above. Styles will be combined when they match multiple cases. Standard ansi escape codes (e.g. bold, underlined) can all be added.

#### Kitty text size protocol

Size is supported through the kitty text size protocol. This will currently only work with the kitty terminal version 0.40.0+. It will not work if you are running tmux, screen ect. It will only show the size when directly printed to terminal, not through the tui. This feature is very new, so availability in other terminals will depend on uptake.

Valid values for size are `normal` (to reset size in a nested element), `double`, `triple` and `half`.
The first letter is not case-sensitive.
As an alternative the values 1, 2 or 3 can also be used for normal, double and triple.

This feature needs to be specifically switched on in the misc section.

```toml
[styles.h1]
bold = true

[styles.a]
fg = "Cyan"

[styles.blockquote]
fg = "Gray"
italic = true
size = "Double"
```

This means:

- `<h1>` will be **bold**.
- `<a>` (links) will be **cyan**.
- `<blockquote>` will be **gray** and **italicised**.

### Style Precedence and Merging

Styles are cumulative and follow a priority order. The precedence for matching styles is:

- Basic (default) style – applies when no specific match exists.
- Tag selector – applies to all elements of a given type (e.g., div).
- Untagged class selector – applies to all elements with the class (e.g., .that).
- Tagged class selector – applies to a specific tag with the class (e.g., div.that).
- Untagged ID selector – applies to the element with a specific ID (e.g., #this).
- Tagged ID selector – applies to a specific tag with a specific ID (e.g., div#this).

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

```toml
[syntax]
default_language = "rust"
theme = "base16-ocean.dark"
```

## 🗄️ Cache Settings

Caching stores the raw HTML associated with a URL, allowing for faster retrieval of previously accessed results. This is particularly useful for scripts where you may need to select multiple elements from the same page by repeatedly calling the search function with different selectors.
### Configuration Options

### `cache_mode`
- **Description**: Determines the caching mode. Caching is disabled by default.
- **Options**:
  - `disabled`: No caching is performed.
  - `read`: Only reads from the cache; does not write new entries.
  - `write`: Only writes to the cache; does not read from it.
  - `readwrite`: Both reads from and writes to the cache.

  This can be overriden with the `--cache`, `--no-cache`, `--cache-mode` or `--flash-cache` flags.

### `max_size`
- **Description**: Specifies the maximum size of the cache. During testing, it was observed that approximately 2MB is used per 100 entries, though this may vary depending on the size of the pages being cached.
- **Type**: Integer
- **Default**: `100`

### `ttl` (Time to Live)
- **Description**: Defines how long the cached value should remain valid, in seconds. Note that the cached data is stored with the TTL being added to the cached time. This means that if you change this to a longer value and then change it back, the longer-lived data might persist. To remove such data, use the `--clear-cache` flag.
- **Type**: Integer
- **Default**: `300` (5 minutes)

```toml
[cache]
cache_mode = "readwrite"
max_size = 100
ttl = 300
```

Note that the `--flash-cache` flag overrides this config setting readwrite mode, infinite max size and a ttl of 5 seconds while it is applied.

## 🛂 Headers

This section allows you to define the headers that will be added when you make the request.

```toml
[headers]
"Accept" = "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
"Accept-Language" = "en-US,en;q=0.9"
"User-Agent" = "Lynx/2.8.8dev.3 libwww-FM/2.14 SSL-MM/1.4.1"
```

# 🌍 Environment Variables

Certain functionality in `is-fast` can be customized via environment variables. Below are the key environment variables you can configure:

## Directory Configuration
| Variable Name          | Description                                              |
|------------------------|----------------------------------------------------------|
| `IS_FAST_CONFIG_DIR`   | Full path where the configuration file should be stored. |
| `IS_FAST_DATABASE_DIR` | Full path where the database file should be stored.      |
| `IS_FAST_LOG_DIR`      | Full path where log files should be stored.              |

**Note:** These paths must be absolute and cannot be relative to the home directory.

## Search API Configuration
If you plan to use external search engines, you must configure the respective API keys. [See the search engine configuration section above for more details](#engine).

| Environment Variable              | Description                                         |
|-----------------------------------|-----------------------------------------------------|
| `IS_FAST_GOOGLE_API_KEY`          | API key for Google Custom Search.                   |
| `IS_FAST_GOOGLE_SEARCH_ENGINE_ID` | Search Engine ID for Google Custom Search.          |
| `IS_FAST_KAGI_API_KEY`            | API key for Kagi search (currently in closed Beta). |

# 🌐 Using `is-fast` to Open URLs Directly

`is-fast` allows you to open a specific URL directly in its terminal viewer, bypassing the search functionality. This is
done using the `--direct` option.

### `--direct` / `-d`

**Open a given URL directly in the TUI viewer.**

If this option is provided, `is-fast` will immediately load and render the contents of the given URL inside the terminal
interface.

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

```sh
is-fast --file example.html
is-fast -f example.html
```

### `--url` / `-u`

**Associate the HTML file with a reference URL.**

This option is only valid when `--file` is used. It allows you to provide a URL that will be used for informing which
selector should be used with this file.

```sh
is-fast --file example.html --url example.com
is-fast -f example.html -u example.com
```

---

# 🔄 Using `--piped`, `|` or `>` to Output to Standard Output

Instead of rendering the content inside the TUI viewer, `is-fast` provides an option to output the processed result
directly to **standard output (stdout)**. This allows you to **pipe the output** to other commands or **write it to a
file**. This can be directly invoked using the `--piped` command in the case that you just want to print to stdout, or 
added implicitly in the case that the output is not the terminal. The result is in plain text unless `--color=always` 
is applied, but otherwise with the formatting you would see in the TUI.

## Output type

### Search command

When used with a regular search, the first search result will be sent out.

### `--direct` or `--file`

The contents of the page will be output.

### `--history`

The history database will be output in CSV format. If you want to further manipulate/query this data, I recommend 
[mlr](https://github.com/johnkerl/miller) for processing it. If you are more comfortable with json processing, using `mlr`
you can easily convert to json to be processed by [jq](https://github.com/jqlang/jq).

Here is how you could get a list of all the titles this way.

```sh
is-fast --history | mlr --icsv --ojson cat | jq '.[].title'
```

#### Other Uses:

```sh
# Output the contents of a local file to stdout
is-fast --file example.html --piped

# Fetch and output the contents of a URL to stdout
is-fast --direct "https://example.com" --piped

# Save the output to a file
is-fast --file example.html > output.txt

# Pipe the output into another command
is-fast --direct "https://example.com" | grep "keyword"
```

Using `--piped` makes `is-fast` behave more like a **command-line utility** for extracting and processing content,
rather than an interactive TUI viewer.

---

# 📜 Viewing History in `is-fast`

`is-fast` allows you to view and select previously visited pages using the `--history` option.

### `--history`

**Show previously viewed pages.**

If this option is provided, `is-fast` will display a list of previously visited webpages, numbered with the most recent entries at the bottom. You can scroll up and down and select to open. The entries are stored locally in a SQLite database. If you don't wish for your sites to be tracked, then you can switch this feature off in the Configuration. The argument will still show your current history, but new searches will not add to your history. You can delete from your history by using the delete key in the history view, or by running the command `--clear-history`.

```sh
is-fast --history
```

### `--last`

This will show the last page from your history. History must be enabled and have entries for this flag to work. This is very useful for scripts where a search is involved, as search resuts are non deterministic, so repeating with the same search might lead you to have *different results*.

#### Example Usage in a script:

```sh
isf_so() {
    QUESTION=$(is-fast ${*} --site "www.stackoverflow.com" --selector "div.question .js-post-body" --color=always --flash-cache --piped) # Find the question content.
    ANSWER=$(is-fast --last --selector "div.accepted-answer .js-post-body" --color=always --flash-cache --piped) # Separately find the answer content, using last to ensure the same result is shown.
    cat << EOF # Format as desired
QUESTION:

$QUESTION

ANSWER:

$ANSWER
EOF
}
```

### `--no-history`

When this flag is used with a search command will not log history for that request.

```sh
is-fast --no-history "how to deal with an obnoxious boss"
```

---

# ⚡ Caching in `is-fast`

`is-fast` includes an optional caching system to speed up the loading of static pages when revisiting them. By default, caching is disabled, but you can enable it and configure its behavior. The default behaviour of the cache when enabled is to have a TTL of 5 minutes, and a max cache size of 1000. When testing the average size of 1000 results was around 23MB, but this will vary depending on the size of the html you are processing.

As the vast majority of the time is-fast spends is waiting for the results of the webscraping to be returned, when a cache hit occurs the result is basically instant. This is very useful if you are reading a little, closing the program, making some changes, then coming back to the same result.

Note, if the provided flag conflicts with the config, the flag will always take priority. If multiple flags are provided, then is-fast will fail safe to disabled.

### `--cache`

This will cache the result even if caching is normally disabled.

```sh
is-fast --cache "Java how to use entity manager"
```

### `--no-cache`

This will not cache the result even if caching is normally enabled.

```sh
is-fast --no-cache --direct "www.football.com/live/game" --selector "div.scores"
```

### `--flash-cache`

This uses a special mode where the cache size is maximum for the duration of the request, but the TTL is only 5 seconds. This is useful for scripting, where you want temporary caching without filling your cache.

```sh
isf_find() {
    local index=1
    local element
    
    while :; do
        element=$(is-fast --direct "en.wikipedia.org/wiki/rome" --selector "div.mw-content-ltr > p" --nth-element "$index" --color=always --flash-cache --piped)
        
        # Break if the element is empty - means all elements have been searched.
        if [[ -z "$element" ]]; then
            break
        fi
        
        # If the element contains "given word", print and exit
        if echo "$element" | grep -qi "$1"; then
            echo "$element"
            return
        fi
        
        ((index++))
    done
}
```

### `--cache-mode`

Allows you to explicitly set the cache mode. Available options are `readwrite`, `read`, `write`, `never`, and `flash`.

The write mode is useful if you have a bad cached value stored, as it will override the bad value with the newer one.

```sh
is-fast --cache-mode write --direct "www.previously_bad_result.com"
```
---

## 🐞 Logging in `is-fast`

`is-fast` includes an optional logging system to help you debug or monitor the tool's behavior. Logs are written to a file, which by default will be placed inside your config directory. You can override this location by setting the `IS_FAST_LOG_DIR` environment variable.

The logging system is based on Rust's standard logging framework, which is normally controlled through the `RUST_LOG` environment variable. Internally, enabling `--log` has the same effect as setting `RUST_LOG=is_fast=error`. If you want to enable more detailed logging for external crates as well, using the `RUST_LOG` environment variable is recommended.

Example of using `RUST_LOG` to enable verbose logging across crates:

```sh
RUST_LOG="is_fast=debug,reqwest=info" is-fast "How to do rust logging"
```

### `--log`

Enables logging with a default log level of `error`.

```sh
is-fast --log "Java how to use entity manager"
```

### `--log-level`

Sets the specific log level to use. Available options typically include `error`, `warn`, `info`, `debug`, and `trace`.

```sh
is-fast --log --log-level debug "Debug level rust logging"
```

---

# 🔑 Customizing your results

### `--selector/-s`

Apply the given CSS selector to the page. This will only apply to --file and --direct queries.

```sh
is-fast --selector ".interesting" --direct "www.site.com"
```

### `--nth-element`

Normally used in conjunction with `--selector` this allows you to only return the nth element that matches that selector. Multiple options can be provided, either comma separated or flag separated.

```sh
    is-fast --direct "www.example.com/site" --selector "div.sb" --nth-element 1,2 --nth-element 4 # There are multiple div.sb elements - we only want to see the first, second and fourth.
```

### `--site`

This will restrict the search to only the given domain.

```sh
is-fast --site "en.wikipedia.org" "Rust programming language"
```

### `--color`

This allows the caller to specify the color mode. Default value is `tui`, which will only show color in the TUI mode. However it can also be set to `never` and `always`

```sh
is-fast --color=always "How to do a for loop in rust" | bat # Will output to bat with full colors
```
### `--ignore`

This allows the user to specify additional elements to ignore. These follow the same limited css selector logic as in the configuration file. This takes a list value, either from multiple flags or comma separated.

```sh
is-fast --last --ignore="div.sidebar,div#ignore" --ignore=".bad-vibes"
```

### `--style-element`

This flag allows users to apply inline styles to specific elements in the output.

**Format:**  
```
--style-element="tag#id.class.otherclass:fg=red;bg=green;bold"
```
- The selector (`tag#id.class.otherclass`) determines which elements the style applies to.
- The style rules (`fg=red;bg=green;bold`) define the appearance of the element.
- Boolean attributes (like `bold`) default to `true` if no value is provided (`bold=true` is a valid alternative).

**Usage Examples:**  
```sh
is-fast --style-element="h1.title:fg=blue;bold" --style-element="p:fg=gray"
```
This will:
- Style `<h1 class="title">` elements with blue foreground and bold text.
- Style `<p>` elements with a gray foreground.

You can specify multiple `--style-element` flags to apply different styles to different elements. This provides fine-grained control over text appearance in the output.


### `--no-block`

When this flag is applied block elements are ignored. This is useful if you want to get a small amount of information but it ends up being
unexpectedly on different lines.

```sh
is-fast --last --no-block
```

### `--pretty-print`

Customize the format of the output to the terminal with the following commands. This flag does not affect the TUI and would normally be use in conjunction with the `--piped` command:

- **`wrap`**: This will automatically wrap the output.
  ```sh
  is-fast --pretty-print="wrap" "Some search term"
  ```

- **`margin:<value>`**: This will apply a margin to the output. If a margin is applied, it will also automatically wrap the output. The value should be a number indicating the desired margin in characters.
  ```sh
  is-fast --pretty-print="margin:10" "Some search term"
  ```

- **`title:Option(<value>)`**: This will apply a title to the output. Note that the title cannot contain the characters `,` or `:` due to parsing issues. If the title value is not provided then the title of the page will be used instead.
  ```sh
  is-fast --pretty-print="title:My Custom Title" "Some search term"
  ```

- **Combining commands**: You can combine the different commands to apply multiple customizations at once.
  ```sh
  is-fast --pretty-print="wrap,margin:10,title:Search Results" "Some search term"
  ```

### Example usage of `--pretty-print`:

- Apply wrapping with a margin of 10 and a custom title:
  ```sh
  is-fast --pretty-print="wrap,margin:10,title:Rust Programming" "Rust programming language"
  ```

- Apply a title without wrapping:
  ```sh
  is-fast --pretty-print="title:Rust Info" "Rust programming language"
  ```

- Apply wrapping alone:
  ```sh
  is-fast --pretty-print="wrap" "Rust programming language"
  ```

- Apply margin and wrapping together:
  ```sh
  is-fast --pretty-print="margin:15" "Rust programming language"
  ```

---

# 🧹 Clearing Data

To remove stored history or cached pages, use the following options:

- `--clear-history` clears all stored history.
- `--clear-cache` clears all cached pages.
- `--clear-all` clears both cache and history.

```sh
is-fast --clear-history
is-fast --clear-cache
is-fast --clear-all
```

---

# Example scripts

Please see the [scripts](scripts) folder for some fun little functions that show how `is-fast` can be be used in a powerful and flexible way as a cli utility for retrieving information from the web.

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="http://pwnwriter.me"><img src="https://avatars.githubusercontent.com/u/90331517?v=4?s=100" width="100px;" alt="Nabeen Tiwaree"/><br /><sub><b>Nabeen Tiwaree</b></sub></a><br /><a href="#platform-pwnwriter" title="Packaging/porting to new platform">📦</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/rehanzo"><img src="https://avatars.githubusercontent.com/u/60243794?v=4?s=100" width="100px;" alt="Rehan"/><br /><sub><b>Rehan</b></sub></a><br /><a href="#plugin-rehanzo" title="Plugin/utility libraries">🔌</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/d3-X-t3r"><img src="https://avatars.githubusercontent.com/u/1624052?v=4?s=100" width="100px;" alt="d3Xt3r"/><br /><sub><b>d3Xt3r</b></sub></a><br /><a href="#ideas-d3-X-t3r" title="Ideas, Planning, & Feedback">🤔</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

