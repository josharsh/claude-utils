#!/bin/bash

# Claude-Utils Installation Script
set -e

echo "🚀 Claude-Utils Installer"
echo "========================"

# Detect OS
OS="unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    OS="windows"
fi

echo "Detected OS: $OS"

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust found: $(rustc --version)"
fi

# Install system dependencies
if [[ "$OS" == "linux" ]]; then
    echo "Installing Linux dependencies..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y libxcb-xfixes0-dev libxcb-shape0-dev
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y libxcb-devel
    fi
fi

# Install method selection
if [[ "$1" == "--build" ]] || [[ ! -f "Cargo.toml" ]]; then
    # Install from crates.io
    echo "Installing claude-utils from crates.io..."
    cargo install claude-utils
    INSTALL_DIR="$HOME/.cargo/bin"
else
    # Build from source
    echo "Building claude-utils from source..."
    cargo build --release
    
    # Install binary
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    cp target/release/claude-utils "$INSTALL_DIR/"
fi

echo "✅ claude-utils installed to $INSTALL_DIR/claude-utils"

# Add to PATH if needed
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "⚠️  Add $INSTALL_DIR to your PATH:"
    echo ""
    echo "Add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\""
fi

# Start daemon and show token
echo ""
echo "Starting claude-utils daemon..."
"$INSTALL_DIR/claude-utils" start &
DAEMON_PID=$!

# Wait for daemon to start
sleep 2

# Get token
TOKEN=$("$INSTALL_DIR/claude-utils" token 2>/dev/null || echo "")

if [[ -n "$TOKEN" ]]; then
    echo ""
    echo "🔐 Authentication token: $TOKEN"
    echo ""
    echo "Add to your shell profile:"
    echo "export CLAUDE_UTILS_TOKEN=$TOKEN"
    echo ""
    echo "Create .mcp.json in your Claude Code projects:"
    "$INSTALL_DIR/claude-utils" config
fi

# Kill the daemon we started
kill $DAEMON_PID 2>/dev/null || true

echo ""
echo "✅ Installation complete!"
echo ""
echo "Next steps:"
echo "1. Add claude-utils to your PATH (if needed)"
echo "2. Export CLAUDE_UTILS_TOKEN in your shell profile"
echo "3. Run 'claude-utils start' to start the daemon"
echo "4. Add .mcp.json to your Claude Code projects"
echo ""
echo "For more help: claude-utils --help"