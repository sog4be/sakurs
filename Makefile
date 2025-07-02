# Makefile for sakurs - Rust workspace automation
# Usage: make ci-check, make format, make test, etc.

.PHONY: help ci-check format lint test check build clean install-hooks coverage coverage-html coverage-clean

# Default target
help:
	@echo "🚀 Sakurs Development Commands"
	@echo ""
	@echo "CI Verification:"
	@echo "  ci-check    Run all CI checks (format, lint, test, check)"
	@echo "  ci-local    Run CI checks exactly as CI does"
	@echo ""
	@echo "Development:"
	@echo "  format      Format code with cargo fmt"
	@echo "  lint        Run clippy linter"
	@echo "  test        Run all tests"
	@echo "  check       Check compilation"
	@echo "  build       Build all packages"
	@echo ""
	@echo "Coverage:"
	@echo "  coverage       Generate test coverage report"
	@echo "  coverage-html  Generate and open HTML coverage report"
	@echo "  coverage-clean Clean coverage data"
	@echo ""
	@echo "Setup:"
	@echo "  install-hooks  Install git pre-commit hooks"
	@echo "  clean          Clean build artifacts"

# CI verification commands (exact match with CI)
ci-check: format-check lint test check
	@echo "✅ All CI checks passed!"

ci-local:
	@echo "🔍 Running CI checks exactly as CI does..."
	cargo fmt --all -- --check
	cargo clippy --workspace -- -D warnings
	cargo test --workspace
	cargo check --workspace
	@echo "✅ Local CI verification complete!"

# Individual commands
format:
	cargo fmt --all

format-check:
	@echo "📝 Checking code formatting..."
	cargo fmt --all -- --check

lint:
	@echo "🔧 Running clippy..."
	cargo clippy --workspace -- -D warnings

test:
	@echo "🧪 Running tests..."
	cargo test --workspace

check:
	@echo "⚙️ Checking compilation..."
	cargo check --workspace

build:
	@echo "🔨 Building all packages..."
	cargo build --workspace

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Install git hooks
install-hooks:
	@echo "⚙️ Installing git pre-commit hooks..."
	@if [ ! -f .git/hooks/pre-commit ]; then \
		echo "❌ Pre-commit hook not found. Please create it first."; \
		exit 1; \
	fi
	chmod +x .git/hooks/pre-commit
	@echo "✅ Git hooks installed!"

# Coverage commands
coverage:
	@echo "📊 Generating test coverage report..."
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "❌ cargo-llvm-cov not found. Installing..."; \
		cargo install cargo-llvm-cov; \
	fi
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		cargo llvm-cov nextest --all-features --workspace; \
	else \
		cargo llvm-cov test --all-features --workspace; \
	fi
	@echo ""
	cargo llvm-cov report --summary-only

coverage-html:
	@echo "📄 Generating HTML coverage report..."
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "❌ cargo-llvm-cov not found. Installing..."; \
		cargo install cargo-llvm-cov; \
	fi
	cargo llvm-cov report --html
	@echo "✅ Coverage report generated at: target/llvm-cov/html/index.html"
	@if command -v open >/dev/null 2>&1; then \
		open target/llvm-cov/html/index.html; \
	elif command -v xdg-open >/dev/null 2>&1; then \
		xdg-open target/llvm-cov/html/index.html; \
	fi

coverage-clean:
	@echo "🧹 Cleaning coverage data..."
	cargo llvm-cov clean --workspace