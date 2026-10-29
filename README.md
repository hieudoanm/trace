# Trace

Trace is a high-performance, multi-language code intelligence and graph analysis tool. It analyses source code repositories and builds a unified graph of their internal structure and relationships, enabling developers to understand codebase organization, follow function call chains, inspect dependencies, explore symbol references, and determine the impact of code changes.

## Features

- **Multi-language support**: TypeScript, Python, Go, Rust, Kotlin, Swift, Java, C#, PHP, and Ruby
- **Syntax-level analysis**: Tree-sitter based parsing across all supported languages
- **Graph queries**: Find callers, callees, references, dependencies, and impact
- **Cycle detection**: Identify dependency cycles in your codebase
- **Path analysis**: Find paths between symbols and understand relationships
- **Incremental caching**: Fast repeated scans with content-addressed caching
- **Deterministic output**: Reproducible results for identical source trees
- **Machine-readable export**: JSON output with schema validation
- **Local analysis**: No cloud services or external APIs required

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/hieudoanm/trace.git
cd trace

# Build with Cargo
cargo build --release

# The binary will be at target/release/trace
```

### Requirements

- Rust 1.85 or later
- Cargo (comes with Rust)

## Quick Start

### Scan a repository

```bash
# Scan the current directory
trace scan

# Scan a specific path
trace scan /path/to/repository

# Write output to a file
trace scan . -o graph.json

# Filter by language
trace scan . --language typescript

# Exclude files matching a pattern
trace scan . --exclude "node_modules/**"
```

### Query a graph

```bash
# Get statistics
trace stats

# Find callers of a function
trace callers myFunction

# Find callees of a function
trace callees myFunction

# List references to a symbol
trace refs MyClass

# Show dependencies
trace deps src/main.ts

# Show transitive dependencies
trace deps src/main.ts --transitive

# Impact analysis
trace impact src/utils.ts

# Find a path between two symbols
trace path mainFunction helperFunction

# Detect dependency cycles
trace cycles

# Check configuration and parser availability
trace check

# List supported languages
trace languages
```

## CLI Reference

### Global Options

- `-v, --verbose`: Enable verbose logging
- `-q, --quiet`: Silence all non-error output
- `-h, --help`: Print help
- `-V, --version`: Print version

### Commands

#### `trace scan [PATH]`

Analyse a repository and generate a graph.

**Options:**

- `--format <FORMAT>`: Output format: json or dot (default: json)
- `-o, --output <PATH>`: Write output to PATH instead of stdout
- `--include <GLOB>`: Include only files matching this glob pattern
- `--exclude <GLOB>`: Exclude files matching this glob pattern
- `--language <LANGUAGE>`: Only analyse the given language(s)
- `--no-cache`: Disable the syntax cache
- `--jobs <N>`: Number of parallel jobs
- `--config <PATH>`: Path to a trace.toml configuration file

#### `trace graph [PATH]`

Load or construct a graph and provide filtered graph output.

**Options:**

- `--format <FORMAT>`: Output format: json or dot (default: json)
- `-o, --output <PATH>`: Write output to PATH instead of stdout
- `--language <LANGUAGE>`: Filter by language
- `--kind <KIND>`: Filter by node kind

#### `trace callers <SYMBOL> [PATH]`

Display callers of a symbol.

**Options:**

- `--max-depth <N>`: Maximum traversal depth (default: 32)
- `--max-results <N>`: Maximum number of results (default: 1000)

#### `trace callees <SYMBOL> [PATH]`

Display callees of a symbol.

**Options:**

- `--max-depth <N>`: Maximum traversal depth (default: 32)
- `--max-results <N>`: Maximum number of results (default: 1000)

#### `trace refs <SYMBOL> [PATH]`

List references to a symbol.

**Options:**

- `--max-depth <N>`: Maximum traversal depth (default: 32)
- `--max-results <N>`: Maximum number of results (default: 1000)

#### `trace deps <TARGET> [PATH]`

Display direct or transitive dependencies of a target.

**Options:**

- `--transitive`: Follow transitive dependencies
- `--max-depth <N>`: Maximum traversal depth (default: 32)

#### `trace impact <TARGET> [PATH]`

Display static impact analysis for a target.

**Options:**

- `--max-depth <N>`: Maximum traversal depth (default: 32)
- `--max-results <N>`: Maximum number of results (default: 1000)

#### `trace path <SOURCE> <TARGET> [PATH]`

Find a path between two symbols or nodes.

**Options:**

- `--max-depth <N>`: Maximum traversal depth (default: 32)

#### `trace cycles [PATH]`

Detect dependency cycles.

#### `trace stats [PATH]`

Report graph statistics.

#### `trace languages`

Report supported languages and analysis capabilities.

#### `trace check [PATH]`

Validate configuration, discovery, and parser availability.

## Architecture

Trace is organized as a Rust workspace with the following crates:

- **trace-core**: Core graph model, stable identities, and query engine
- **trace-parser**: Repository discovery and tree-sitter syntax extraction
- **trace-analysis**: Symbol resolution, graph construction, and caching
- **trace-export**: Deterministic graph export formats
- **trace-cli**: Command-line interface

The core engine is independent of the visualization layer and does not execute or upload source code.

## Language Support

Trace supports the following languages with syntax-level analysis:

| Language   | Extensions | Syntax | Symbols | Imports | Calls | Cross-file | Semantic |
| ---------- | ---------- | ------ | ------- | ------- | ----- | ---------- | -------- |
| TypeScript | .ts, .tsx  | ✅      | ✅       | ✅       | ✅     | ✅          | Planned  |
| Python     | .py        | ✅      | ✅       | ✅       | ✅     | ✅          | Planned  |
| Go         | .go        | ✅      | ✅       | ✅       | ✅     | ✅          | Planned  |
| Rust       | .rs        | ✅      | ✅       | ✅       | ✅     | ✅          | Planned  |
| Kotlin     | .kt, .kts  | ✅      | ✅       | ✅       | ✅     | ✅          | Planned  |
| Swift      | .swift     | ✅      | ✅       | ✅       | ✅     | ✅          | Planned  |

### Current Limitations

- **TypeScript**: Relative ES module imports are resolved. tsconfig path aliases are not applied. Method resolution uses lexical scope and simple local type inference.
- **Python**: Absolute and relative module imports are resolved against discovered files. Dynamic imports, monkey-patching, and decorator effects are not modelled.
- **Go**: Imports resolve through go.mod module paths and package directories. Interface method sets are reported syntactically; satisfaction is not type-checked.
- **Rust**: crate/self/super use paths and simple module files are resolved. Macro expansion and trait method resolution are not performed.
- **Kotlin**: Imports and same-package declarations are resolved. Overload selection and extension resolution are conservative.
- **Swift**: Files in the same Swift package/module root share a lexical scope. Protocol witness and dynamic dispatch resolution are not performed.

## Configuration

Trace supports an optional project-level configuration file `trace.toml`.

### Example Configuration

```toml
[analysis]
# Maximum file size in bytes (default: 1048576)
max_file_size = 1048576

