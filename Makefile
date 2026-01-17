# Makefile - Sovereign Organism Management
#
# Use this to build, test, and maintain the agency.

.PHONY: build test proof bundle clean setup

# Build the agency core
build:
	cargo build --bin rust_agency

# Run all verification suites (Logic + Integration + Architecture + Load)
test:
	@echo "🧪 Running Comprehensive Test Suite..."
	cargo test --test comprehensive_features
	@echo "\n🏛️ Running Architecture Tests..."
	cargo test --test architecture
	@echo "\n🧬 Running Unit Tests..."
	cargo test --lib

# Execute the live Proof of Life demonstration
proof: bundle
	ORT_STRATEGY=download cargo run --bin proof_of_life

# Bundle finicky dependencies (ONNX) for portability
bundle:
	./scripts/bundle_onnx.sh

# Initial setup: Install deps and fetch models
setup:
	cargo build
	./scripts/bundle_onnx.sh
	ORT_STRATEGY=download cargo run --bin proof_of_life

# Clean build artifacts
clean:
	cargo clean
	rm -f libonnxruntime.dylib
	rm -rf artifacts/bin
