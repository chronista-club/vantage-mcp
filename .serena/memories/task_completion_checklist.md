# Task Completion Checklist

When completing a task in the Ichimi Server project, ensure you follow these steps:

## Before Marking Task Complete

### 1. Code Quality Checks
```bash
# Format code
cargo fmt

# Run linter (must pass)
cargo clippy -- -D warnings

# Check formatting
cargo fmt -- --check
```

### 2. Testing
```bash
# Run all tests
cargo test

# For integration changes, run specific tests
cargo test test_integration
cargo test test_persistence
```

### 3. Build Verification
```bash
# Ensure debug build works
cargo build

# Ensure release build works
cargo build --release
```

### 4. Frontend Checks (if UI changes)
```bash
cd ui/web

# Type checking
bun run type-check

# Build production bundle
bun run build
```

### 5. Documentation Updates
- Update README.md if features changed
- Update docs/README.ja.md for Japanese documentation
- Update .claude/CLAUDE.md if architecture changed
- Add/update code comments for complex logic

### 6. Commit Guidelines
- Stage all changes: `git add -A`
- Use conventional commit message format
- Include breaking changes in commit body if applicable
- Reference issue numbers if applicable

### 7. Pre-Push Checklist
- [ ] All tests pass
- [ ] Code is formatted
- [ ] Linter has no warnings
- [ ] Documentation is updated
- [ ] Commit messages are clear
- [ ] No debug code or console.log statements
- [ ] No hardcoded values or secrets

## Special Considerations

### For Version Updates
1. Update version in `Cargo.toml`
2. Update version badges in README files
3. Create git tag: `git tag -a v0.1.0-betaXX -m "Release message"`

### For API Changes
1. Update REST API documentation in README
2. Test all API endpoints
3. Ensure backward compatibility or document breaking changes

### For Persistence Changes
1. Test KDL file format compatibility
2. Test import/export functionality
3. Verify auto-save/restore works

### For MCP Tool Changes
1. Update tool list in documentation
2. Test tool with Claude Code
3. Verify tool help text is clear

## Final Verification
```bash
# Clean build from scratch
cargo clean
cargo build --release
cargo test
```

If all checks pass, the task can be considered complete!