# Follow symbolic links (default: false)
follow_symlinks = false

# Include generated code (default: false)
include_generated = false

# Enabled languages (default: all)
languages = ["typescript", "python", "go", "rust", "kotlin", "swift"]

[cache]
# Enable caching (default: true)
enabled = true

# Cache directory (default: ".trace-cache")
directory = ".trace-cache"

[output]
# Default output format (default: "json")
format = "json"

# Semantic analysis mode (default: "off")
semantic_mode = "off"
```

### Configuration Priority

Command-line arguments override configuration file values, which override defaults.

## Graph Schema

Trace outputs a versioned JSON graph schema containing:

- **Repository metadata**: ID, name, root path, file count, language count
- **Analysis metadata**: Format, cache status, semantic mode, enabled languages
- **Language information**: Supported languages and grammar versions
- **Nodes**: Source code entities (files, modules, classes, functions, etc.)
- **Edges**: Relationships between entities (contains, imports, calls, references, etc.)
- **Diagnostics**: Parse errors, unresolved imports, ambiguous symbols
- **Statistics**: Counts by node kind, edge kind, language, and resolution status

### Node Kinds

- `repository`, `file`, `module`, `package`, `namespace`
- `class`, `struct`, `interface`, `trait`, `enum`
- `function`, `method`, `constructor`
- `variable`, `constant`, `type`, `property`, `parameter`
- `external_symbol`, `unknown`

### Edge Kinds

- `contains`, `imports`, `exports`
- `calls`, `references`
- `inherits`, `implements`, `uses_type`
- `defines`, `overrides`, `depends_on`

### Resolution Status

- `resolved_internal`: Resolved to a symbol within the repository
- `resolved_external`: Resolved to an external dependency
- `unresolved`: Could not be resolved
- `ambiguous`: Multiple possible targets

## Development

### Building

```bash
# Build the workspace
cargo build --workspace

# Build with optimizations
cargo build --release

# Run tests
cargo test --workspace

# Run formatting checks
cargo fmt --check

# Run linter
cargo clippy --workspace --all-targets --all-features -- -D warnings
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
```

### Running the CLI

```bash
# Build and run
cargo run -- scan /path/to/repo

# Or use the built binary
./target/debug/trace scan /path/to/repo
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Trace uses:

- [tree-sitter](https://tree-sitter.github.io/) for incremental syntax parsing
- [petgraph](https://github.com/petgraph/petgraph) for graph algorithms
- [serde](https://serde.rs/) for serialization
