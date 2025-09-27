# Ichimi Server Project Overview

## Project Purpose
Ichimi Server is a powerful process management server for Claude Code via the Model Context Protocol (MCP). It enables Claude to manage processes as resources, providing capabilities for starting, stopping, monitoring processes, and capturing their outputs.

## Tech Stack

### Backend (Rust)
- **Language**: Rust (edition 2024)
- **MCP SDK**: rmcp (Model Context Protocol Rust SDK)
- **Async Runtime**: Tokio
- **Web Framework**: Axum
- **Serialization**: Serde, Serde JSON
- **Persistence**: 
  - KDL format for configuration files
  - SurrealDB for advanced persistence
- **Error Handling**: anyhow, thiserror
- **Logging**: tracing, tracing-subscriber

### Frontend (Web Dashboard)
- **Framework**: Vue 3 with TypeScript
- **Build Tool**: Vite
- **State Management**: Pinia
- **Router**: Vue Router
- **UI Framework**: Tabler
- **Package Manager**: Bun

## Key Features
- Process lifecycle management (create, start, stop, delete)
- Real-time log capture (stdout/stderr)
- KDL format persistence (`.ichimi/processes.kdl`)
- Auto-start processes with `auto_start` flag
- Web dashboard with modern Vue 3 SPA
- MCP-compliant server with 12+ tools
- CI/CD monitoring via GitHub Actions
- Clipboard management
- Learning engine with LIVE QUERY

## Version
Current version: 0.1.0-beta19 (moving towards 0.1.0 stable)