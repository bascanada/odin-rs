# Odin 2 Rust - Makefile
# ========================

.PHONY: all build test samples clean check fmt lint doc help wavetables demos demo-morphing demo-morph-presets demo-all

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
	cargo test -p odin2-core --features std

# Run tests with output
test-verbose:
	@echo "Running all tests (verbose)..."
	cargo test -p odin2-core --features std -- --nocapture

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

# ========================================
# Demo Commands
# ========================================

# Generate emotional preset morphing examples (Happy→Sad, 2D space)
# Creates 10 WAV files demonstrating 1D and 2D emotional morphing
demo-morphing:
	@echo "=== Generating Emotional Preset Morphing Examples ==="
	cargo run -p odin2-core --example preset_morph --features std -- --generate-audio
	@echo ""
	@echo "Generated files in samples/morphing/:"
	@ls -1 samples/morphing/*.wav 2>/dev/null || echo "No files found"

# Morph between factory presets (default: Pad Evolution demo)
# Usage: make demo-morph-presets DEMO=1  (1, 2, or 3)
demo-morph-presets:
	@echo "=== Morphing Factory Presets Demo ==="
	cd crates/odin2-core && cargo run --bin morph-demo --features std -- --demo $(or $(DEMO),1) --play

# Generate all built-in presets (10 presets with default melody)
# Generates analog_saw, supersaw, wavetable_pad, filtered_bass, etc.
demo-all:
	@echo "=== Generating All Built-in Presets ==="
	cd crates/odin2-core && cargo run --bin odin2-demo --features std
	@echo ""
	@echo "Generated files in samples/demos/:"
	@ls -1 samples/demos/*.wav 2>/dev/null || echo "No files found"

# Run all demos sequentially
demos: demo-morphing demo-all
	@echo ""
	@echo "=== All Demos Complete ==="
	@echo ""
	@echo "Audio files saved in:"
	@echo "  - samples/morphing/ (10 emotional morphing examples)"
	@echo "  - samples/demos/ (10 built-in preset examples)"

# ========================================

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
	rm -rf samples/*.wav samples/morphing/*.wav samples/demos/*.wav samples/tests/*.wav

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
	@echo "Build & Test:"
	@echo "  make              - Build and test"
	@echo "  make build        - Build release"
	@echo "  make build-debug  - Build debug"
	@echo "  make test         - Run all tests"
	@echo "  make test-verbose - Run tests with output"
	@echo "  make samples      - Generate audio samples to samples/"
	@echo ""
	@echo "Audio Demos:"
	@echo "  make demos            - Run all demos (morphing + presets)"
	@echo "  make demo-morphing    - Generate emotional morphing examples (10 files)"
	@echo "  make demo-morph-presets - Morph factory presets (use DEMO=1,2,3)"
	@echo "  make demo-all         - Generate all built-in presets (10 files)"
	@echo ""
	@echo "Development:"
	@echo "  make wavetables   - Convert C++ wavetables to Rust"
	@echo "  make check        - Quick syntax check"
	@echo "  make fmt          - Format code"
	@echo "  make lint         - Run clippy linter"
	@echo "  make doc          - Generate and open documentation"
	@echo "  make clean        - Clean all build artifacts"
	@echo "  make clean-samples- Clean audio samples from samples/"
	@echo "  make watch        - Watch for changes (requires cargo-watch)"
	@echo "  make watch-test   - Watch and test on changes"
	@echo "  make help         - Show this help"
