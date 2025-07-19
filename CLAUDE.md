# CLAUDE.md

## Important Project Guidelines

Before making any changes, please review:
- @CONTRIBUTING.md - Git workflow, branch naming, and development guidelines
- @.github/PULL_REQUEST_TEMPLATE.md - PR checklist and required information

When asked to create PRs or commits, always reference these documents to ensure compliance with project standards.

## Tech Stack
- Rust 1.81+ (workspace with 3 crates)
- Cargo workspace resolver = "2"
- Key dependencies: thiserror, serde, tracing
- Testing: criterion (benchmarks), proptest (property testing)

## Project Structure
- @sakurs-core/ - Core Rust library implementing Δ-Stack Monoid algorithm
- @sakurs-cli - Command-line interface for batch processing  
- @sakurs-py - Python bindings via PyO3
- @docs/ARCHITECTURE.md - Detailed system architecture and design decisions
- @docs/DELTA_STACK_ALGORITHM.md - Detailed algorithm description of this system's sentence boundary detection

## Commands
- `cargo build` - Build all workspace crates
- `cargo test` - Run all tests
- `cargo fmt` - Format code
- `cargo clippy` - Lint and catch common mistakes
- `cargo check` - Fast compilation check

## Development Environment Tools

### Available System Tools
These tools are commonly available and can be used for better code exploration and development:
- `tree` - Display directory structure hierarchically (if installed)
- `rg` (ripgrep) - Fast text search across files with better performance than grep
- `fd` - Fast file search alternative to find
- `bat` - Enhanced cat with syntax highlighting (if installed)
- `gh` - GitHub CLI for PR operations

### Recommended Tool Usage
- **Directory structure**: Use `tree -L 3 -I 'target|__pycache__|*.pyc|.git'` for clean overview
- **Code search**: Always prefer `rg` over `grep` for performance
  - Example: `rg --type rust "pattern"` for Rust-specific searches
- **File finding**: Use `fd` when available, fallback to `find`
- **PR operations**: Use `gh pr create` with the template format

### Tool Availability Check
Before using optional tools, check availability:
```bash
command -v tree >/dev/null 2>&1 && echo "tree is available" || echo "tree not found, use ls -R"
```

## Project-Specific Tools and Scripts

### Makefile Commands
- `make coverage` - Generate test coverage report with summary
- `make coverage-html` - Generate and open HTML coverage report
- `make coverage-clean` - Clean coverage data

## Code Style & Conventions
- Follow standard Rust naming conventions
- Use `thiserror` for error handling across crates
- Implement `Send + Sync` for parallel processing traits
- Add rustdoc comments for public APIs
- Use workspace dependencies for consistency
- **Keep code clean**: Remove outdated TODO comments, temporary debug code, and commented-out blocks before committing

## Architecture Notes
- Hexagonal architecture (Ports & Adapters pattern)
- Domain core implements mathematical monoid properties
- Language rules as pluggable traits
- Designed for true parallelism via rayon

