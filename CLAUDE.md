# CLAUDE.md

## Tech Stack
- Rust 1.79+ (workspace with 3 crates)
- Cargo workspace resolver = "2"
- Key dependencies: thiserror, serde, tracing
- Testing: criterion (benchmarks), proptest (property testing)

## Project Structure
- `sakurs-core/` - Core Rust library implementing Δ-Stack Monoid algorithm
- `sakurs-cli/` - Command-line interface for batch processing  
- `sakurs-py/` - Python bindings via PyO3
- `docs/ARCHITECTURE.md` - Detailed system architecture and design decisions

## Commands
- `cargo build` - Build all workspace crates
- `cargo test` - Run all tests
- `cargo fmt` - Format code
- `cargo clippy` - Lint and catch common mistakes
- `cargo check` - Fast compilation check

## Code Style & Conventions
- Follow standard Rust naming conventions
- Use `thiserror` for error handling across crates
- Implement `Send + Sync` for parallel processing traits
- Add rustdoc comments for public APIs
- Use workspace dependencies for consistency

## Architecture Notes
- Hexagonal architecture (Ports & Adapters pattern)
- Domain core implements mathematical monoid properties
- Language rules as pluggable traits
- Designed for true parallelism via rayon

## Development Workflow
- ALWAYS run `cargo fmt` and `cargo clippy` before commits
- Read `docs/ARCHITECTURE.md` before modifying core algorithm
- Add tests for new functionality - both unit and property tests
- Benchmark performance-critical changes