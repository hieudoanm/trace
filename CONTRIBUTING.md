# Contributing to Trace

Thank you for your interest in contributing to Trace! This document provides guidelines and instructions for contributing to the project.

## Development Setup

### Prerequisites

- Rust 1.85 or later
- Cargo (comes with Rust)
- Git

### Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/your-username/trace.git
   cd trace
   ```
3. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. Build the project:
   ```bash
   cargo build --workspace
   ```

## Development Workflow

### Building

```bash
# Debug build
cargo build --workspace

# Release build
cargo build --release

# Build a specific crate
cargo build -p trace-core
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p trace-core
cargo test -p trace-parser
cargo test -p trace-analysis
cargo test -p trace-export
cargo test -p trace-cli

# Run tests with output
cargo test --workspace -- --nocapture

# Run a specific test
cargo test test_name
```

### Formatting and Linting

```bash
# Format code
cargo fmt

# Check formatting without modifying files
cargo fmt --check

# Run Clippy linter
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Fix Clippy suggestions automatically
cargo clippy --workspace --all-targets --all-features --fix
```

### Running the CLI

```bash
# Run from source
cargo run -- scan /path/to/repo

# Or use the debug binary
./target/debug/trace scan /path/to/repo

# Use the release binary
./target/release/trace scan /path/to/repo
```

## Project Structure

```
trace/
├── Cargo.toml              # Workspace configuration
├── Cargo.lock              # Dependency lock file
├── rust-toolchain.toml     # Rust toolchain specification
├── AGENTS.md               # Complete implementation specification
├── README.md               # Project README
├── CONTRIBUTING.md         # This file
├── LICENSE                 # MIT License
├── Makefile                # Build automation
├── .github/                # GitHub workflows
├── crates/                 # Workspace crates
│   ├── trace-cli/          # Command-line interface
│   ├── trace-core/         # Core graph model
│   ├── trace-parser/       # Repository discovery and parsing
│   ├── trace-analysis/     # Symbol resolution and graph construction
│   └── trace-export/       # Graph export formats
├── fixtures/               # Test fixtures for each language
│   ├── typescript/
│   ├── python/
│   ├── go/
│   ├── rust/
│   ├── kotlin/
│   └── swift/
├── schemas/                # JSON schemas
│   └── graph.schema.json
└── tests/                  # Integration tests
```

## Crate Responsibilities

### trace-cli

- CLI argument parsing and command handling
- Configuration loading
- Progress reporting
- Exit codes and user-facing diagnostics
- Invoking the analysis pipeline

### trace-core

- Stable node and edge types
- Canonical IDs
- Source locations
- Graph storage and indexing
- Graph validation
- Graph query interfaces
- Analysis capability and confidence metadata

### trace-parser

- File discovery integration
- Language detection
- Tree-sitter parser registry
- Syntax extraction
- Source location mapping
- Parser diagnostics

### trace-analysis

- Language adapters
- Symbol indexing
- Module resolution
- Call resolution
- Reference analysis
- Dependency analysis
- Cross-file and cross-language relationships
- Incremental analysis orchestration

### trace-export

- JSON export
- JSON schema compatibility
- Optional DOT and SVG exports
- Deterministic serialization

## Code Style

### General Guidelines

- Follow Rust best practices and idioms
- Use descriptive variable and function names
- Keep functions focused and concise
- Add documentation comments (`///`) for public APIs
- Use `thiserror` for library errors and `anyhow` for application-level errors
- Prefer `tracing` for structured logging over `println!` or `eprintln!`

### Example

```rust
/// Resolves a symbol by its qualified name.
///
/// # Arguments
///
/// * `name` - The qualified name of the symbol to resolve
///
/// # Returns
///
/// Returns `Some(SymbolRef)` if found, `None` otherwise.
pub fn resolve_symbol(&self, name: &str) -> Option<SymbolRef> {
    self.index.get(name).cloned()
}
```

## Testing Guidelines

### Unit Tests

- Write unit tests for all non-trivial functions
- Test edge cases and error conditions
- Use descriptive test names
- Keep tests fast and independent

### Integration Tests

- Test across crate boundaries
- Use fixtures for realistic test data
- Test error handling and diagnostics
- Validate against the JSON schema

### Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_imported_symbols() {
        let mut index = RepositoryIndex::build(root, parsed_files).unwrap();
        let result = resolve_imports(&mut index);
        assert!(!result.edges.is_empty());
    }
}
```

## Adding Language Support

When adding support for a new language:

1. Add the language to `trace-core/src/language.rs`
2. Add the tree-sitter grammar to `Cargo.toml`
3. Create an adapter in `trace-parser/src/adapters/`
4. Add tests in `fixtures/` for the new language
5. Update the language support matrix in `README.md`
6. Document any language-specific limitations

## Adding a New CLI Command

1. Add the command to the `Commands` enum in `trace-cli/src/main.rs`
2. Implement the command handler function
3. Add help text and argument descriptions
4. Test the command manually and with fixtures
5. Update the CLI reference in `README.md`

## Pull Request Process

1. Update the relevant documentation (README, AGENTS.md, code comments)
2. Ensure all tests pass: `cargo test --workspace`
3. Ensure formatting is correct: `cargo fmt --check`
4. Ensure Clippy passes: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
5. Commit your changes with a clear message
6. Push to your fork
7. Create a pull request with a descriptive title and description

### Commit Message Format

```
Summary of changes in one line (50 chars or less)

More detailed explanatory text, if necessary. Wrap it to about 72
characters or so. In some contexts, the first line is treated as the
subject of the commit and the rest of the text as the body.

- Bullet points are okay
- Use a hanging indent
```

## Release Process

Releases are managed by the maintainers. The process includes:

1. Update version numbers in `Cargo.toml`
2. Update `CHANGELOG.md` with release notes
3. Tag the release in Git
4. Build release binaries for supported platforms
5. Publish to crates.io (if applicable)
6. Create GitHub release

## Getting Help

- Open an issue for bugs or feature requests
- Check existing issues for similar problems
- Read the [AGENTS.md](AGENTS.md) file for detailed implementation guidance
- Join discussions in GitHub issues for design questions

## License

By contributing to Trace, you agree that your contributions will be licensed under the MIT License.
