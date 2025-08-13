# Watch Mode Architecture

## Overview

Watch mode transforms claude-utils from a passive MCP server into an active clipboard monitor that enables true ⌘V paste functionality in terminals.

## How It Works

### 1. Clipboard Monitoring
```
ClipboardWatcher (500ms poll) → Detects new content → Emits ClipboardEvent
```

- Polls clipboard every 500ms (configurable)
- Calculates SHA-256 hash to detect changes
- Ignores duplicate content
- Handles both text and images

### 2. Content Processing
```
ClipboardProcessor → Stages files → Creates symlinks → Updates clipboard
```

When an image is detected:
1. Saves to `/tmp/claude-utils/clip-[hash].png`
2. Creates symlink `~/Desktop/claude-paste-[timestamp].png`
3. Creates "latest" symlink `~/Desktop/claude-paste.png`
4. Updates clipboard with dual format (macOS) or path (other OS)

### 3. Dual Clipboard Format (macOS)

On macOS, we use NSPasteboard to set multiple representations:
- `public.utf8-plain-text`: File path for terminals
- `public.png`: Original image for image apps

This means:
- Terminal apps (when you press ⌘V) → Get the file path
- Image apps (Preview, Photoshop) → Get the original image

### 4. User Experience

```
User copies image → 
  claude-utils detects (within 500ms) →
    Saves & creates symlink →
      Updates clipboard →
        User presses ⌘V →
          Path appears in terminal!
```

## Implementation Details

### Platform-Specific Code

**macOS** (`platform::DualClipboard`):
- Uses objc/cocoa crates for NSPasteboard access
- Sets multiple pasteboard types simultaneously
- Preserves original image data

**Linux/Windows** (fallback):
- Sets text-only clipboard with file path
- Could be enhanced with X11/Win32 specific code

### File Management

- **Staging**: `/tmp/claude-utils/clip-[8-char-hash].[ext]`
- **Symlinks**: `~/Desktop/claude-paste-YYYYMMDD-HHMMSS.[ext]`
- **Latest**: `~/Desktop/claude-paste.[ext]` (always points to newest)
- **Cleanup**: Keeps only 5 most recent symlinks

### Performance Considerations

- 500ms polling interval balances responsiveness vs CPU usage
- Content hashing prevents unnecessary processing
- Async architecture prevents blocking
- File deduplication saves disk space

## Configuration Options

```bash
claude-utils start --watch \
  --symlink-dir ~/Documents \      # Custom symlink location
  --no-dual-format \               # Disable dual clipboard (path only)
  --no-notifications               # Disable system notifications
```

## Security

- All files are user-readable only
- Symlinks are created in user-controlled directories
- No network access or external communication
- Original clipboard content preserved

## Known Limitations

1. **Polling-based**: Not event-driven (requires periodic checks)
2. **Platform differences**: Dual format only on macOS currently
3. **Binary content**: Only handles images, not other binary formats
4. **Clipboard conflicts**: May interfere with clipboard managers

## Future Enhancements

1. **Event-based monitoring**: Use OS clipboard change notifications
2. **More formats**: Support PDFs, videos, etc.
3. **Smart naming**: Detect image content for better filenames
4. **Clipboard history**: Keep last N clipboard items
5. **Terminal integration**: Direct PTY injection (ultimate goal)