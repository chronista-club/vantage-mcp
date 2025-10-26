# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-beta1] - 2025-08-22

### Changed
- **BREAKING**: Removed SurrealDB dependency for simpler in-memory persistence
- Web dashboard is now enabled by default
- Browser automatically opens when web dashboard starts
- Significantly reduced binary size and startup time

### Added
- `--no-web` option to disable web dashboard
- `--no-open` option to prevent automatic browser opening
- Improved CLI argument handling with proper help and version commands

### Removed
- SurrealDB dependency and all related code
- Complex ORM model layer (replaced with simple HashMap storage)
- `.surql` export format (now uses JSON)

## [0.1.0-alpha1] - 2025-08-22

### Added
- Initial alpha release of Vantage MCP
- Core process management functionality (start, stop, monitor)
- Real-time stdout/stderr capture with circular buffers
- SurrealDB-based persistence with in-memory storage
- Export/import functionality for process definitions
- Model trait and ORM-like database operations
- MCP (Model Context Protocol) server implementation
- Process filtering and search capabilities
- Auto-export functionality with configurable intervals
- Environment variable and working directory support
- Graceful shutdown with configurable grace periods
- Comprehensive test suite

### Technical
- Built with Rust for performance and safety
- Uses SurrealDB for flexible data persistence
- Implements SCHEMALESS tables for maximum flexibility
- Async/await throughout for efficient I/O handling
- Tokio runtime for concurrent operations

### Installation
- Multiple installation methods supported:
  - Quick install script
  - Homebrew formula
  - Cargo install
  - Pre-built binaries for multiple platforms
  - Source build

### Known Issues
- Web dashboard feature is still in development
- Some edge cases in process state recovery after restart

## [Unreleased]

### Planned
- Web dashboard with real-time updates
- Process resource monitoring (CPU, memory)
- Process groups and dependencies
- Scheduled process execution
- Remote process management
- Enhanced security features