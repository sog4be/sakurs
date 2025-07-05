.PHONY: help download benchmark test clean

# Default target
help:
	@echo "Available targets:"
	@echo "  download   - Download and process UD Japanese GSD data"
	@echo "  benchmark  - Run UD Japanese GSD benchmarks"
	@echo "  test       - Test the download script"
	@echo "  clean      - Remove processed data"

# Download and process the UD Japanese GSD data
download:
	@echo "📥 Downloading UD Japanese GSD r2.16..."
	cd "$${PWD%/benchmarks/data/ud_japanese_gsd}/benchmarks" && uv run python data/ud_japanese_gsd/download.py

# Force re-download
download-force:
	@echo "🔄 Force re-downloading UD Japanese GSD r2.16..."
	cd "$${PWD%/benchmarks/data/ud_japanese_gsd}/benchmarks" && uv run python data/ud_japanese_gsd/download.py --force

# Run benchmarks for UD Japanese GSD
benchmark:
	@echo "🏃 Running UD Japanese GSD benchmarks..."
	cd ../../ && cargo bench --bench ud_japanese_gsd

# Test the download script with sample data
test:
	@echo "🧪 Testing UD Japanese GSD processing..."
	cd "$${PWD%/benchmarks/data/ud_japanese_gsd}/benchmarks" && uv run python -c "import sys; sys.path.insert(0, 'data/ud_japanese_gsd'); from loader import is_available, load_sample; print('Available:', is_available()); print('Sample:', load_sample())"

# Clean processed data
clean:
	@echo "🧹 Cleaning UD Japanese GSD cache..."
	rm -rf cache/
	@echo "✅ Cache cleaned"

# Check if data is available
check:
	@echo "🔍 Checking UD Japanese GSD data availability..."
	@if [ -f "cache/ud_japanese_gsd.json" ]; then \
		echo "✅ Full UD Japanese GSD data available"; \
		ls -lh cache/ud_japanese_gsd.json; \
	elif [ -f "cache/test_ud_japanese_gsd.json" ]; then \
		echo "⚠️  Test UD Japanese GSD data available"; \
		ls -lh cache/test_ud_japanese_gsd.json; \
	else \
		echo "❌ No UD Japanese GSD data found"; \
		echo "Run 'make download' to download the data"; \
	fi