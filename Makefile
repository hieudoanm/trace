.PHONY: help build build-release test test-all fmt fmt-check clippy clean run install check

# Default target
help:
	@echo "Trace - Build and Development Commands"
	@echo ""
	@echo "Build:"
	@echo "  make build          - Build the project in debug mode"
	@echo "  make build-release  - Build the project in release mode"
	@echo "  make install        - Install the trace binary"
	@echo ""
	@echo "Test:"
	@echo "  make test           - Run all tests"
	@echo "  make test-all       - Run tests with output"
	@echo ""
	@echo "Linting:"
	@echo "  make fmt            - Format code"
	@echo "  make fmt-check      - Check code formatting"
	@echo "  make clippy         - Run Clippy linter"
	@echo "  make check          - Run all checks (fmt, clippy, test)"
	@echo ""
	@echo "Clean:"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "Run:"
	@echo "  make run            - Build and run trace on current directory"
	@echo "  make run-<cmd>      - Build and run a specific command"

# Build in debug mode
build:
	cargo build --workspace

# Build in release mode
build-release:
	cargo build --workspace --release

# Install the binary
install:
	cargo install --path crates/trace-cli

# Run tests
test:
	cargo test --workspace

# Run tests with output
test-all:
	cargo test --workspace -- --nocapture

# Format code
fmt:
	cargo fmt --all

# Check code formatting
fmt-check:
	cargo fmt --all -- --check

# Run Clippy
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Run all checks
check: fmt-check clippy test

# Build and run trace on current directory
run: build
	./target/debug/trace scan .

# Build and run trace with a specific command
run-scan: build
	./target/debug/trace scan $(ARGS)

run-stats: build
	./target/debug/trace stats $(ARGS)

run-callers: build
	./target/debug/trace callers $(ARGS)

run-callees: build
	./target/debug/trace callees $(ARGS)

run-deps: build
	./target/debug/trace deps $(ARGS)

run-impact: build
	./target/debug/trace impact $(ARGS)

run-cycles: build
	./target/debug/trace cycles $(ARGS)

run-check: build
	./target/debug/trace check $(ARGS)

run-languages: build
	./target/debug/trace languages
