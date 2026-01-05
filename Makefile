# Odin 2 Rust - Makefile
# ========================

.PHONY: all build test samples clean check fmt lint doc help wavetables

# Default target
all: build test

# Build the project
build:
	@echo "Building odin2-core..."
	cargo build --release -p odin2-core

# Build in debug mode
build-debug:
	@echo "Building odin2-core (debug)..."
	cargo build -p odin2-core

# Run all tests
test:
	@echo "Running all tests..."
	cargo test -p odin2-core

# Run tests with output
test-verbose:
	@echo "Running all tests (verbose)..."
	cargo test -p odin2-core -- --nocapture

# Generate audio samples
samples: build-debug
	@echo "Generating audio samples..."
	@mkdir -p samples
	cargo test -p odin2-core --test audio_generation -- --nocapture
	@echo "Moving samples to samples/ directory..."
	@mv crates/odin2-core/target/test_*.wav samples/ 2>/dev/null || true
	@echo ""
	@echo "Generated samples:"
	@ls -la samples/*.wav 2>/dev/null || echo "No samples found"

# Quick check (no codegen)
check:
	@echo "Checking code..."
	cargo check -p odin2-core

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt -p odin2-core

# Format check (CI)
fmt-check:
	@echo "Checking format..."
	cargo fmt -p odin2-core -- --check

# Lint with clippy
lint:
	@echo "Running clippy..."
	cargo clippy -p odin2-core -- -W clippy::all

# Generate documentation
doc:
	@echo "Generating documentation..."
	cargo doc -p odin2-core --no-deps --open

# Clean build artifacts
clean:
	@echo "Cleaning..."
	cargo clean
	rm -rf samples/*.wav

# Clean only samples
clean-samples:
	@echo "Cleaning samples..."
	rm -rf samples/*.wav

# Convert wavetables from C++ to Rust
wavetables:
	@echo "Converting wavetables from C++ to Rust..."
	python3 scripts/convert_wavetables.py

# Watch for changes and rebuild (requires cargo-watch)
watch:
	cargo watch -x "check -p odin2-core"

# Watch and run tests on change
watch-test:
	cargo watch -x "test -p odin2-core"

# Benchmark (placeholder for future benchmarks)
bench:
	@echo "Running benchmarks..."
	cargo bench -p odin2-core

# Show help
help:
	@echo "Odin 2 Rust - Build Commands"
	@echo "============================"
	@echo ""
	@echo "  make              - Build and test"
	@echo "  make build        - Build release"
	@echo "  make build-debug  - Build debug"
	@echo "  make test         - Run all tests"
	@echo "  make test-verbose - Run tests with output"
	@echo "  make samples      - Generate audio samples to samples/"
	@echo "  make wavetables   - Convert C++ wavetables to Rust"
	@echo "  make check        - Quick syntax check"
	@echo "  make fmt          - Format code"
	@echo "  make lint         - Run clippy linter"
	@echo "  make doc          - Generate and open documentation"
	@echo "  make clean        - Clean all build artifacts"
	@echo "  make clean-samples- Clean only audio samples"
	@echo "  make watch        - Watch for changes (requires cargo-watch)"
	@echo "  make watch-test   - Watch and test on changes"
	@echo "  make help         - Show this help"
