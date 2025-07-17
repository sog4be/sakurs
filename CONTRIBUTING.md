# Contributing to sakurs

Thank you for your interest in contributing to Sakurs! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please treat all contributors with respect and professionalism.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/sakurs.git
   cd sakurs
   ```
3. **Understand the architecture**: Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) to understand the project structure and design decisions
4. **Set up the development environment**:
   ```bash
   # Install pre-commit hooks
   make install-hooks
   
   # Build the project
   cargo build
   ```

## How to Contribute

### Adding New Language Support

One of the easiest ways to contribute is by adding support for new languages:

1. Read the [Adding Languages Guide](docs/ADDING_LANGUAGES.md)
2. Create a TOML configuration file for your language
3. Add comprehensive tests
4. Submit a pull request with example texts

### Reporting Issues

- Check existing issues before creating a new one
- Use clear, descriptive titles
- Include steps to reproduce bugs
- Provide system information when relevant

### Submitting Changes

1. **Create a new branch** for your feature or fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following our coding standards:
   - Write clear, self-documenting code
   - Follow Rust naming conventions
   - Keep commits focused and atomic

3. **Run tests and checks**:
   ```bash
   # Run all CI checks (recommended)
   make ci-check
   
   # Or run individual checks
   cargo test --all-features --workspace
   cargo fmt --all -- --check
   cargo clippy --all-features --workspace -- -D warnings
   ```

4. **Commit your changes**:
   ```bash
   git commit -m "feat: add new feature description"
   ```
   
   Follow conventional commit format:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `test:` for test additions/changes
   - `refactor:` for code refactoring
   - `chore:` for maintenance tasks

5. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Create a Pull Request** on GitHub

## Git Branch Strategy

This project follows a **simplified GitHub Flow** strategy optimized for open-source collaboration and continuous integration.

### Branch Types

#### Main Branch (`main`)
- Contains production-ready, stable code
- All code must pass CI/CD checks before merging
- Protected branch requiring pull request reviews
- Deploy-ready at all times

#### Feature Branches
- Created from `main` for new features or bug fixes
- Short-lived (typically 1-7 days)
- Merged back to `main` via pull requests
- Deleted after successful merge

### Branch Naming Convention

Use descriptive names following these patterns:

```bash
# Features
feature/add-streaming-api
feature/improve-performance
feature/japanese-language-support

# Bug fixes
fix/memory-leak-in-parser
fix/incorrect-boundary-detection
fix/issue-123

# Documentation
docs/update-architecture
docs/add-benchmarks
docs/fix-typos

# Maintenance
chore/update-dependencies
chore/cleanup-tests
chore/improve-ci
```

### Workflow Steps

1. **Start from main**:
   ```bash
   git checkout main
   git pull origin main
   ```

2. **Create feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Work in small commits**:
   ```bash
   git add .
   git commit -m "feat: implement core functionality"
   git commit -m "test: add unit tests for new feature"
   git commit -m "docs: update API documentation"
   ```

4. **Keep branch up-to-date**:
   ```bash
   git checkout main
   git pull origin main
   git checkout feature/your-feature-name
   git rebase main  # or merge main if you prefer
   ```

5. **Push and create PR**:
   ```bash
   git push origin feature/your-feature-name
   # Create PR on GitHub
   ```

### Branch Management Rules

#### Do's ✅
- Keep branches focused on a single feature/fix
- Use descriptive branch names
- Rebase or merge main regularly to stay current
- Delete branches after merging
- Write clear commit messages
- Keep PRs small and reviewable

#### Don'ts ❌
- Don't work directly on `main`
- Don't create long-lived feature branches
- Don't mix unrelated changes in one branch
- Don't force push to shared branches
- Don't merge without code review

### Special Cases

#### Hotfixes
For urgent production fixes:
```bash
# Create hotfix branch from main
git checkout -b fix/critical-security-issue main

# Make minimal changes
git commit -m "fix: resolve critical security vulnerability"

# Fast-track review and merge
```

#### Release Preparation
For preparing releases:
```bash
# Create release preparation branch
git checkout -b chore/prepare-v1.2.0 main

# Update version numbers, changelog, etc.
git commit -m "chore: prepare release v1.2.0"
```

### Integration with Issues

Link branches to issues for better tracking:
- Branch names can reference issues: `fix/issue-123-memory-leak`
- Commit messages can auto-close issues: `fix: resolve memory leak (closes #123)`
- PRs should reference related issues in description

### Pull Request Guidelines

- Provide a clear description of changes
- Reference any related issues
- Ensure all tests pass
- Keep PRs focused on a single concern
- Be responsive to feedback

## Development Guidelines

### Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality. The hooks run automatically before each commit and check:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy --all-features`)
- Tests (can be skipped with `SKIP_TESTS=1 git commit`)

To install the hooks:
```bash
make install-hooks
```

### Code Style

- Use `cargo fmt` for consistent formatting
- Run `cargo clippy --all-features --workspace -- -D warnings` and address all warnings
- Write meaningful variable and function names
- Add comments for complex logic

### Testing

- Write tests for new functionality
- Ensure existing tests pass
- Aim for good test coverage
- Include both unit and integration tests where appropriate

### Documentation

- Update documentation for API changes
- Add rustdoc comments for public APIs
- Keep README.md up to date
- Refer to [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for understanding the system design before making architectural changes

## Questions?

If you have questions, feel free to:
- Open an issue for discussion
- Ask in the project's communication channels

Thank you for contributing to Sakurs!