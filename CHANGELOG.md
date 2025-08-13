# Changelog

All notable changes to claude-utils will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-08-12

### Fixed
- Fixed compilation error in file_manager module with elapsed time handling
- Fixed unused import warnings across multiple modules
- Fixed MCP server initialization to use Arc-wrapped managers
- Fixed all clippy warnings and improved code quality
- Applied proper code formatting with rustfmt

### Changed
- Updated dependencies to latest versions
- Improved error handling consistency

## [0.1.0] - 2025-01-10

### Added
- Initial release of claude-utils
- MCP server implementation for Claude Code integration
- Clipboard watching mode (`--watch`) for automatic image-to-path conversion
- Smart file staging with SHA-256 deduplication
- Desktop symlink creation with timestamping
- Dual clipboard format support (path for terminals, image for other apps) on macOS
- Token-based authentication for security
- Comprehensive CLI with subcommands
- Cross-platform support (macOS, Linux, Windows)
- System notifications for clipboard events
- Automatic cleanup of old staged files
- Integration tests and documentation

### Features
- `claude-utils start` - Start the MCP daemon
- `claude-utils start --watch` - Enable clipboard watching mode
- `claude-utils clip get` - Get current clipboard content
- `claude-utils clip paste` - Output clipboard content/path
- `claude-utils token` - Display authentication token
- `claude-utils config` - Generate MCP configuration

### Known Limitations
- Dual clipboard format only partially implemented on macOS
- Terminal paste interception (⌘V hijacking) not yet implemented
- Requires explicit Claude Code queries rather than automatic paste detection