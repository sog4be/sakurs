# Makefile for UD Japanese-BCCWJ data management

.PHONY: all download clean info help

# Default cache directory
CACHE_DIR = cache

all: download

# Download the corpus
download:
	@echo "Downloading UD Japanese-BCCWJ r2.16..."
	@python download.py
	@echo "Download complete!"

# Force re-download
download-force:
	@echo "Force re-downloading UD Japanese-BCCWJ r2.16..."
	@python download.py --force
	@echo "Download complete!"

# Show corpus information
info:
	@if [ -f "$(CACHE_DIR)/ud_japanese_bccwj.json" ]; then \
		echo "UD Japanese-BCCWJ corpus information:"; \
		echo "==================================="; \
		python -c "import json; \
		data = json.load(open('$(CACHE_DIR)/ud_japanese_bccwj.json', 'r')); \
		print(f\"Version: {data['metadata']['version']}\"); \
		print(f\"Documents: {len(data['documents'])}\"); \
		splits = {}; \
		[splits.update({d['split']: splits.get(d['split'], 0) + 1}) for d in data['documents']]; \
		print(f\"Splits: {splits}\"); \
		print(f\"License: {data['metadata']['license']}\")"; \
	else \
		echo "Corpus not downloaded yet. Run 'make download' first."; \
	fi

# Clean cache
clean:
	@echo "Cleaning cache..."
	@rm -rf $(CACHE_DIR)
	@echo "Cache cleaned."

# Help
help:
	@echo "UD Japanese-BCCWJ Data Management"
	@echo "================================"
	@echo "Available targets:"
	@echo "  make download       - Download the corpus"
	@echo "  make download-force - Force re-download"
	@echo "  make info          - Show corpus information"
	@echo "  make clean         - Remove cached data"
	@echo "  make help          - Show this help"