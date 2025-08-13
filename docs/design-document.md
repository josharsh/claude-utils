# Claude-Utils Design Document: Clipboard Bridge

## Overview

Claude-Utils is a cross-platform companion toolkit for Anthropic's Claude Code CLI, with the clipboard bridge as its flagship module. This design document outlines the architecture, implementation strategy, and user experience for seamless clipboard integration.

## 1. Architecture Overview

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        Claude Code CLI                       │
│                                                              │
│  ┌─────────────┐   MCP Protocol    ┌──────────────────┐    │
│  │   Claude    │ ◄──────────────►  │  MCP Client      │    │
│  │   Agent     │   JSON-RPC/SSE    │  (Built-in)      │    │
│  └─────────────┘                   └──────────────────┘    │
└────────────────────────────────┬────────────────────────────┘
                                 │
                                 │ HTTP+SSE
                                 │ localhost:3830
                                 │
┌────────────────────────────────▼────────────────────────────┐
│                     claude-utils-clipd                       │
│                                                              │
│  ┌──────────────┐    ┌─────────────┐    ┌──────────────┐  │
│  │ MCP Server   │    │  Clipboard  │    │   File       │  │
│  │ (HTTP+SSE)   │◄──►│  Monitor    │◄──►│  Manager     │  │
│  └──────────────┘    └─────────────┘    └──────────────┘  │
│         ▲                    │                    │         │
│         │                    ▼                    ▼         │
│  ┌──────────────┐    ┌─────────────┐    ┌──────────────┐  │
│  │   Terminal   │    │  Platform   │    │   Staging    │  │
│  │  Interceptor │    │  Adapter    │    │  Directory   │  │
│  └──────────────┘    └─────────────┘    └──────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

1. **MCP Server**: HTTP+SSE server implementing Model Context Protocol
2. **Clipboard Monitor**: Platform-specific clipboard access layer
3. **File Manager**: Handles staging, caching, and cleanup of clipboard files
4. **Terminal Interceptor**: Captures paste events and routes through clipboard bridge
5. **Platform Adapter**: Abstracts OS-specific clipboard APIs

### Key Innovation: Transparent Paste Interception

The breakthrough design element is intercepting native paste keystrokes (⌘V/Ctrl+V) at the terminal level, automatically routing clipboard content through our MCP server without requiring users to learn new commands or change their workflow.

## 2. API Specification

### MCP Server Endpoints

