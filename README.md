# Ichimi Server

A powerful process management server for Claude Code via the Model Context Protocol (MCP).

## Features

- 🚀 **Process Management**: Start, stop, and monitor any process
- 📊 **Real-time Logging**: Capture and stream stdout/stderr outputs
- 🔍 **Status Monitoring**: Track process states and metrics
- 🎯 **Flexible Filtering**: List and search processes with filters
- 💾 **Memory Efficient**: Circular buffer for log management
- 🔌 **MCP Native**: Built specifically for Claude Code integration

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/chronista-club/ichimi-server
cd ichimi-server

# Build the server
cargo build --release

# The binary will be available at:
# target/release/ichimi-server
```

### Using Cargo

```bash
cargo install ichimi-server
```

## Configuration

### Claude Code Configuration

Add the server to your `.mcp.json` or Claude Code settings:

```json
{
    "mcpServers": {
        "ichimi": {
            "type": "stdio",
            "command": "ichimi-server",
            "env": {
                "RUST_LOG": "info"
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
│   └── process/
│       ├── mod.rs       # Process module exports
│       ├── manager.rs   # Process lifecycle management
│       ├── buffer.rs    # Circular buffer for logs
│       └── types.rs     # Type definitions
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

## Acknowledgments

- Built with [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Rust MCP SDK
- Inspired by the Model Context Protocol specification
- Part of the Chronista Club ecosystem

## Support

For issues, questions, or suggestions:
- Open an issue on [GitHub](https://github.com/chronista-club/ichimi-server/issues)
- Check the [documentation](https://github.com/chronista-club/ichimi-server/wiki)

---

*Ichimi Server - Making process management simple and powerful for Claude Code*