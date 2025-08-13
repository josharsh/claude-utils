# Claude-Utils 🚀

[![Crates.io](https://img.shields.io/crates/v/claude-utils.svg)](https://crates.io/crates/claude-utils)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue)](https://github.com/josharsh/claude-utils)

Cross-platform companion toolkit for Anthropic's Claude Code CLI, featuring seamless clipboard integration.


## The Problem

Claude Code users cannot paste screenshots or images directly into the terminal using standard keyboard shortcuts (⌘V/Ctrl+V). Current workarounds require saving files and drag-dropping, breaking the keyboard-centric workflow.

## The Solution

Claude-Utils provides a transparent clipboard bridge that:
- 🎯 **Just Works™**: Press ⌘V/Ctrl+V to paste anything into Claude Code
- 🖼️ **Image Support**: Automatically stages images and provides file paths
- 📋 **Text Support**: Handles text up to 64KB inline
- 🔐 **Secure**: Local-only with token authentication
- 🌍 **Cross-Platform**: macOS, Linux, and Windows support
- ✨ **Watch Mode**: Automatically converts copied images to paths for seamless pasting

## Quick Start

### Installation

```bash
# Install from crates.io (recommended)
cargo install claude-utils

# macOS via Homebrew
brew tap josharsh/homebrew-claude-utils
brew install claude-utils

# Build from source
git clone https://github.com/josharsh/claude-utils
cd claude-utils
cargo build --release
```

### Basic Usage

#### Standard Mode
1. Start the daemon:
```bash
claude-utils start
```

2. Configure Claude Code and set token as shown
3. In Claude Code: "Analyze the image in my clipboard"

#### Watch Mode (Recommended) 🎯
1. Start with watch mode:
```bash
claude-utils start --watch
```

2. Copy any image (screenshot, etc.)
3. Image automatically saved to `~/Desktop/claude-paste.png`
4. Press ⌘V in terminal → path appears!
5. Original image still available for other apps

The magic: When you copy an image, claude-utils:
- Saves it to a timestamped file
- Creates a symlink on your Desktop
- Sets clipboard to the file path (for terminal)
- Preserves original image (for other apps)

## Features

### Clipboard Bridge
- **Automatic Detection**: Detects content type (text/image)
- **Smart Staging**: Large content automatically saved to temp files
- **Deduplication**: Same content won't be staged twice
- **Auto-Cleanup**: Old files cleaned up after 15 minutes

### MCP Server
- Full Model Context Protocol implementation
- JSON-RPC 2.0 compliant
- Server-Sent Events (SSE) support for real-time updates
- Tools: `clipboard.get`, `clipboard.set`

### Security
- Runs on localhost only (127.0.0.1:3830)
- Token-based authentication
- Read-only by default (use `--write` flag for clipboard writes)
- Secure file permissions (0600 for tokens)

## Advanced Usage

### CLI Commands

```bash
# Start with watch mode (recommended!)
claude-utils start --watch

# Watch mode with custom options
claude-utils start --watch --symlink-dir ~/Documents --no-notifications

# Start with custom port
claude-utils start --port 8080

# Disable authentication (development only)
claude-utils start --no-auth

# Enable clipboard write operations
claude-utils start --write

# Custom staging directory
claude-utils start --staging-dir /path/to/staging

# Show authentication token
claude-utils token

# Quick clipboard operations
claude-utils clip get              # Get clipboard as JSON
claude-utils clip get --format text # Get as plain text
claude-utils clip paste            # Paste (outputs file path for images)
```

### Watch Mode Options

```bash
--watch              # Enable clipboard watching
--symlink-dir PATH   # Where to create symlinks (default: ~/Desktop)
--no-dual-format     # Disable dual clipboard format (macOS)
--no-notifications   # Disable system notifications
```

### MCP Integration

The server exposes these tools for Claude Code:

```json
{
  "tools": [
    {
      "name": "clipboard.get",
      "description": "Get current clipboard content",
      "input_schema": {
        "type": "object",
        "properties": {
          "format": {
            "type": "string",
            "enum": ["auto", "text", "image"]
          }
        }
      }
    },
    {
      "name": "clipboard.set",
      "description": "Set clipboard content (requires --write flag)",
      "input_schema": {
        "type": "object",
        "properties": {
          "type": {"type": "string"},
          "data": {"type": "string"}
        }
      }
    }
  ]
}
```

## Architecture

```
Claude Code <---> MCP Client <---> claude-utils-clipd <---> System Clipboard
                     |                     |
                 JSON-RPC              File Staging
                  + SSE                    |
                                    /tmp/claude-utils/
```

## Platform Notes

### macOS
- Full support for text and images
- Uses native NSPasteboard API

### Linux
- X11 and Wayland support (via arboard)
- May require `xclip` or `wl-clipboard` packages

### Windows
- Native Win32 clipboard API
- WSL2 users: Bridges Windows clipboard to WSL

## Building from Source

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/josharsh/claude-utils
cd claude-utils
cargo build --release

# Binary will be at ./target/release/claude-utils
```

## Troubleshooting

### "Authentication required" error
- Ensure `CLAUDE_UTILS_TOKEN` environment variable is set
- Check token with: `claude-utils token`

### Images not pasting
- Check if daemon is running: `curl http://localhost:3830/health`
- Verify clipboard has image content: `claude-utils clip get`

### Permission denied errors
- Ensure staging directory is writable
- Check file permissions on `~/.claude-utils/auth.token`

## Future Roadmap

- [ ] Terminal paste interception (the "magic" experience)
- [ ] Screenshot capture endpoint
- [ ] OCR capabilities
- [ ] Clipboard history
- [ ] Multi-agent orchestration support

## Contributing

Contributions welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

## License

MIT License - see [LICENSE](LICENSE) file.

## Acknowledgments

Built with:
- [arboard](https://github.com/1Password/arboard) - Cross-platform clipboard
- [axum](https://github.com/tokio-rs/axum) - Web framework
- [Model Context Protocol](https://modelcontextprotocol.io) - MCP specification

---

Made with ❤️ for the Claude Code community