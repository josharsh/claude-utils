# Contributing to Claude-Utils

First off, thank you for considering contributing to Claude-Utils! 🎉

## Code of Conduct

Be kind, respectful, and considerate. We're all here to make Claude Code better.

## How Can I Contribute?

### Reporting Bugs

1. Check if the issue already exists
2. Include:
   - OS and version
   - Claude-Utils version (`claude-utils --version`)
   - Steps to reproduce
   - Expected vs actual behavior
   - Error messages/logs

### Suggesting Features

1. Check existing issues/discussions
2. Explain the use case
3. Provide examples
4. Consider implementation approach

### Pull Requests

1. Fork the repo
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for new functionality
4. Ensure all tests pass (`cargo test`)
5. Run clippy (`cargo clippy -- -D warnings`)
6. Format code (`cargo fmt`)
7. Commit with clear message
8. Push and create PR

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR-USERNAME/claude-utils
cd claude-utils

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and test
cargo build
cargo test

# Run locally
cargo run -- start --watch --no-auth
```

## Project Structure

```
claude-utils/
├── src/
│   ├── bin/claude-utils.rs   # CLI entry point
│   ├── clipboard/            # Clipboard management
│   ├── file_manager/         # File staging
│   ├── mcp/                  # MCP protocol implementation
│   └── lib.rs               # Library root
├── tests/                    # Integration tests
└── docs/                     # Documentation
```

## Testing

- Unit tests: In module files
- Integration tests: In `tests/`
- Run all tests: `cargo test`
- Check code quality: `cargo clippy`

## Commit Messages

Follow conventional commits:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `chore:` Maintenance
- `test:` Tests
- `refactor:` Code restructuring

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create PR to `main`
4. After merge, maintainer creates release tag

## Questions?

Open an issue or discussion. We're happy to help!