#### Health Check
```http
GET /health
```
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "platform": "darwin|linux|win32",
  "capabilities": ["text", "image", "watch"]
}
```

#### Get Clipboard Content
```http
POST /tools/clipboard.get
```
Request:
```json
{
  "format": "auto|text|image",  // auto detects content type
  "inline_size_limit": 65536    // 64KB for inline text
}
```
Response:
```json
{
  "type": "text/plain|image/png|image/jpeg",
  "data": "base64_or_utf8_string",  // inline if small
  "file": "/tmp/claude-utils/clip-sha256.png",  // if large
  "metadata": {
    "size": 1024,
    "width": 1920,    // for images
    "height": 1080,   // for images
    "timestamp": "2025-01-10T10:30:00Z"
  }
}
```

#### Watch Clipboard Changes (SSE)
```http
POST /tools/clipboard.watch
```
Streams clipboard changes in real-time via Server-Sent Events.

#### Set Clipboard Content
```http
POST /tools/clipboard.set
```
Request:
```json
{
  "type": "text/plain|image/png",
  "data": "base64_or_utf8_string"
}
```
Note: Requires `--write` flag for security.

### MCP Integration Schema

```json
{
  "name": "claude-utils-clipboard",
  "description": "Seamless clipboard integration for Claude Code",
  "tools": [
    {
      "name": "clipboard.get",
      "description": "Get current clipboard content",
      "input": {
        "type": "object",
        "properties": {
          "format": {"type": "string", "enum": ["auto", "text", "image"]}
        }
      }
    },
    {
      "name": "clipboard.paste_intercept",
      "description": "Handle intercepted paste event",
      "input": {
        "type": "object",
        "properties": {
          "terminal": {"type": "string"},
          "timestamp": {"type": "string"}
        }
      }
    }
  ]
}
```

## 3. Terminal Paste Interception Strategy

### The Magic: PTY Injection + Clipboard Bridge

The core innovation involves:
1. Detecting paste sequences at the PTY level
2. Capturing clipboard content before it reaches the terminal
3. Staging large/binary content to files
4. Injecting appropriate content (text or file path) into the terminal

### Platform-Specific Implementation

#### macOS
- **Clipboard API**: `NSPasteboard` via `objc` crate
- **Terminal Detection**: 
  - Terminal.app: Apple Events
  - iTerm2: Custom escape sequences
  - Generic: PTY interception
- **Paste Sequence**: Intercept Cmd+V at NSEvent level

#### Linux
- **Clipboard API**: 
  - X11: `x11-clipboard` or `xcb`
  - Wayland: `wl-clipboard-rs`
- **Terminal Detection**: Bracketed paste mode (`ESC [ 200 ~`)
- **Paste Sequence**: Intercept Ctrl+Shift+V or Ctrl+V

#### Windows
- **Clipboard API**: Win32 Clipboard API
- **Terminal Detection**: Console API hooks
- **WSL2 Special Case**: Bridge Windows clipboard to WSL environment

### Implementation Pseudocode

```rust
pub struct TerminalInterceptor {
    pty_master: PtyMaster,
    clipboard_bridge: ClipboardBridge,
    paste_detector: PasteDetector,
}

impl TerminalInterceptor {
    pub fn intercept_paste(&mut self) -> Result<()> {
        // 1. Detect paste sequences
        match self.paste_detector.detect() {
            PasteEvent::Detected(terminal_type) => {
                // 2. Get clipboard content
                let content = self.clipboard_bridge.get_content()?;
                
                // 3. Process based on content type
                let injection = match content {
                    ClipboardContent::Text(text) if text.len() < 65536 => {
                        // Small text: inject directly
                        InjectionContent::Direct(text)
                    },
                    ClipboardContent::Text(text) => {
                        // Large text: stage to file
                        let path = self.stage_text(&text)?;
                        InjectionContent::FilePath(path)
                    },
                    ClipboardContent::Image(data) => {
                        // Images: always stage to file
                        let path = self.stage_image(&data)?;
                        InjectionContent::FilePath(path)
                    }
                };
                
                // 4. Inject into PTY
                self.inject_to_pty(injection)?;
            },
            PasteEvent::None => {}
        }
        
        Ok(())
    }
}
```

## 4. Smart File Staging

### File Management Strategy

```rust
pub struct FileManager {
    staging_dir: PathBuf,
    cache: LruCache<String, FileMetadata>,
    cleanup_interval: Duration,
}

impl FileManager {
    pub fn stage_image(&self, data: &[u8]) -> Result<StagedFile> {
        // Generate content hash for deduplication
        let hash = sha256(data);
        let extension = detect_image_format(data)?;
        let filename = format!("clip-{}.{}", &hash[..8], extension);
        let path = self.staging_dir.join(&filename);
        
        // Only write if not already cached
        if !path.exists() {
            fs::write(&path, data)?;
            
            // Generate thumbnail for quick preview
            if let Ok(thumb) = generate_thumbnail(data, 256) {
                let thumb_path = path.with_extension("thumb.png");
                thumb.save(&thumb_path)?;
            }
        }
        
        Ok(StagedFile {
            path: path.to_string(),
            size: data.len(),
            format: extension,
            thumbnail: Some(format!("{}.thumb.png", filename))
        })
    }
    
    pub fn cleanup_old_files(&self) -> Result<()> {
        // Remove files older than 15 minutes
        let cutoff = SystemTime::now() - Duration::from_secs(900);
        
        for entry in fs::read_dir(&self.staging_dir)? {
            let entry = entry?;
            if let Ok(metadata) = entry.metadata() {
                if metadata.modified()? < cutoff {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        
        Ok(())
    }
}
```

### Staging Directory Structure
```
$TMPDIR/claude-utils/
├── clip-a1b2c3d4.png      # Full image
├── clip-a1b2c3d4.thumb.png # 256x256 thumbnail
├── clip-e5f6g7h8.jpg      # Another image
└── clip-i9j0k1l2.txt      # Large text file
```

## 5. Security Model

### Authentication

```rust
pub struct AuthManager {
    token_path: PathBuf,
}

impl AuthManager {
    pub fn initialize() -> Result<String> {
        let token_path = dirs::home_dir()
            .unwrap()
            .join(".claude-utils")
            .join("auth.token");
        
        if token_path.exists() {
            // Read existing token
            fs::read_to_string(&token_path)
        } else {
            // Generate new token on first run
            let token = generate_secure_random(32);
            
            // Create directory with restricted permissions
            let dir = token_path.parent().unwrap();
            fs::create_dir_all(dir)?;
            
            // Write token with 0600 permissions
            let mut file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o600)
                .open(&token_path)?;
            
            file.write_all(token.as_bytes())?;
            Ok(token)
        }
    }
}
```

### Security Policies

1. **Read-Only by Default**: No clipboard writes without explicit flag
2. **Local Only**: Bind to 127.0.0.1, no remote access
3. **Token Authentication**: Required for all API calls
4. **Temporary Files**: Auto-cleanup after 15 minutes
5. **Permission Restrictions**: 
   - Config files: 0600 (user read/write only)
   - Staged files: 0644 (user write, others read)

## 6. Installation & Discovery

### Zero-Configuration Setup

```bash
# macOS
brew install claude-utils

# Linux
sudo apt install claude-utils  # Debian/Ubuntu
sudo dnf install claude-utils  # Fedora

# Windows
winget install claude-utils

# Start the daemon
claude-utils start
```

### Auto-Discovery Mechanisms

1. **Default Port**: Always tries 3830 first
2. **Environment Variable**: `CLAUDE_UTILS_PORT`
3. **Config File**: `~/.claude-utils/config.json`
4. **mDNS Broadcast**: `_claude-utils._tcp.local`

### MCP Registration

Automatically generates `.mcp.json`:
```json
{
  "claude-utils": {
    "command": "claude-utils",
    "args": ["clipd", "--port", "3830"],
    "env": {
      "CLAUDE_UTILS_TOKEN": "${CLAUDE_UTILS_TOKEN}"
    }
  }
}
```

## 7. User Experience Flow

### The Seamless Experience

```
1. User copies screenshot (Cmd+Shift+Ctrl+4 on macOS)
2. User focuses Claude Code terminal
3. User presses Cmd+V (native paste)
4. Claude-utils intercepts paste event
5. Detects image in clipboard
6. Stages to /tmp/claude-utils/clip-abc123.png
7. Injects path into terminal: "/tmp/claude-utils/clip-abc123.png"
8. Claude Code reads the file path
9. Claude analyzes image immediately
```

### Fallback Mechanisms

For terminals that don't support interception:
```bash
# Explicit command
claude-utils clip paste

# Or in Claude Code:
"Please analyze the image in my clipboard"
# Claude automatically calls clipboard.get tool
```

## 8. Implementation Plan

### Phase 1: MVP (Days 1-7)
- [ ] Basic MCP server with clipboard.get endpoint
- [ ] Text clipboard support (UTF-8, up to 64KB inline)
- [ ] Image clipboard support (PNG/JPEG via file staging)
- [ ] macOS platform implementation
- [ ] Basic authentication system

### Phase 2: Terminal Integration (Days 8-14)
- [ ] Terminal paste interception for macOS
- [ ] Linux platform support (X11 and Wayland)
- [ ] Windows platform support (including WSL2)
- [ ] Auto-discovery mechanism
- [ ] Performance optimizations

### Phase 3: Polish & Release (Days 15-21)
- [ ] Thumbnail generation for images
- [ ] Clipboard history (last 10 items)
- [ ] Installation packages (brew, apt, winget)
- [ ] Comprehensive documentation
- [ ] Demo video/GIF creation

## 9. Technical Stack

### Core Dependencies
- **Language**: Rust (performance, memory safety, cross-platform)
- **Clipboard**: `arboard` crate (unified clipboard API)
- **Web Framework**: `axum` (async HTTP + SSE support)
- **Terminal**: `crossterm` + platform-specific PTY libraries
- **Image Processing**: `image` crate
- **Serialization**: `serde_json`

### Platform-Specific Dependencies
- **macOS**: `objc`, `cocoa` crates
- **Linux**: `x11-clipboard`, `wl-clipboard-rs`
- **Windows**: `winapi`, `windows-rs`

### Build & Distribution
- **Packaging**: `cargo-bundle` for native installers
- **CI/CD**: GitHub Actions for multi-platform builds
- **Release**: Homebrew tap, APT repository, Windows Package Manager

## 10. Future Expansion Hooks

### Planned Modules
1. **Screenshot Capture**: `/screenshot/capture` endpoint
2. **OCR Service**: `/ocr/analyze` for text extraction
3. **Code Graph**: `/graph/query` for codebase visualization
4. **Multi-Agent**: `/conductor/events` for agent orchestration

### Extension Points
- Plugin system for custom clipboard processors
- Webhook support for clipboard events
- Integration with popular IDEs beyond terminal
- Cloud sync for clipboard history

## Success Metrics

1. **Performance**: <100ms paste interception latency
2. **Reliability**: 99.9% successful paste operations
3. **Adoption**: 50% of Claude Code users within 3 months
4. **User Satisfaction**: "Can't imagine Claude Code without it"

## Conclusion

Claude-Utils with its clipboard bridge module addresses a critical gap in the Claude Code experience. By providing seamless, platform-native clipboard integration, we transform a multi-step workaround into a single, natural action. The architecture is designed for reliability, security, and extensibility, ensuring Claude-Utils becomes an essential companion to Claude Code.