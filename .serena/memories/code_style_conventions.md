# Code Style and Conventions

## Rust Code Style

### General Principles
- Follow standard Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Use `rustfmt` for automatic formatting
- Follow `clippy` recommendations
- Prefer explicit error handling with `Result<T, E>`
- Use `async/await` for asynchronous operations

### Project-Specific Patterns

#### Arc<RwLock> Pattern
- Used for thread-safe concurrent access to shared state
- ProcessManager uses `Arc<RwLock<HashMap>>` for process management
- Each process wrapped in `Arc<RwLock>` for fine-grained locking

#### State Machine Pattern
- Process states: `NotStarted` → `Running` → `Stopped`/`Failed`
- State transitions are atomic with timestamps

#### Error Handling
- Use `anyhow` for application errors
- Use `thiserror` for library errors
- All process operations return `Result<T, String>`
- MCP errors use `ErrorCode::INTERNAL_ERROR`

#### Module Organization
```
crates/
├── vantage/           # Main server crate
│   ├── src/
│   │   ├── lib.rs    # Core server with MCP tools
│   │   ├── bin/      # Binary entry points
│   │   ├── process/  # Process management
│   │   ├── web/      # Web server
│   │   ├── messages/ # MCP message types
│   │   ├── ci/       # CI/CD monitoring
│   │   └── events/   # Event system
└── vantage-persistence/ # Persistence layer
    ├── src/
    │   ├── lib.rs    # Persistence interface
    │   ├── kdl/      # KDL format
    │   └── surrealdb/ # SurrealDB integration
```

### Documentation
- Use doc comments (`///`) for public APIs
- Include examples in doc comments when helpful
- Document safety requirements for unsafe code

## Frontend Code Style (Vue 3 + TypeScript)

### Vue Components
- Use Single File Components (SFC) with `<script setup>` syntax
- TypeScript for all components
- Composition API preferred over Options API

### TypeScript
- Strict mode enabled
- Define interfaces for all data structures
- Use type inference where appropriate
- Avoid `any` type

### File Organization
```
ui/web/src/
├── components/   # Reusable components
├── views/        # Page components
├── stores/       # Pinia stores
├── api/          # API client
├── types/        # TypeScript types
├── router/       # Vue Router config
└── themes.ts     # Theme configuration
```

### Naming Conventions
- Components: PascalCase (e.g., `ProcessCard.vue`)
- Composables: camelCase with `use` prefix (e.g., `useProcess`)
- Stores: camelCase (e.g., `processStore`)
- Types/Interfaces: PascalCase with `I` prefix for interfaces

## Commit Message Convention
```
type: description

- feat: New feature
- fix: Bug fix
- docs: Documentation changes
- style: Code style changes
- refactor: Code refactoring
- test: Test additions/changes
- chore: Build/tooling changes
```

## Testing Conventions
- Unit tests next to source files
- Integration tests in `tests/` directory
- Use descriptive test names
- Test both success and failure cases