# Setting up Claude-Utils with Claude Code

## Quick Setup

1. **Install and start claude-utils:**
```bash
# Install (once available via package managers)
brew install claude-utils  # macOS
# OR build from source
cargo install --path .

# Start the daemon
claude-utils start
```

2. **Save the authentication token:**
When you first run `claude-utils start`, it will display a token like:
```
Authentication token: a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
Set CLAUDE_UTILS_TOKEN=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6 in your environment
```

Add this to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):
```bash
export CLAUDE_UTILS_TOKEN=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
```

3. **Configure Claude Code:**

In your project directory, create or update `.mcp.json`:
```bash
claude-utils config > .mcp.json
```

Or manually create `.mcp.json`:
```json
{
  "claude-utils": {
    "command": "claude-utils",
    "args": ["start"],
    "env": {
      "CLAUDE_UTILS_TOKEN": "${CLAUDE_UTILS_TOKEN}"
    }
  }
}
```

4. **Test the integration:**

In Claude Code, you can now:
- Copy an image/screenshot to your clipboard
- Type: "Analyze the image in my clipboard"
- Claude will automatically use the `clipboard.get` tool

## Usage Examples

### Analyzing UI Screenshots
```
You: "I just took a screenshot of my app's dashboard. Can you help me improve the layout?"
Claude: *automatically reads clipboard via claude-utils and analyzes the image*
```

### Code Documentation with Diagrams
```
You: "I've copied a system architecture diagram. Can you help me implement this design?"
Claude: *reads the diagram and provides implementation guidance*
```

### Quick Text Sharing
```
You: "I have some log output in my clipboard, can you help debug it?"
Claude: *reads text from clipboard and analyzes the logs*
```

## Advanced Configuration

### Custom Port
If port 3830 is in use:
```bash
# Start on different port
claude-utils start --port 8080

# Update .mcp.json
{
  "claude-utils": {
    "command": "claude-utils",
    "args": ["start", "--port", "8080"],
    "env": {
      "CLAUDE_UTILS_TOKEN": "${CLAUDE_UTILS_TOKEN}",
      "CLAUDE_UTILS_PORT": "8080"
    }
  }
}
```

### Development Mode (No Auth)
For local development only:
```bash
claude-utils start --no-auth
```

### Enable Clipboard Writing
If you want Claude to be able to set your clipboard:
```bash
claude-utils start --write
```

## Verifying Setup

1. **Check daemon health:**
```bash
curl http://localhost:3830/health
```

Should return:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "platform": "darwin",
  "capabilities": ["text", "image", "watch"],
  "auth_required": true
}
```

2. **Test clipboard access:**
```bash
# Copy some text to clipboard, then:
claude-utils clip get
```

3. **In Claude Code:**
Ask Claude to read your clipboard:
```
"What's currently in my clipboard?"
```

## Troubleshooting

### "Tool not found" error
- Ensure `.mcp.json` is in your project root
- Restart Claude Code after adding the configuration

### "Authentication failed"
- Verify `CLAUDE_UTILS_TOKEN` is set correctly
- Run `claude-utils token` to see the current token

### Images not working
- Ensure you have image content in clipboard (not just a file)
- Try `claude-utils clip get` to verify clipboard content
- Check staging directory permissions

## Tips

1. **Keep daemon running**: Add to your system startup scripts
2. **Quick paste**: After copying, just ask Claude about "the image/text in my clipboard"
3. **Multiple projects**: The `.mcp.json` file is per-project, but the daemon is system-wide