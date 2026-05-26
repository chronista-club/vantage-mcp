# Vantage MCP

> [!NOTE]
> **This repository has been archived (2026-05-26)**
> Successor: [vantage-point](https://github.com/chronista-club/vantage-point) — vantage-mcp's process management & MCP server have been integrated into vantage-point's Lane/Wing infrastructure as built-in capabilities.
> Conceptual designs (SurrealDB-backed smart features, Agent Orchestration) live on in creo-memories and vantage-point's design docs.

**English** | [日本語](./docs/README.ja.md)

Process as a Resource - Manage processes as resources

A powerful process management server for Claude Code via the Model Context Protocol (MCP).

![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)
![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green)

## ✨ Features

### Core Features
- 🚀 **Process Management**: Control start, stop, and monitoring of any process via MCP tools
- 📊 **Real-time Logging**: Capture and stream stdout/stderr outputs
- 🔍 **Status Monitoring**: Track process states and metrics
- 🎯 **Flexible Filtering**: Search processes by state or pattern
- 💾 **Persistence**: Configuration management in KDL format (`.vantage/processes.kdl`)
- 🔄 **Auto-start**: Automatic process startup with `auto_start` flag

### Web Dashboard
- 🌐 **Modern UI**: Sophisticated SPA with Vue 3 + TypeScript + Tabler
- 📈 **Real-time Updates**: Monitor process states with auto-refresh
- 🔍 **Search Features**: Process search and filtering
- 🌙 **Dark Mode**: Light/dark theme switching
- 📱 **Responsive**: Mobile to desktop support
- 🎯 **Type Safety**: Complete typing with TypeScript
- 📦 **Component-Oriented**: Vue 3 SFC (Single File Component) architecture

### MCP Integration
- 🔌 **MCP Compliant Server**: Fully compliant with Model Context Protocol
- 🤖 **Claude Code Ready**: Direct integration with Claude Code
- 🛠️ **Rich Tools**: 12+ MCP tools provided
- 📡 **Web API**: RESTful API for external integration

## 🚀 Installation

### Using Cargo (Recommended)

```bash
# Install from GitHub repository
cargo install --git https://github.com/chronista-club/vantage-mcp --tag v0.2.0

# Or install latest from main branch
cargo install --git https://github.com/chronista-club/vantage-mcp
```

### From Source

```bash
# Clone the repository
git clone https://github.com/chronista-club/vantage-mcp
cd vantage-mcp

# Release build
cargo build --release

# Binary will be at:
# target/release/vantage
```

## Configuration

### Claude Code Configuration

Add the server to your `.mcp.json` or Claude Code settings:

```json
{
    "mcpServers": {
        "vantage": {
            "type": "stdio",
            "command": "vantage",
            "env": {
                "RUST_LOG": "info",
                "VANTAGE_AUTO_EXPORT_INTERVAL": "300"
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

You should see "vantage" server as "connected".

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
- `export_processes` - Export all processes to a YAML file
- `import_processes` - Import processes from a YAML file

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

## 📝 Persistence

### KDL Configuration Files

Vantage MCP uses [KDL (Cuddly Data Language)](https://kdl.dev/) format for process persistence. Configuration files are automatically saved to `.vantage/processes.kdl`.

#### Example KDL Configuration

```kdl
// Vantage MCP Process Configuration
meta {
    version "1.0.0"
}

// Web server process
process "webserver" {
    command "python"
    args "-m" "http.server" "8000"
    cwd "/path/to/public"
    auto_start #false
}

// Background worker
process "worker" {
    command "/usr/local/bin/worker"
    args "--config" "worker.conf"
    cwd "/app"
    auto_start #true  // Auto-start on server launch
}
```

#### Configuration Fields

| Field | Description | Required |
|-------|-------------|----------|
| `command` | Path to executable | ✅ |
| `args` | Command line arguments (multiple allowed) | ❌ |
| `cwd` | Working directory | ❌ |
| `auto_start` | Auto-start on server launch | ❌ |

### YAML Export/Import

Process configurations can be exported/imported in YAML format for backup and migration:

```bash
# Export processes to YAML file
curl http://127.0.0.1:12700/api/export > vantage_export.yaml

# Import processes from YAML file
curl -X POST http://127.0.0.1:12700/api/import \
  -H "Content-Type: application/yaml" \
  -d @vantage_export.yaml
```

## 🌐 Web Dashboard

### Starting the Dashboard

```bash
# Start server (Web dashboard automatically enabled on port 12700)
vantagemcp

# Don't open browser automatically
vantagemcp --no-open
```

The web dashboard will be available at `http://localhost:12700` (or another port if 12700 is in use)

### Dashboard Features

#### Main Screen
- **Stats Cards**: Display total processes, running, stopped, and error states
- **Process List**: Table view of all processes
- **Real-time Updates**: Auto-refresh every 5 seconds
- **Search**: Search by process ID or command

#### Process Operations
- **Start/Stop**: One-click process control
- **Log Viewing**: Display latest stdout/stderr logs
- **Delete**: Remove unwanted processes
- **Add New**: Modal dialog for process creation

#### UI/UX
- **Responsive Design**: Mobile-friendly
- **Dark Mode**: Light/dark theme switching
- **Modern Design**: Tabler UI framework

### REST API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/status` | GET | Server status |
| `/api/dashboard` | GET | Dashboard stats |
| `/api/processes` | GET | List processes |
| `/api/processes` | POST | Add process |
| `/api/processes/:id` | GET | Process details |
| `/api/processes/:id` | DELETE | Delete process |
| `/api/processes/:id/start` | POST | Start process |
| `/api/processes/:id/stop` | POST | Stop process |
| `/api/processes/:id/logs` | GET | Get logs |

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
vantage-mcp/
├── crates/
│   ├── vantage/                 # Main server crate
│   │   ├── src/
│   │   │   ├── lib.rs          # Core server implementation
│   │   │   ├── bin/
│   │   │   │   └── vantage_server.rs  # Binary entry point
│   │   │   ├── process/        # Process management
│   │   │   │   ├── mod.rs
│   │   │   │   ├── manager.rs
│   │   │   │   ├── buffer.rs
│   │   │   │   └── protocol.rs
│   │   │   ├── web/            # Web server
│   │   │   │   ├── server.rs
│   │   │   │   ├── handlers.rs
│   │   │   │   └── api.rs
│   │   │   ├── messages/       # MCP message types
│   │   │   ├── ci/             # CI/CD monitoring
│   │   │   └── events/         # Event system
│   │   └── tests/
│   └── vantage-persistence/     # Persistence layer
│       ├── src/
│       │   ├── lib.rs          # Persistence interface
│       │   ├── kdl/            # KDL format persistence
│       │   ├── persistence/    # In-memory storage implementation
│       │   └── yaml/           # YAML snapshot export/import
│       └── tests/
├── ui/
│   └── web/                    # Vue 3 SPA
│       ├── src/
│       │   ├── App.vue         # Root component
│       │   ├── main.ts         # Entry point
│       │   ├── router/         # Vue Router config
│       │   ├── stores/         # Pinia stores
│       │   ├── components/     # Vue components
│       │   ├── views/          # Page components
│       │   ├── api/            # API client
│       │   ├── types/          # TypeScript types
│       │   └── themes.ts       # Theme configuration
│       ├── package.json
│       ├── tsconfig.json
│       └── vite.config.ts
│       ├── dist/               # Production build
├── .vantage/                    # Data directory
│   └── processes.kdl           # Process config file
└── examples/                   # Usage examples
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

## 🔑 Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|  
| `RUST_LOG` | Log level (error, warn, info, debug, trace) | info |
| `VANTAGE_DATA_DIR` | Directory for data files | ~/.vantage/data |
| `VANTAGE_IMPORT_FILE` | File to import on startup | ~/.vantage/data/processes.yaml |
| `VANTAGE_EXPORT_FILE` | Export destination on shutdown | ~/.vantage/data/processes.yaml |
| `VANTAGE_STOP_ON_SHUTDOWN` | Stop processes on vantage exit (true/false) | false (continue) |
| `VANTAGE_AUTO_EXPORT_INTERVAL` | Auto-export interval in seconds | - |

## 🙏 Acknowledgments

- [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Rust MCP SDK
- [Tera](https://tera.netlify.app/) - Template engine
- UI framework: [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/) + [Tabler](https://tabler.io/)
- [KDL](https://kdl.dev/) - Configuration format
- Inspired by the Model Context Protocol specification
- Part of the Chronista Club ecosystem

## Support

For issues, questions, or suggestions:
- Open an issue on [GitHub](https://github.com/chronista-club/vantage-mcp/issues)
- Check the [documentation](https://github.com/chronista-club/vantage-mcp/wiki)

---

*Vantage MCP - Making process management simple and powerful for Claude Code*
