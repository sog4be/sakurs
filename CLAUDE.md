# CLAUDE.md

## Important Project Guidelines

Before making any changes, please review:
- **CONTRIBUTING.md** - Git workflow, branch naming, and development guidelines
- **.github/PULL_REQUEST_TEMPLATE.md** - PR checklist and required information

When asked to create PRs or commits, always reference these documents to ensure compliance with project standards.

## Tech Stack
- Rust 1.81+ (workspace with 3 crates)
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
- **MANDATORY**: Run CI verification commands before every commit:
  ```bash
  cargo fmt --all -- --check
  cargo clippy --workspace -- -D warnings  
  cargo test --workspace
  cargo check --workspace
  ```
- ALWAYS run `cargo fmt` and `cargo clippy` before commits
- Read `docs/ARCHITECTURE.md` before modifying core algorithm
- Add tests for new functionality - both unit and property tests
- Benchmark performance-critical changes
- Follow branch naming conventions from CONTRIBUTING.md
- Use conventional commit format as described in CONTRIBUTING.md

## CI Verification Commands
**Run these exact commands before committing to avoid CI failures:**
```bash
# Format check (matches CI exactly)
cargo fmt --all -- --check

# Lint with warnings as errors (matches CI)
cargo clippy --workspace -- -D warnings

# Test all packages
cargo test --workspace

# Compilation check
cargo check --workspace

# Test coverage (optional but recommended)
make coverage
```

## Test Coverage
The project uses `cargo-llvm-cov` for accurate test coverage reporting:

```bash
# Generate coverage report with summary
make coverage

# Generate and open HTML coverage report
make coverage-html

# Clean coverage data
make coverage-clean
```

Coverage reports are automatically generated in CI and displayed in GitHub Actions job summaries.

## Pull Request Guidelines
When creating or helping with PRs:
1. Always use the PR template structure from `.github/PULL_REQUEST_TEMPLATE.md`
2. Ensure all checklist items are addressed
3. Follow the commit message conventions (feat:, fix:, docs:, etc.)
4. Include the AI attribution footer in commits:
   ```
   🤖 Generated with [Claude Code](https://claude.ai/code)
   
   Co-Authored-By: Claude <noreply@anthropic.com>
   ```

## Git Workflow
- Branch from `main` for new features
- Use descriptive branch names: `feature/`, `fix/`, `docs/`, `chore/`
- Keep commits atomic and focused
- Refer to CONTRIBUTING.md for detailed Git workflow and examples

## Temporary Files and Reports
For temporary analysis reports, documentation, and other working files:

### File Organization
- **Location**: `temp/` directory in project root
- **Naming Convention**: `yyyy-mm-dd_${report-name}.md`
- **Examples**:
  ```
  temp/2025-07-02_test-coverage-analysis.md
  temp/2025-07-02_performance-benchmarks.md
  temp/2025-07-02_architecture-review.md
  ```

### Usage Guidelines
- Use for analysis reports, investigation findings, temporary documentation
- Include date for chronological tracking and cleanup
- Use descriptive names with hyphens for readability
- Clean up periodically - these files are not meant for long-term storage
- Add `temp/` to `.gitignore` if temporary files should not be committed

### When to Use
- Code coverage analysis reports
- Performance investigation findings
- Architecture decision documentation drafts
- Debug session logs and findings
- Temporary research and analysis documents