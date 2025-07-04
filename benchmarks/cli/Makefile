# Sakurs CLI Benchmark Makefile

.PHONY: all install prepare benchmark-accuracy benchmark-performance benchmark-comparison benchmark-all clean help

# Default target
all: benchmark-all

# Install dependencies
install:
	@echo "Installing Python dependencies..."
	pip install -r requirements.txt
	@echo "Checking for Hyperfine..."
	@which hyperfine > /dev/null || (echo "Please install Hyperfine: https://github.com/sharkdp/hyperfine" && exit 1)
	@echo "Checking for sakurs..."
	@which sakurs > /dev/null || (echo "Please build and add sakurs to PATH" && exit 1)

# Prepare benchmark data
prepare:
	@echo "Preparing benchmark data..."
	python scripts/prepare_data.py

# Run accuracy benchmarks
benchmark-accuracy: prepare
	@echo "Running accuracy benchmarks..."
	@mkdir -p results
	bash scenarios/accuracy/run_all.sh

# Run performance benchmarks  
benchmark-performance: prepare
	@echo "Running performance benchmarks..."
	@mkdir -p results
	bash scenarios/performance/run_all.sh

# Run comparison benchmarks
benchmark-comparison: prepare
	@echo "Running comparison benchmarks..."
	@mkdir -p results
	bash scenarios/comparison/run_all.sh

# Run all benchmarks and generate report
benchmark-all: benchmark-accuracy benchmark-performance benchmark-comparison
	@echo "Generating comprehensive report..."
	python scenarios/comparison/generate_report.py
	@echo "Benchmarks complete! Results in results/"

# Clean up results
clean:
	@echo "Cleaning benchmark results..."
	rm -rf results/*
	@echo "Clean complete"

# Show help
help:
	@echo "Sakurs CLI Benchmark Targets:"
	@echo "  make install              - Install dependencies"
	@echo "  make prepare              - Prepare benchmark data"
	@echo "  make benchmark-accuracy   - Run accuracy benchmarks"
	@echo "  make benchmark-performance - Run performance benchmarks"
	@echo "  make benchmark-comparison  - Run baseline comparisons"
	@echo "  make benchmark-all        - Run all benchmarks"
	@echo "  make clean                - Clean results"
	@echo "  make help                 - Show this help"