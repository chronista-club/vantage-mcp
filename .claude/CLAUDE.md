# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ichimi Server is a process management server for Claude Code via Model Context Protocol (MCP). It allows Claude to start, stop, monitor processes, and capture their output through MCP tools.

## Build and Development Commands

```bash
# Build commands
cargo build           # Debug build
cargo build --release # Release build (optimized)

# Testing
cargo test           # Run all tests
cargo test [test_name] # Run specific test

# Code quality
cargo fmt            # Format code
cargo fmt -- --check # Check formatting without changing files
cargo clippy         # Run linter
cargo clippy -- -D warnings # Fail on warnings

# Run the server
cargo run --bin ichimi-server
./target/release/ichimi-server # Run release build
```

## Architecture

### Module Structure

The codebase is organized into functional modules:

- **`src/lib.rs`**: Main server implementation with MCP tool handlers. Each tool method is decorated with `#[tool]` attribute and maps to an MCP tool exposed to Claude.

- **`src/messages/`**: Request/response message structures
  - `basic.rs`: Simple message types (echo, ping)
  - `process.rs`: Process management request types
  
- **`src/process/`**: Core process management logic
  - `manager.rs`: `ProcessManager` - handles process lifecycle, maintains process registry
  - `buffer.rs`: `CircularBuffer` - memory-efficient log storage with fixed capacity
  - `types.rs`: Domain types (`ProcessState`, `ProcessInfo`, `ProcessStatus`)

### Key Design Patterns

1. **Arc<RwLock> Pattern**: The `ProcessManager` uses `Arc<RwLock<HashMap>>` for thread-safe concurrent access to managed processes. Each process is also wrapped in `Arc<RwLock>` for granular locking.

2. **State Machine**: Processes transition through states: `NotStarted` → `Running` → `Stopped`/`Failed`. State transitions are atomic and include timestamps.

3. **Async Output Capture**: When a process starts, two async tasks are spawned to capture stdout/stderr into circular buffers, preventing memory exhaustion from long-running processes.

4. **Tool Router**: The `#[tool_router]` macro generates MCP tool routing. Tools are async functions that return `CallToolResult`.

## MCP Integration Points

The server exposes these tools to Claude:
- Basic: `echo`, `ping`, `get_status`
- Process Management: `create_process`, `start_process`, `stop_process`, `get_process_status`, `get_process_output`, `list_processes`, `remove_process`

Each tool maps directly to a method in `IchimiServer` impl block in `lib.rs`.

## Process Lifecycle

1. **Create**: Register process configuration (command, args, env, cwd)
2. **Start**: Spawn tokio process, capture PID, start output handlers
3. **Monitor**: Track state, capture stdout/stderr to circular buffers
4. **Stop**: Send SIGTERM, wait for grace period, force kill if needed
5. **Remove**: Clean up process from registry

## Error Handling

- All process operations return `Result<T, String>` 
- Errors are converted to MCP errors with `ErrorCode::INTERNAL_ERROR`
- Process failures are captured in `ProcessState::Failed` with error details

## Testing Considerations

Currently no unit tests exist. When adding tests:
- Mock `tokio::process::Command` for process operations
- Test state transitions in `ProcessManager`
- Verify circular buffer behavior at capacity
- Test concurrent access patterns