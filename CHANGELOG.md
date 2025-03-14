# Changelog

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
