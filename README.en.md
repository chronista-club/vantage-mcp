# Ichimi Server

**English** | [日本語](./README.md)

A powerful process management server for Claude Code via the Model Context Protocol (MCP).

## Features

- 🚀 **Process Management**: Start, stop, and monitor any process
- 📊 **Real-time Logging**: Capture and stream stdout/stderr outputs
- 🔍 **Status Monitoring**: Track process states and metrics
- 🎯 **Flexible Filtering**: List and search processes with filters
- 💾 **Persistence**: Export/import processes with SurrealDB in-memory database
- 🌐 **Web Dashboard**: Optional web UI for browser-based management
- 🔄 **Auto-backup**: Automatic export at configurable intervals
- 🔌 **MCP Native**: Built specifically for Claude Code integration

## Installation

### Quick Install (Recommended)

The easiest way to install Ichimi Server:

```bash
curl -fsSL https://raw.githubusercontent.com/chronista-club/ichimi-server/main/install.sh | bash
```

This script will:
- Detect your platform (macOS/Linux, x86_64/ARM)
- Download the appropriate binary from GitHub releases
- Install to `~/.local/bin`
- Fall back to building from source if needed

### Using Cargo

```bash
cargo install --git https://github.com/chronista-club/ichimi-server --bin ichimi
```

### From Source

```bash
# Clone the repository
git clone https://github.com/chronista-club/ichimi-server
cd ichimi-server

# Build the server
cargo build --release

# Install to PATH (optional)
cargo install --path . --bin ichimi
```

### Manual Download

