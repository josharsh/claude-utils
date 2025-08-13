#!/bin/bash

# Claude-Utils Quick Start Example
echo "🚀 Claude-Utils Quick Start"
echo "=========================="
echo ""

# Check if claude-utils is installed
if ! command -v claude-utils &> /dev/null; then
    echo "❌ claude-utils not found. Please install it first:"
    echo "   cargo install claude-utils"
    echo "   OR"
    echo "   brew tap josharsh/claude-utils && brew install claude-utils"
    exit 1
fi

echo "✅ claude-utils is installed!"
echo ""

# Start the daemon in watch mode
echo "Starting claude-utils in watch mode..."
echo "This will:"
echo "- Monitor your clipboard for images"
echo "- Save them to ~/Desktop with timestamps"
echo "- Convert clipboard to file paths for terminal pasting"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Start with watch mode enabled
claude-utils start --watch --no-auth