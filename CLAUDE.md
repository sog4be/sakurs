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
1. **MANDATORY**: Always use the PR template structure from `.github/PULL_REQUEST_TEMPLATE.md`
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
- Refer to CONTRIBUTING.md for detailed Git workflow and examples

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
- **Examples**:
  ```
  temp/2025-07-02-10:30:00_test-coverage-analysis.md
  temp/2025-07-02-14:15:30_performance-benchmarks.md
  temp/2025-07-02-16:45:00_architecture-review.md
  ```

### Usage Guidelines
- Use for analysis reports, investigation findings, temporary documentation
- Include date and time for precise chronological tracking
- Use 24-hour format (HH:MM:SS) for consistency
- Use descriptive names with hyphens for readability
- Clean up periodically - these files are not meant for long-term storage
- Add `temp/` to `.gitignore` if temporary files should not be committed

### When to Use
- Code coverage analysis reports
- Performance investigation findings
- Architecture decision documentation drafts
- Debug session logs and findings
- Temporary research and analysis documents
- Progress tracking and status reports