.PHONY: help install download clean format lint test

help:
	@echo "📚 Brown Corpus Data Processing"
	@echo ""
	@echo "Commands:"
	@echo "  install    Install Python dependencies with uv"
	@echo "  download   Download and process Brown Corpus"
	@echo "  clean      Remove cached data"
	@echo "  format     Format Python code"
	@echo "  lint       Lint Python code"
	@echo "  test       Run tests (if any)"

install:
	@echo "📦 Installing dependencies with uv..."
	@if ! command -v uv >/dev/null 2>&1; then \
		echo "❌ uv not found. Please install it first:"; \
		echo "  curl -LsSf https://astral.sh/uv/install.sh | sh"; \
		exit 1; \
	fi
	cd ../../ && uv sync

download: install
	@echo "📥 Downloading Brown Corpus..."
	cd ../../ && uv run python -m data.brown_corpus.download

clean:
	@echo "🧹 Cleaning cache..."
	rm -rf cache/
	rm -rf __pycache__/
	find . -name "*.pyc" -delete
	find . -name ".DS_Store" -delete

format:
	@echo "🎨 Formatting Python code..."
	cd ../../ && uv run ruff format benchmarks/
	cd ../../ && uv run ruff check --fix benchmarks/

lint:
	@echo "🔍 Linting Python code..."
	cd ../../ && uv run ruff check benchmarks/
	cd ../../ && uv run mypy benchmarks/ || true

test:
	@echo "🧪 Running tests..."
	@if [ -f test_*.py ]; then \
		uv run pytest -v; \
	else \
		echo "No tests found."; \
	fi