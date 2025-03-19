# Changelog

## [0.12.1]
### Fixed
- Unicode is now correctly rendered for all sites.
- Preloading no longer starts a new thread if one is not needed.

### Changed
- Removed a number of dependencies, where two dependencies were used that performed the same function.

## [0.12.0]
### Added
- New flag for adding additional ignore tags. `--ignore="div"`
- New flag for not applying block elements (reducing non formatted or `<br>` code to a single line).
- All tags for ignored and blocked elements support basic css selector features (.class or #id)
- Title `--pretty-print` value will now default to the page title if no title is provided.
- Support multiple elements with one flag for `--nth-element`
- A number of new default page selectors.
- New script for doing quick conversion checks.

### Fixed
- Google search page is now supported to view, as are a number of other pages that were previously blocked.
- Spaces in direct urls are now converted to + for ease of scripting use.

### Changed
- Refactor of code across many files, splitting up logic especially in the Config struct/s.
- Updated a number of dependencies in the cargo lock.

## [0.11.4]
### Changed
- Increased the level of details in the logs.

### Fixed
- Allow the title to be updated in the history.
- Output shown and exit with error code when no results are found.

## [0.11.3]
### Fixed
- Increased the timeout due to the previous timeout being too short.

## [0.11.2]
### Added
- Logging to file when the `RUST_LOG` env var is enabled.
    - The log file will only be created if that environment variable is enabled.
    - Logs will be placed in the is-fast config directory.
    - When enabled, there will be an output to stderr to show the log location.
- Explicit flag for cache level (readwrite, read, write, none, flash).
- Better error messages when the page fails to load.

### Changed
- Page title is now extracted from HTML, meaning that there is no need to retrieve from the search results.

### Fixed
- History tracking now works with everything except the `--file` flag.
- Parallel loading of the next page is much more efficient.
- Errors or non-content now lead to that url being purged from the cache, preventing a bad cache causing lasting issues.

## [0.11.1]
### Changed
- Error when updating

## [0.11.0]
### Added
- Additional configuration for printed output using the --pretty-print command.
- Format is command:value,command:value
 - Can wrap the output. - Command = wrap, no value needed, use --pretty-print="wrap"
 - Can apply a margin to the output. - Command = margin, value = number, use --pretty-print="margin:10"
 - NOTE: Margin this will automatically wrap. I think this should be the desired behaviour anyway if you want margins.
 - Can apply a title to the output. - Command = title, value = string, use --pretty-print="title:TITLE"
 - NOTE: The title cannot contain the characters , or : due to parsing issues.

### Changed
- Updated documentation to include these configuration changes.
- Updated the example scripts to take advantage of these new features.

### Fixed
- Bug where ad results would sometimes be retrieved from duckduckgo

## [0.10.1]
### Added
- The ability to cache your results. Although this option is off by default, enabling it speeds up the time mostly static pages take to reload if you close and open them again.
- Added configuration for this
  - TTL
  - Max cache size
  - Setting for it (Disabled, Read, Write, ReadWrite)
- Added command flags for this:
  - `--cache` will cache the result even if caching is normally disabled.
  - `--no-cache` will not cache the result even if caching is normally enabled.
  - `--flash-cache` uses a special mode where the cache size is maximum for the duration of the request, but the TTL is only 5 seconds. This has some application for scripting, where you don't want to fill your cache, but you want results to persist there throughout the duration of your script.
  - `--clear-cache` removes all items from the cache.
- Added additional command flags for history too.
  - `--clear-history` clears the history.
  - `--clear-all` clears both cache and history.
  - `--no-history` will not log history for that request.

## [0.10.0]
### Fixed
- Accidentally skipped updating in 0.10.1

### Changed
- Enabled command flags to be able to be passed into the config, centralizing the configuration logic.
- Split config file into the raw and processed config.

### Fixed
- Clippy pedantic issues.
- Bug where input is buffered while waiting for the loading screen causing unexpected behaviour on page load.

## [0.9.0]
### Added
- A number of exciting features for scripting with `is-fast`
- Scripts Directory containing a number of example scripts for how you could use `is-fast` for useful programs.
- New flag `--last` - will immediately open the last viewed page (requires history to be enabled)
- New flag `--nth-element` - will select the nth element that matches the css selector in the case there are multiple matches.
- Contributers section to give the kind people that contribute to this project the appreciation they deserve ❤️
- Generation of flag autocomplete scripts as part of the build process.

### Changed
- Moved the selection logic from the link to the page.
- Page now responsible for user passed flags.
- Simplifies the link creation in the search engine.

## [0.8.5] - 2025-03-11
### Added
- Support for 32 bit linux releases

## [0.8.4] - 2025-03-11
### Fixed
- Automated changelog and release notes 🤞
