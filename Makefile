# Makefile for sakurs - Rust workspace automation
# Usage: make ci-check, make format, make test, etc.

.PHONY: help ci-check format lint test check build clean install-hooks coverage coverage-html coverage-threshold coverage-clean check-llvm-tools check-python format-python lint-python type-check-python py-dev py-test py-bench py-build wheels

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
	@echo "  coverage          Generate test coverage report"
	@echo "  coverage-html     Generate and open HTML coverage report"
	@echo "  coverage-threshold Check coverage threshold (80%)"
	@echo "  coverage-clean    Clean coverage data"
	@echo ""
	@echo "Python Development:"
	@echo "  check-python     Run Python checks (ruff, mypy)"
	@echo "  format-python    Format Python code"
	@echo "  lint-python      Lint Python code"
	@echo "  type-check-python Run mypy type checking"
	@echo "  py-dev           Build Python bindings for development"
	@echo "  py-test          Run Python tests"
	@echo "  py-bench         Run Python benchmarks"
	@echo "  py-build         Build Python wheel for release"
	@echo "  wheels           Build wheels for distribution"
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
	cargo clippy --all-features --workspace -- -D warnings
	cargo test --all-features --workspace
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
	cargo clippy --all-features --workspace -- -D warnings

test:
	@echo "🧪 Running tests..."
	cargo test --all-features --workspace

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
	@if [ -f .githooks/pre-commit ]; then \
		cp .githooks/pre-commit .git/hooks/pre-commit; \
		chmod +x .git/hooks/pre-commit; \
		echo "✅ Git hooks installed from .githooks/pre-commit!"; \
	elif [ -f .git/hooks/pre-commit ]; then \
		chmod +x .git/hooks/pre-commit; \
		echo "✅ Existing git hooks made executable!"; \
	else \
		echo "❌ Pre-commit hook not found. Please create .githooks/pre-commit first."; \
		exit 1; \
	fi

# Coverage commands
check-llvm-tools:
	@if ! rustup component list --installed | grep -q llvm-tools-preview; then \
		echo "📦 Installing llvm-tools-preview..."; \
		rustup component add llvm-tools-preview; \
	fi

coverage: check-llvm-tools
	@echo "📊 Generating test coverage report..."
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "❌ cargo-llvm-cov not found. Installing..."; \
		cargo install cargo-llvm-cov || (echo "❌ Failed to install cargo-llvm-cov" && exit 1); \
	fi
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		cargo llvm-cov nextest --all-features --workspace --ignore-filename-regex='benchmarks/' || (echo "❌ Coverage generation failed" && exit 1); \
	else \
		cargo llvm-cov test --all-features --workspace --ignore-filename-regex='benchmarks/' || (echo "❌ Coverage generation failed" && exit 1); \
	fi
	@echo ""
	cargo llvm-cov report --summary-only

coverage-html: check-llvm-tools
	@echo "📄 Generating HTML coverage report..."
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "❌ cargo-llvm-cov not found. Installing..."; \
		cargo install cargo-llvm-cov || (echo "❌ Failed to install cargo-llvm-cov" && exit 1); \
	fi
	cargo llvm-cov report --html --ignore-filename-regex='benchmarks/' || (echo "❌ HTML coverage generation failed" && exit 1)
	@echo "✅ Coverage report generated at: target/llvm-cov/html/index.html"
	@if command -v open >/dev/null 2>&1; then \
		open target/llvm-cov/html/index.html; \
	elif command -v xdg-open >/dev/null 2>&1; then \
		xdg-open target/llvm-cov/html/index.html; \
	fi

coverage-threshold: check-llvm-tools
	@echo "🎯 Checking coverage threshold..."
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "❌ cargo-llvm-cov not found. Installing..."; \
		cargo install cargo-llvm-cov || (echo "❌ Failed to install cargo-llvm-cov" && exit 1); \
	fi
	cargo llvm-cov report --fail-under-lines 80 --ignore-filename-regex='benchmarks/' || (echo "❌ Coverage below 80% threshold" && exit 1)

