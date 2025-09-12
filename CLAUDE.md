# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Shengji is an online implementation of the Chinese trick-taking card game 升级 ("Tractor" or "Finding Friends"). It features a Rust backend with WebSocket support and a React TypeScript frontend with WebAssembly integration.

## Commands

### Development
```bash
# Run frontend in development mode with hot reloading
cd frontend && yarn watch

# Run backend in development mode
cd backend && cargo run --features dynamic

# Full development setup (run in separate terminals)
cd frontend && yarn watch
cd backend && cargo run --features dynamic
```

### Building
```bash
# Build production frontend
cd frontend && yarn build

# Build release backend
cargo build --release

# Full production build
cd frontend && yarn build && cd ../backend && cargo run
```

### Testing
```bash
# Run all Rust tests
cargo test --all

# Run specific Rust test
cargo test test_name

# Run frontend tests
cd frontend && yarn test

# Run frontend tests in watch mode
cd frontend && yarn test --watch
```

### Code Quality
```bash
# Lint TypeScript
cd frontend && yarn lint

# Fix TypeScript lint issues
cd frontend && yarn lint --fix

# Lint Rust
cargo clippy

# Format TypeScript
cd frontend && yarn prettier --write

# Check TypeScript formatting
cd frontend && yarn prettier --check

# Format Rust
cargo fmt --all

# Check Rust formatting
cargo fmt --all -- --check
```

### Type Generation
```bash
# Generate TypeScript types from Rust schemas (run from frontend directory)
cd frontend && yarn types && yarn prettier --write && yarn lint --fix
```

## Architecture

### Rust Workspace Structure
- **backend/**: Axum web server handling WebSocket connections and game API
- **core/**: Game state management, message types, and serialization
- **mechanics/**: Core game logic including bidding, tricks, and scoring
- **storage/**: Storage abstraction layer supporting in-memory and Redis backends
- **frontend/shengji-wasm/**: WebAssembly bindings for client-side game mechanics

### Frontend Structure
- **frontend/src/**: React components and application logic
- **frontend/src/state/**: WebSocket connection and state management
- **frontend/src/ChatMessage.tsx**: In-game chat implementation
- **frontend/src/Draw.tsx**: Card rendering and game board visualization
- **frontend/src/Play.tsx**: Main gameplay component
- **frontend/json-schema-bin/**: Utility for generating TypeScript types from Rust

### Type Safety Strategy
The project maintains type safety between Rust and TypeScript by:
1. Defining types in Rust using serde serialization
2. Generating JSON schemas from Rust types
3. Converting schemas to TypeScript definitions via json-schema-bin
4. Sharing game logic through WebAssembly for client-side validation

### WebSocket Communication
- All game state updates flow through WebSocket connections
- Messages are typed and validated on both client and server
- State synchronization happens automatically via the WebSocketProvider

## Development Notes

### When modifying game mechanics:
1. Update logic in `mechanics/src/`
2. If changing message types, update `core/src/message.rs`
3. Regenerate TypeScript types with `yarn types`
4. Update frontend components to handle new mechanics

### When adding new features:
1. Implement server-side logic in appropriate Rust module
2. Add message types if needed in `core/`
3. Generate TypeScript types
4. Implement UI in React components
5. Ensure WebSocket message handling is updated

### Testing approach:
- Unit test game mechanics in Rust (`mechanics/src/`)
- Integration test API endpoints in `backend/`
- Component testing for React UI elements
- Manual testing for WebSocket interactions and gameplay flow
