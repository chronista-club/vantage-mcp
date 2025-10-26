# Suggested Commands for Vantage MCP Development

## Build Commands
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check compilation without building
cargo check
```

## Test Commands
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

## Code Quality
```bash
# Format code
cargo fmt

# Check formatting without changes
cargo fmt -- --check

# Run linter
cargo clippy

# Strict linting (fail on warnings)
cargo clippy -- -D warnings
```

## Running the Server
```bash
# Run debug build
cargo run --bin vantage

# Run with web dashboard
cargo run --bin vantage -- --web

# Run with custom port
cargo run --bin vantage -- --web --web-port 8080

# Run web-only mode (no MCP)
cargo run --bin vantage -- --web-only

# Run release build
./target/release/vantage
```

## Environment Variables
```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run with info logging
RUST_LOG=info cargo run

# Run with auto-export
VANTAGE_AUTO_EXPORT_INTERVAL=300 cargo run
```

## Frontend Development (Vue 3)
```bash
# Navigate to UI directory
cd ui/web

# Install dependencies
bun install

# Development server (port 5173)
bun run dev

# Production build
bun run build

# Preview production build
bun run preview
```

## Git Commands
```bash
# Check status
git status

# Stage changes
git add -A

# Commit with message
git commit -m "type: description"

# Push to remote
git push origin main

# Create and push tag
git tag -a v0.1.0-betaXX -m "Release v0.1.0-betaXX"
git push origin v0.1.0-betaXX
```

## GitHub CLI
```bash
# Create release
gh release create v0.1.0-betaXX --title "Title" --notes-file release-notes.md --prerelease

# View CI runs
gh run list

# Watch CI run
gh run watch
```

## Makefile Shortcuts
```bash
# Build debug
make build

# Build release
make build-release

# Run tests
make test

# Format code
make fmt

# Run linter
make lint

# Check formatting
make fmt-check
```

## System Utilities (macOS/Darwin)
```bash
# List files with details
ls -la

# Find files
find . -name "*.rs"

# Search in files (ripgrep recommended)
rg "pattern"

# Process management
ps aux | grep vantage
kill -TERM <pid>

# Port checking
lsof -i :12700
```