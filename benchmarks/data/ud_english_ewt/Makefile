.PHONY: help download benchmark test clean

# Default target
help:
	@echo "Available targets:"
	@echo "  download   - Download and process UD English EWT data"
	@echo "  benchmark  - Run UD English EWT benchmarks"
	@echo "  test       - Test the download script"
	@echo "  clean      - Remove processed data"

# Download and process the UD English EWT data
download:
	@echo "📥 Downloading UD English EWT r2.16..."
	cd "$${PWD%/benchmarks/data/ud_english_ewt}/benchmarks" && uv run python data/ud_english_ewt/download.py

# Force re-download
download-force:
	@echo "🔄 Force re-downloading UD English EWT r2.16..."
	cd "$${PWD%/benchmarks/data/ud_english_ewt}/benchmarks" && uv run python data/ud_english_ewt/download.py --force

# Run benchmarks for UD English EWT
benchmark:
	@echo "🏃 Running UD English EWT benchmarks..."
	cd ../../ && cargo bench --bench ud_english_ewt

# Test the download script with sample data
test:
	@echo "🧪 Testing UD English EWT processing..."
	cd "$${PWD%/benchmarks/data/ud_english_ewt}/benchmarks" && uv run python -c "import sys; sys.path.insert(0, 'data/ud_english_ewt'); from loader import is_available, load_sample; print('Available:', is_available()); print('Sample:', load_sample())"

# Clean processed data
clean:
	@echo "🧹 Cleaning UD English EWT cache..."
	rm -rf cache/
	@echo "✅ Cache cleaned"

# Check if data is available
check:
	@echo "🔍 Checking UD English EWT data availability..."
	@if [ -f "cache/ud_english_ewt.json" ]; then \
		echo "✅ Full UD English EWT data available"; \
		ls -lh cache/ud_english_ewt.json; \
	elif [ -f "cache/test_ud_english_ewt.json" ]; then \
		echo "⚠️  Test UD English EWT data available"; \
		ls -lh cache/test_ud_english_ewt.json; \
	else \
		echo "❌ No UD English EWT data found"; \
		echo "Run 'make download' to download the data"; \
	fi