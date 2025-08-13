# Setting Up Homebrew Tap for claude-utils

## Quick Fix for "Repository not found" Error

### 1. Create the Tap Repository on GitHub

1. Go to: https://github.com/new
2. Repository name: `homebrew-claude-utils` (MUST be exactly this)
3. Description: "Homebrew tap for claude-utils"
4. Public repository: ✅
5. Initialize with README: ❌ (leave unchecked)
6. Click "Create repository"

### 2. Set Up the Tap Repository Locally

```bash
# Clone the empty repository
cd ~/Documents  # or wherever you want
git clone https://github.com/josharsh/homebrew-claude-utils.git
cd homebrew-claude-utils

# Create the Formula directory
mkdir -p Formula

# Copy the formula (from claude-utils directory)
cp ~/Downloads/NewFolder/josharsh_npm/claude-utils/claude-utils.rb Formula/

# Create a README
cat > README.md << 'EOF'
# homebrew-claude-utils

Homebrew tap for [claude-utils](https://github.com/josharsh/claude-utils).

## Installation

```bash
brew tap josharsh/claude-utils
brew install claude-utils
```

## What is claude-utils?

Cross-platform companion toolkit for Anthropic's Claude Code CLI, featuring seamless clipboard integration.
EOF

# Commit and push
git add .
git commit -m "Add claude-utils formula"
git push origin main
```

### 3. Test Installation

Now the tap should work:

```bash
# Remove failed tap if it exists
brew untap josharsh/claude-utils 2>/dev/null || true

# Add the tap
brew tap josharsh/claude-utils

# Install claude-utils
brew install claude-utils
```

## Alternative: Quick Install Script

If you want to automate this:

```bash
#!/bin/bash
# save as setup-tap.sh

REPO_NAME="homebrew-claude-utils"
GITHUB_USER="josharsh"

# Clone the repository
git clone "https://github.com/${GITHUB_USER}/${REPO_NAME}.git" || {
    echo "❌ Repository not found. Please create it on GitHub first:"
    echo "   https://github.com/new"
    echo "   Name: ${REPO_NAME}"
    exit 1
}

cd "${REPO_NAME}"
mkdir -p Formula
cp ../claude-utils/claude-utils.rb Formula/

# Create README
cat > README.md << 'EOF'
# homebrew-claude-utils

Homebrew tap for [claude-utils](https://github.com/josharsh/claude-utils).

## Installation

```bash
brew tap josharsh/claude-utils
brew install claude-utils
```
EOF

# Commit and push
git add .
git commit -m "Add claude-utils formula v0.1.1"
git push origin main

echo "✅ Tap repository set up successfully!"
echo "Test with: brew tap ${GITHUB_USER}/claude-utils"
```

## Notes

- The repository MUST be named `homebrew-claude-utils` for the tap `josharsh/claude-utils` to work
- The formula MUST be in a `Formula/` directory
- The formula file should be named `claude-utils.rb`

## Current Formula Status

The current formula installs from source using Rust/Cargo. Once you create GitHub releases with pre-built binaries, we can update the formula to download those instead (much faster installation).