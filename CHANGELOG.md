# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-09-08

### Added
- Initial stable release.
- Semantic code search using TF-IDF ranking.
- Persistent indexing cache for faster searches.
- Advanced filtering by extension, file size, and glob patterns.
- Search by filename.
- Global configuration support.
- Search aliases to save complex queries.
- Interactive mode to navigate search results.
- Parallel indexing with `rayon` for improved performance.

### Changed
- Refactored project into a modular structure (`cli`, `core`, `cache`).
- Improved CLI output and error messages.

### Fixed
- Fixed issues with alias saving and execution.
- Resolved crate name conflict on crates.io (`semcode-search`).
