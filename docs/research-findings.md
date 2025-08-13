# Claude Code Research Findings

## Executive Summary

This document contains comprehensive research findings about Claude Code's architecture, MCP integration, and the clipboard functionality gap that Claude-Utils aims to solve.

## 1. Claude Code Architecture Overview

### Core Functionality
- **Terminal-first design**: Claude Code is an "agentic coding tool that lives in your terminal"
- **Direct action capability**: Takes action by editing files, running commands, creating commits
- **Unix philosophy**: Composable, scriptable, and pipeable
- **Agent-based architecture**: Follows an implicit Think-Act-Observe loop

### Key Capabilities
- Build features from descriptions by creating plans and writing code
- Debug and fix issues by analyzing codebases
- Navigate any codebase with contextual awareness
- Automate tedious tasks like fixing lint issues or resolving merge conflicts

### Installation and Basic Usage
```bash
# Install Claude Code
npm install -g @anthropic-ai/claude-code

# Navigate to project
cd your-awesome-project

# Start coding
claude
```

## 2. CLI REPL Functionality

### Command Structure
- `claude`: Starts an interactive Read-Eval-Print Loop (REPL)
- `claude "explain this project"`: Start with an initial prompt
- `claude -p "explain this function"`: Print mode - query and exit
- `cat logs.txt | claude -p "explain"`: Supports piping content

### Key Flags
- `--model`: Select specific model (e.g., "sonnet" or "opus")
- `--verbose`: Enable detailed logging
- `--max-turns`: Limit number of interaction turns
- `--output-format`: Control response format (text, JSON, stream)
- `-c`: Continue previous conversations
- `-r`: Resume specific sessions

## 3. Model Context Protocol (MCP) Integration

### What is MCP?
MCP (Model Context Protocol) is an open protocol standard that enables unified context interaction between AI models and development environments. Think of it as the "USB-C for AI" - providing a universal way for AI models to connect to different tools and services.

### MCP Server Capabilities
1. **Resources**: File-like data that can be read by clients
2. **Tools**: Functions that can be called by the LLM (with user approval)
3. **Prompts**: Pre-written templates for specific tasks

### Configuration Methods

#### CLI Commands
```bash
# Basic syntax
claude mcp add <name> <command> [args...]

# Example: Adding a local server
claude mcp add my-server -e API_KEY=123 -- /path/to/server arg1 arg2

# Remote servers with SSE
claude mcp add --transport sse sse-server https://example.com/sse-endpoint
```

#### Project-Scoped Configuration
```json
// .mcp.json at project root
{
  "my-server": {
    "command": "/path/to/server",
    "args": ["arg1", "arg2"],
    "env": {
      "API_KEY": "${API_KEY}"
    }
  }
}
```

### Authentication
- Supports OAuth 2.0 for secure connections
- Native OAuth flow eliminates manual API key management
- Environment variable discovery: `CLAUDE_UTILS_PORT`

### MCP Prompts as Slash Commands
- Type `/` to see all available commands
- Format: `/mcp__servername__promptname`

## 4. Agent Architecture: Think-Act-Observe Loop

### Evidence of TAO Pattern
While not explicitly documented, Claude Code demonstrates agent-like behavior:

1. **Iterative Problem-Solving**: Can loop through tasks programmatically
2. **Tool-Calling Capabilities**: Direct file editing, command execution, git operations
3. **Feedback Loops**: Autonomous assessment and retry capabilities
4. **Planning and Execution**: Research → Plan → Execute pattern

### Implementation Details
- **Research Phase**: Gathering context before action
- **Planning Phase**: Breaking down complex tasks
- **Execution Phase**: Taking concrete actions
- **Observation Phase**: Evaluating results and adjusting

### Sub-Agent Capabilities
- `Task` tool can spin off sub-agents with same tool access
- Enables parallel exploration and complex workflows

## 5. The Clipboard Problem

### Verified GitHub Issues
- **Issue #1361**: Can't paste image from clipboard (Linux bug)
- **Issue #834**: Ctrl+V doesn't work in TUI (Ubuntu 22.04)
- **Issue #618**: Image analysis requires web interface (macOS)
- **Issue #2102**: 90% failure rate on macOS clipboard paste

### Current Pain Points
1. **No direct paste support**: Users cannot use ⌘/Ctrl+V for images
2. **Platform inconsistencies**: Different behavior across OS
3. **Workflow disruption**: Requires saving files and drag-drop
4. **Terminal limitations**: TUI doesn't handle rich content

### Existing Workarounds

#### Official (macOS)
- Use `cmd+ctrl+shift+4` to screenshot to clipboard
- Press `ctrl+v` (not `cmd+v`) to paste
- Note: Doesn't work remotely

#### Community Solutions
- Save screenshots to files first
- Drag and drop into prompt
- Use Puppeteer MCP server for browser screenshots
- Windows-to-WSL2 bridge projects

## 6. Existing MCP Clipboard Servers

### @standardbeagle/mcp-clip
- **Strengths**: 
  - Supports both text and images
  - High-performance, lock-free design
  - Cross-platform support
- **Limitations**:
  - WSL2-focused implementation
  - Not seamlessly integrated with paste action
  - Requires explicit MCP configuration

### Gap Analysis
- No unified cross-platform solution
- No transparent paste interception
- Most require explicit commands or configuration
- Platform-specific implementations dominate

## 7. Key Integration Points for Claude-Utils

### Discovery Mechanisms
1. Environment variables: `CLAUDE_UTILS_PORT`, `CLAUDE_UTILS_TOKEN`
2. Default port conventions (e.g., 3830)
3. Configuration files: `~/.claude-utils/config.json`
4. MCP server registration via `.mcp.json`

### API Patterns
- Health endpoint: `GET /health`
- MCP tool endpoints follow `/tools/{name}` pattern
- SSE for real-time updates
- JSON-RPC for command execution

### Security Considerations
- Local-only by default (127.0.0.1)
- Token-based authentication
- Read-only operations by default
- Explicit flags for write operations

## 8. Developer Workflow Insights

### Current Workflow
1. Take screenshot or copy content
2. Save to file (disrupts flow)
3. Navigate to file location
4. Drag into Claude Code or reference path
5. Claude processes the file

### Desired Workflow
1. Copy/screenshot content
2. Press ⌘/Ctrl+V in Claude Code
3. Claude immediately processes content

### Impact
- Reduces 5 steps to 2
- Maintains keyboard-only flow
- Eliminates file management overhead
- Enables rapid iteration on UI/visual tasks

## Conclusion

Claude Code's architecture is well-suited for clipboard integration via MCP. The main challenges are:
1. Terminal paste interception across platforms
2. Transparent file staging for large content
3. Seamless discovery and authentication
4. Maintaining the "it just works" experience

Claude-Utils can fill this gap by providing a daemon that bridges system clipboard to Claude Code via MCP, with smart handling of both text and rich content.