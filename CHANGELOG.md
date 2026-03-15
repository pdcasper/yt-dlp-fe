# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-03-15

### Added
- Native folder picker button to select custom download directory
- Download directory persists between sessions
- Responsive UI that adapts to window size

### Changed
- Improved container layout with flexbox for better responsiveness

## [0.2.0] - 2026-03-15

### Changed
- Replaced `std::process::Command` with `tauri-plugin-shell` to execute yt-dlp without opening a Terminal window on macOS
- Updated build targets to generate `.dmg` and `.app` for macOS

### Fixed
- Terminal window no longer opens during downloads on macOS

## [0.1.0] - 2026-03-15

### Added
- Initial release
- YouTube to MP3 download functionality using yt-dlp
- Simple GUI for entering video URLs
- Download progress events (started, complete, error)
