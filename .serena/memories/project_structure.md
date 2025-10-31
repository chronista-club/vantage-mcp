# Vantage MCP Project Structure

## Repository Layout
```
vantage-mcp/
├── crates/                     # Rust workspace crates
│   ├── vantage/                 # Main server crate
│   │   ├── src/
│   │   │   ├── lib.rs         # Core MCP server implementation
│   │   │   ├── bin/           # Binary entry points
│   │   │   │   └── vantage_server.rs
│   │   │   ├── process/       # Process management
│   │   │   │   ├── manager.rs # ProcessManager core
│   │   │   │   ├── buffer.rs  # Circular buffer for logs
│   │   │   │   ├── protocol.rs
│   │   │   │   └── shell.rs
│   │   │   ├── web/           # Web dashboard server
│   │   │   │   ├── server.rs  # HTTP server
│   │   │   │   ├── handlers.rs # Request handlers
│   │   │   │   ├── api.rs     # API routes
│   │   │   │   └── assets.rs  # Static file serving
│   │   │   ├── messages/      # MCP message types
│   │   │   │   ├── basic.rs
│   │   │   │   ├── process.rs
│   │   │   │   ├── ci.rs
│   │   │   │   └── clipboard.rs
│   │   │   ├── ci/            # CI/CD monitoring
│   │   │   ├── events/        # Event system
│   │   │   ├── learning/      # Learning engine
│   │   │   └── security/      # Security features
│   │   └── tests/             # Integration tests
│   └── vantage-persistence/    # Persistence layer crate
│       ├── src/
│       │   ├── lib.rs         # Persistence interface
│       │   ├── types.rs       # Shared types
│       │   ├── kdl/           # KDL format support
│       │   ├── surrealdb/     # SurrealDB integration
│       │   └── db/            # Database abstraction
│       └── tests/
├── ui/                        # Frontend code
│   └── web/                   # Vue 3 web dashboard
│       ├── src/
│       │   ├── App.vue        # Root component
│       │   ├── main.ts        # Entry point
│       │   ├── components/    # Reusable components
│       │   ├── views/         # Page components
│       │   ├── stores/        # Pinia state management
│       │   ├── api/           # API client
│       │   ├── types/         # TypeScript definitions
│       │   ├── router/        # Vue Router
│       │   └── themes.ts      # Theme configuration
│       ├── dist/              # Production build output
│       ├── package.json
│       ├── tsconfig.json
│       └── vite.config.ts
├── docs/                      # Documentation
│   └── README.ja.md          # Japanese README
├── scripts/                   # Utility scripts
├── examples/                  # Usage examples
├── .github/                   # GitHub Actions workflows
├── .vantage/                   # Runtime data directory
│   ├── processes.kdl         # Process configurations
│   └── snapshot.surql        # Database snapshots
├── .serena/                   # Serena configuration
│   ├── project.yml
│   ├── memories/             # Project memories
│   └── cache/                # Language server cache
├── .claude/                   # Claude-specific docs
│   └── CLAUDE.md
├── Cargo.toml                # Workspace configuration
├── Cargo.lock
├── Makefile                  # Build shortcuts
├── README.md                 # Main documentation (English)
├── LICENSE-MIT
├── LICENSE-APACHE
└── rust-toolchain.toml       # Rust version specification
```

## Key Architectural Components

### 1. MCP Server Layer (`crates/vantage-atom/src/lib.rs`)
- Implements `VantageServer` struct
- Defines all MCP tools with `#[tool]` attributes
- Routes tool calls to appropriate handlers

### 2. Process Management (`crates/vantage-atom/src/process/`)
- `ProcessManager`: Core process lifecycle management
- Thread-safe with `Arc<RwLock<HashMap>>`
- Circular buffer for efficient log storage
- Graceful shutdown with configurable timeout

### 3. Web Dashboard (`crates/vantage-atom/src/web/`)
- Axum-based HTTP server
- RESTful API endpoints
- Static file serving for Vue SPA
- Auto port selection if default port busy

### 4. Persistence Layer (`crates/vantage-atom-persistence/`)
- Abstraction over multiple storage backends
- KDL format for human-readable configs
- SurrealDB for advanced queries
- Auto-save and restore functionality

### 5. Frontend (`ui/web/`)
- Modern Vue 3 with Composition API
- TypeScript for type safety
- Pinia for state management
- Vite for fast development
- Tabler UI components

## Data Flow
1. MCP Client → VantageServer (MCP tools)
2. VantageServer → ProcessManager (process operations)
3. ProcessManager → Persistence Layer (save state)
4. Web API → ProcessManager (HTTP operations)
5. Vue Frontend → Web API (REST calls)

## Configuration Files
- `.mcp.json`: MCP server configuration
- `.vantage/processes.kdl`: Process definitions
- `rust-toolchain.toml`: Rust version (2024 edition)
- `.mise.toml`: Development environment setup