coverage-clean:
	@echo "🧹 Cleaning coverage data..."
	cargo llvm-cov clean --workspace || echo "⚠️ Coverage clean failed (may not be installed)"

# Python development commands
check-python:
	@echo "🐍 Running Python checks..."
	@if [ -d sakurs-py ]; then \
		cd sakurs-py && \
		echo "  Running ruff check..." && \
		uv run ruff check . && \
		echo "  Running ruff format check..." && \
		uv run ruff format --check . && \
		echo "  Running mypy..." && \
		uv run mypy . || true; \
		echo "✅ Python checks complete!"; \
	else \
		echo "⚠️ sakurs-py directory not found"; \
	fi

format-python:
	@echo "🎨 Formatting Python code..."
	@if [ -d sakurs-py ]; then \
		cd sakurs-py && \
		uv run ruff format . && \
		uv run ruff check --fix . && \
		echo "✅ Python code formatted!"; \
	else \
		echo "⚠️ sakurs-py directory not found"; \
	fi

lint-python:
	@echo "🔍 Linting Python code..."
	@if [ -d sakurs-py ]; then \
		cd sakurs-py && \
		uv run ruff check . --fix && \
		echo "✅ Python linting complete!"; \
	else \
		echo "⚠️ sakurs-py directory not found"; \
	fi

type-check-python:
	@echo "🔍 Running mypy type checking..."
	@if [ -d sakurs-py ]; then \
		cd sakurs-py && \
		uv run mypy . && \
		echo "✅ Type checking complete!"; \
	else \
		echo "⚠️ sakurs-py directory not found"; \
	fi

# Python development commands
py-dev:
	@echo "🐍 Building Python bindings for development..."
	@if [ -d sakurs-py ]; then \
		cd sakurs-py && \
		uv sync --no-install-project && \
		uv run --no-sync maturin build --release --features extension-module -o dist && \
		WHEEL_FILE=$$(ls -t dist/*.whl | head -1) && \
		uv pip install --force-reinstall "$$WHEEL_FILE" && \
		echo "✅ Python bindings built and installed from wheel!"; \
		echo "💡 Note: Use .venv/bin/python directly instead of 'uv run' to avoid editable install issues"; \
	else \
		echo "❌ sakurs-py directory not found"; \
		exit 1; \
	fi

py-test: py-dev
	@echo "🧪 Running Python tests..."
	@if [ -d sakurs-py ]; then \
		cd sakurs-py && \
		.venv/bin/python -m pytest tests/ -v && \
		echo "✅ Python tests passed!"; \
	else \
		echo "❌ sakurs-py directory not found"; \
		exit 1; \
	fi

py-bench: py-dev
	@echo "⚡ Running Python benchmarks..."
	@if [ -d sakurs-py ]; then \
		cd sakurs-py && \
		.venv/bin/python -m pytest benchmarks/ --benchmark-only && \
		echo "✅ Python benchmarks complete!"; \
	else \
		echo "❌ sakurs-py directory not found"; \
		exit 1; \
	fi

py-build:
	@echo "📦 Building Python wheel..."
	@if [ -d sakurs-py ]; then \
		cd sakurs-py && \
		uv sync --no-install-project && \
		uv run --no-sync maturin build --features extension-module --release && \
		echo "✅ Python wheel built!"; \
	else \
		echo "❌ sakurs-py directory not found"; \
		exit 1; \
	fi

# Build wheels for distribution
wheels:
	@echo "📦 Building wheels for distribution..."
	@if [ -d sakurs-py ]; then \
		cd sakurs-py && \
		uv sync --no-install-project && \
		uv run --no-sync maturin build --features extension-module --release --strip && \
		echo "✅ Distribution wheels built!"; \
	else \
		echo "❌ sakurs-py directory not found"; \
		exit 1; \
	fi