## Coding Policy
- **DRY (Don't Repeat Yourself)**: Avoid duplicating code or logic. Abstract repeated functionality into reusable modules, functions, or components to enhance maintainability and readability.
- **SOLID Principles**:
  * **Single Responsibility Principle**: Each class or function should have a single, well-defined responsibility.
  * **Open/Closed Principle**: Software entities should be open for extension but closed for modification.
  * **Liskov Substitution Principle**: Derived classes must be substitutable for their base classes without altering correctness.
  * **Interface Segregation Principle**: Interfaces should be minimal and specific, ensuring clients are not forced to depend on methods they do not use.
  * **Dependency Inversion Principle**: Depend on abstractions (interfaces or abstract classes), not concrete implementations.
- **YAGNI (You Aren't Gonna Need It)**: Do not implement features based purely on speculation. Code only what is necessary now, and avoid complexity from unnecessary future-proofing.
- **Simple is Better**: Prioritize clarity and simplicity—but recognize that true simplicity emerges through thoughtful refinement of complexity. Aim for elegance by deeply understanding and simplifying inherently complex systems.

## Development Workflow
- **MANDATORY**: Run CI verification commands before every commit:
  ```bash
  cargo fmt --all -- --check
  cargo clippy --workspace -- -D warnings  
  cargo test --workspace
  cargo check --workspace
  ```
- ALWAYS run `cargo fmt` and `cargo clippy` before commits
- Read @docs/ARCHITECTURE.md before modifying core algorithm
- Add tests for new functionality - both unit and property tests
- Benchmark performance-critical changes
- Follow branch naming conventions from @CONTRIBUTING.md
- Use conventional commit format as described in @CONTRIBUTING.md

## Performance Tuning
- Default chunk size: 256KB (optimal for most use cases)
- Chunk size can be customized via `--chunk-kb` CLI option
- Thread count can be specified via `--threads` option
- For best performance with large files:
  - Use larger chunk sizes (512KB-1MB) to reduce overhead
  - Adjust thread count based on CPU cores and file size
  - Example: `sakurs process -i large.txt --threads 8 --chunk-kb 1024`

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
1. **MANDATORY**: Always use the PR template structure from @.github/PULL_REQUEST_TEMPLATE.md
   - Use `gh pr create --body "$(cat <<'EOF' ... EOF)"` with template format
   - Fill out ALL required sections: Summary, Type of Change, Changes Made, etc.
   - Mark checkboxes with `[x]` for completed items
   - Do NOT skip sections or use free-form descriptions
2. Ensure all checklist items are addressed before marking as ready for review
3. Follow the commit message conventions (feat:, fix:, docs:, etc.)
4. Include the AI attribution footer in commits:
   ```
   🤖 Generated with [Claude Code](https://claude.ai/code)
   
   Co-Authored-By: Claude <noreply@anthropic.com>
   ```

## PR Template Compliance Checklist
Before creating any PR, verify these requirements:

### ✅ Required Sections (ALL must be completed)
- [ ] **Summary**: Brief description of what the PR accomplishes and why it's needed
- [ ] **Type of Change**: Mark ALL applicable types with `[x]`
- [ ] **Changes Made**: Detailed list with Core Changes, Testing Changes, Documentation Changes
- [ ] **How Has This Been Tested**: Test environment details and test cases
- [ ] **Algorithm/Architecture Impact**: Mark applicable items for algorithm/architecture changes
- [ ] **Checklist**: Complete ALL items in Code Quality, Testing, Documentation, Dependencies sections

### ❌ Common Mistakes to Avoid
- ❌ Using free-form markdown instead of template sections
- ❌ Skipping "Type of Change" checkboxes
- ❌ Missing "Changes Made" subsections (Core/Testing/Documentation)
- ❌ Incomplete test information in "How Has This Been Tested"
- ❌ Unmarked checklist items (leave as `[ ]` if not applicable, explain why)

### 📝 Template Usage Example
```bash
gh pr create --title "feat: implement feature X" --body "$(cat <<'EOF'
## Summary
Brief description here...

## Type of Change
- [x] ✨ New feature
- [ ] 🐛 Bug fix
...
EOF
)"
```

## Git Workflow
- Branch from `main` for new features
- Use descriptive branch names: `feature/`, `fix/`, `docs/`, `chore/`
- Keep commits atomic and focused
- Refer to @CONTRIBUTING.md for detailed Git workflow and examples

## Committing Changes with Git

When the user asks you to create a new git commit, follow these steps carefully:

1. **Pre-commit Verification** (MANDATORY):
   - Run `git status` to identify all changes (staged, unstaged, and untracked files)
   - Review the list and identify files that should NOT be committed:
     - Coverage reports (lcov.info, *.profraw, *.profdata)
     - Build artifacts in target/
     - Temporary files in temp/ or tmp/
     - IDE-specific files (.vscode/, .idea/)
     - OS-specific files (.DS_Store, Thumbs.db)
   - If any unwanted files are present:
     - Remove them with `rm` or `git clean`
     - Ensure they are in .gitignore
     - Run `git status` again to verify cleanup

2. **Change Review** (MANDATORY):
   - Run `git diff --cached` to review staged changes
   - Run `git diff` to review unstaged changes
   - For each file, verify:
     - The changes are intentional and correct
     - No debugging code or console.log statements remain
     - No sensitive information (passwords, API keys) is included
     - Code follows project conventions
     - **Remove unnecessary comments**: Delete outdated TODO comments, temporary debug comments, commented-out code blocks, and development notes that are no longer relevant

3. **Commit Process**:
   - Stage only the necessary files with `git add`
   - Run final CI checks: `cargo fmt --all -- --check` and `cargo clippy --workspace -- -D warnings`
   - Create commit with conventional commit format
   - Include AI attribution footer

## Temporary Files and Reports
For temporary analysis reports, documentation, and other working files:

### File Organization
- **Location**: `temp/` directory in project root
- **Naming Convention**: `yyyy-mm-dd-HH:MM:SS_${report-name}.md`
- **Automatic Timestamp Generation**:
  ```bash
  # Generate accurate timestamp
  TIMESTAMP=$(date "+%Y-%m-%d-%H:%M:%S")
  echo "temp/${TIMESTAMP}_report-name.md"
  
  # Example output:
  # temp/2025-07-02-23:15:42_test-coverage-analysis.md
  ```

### Usage Guidelines
- Use for analysis reports, investigation findings, temporary documentation
- **Always use automatic timestamp generation** for accuracy and consistency
- Use 24-hour format (HH:MM:SS) for consistency
- Use descriptive names with hyphens for readability
- Clean up periodically - these files are not meant for long-term storage
- Add `temp/` to `.gitignore` if temporary files should not be committed

### Automated File Creation
```bash
# Recommended approach for creating temp files
TIMESTAMP=$(date "+%Y-%m-%d-%H:%M:%S")
FILENAME="temp/${TIMESTAMP}_your-report-name.md"
echo "Creating: $FILENAME"

# Use with your content creation
cat > "$FILENAME" << 'EOF'
# Your Report Title

## Content goes here...
EOF
```

### When to Use
- Code coverage analysis reports
- Performance investigation findings
- Architecture decision documentation drafts
- Debug session logs and findings
- Temporary research and analysis documents
- Progress tracking and status reports

## Python Bindings Development

### Development Build Process
**IMPORTANT**: Avoid using `maturin develop` for Python bindings development as it creates editable installs that don't update properly when Rust code changes.

Instead, use:
```bash
# Build and install wheel (recommended)
cd sakurs-py
maturin build --release --features extension-module
uv pip install --force-reinstall target/wheels/*.whl

# Or use the Makefile
make py-dev

# After installation, use .venv/bin/python directly
.venv/bin/python -m pytest tests/
```

### Common Issues
- **Stale binaries**: If Python tests pass but don't reflect recent Rust changes, you may have a stale `.so` file from an editable install
- **uv run reverts**: The `uv run` command may automatically revert to editable installs. Use `.venv/bin/python` directly after installing the wheel
- **Development workflow**: Always rebuild and reinstall the wheel when making Rust code changes

### Testing Python Bindings
```bash
# Run tests after building
cd sakurs-py
.venv/bin/python -m pytest tests/ -v

# Or use make command
make py-test
```
