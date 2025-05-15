# Changelog

This file documents recent notable changes to this project. The format of this
file is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and
this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Moved from the UI repository. Development up to that point included:
  - Extracting keys from JSON files and storing them in a `HashSet`.
  - Collecting `.rs` file paths from the UI repository and extracting all
    strings using Regex.
  - Filtering out non-key strings from the collected strings.
- Added a `ci.yml` file for CI configuration.
- Collected strings inside the `text!` and `get_text!` macros from frontary repository.
- Cloned frontary repository and the UI repository at runtime and stored them in
  a temporary directory.
- Retained valid serde keys.
- Filtered strings in `Request::post` headers.
- Excluded fixed non-key strings and single-character alphabetic strings.
- Excluded strings in Frontary Model components and chart-related ID strings.
- Compared `all_strings` against the keys in ko-KR.json and en-US.json.
- Added optional CLI flags `--ui-path <PATH>` and `--frontary-path <PATH>` to
  allow using local copies of the UI and Frontary repositories.