Download pre-built binaries from the [releases page](https://github.com/chronista-club/ichimi-server/releases):

- `ichimi-linux-x86_64.tar.gz` - Linux x86_64
- `ichimi-linux-aarch64.tar.gz` - Linux ARM64
- `ichimi-macos-x86_64.tar.gz` - macOS Intel
- `ichimi-macos-aarch64.tar.gz` - macOS Apple Silicon

Extract and move to your PATH:
```bash
tar xzf ichimi-*.tar.gz
sudo mv ichimi /usr/local/bin/
```

## Configuration

### Claude Code Configuration

Add the server to your `.mcp.json` or Claude Code settings:

```json
{
    "mcpServers": {
        "ichimi": {
            "type": "stdio",
            "command": "ichimi",
            "env": {
                "RUST_LOG": "info",
                "ICHIMI_AUTO_EXPORT_INTERVAL": "300"
            }
        }
    }
}
```

### Verify Connection

In Claude Code, run:
```
/mcp
```

You should see "ichimi" server as "connected".

## Usage

### Available Tools

#### Basic Tools
- `echo` - Echo back messages for testing
- `ping` - Simple health check
- `get_status` - Get server status and uptime

#### Process Management
- `create_process` - Register a new process configuration
- `start_process` - Start a registered process
- `stop_process` - Stop a running process gracefully
- `get_process_status` - Get detailed process status
- `get_process_output` - Retrieve process stdout/stderr logs
- `list_processes` - List all managed processes with filters
- `remove_process` - Remove a process from management
- `export_processes` - Export all processes to a .surql file
- `import_processes` - Import processes from a .surql file

### Examples

#### Managing a Web Server

```python
# Register a web server process
create_process(
    id="webserver",
    command="python",
    args=["-m", "http.server", "8000"],
    env={"PYTHONUNBUFFERED": "1"},
    cwd="./public"
)

# Start the server
start_process(id="webserver")

# Check the logs
get_process_output(id="webserver", stream="Both", lines=50)

# Stop gracefully
stop_process(id="webserver", grace_period_ms=5000)
```

#### Running a Database

```python
# Start PostgreSQL
create_process(
    id="postgres",
    command="postgres",
    args=["-D", "/usr/local/var/postgres"],
    env={"PGDATA": "/usr/local/var/postgres"}
)

start_process(id="postgres")

# Monitor status
get_process_status(id="postgres")
```

#### Batch Process Management

```python
# List all running processes
list_processes(filter={"state": "Running"})

# Find specific processes by pattern
list_processes(filter={"name_pattern": "worker"})

# Stop all workers
for process in list_processes(filter={"name_pattern": "worker"}):
    stop_process(id=process["id"])
```

## API Reference

### Process States

- `NotStarted` - Process registered but not yet started
- `Running` - Process is currently running with PID
- `Stopped` - Process terminated normally with exit code
- `Failed` - Process failed with error message

### Output Streams

- `Stdout` - Standard output only
- `Stderr` - Standard error only
- `Both` - Combined stdout and stderr

### Process Filters

- `state` - Filter by process state (Running/Stopped/Failed/All)
- `name_pattern` - Filter by ID pattern (supports wildcards)

## Persistence

### Automatic Backup

Ichimi Server uses an in-memory SurrealDB database for process persistence. Data can be exported/imported to `.surql` files for backup and recovery.

```bash
# Enable automatic export every 5 minutes (300 seconds)
ICHIMI_AUTO_EXPORT_INTERVAL=300 ichimi

# Import data on startup
ICHIMI_IMPORT_FILE=/path/to/backup.surql ichimi

# Default export location
# ~/.ichimi/data/ichimi_export.surql
```

### Manual Export/Import

```python
# Export all processes to a file
export_processes(file_path="/path/to/backup.surql")

# Export to default location
export_processes()

# Import processes from a file
import_processes(file_path="/path/to/backup.surql")
```

## Web Dashboard

Ichimi Server includes an optional web dashboard for browser-based management.

### Accessing the Dashboard

```bash
# Start with web dashboard (default port 12700)
ichimi --web

# Specify custom port
ichimi --web --web-port 8080
```

Then open your browser to `http://localhost:12700`

### Dashboard Features

- Real-time process status monitoring
- Start/stop processes with one click
- View process logs (stdout/stderr)
- Search and filter processes
- Responsive design with Tabler UI

## Development

### Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Project Structure

```
ichimi-server/
├── src/
│   ├── lib.rs           # Core server implementation
│   ├── bin/
│   │   └── ichimi_server.rs # Binary entry point
│   ├── process/
│   │   ├── mod.rs       # Process module exports
│   │   ├── manager.rs   # Process lifecycle management
│   │   ├── buffer.rs    # Circular buffer for logs
│   │   └── types.rs     # Type definitions
│   ├── web/
│   │   ├── mod.rs       # Web server module
│   │   └── server.rs    # Dashboard HTTP server
│   ├── messages/
│   │   ├── mod.rs       # Message types
│   │   └── process.rs   # Process-related messages
│   └── persistence.rs   # SurrealDB persistence layer
├── static/              # Web dashboard assets
│   ├── index.html       # Dashboard UI
│   └── favicon.ico      # Icon
├── examples/            # Usage examples
└── tests/              # Integration tests
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|  
| `RUST_LOG` | Log level (error, warn, info, debug, trace) | info |
| `ICHIMI_AUTO_EXPORT_INTERVAL` | Auto-export interval in seconds | - |
| `ICHIMI_IMPORT_FILE` | File to import on startup | - |
| `ICHIMI_DATA_DIR` | Directory for data files | ~/.ichimi/data |

## Acknowledgments

- Built with [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Rust MCP SDK
- Database powered by [SurrealDB](https://surrealdb.com/) - In-memory document database
- UI framework: [Alpine.js](https://alpinejs.dev/) & [Tabler](https://tabler.io/)
- Inspired by the Model Context Protocol specification
- Part of the Chronista Club ecosystem

## Support

For issues, questions, or suggestions:
- Open an issue on [GitHub](https://github.com/chronista-club/ichimi-server/issues)
- Check the [documentation](https://github.com/chronista-club/ichimi-server/wiki)

---

*Ichimi Server - Making process management simple and powerful for Claude Code*