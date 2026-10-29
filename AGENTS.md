# Trace — Complete Project Implementation Specification

## 1. Project Overview

Build **Trace**, a high-performance, multi-language code intelligence and graph analysis tool.

Trace analyses source code repositories and builds a unified graph of their internal structure and relationships. Developers should be able to understand how a codebase is organised, follow function call chains, inspect dependencies, explore symbol references, and determine the impact of code changes.

The target programming languages are:

1. TypeScript
2. Python
3. Go
4. Rust
5. Kotlin
6. Swift
7. Java
8. C#
9. PHP
10. Ruby

The primary implementation language is **Rust**.

The project must be production-oriented, modular, deterministic, testable, efficient, and suitable for open-source distribution.

Do not build a generic graph editor. Trace is specifically a source-code analysis tool.

## 2. Implementation Principles

Follow these principles throughout implementation:

- Implement working functionality rather than producing a design document or placeholder architecture.
- Prefer simple, maintainable abstractions over unnecessary frameworks.
- Use Rust for the core engine, graph representation, analysis orchestration, and CLI.
- Use Tree-sitter for broad, incremental syntax parsing across the ten target languages.
- Use language-specific semantic analysis when syntax alone cannot resolve symbols or function calls.
- Do not claim that a relationship is resolved when it is only inferred from syntax.
- Represent unresolved and partially resolved relationships explicitly.
- Produce deterministic results for identical source trees and configuration.
- Handle malformed source files, missing dependencies, and unsupported syntax without unnecessarily aborting an entire repository scan.
- Keep the core engine independent of the visualisation layer.
- Prefer local analysis and do not upload source code to external services.
- Do not require an LLM, cloud service, or paid API for core functionality.
- Avoid unnecessary runtime dependencies and heavyweight infrastructure.
- Do not implement fake features, empty handlers, or hard-coded sample results as substitutes for real analysis.
- Follow a test-first approach for graph semantics and language adapters.
- Keep documentation consistent with actual implementation.

## 3. Technology Stack

Use the following stack, verifying current stable and mutually compatible versions before implementation.

### Core

- Rust stable toolchain.
- Cargo workspaces only where separate crates provide clear architectural value.
- `clap` for CLI parsing and help output.
- `serde` and `serde_json` for graph serialization.
- `petgraph` for graph algorithms where appropriate.
- `ignore` for repository file discovery and Git ignore rules.
- `walkdir` only if a specific traversal requirement is not adequately covered by the chosen discovery library.
- `rayon` for CPU-bound parallel processing where it improves performance.
- `thiserror` for structured library errors.
- `anyhow` for application-level error propagation.
- `tracing` and `tracing-subscriber` for diagnostics and configurable logging.
- `blake3` or another justified, stable hashing implementation for content fingerprints and cache keys.
- `insta` where snapshot testing is useful.

Avoid adding dependencies without a concrete requirement.

### Parsing

Use Tree-sitter with maintained grammars for:

- TypeScript and TSX.
- JavaScript and JSX where required to analyse JavaScript files inside TypeScript-oriented projects.
- Python.
- Go.
- Rust.
- Kotlin.
- Swift.
- Java.
- C#.
- PHP.
- Ruby.

Treat JavaScript/JSX as an optional companion to TypeScript support, not as an additional primary target language.

Pin compatible grammar and runtime versions. Hide parser implementation details behind a common internal interface.

### Semantic analysis

Use native language tooling where it materially improves correctness.

- **TypeScript:** TypeScript Compiler API for module resolution, symbol identity, types, references, and call resolution.
- **Python:** Python AST and symbol-table facilities initially, with optional integration of additional static-analysis tooling when justified.
- **Go:** `go/parser`, `go/ast`, `go/types`, and the Go module/package loading facilities as appropriate.
- **Rust:** `syn` for syntax-level enrichment when useful; investigate rust-analyzer or compiler-assisted analysis for reliable semantic resolution.
- **Kotlin:** Kotlin PSI and Analysis API or another maintained Kotlin semantic-analysis integration.
- **Swift:** SwiftSyntax and SourceKit or compiler-assisted indexing for semantic information.
- **Java:** Javac / Java compiler symbols and classpath analysis where feasible.
- **C#:** Roslyn-based analysis facilities where feasible.
- **PHP:** PHP parser reflection and AST facilities where feasible.
- **Ruby:** Prism / Ruby AST facilities where feasible.

Do not pretend these language integrations are equivalent in complexity. Build a common contract around them while allowing different levels of semantic precision.

External tools must be optional for basic syntax analysis. Detect their availability, report their capabilities, and provide clear diagnostics when deeper analysis cannot run.

Do not bundle large language toolchains into the initial distribution unless there is a compelling, documented reason.

### Visualisation

The core engine remains focused on the Rust CLI and machine-readable graph output, while providing an embedded, standalone visual explorer:

- Embedded `graph_template.html` bundled into `trace-cli` and emitted to `.trace/graph.html` on `trace scan --write-trace-dir`.
- Powered by D3.js (`d3-force`) and HTML5 Canvas 2D for high performance (60 FPS on 2,700+ nodes, 7,600+ edges) and an Obsidian-style aesthetic.
- Interactive ego-network hover/selection, degree-based node scaling, live search, node/edge type filtering, and physics tuning.
- Decoupled from the core engine: the core produces standard deterministic JSON (`graph.json`).

## 4. Repository Structure

Start with a maintainable Rust workspace. Use a single crate for closely related modules and split functionality into workspace crates only when the separation has clear benefits.

Suggested structure:

    trace/
    ├── Cargo.toml
    ├── Cargo.lock
    ├── rust-toolchain.toml
    ├── README.md
    ├── LICENSE
    ├── CONTRIBUTING.md
    ├── Makefile
    ├── .gitignore
    ├── .github/
    │   └── workflows/
    │       └── ci.yml
    ├── crates/
    │   ├── trace-cli/
    │   │   ├── src/
    │   │   │   ├── cli/             # Modular subcommands (scan, graph, queries, validate)
    │   │   │   ├── graph_template.html # Embedded D3-force Canvas visualizer
    │   │   │   └── main.rs
    │   ├── trace-core/
    │   ├── trace-parser/
    │   │   └── src/adapters/        # 10 Tree-sitter language adapters
    │   ├── trace-analysis/
    │   │   └── src/
    │   │       ├── graph/           # Modular graph construction & edge extraction
    │   │       └── resolve/         # Modular symbol, module & import resolution
    │   └── trace-export/
    ├── schemas/
    │   └── graph.schema.json
    ├── fixtures/
    │   ├── typescript/
    │   ├── python/
    │   ├── go/
    │   ├── rust/
    │   ├── kotlin/
    │   ├── swift/
    │   ├── java/
    │   ├── csharp/
    │   ├── php/
    │   └── ruby/
    ├── tests/
    └── docs/

This is the repository structure. Every crate contains implemented functionality and comprehensive tests.

### Module responsibilities

**trace-cli**

- CLI arguments and commands (partitioned in `src/cli/` into `scan`, `graph`, `queries`, and `validate`).
- Configuration loading.
- Progress reporting.
- Exit codes and user-facing diagnostics.
- Invoking the analysis pipeline.
- Standalone D3-force Canvas visual explorer (`graph_template.html`).

**trace-core**

- Stable node and edge types.
- Canonical IDs.
- Source locations.
- Graph storage and indexing.
- Graph validation.
- Graph query interfaces.
- Analysis capability and confidence metadata.

**trace-parser**

- File discovery integration.
- Language detection.
- Tree-sitter parser registry.
- Syntax extraction across 10 languages (TypeScript, Python, Go, Rust, Kotlin, Swift, Java, C#, PHP, Ruby).
- Source location mapping.
- Parser diagnostics.

**trace-analysis**

- Language adapters.
- Symbol indexing.
- Module resolution (`src/resolve/`).
- Call resolution.
- Reference analysis.
- Dependency analysis.
- Graph construction (`src/graph/`).
- Cross-file and cross-language relationships.
- Incremental analysis orchestration (syntax cache).

**trace-export**

- JSON export.
- JSON schema compatibility.
- Optional DOT and SVG exports when implemented correctly.
- Deterministic serialization.

## 5. Functional Requirements

### 5.1 Repository discovery

Trace must accept a repository path and discover supported source files.

Requirements:

- Respect `.gitignore` by default.
- Exclude `.git`, build outputs, generated code, vendored dependencies, caches, and common dependency directories by default.
- Allow explicit include and exclude patterns.
- Allow users to override relevant defaults.
- Detect supported languages by file extension and, where necessary, project metadata.
- Recognise common source extensions, including `.ts`, `.tsx`, `.js`, `.jsx`, `.py`, `.go`, `.rs`, `.kt`, `.kts`, `.swift`, `.java`, `.cs`, `.php`, and `.rb`.
- Avoid following symbolic links by default.
- Handle unreadable files and malformed source safely.
- Report scanned, skipped, failed, and unsupported files.
- Support monorepos and repositories containing multiple languages.
- Avoid scanning binary files.
- Avoid unexpectedly traversing enormous generated directories.

All traversal and exclusion rules must be documented and tested.

### 5.2 Syntax parsing

Parse source files into a common intermediate representation.

Extract, where supported:

- Files.
- Modules and packages.
- Namespaces.
- Classes.
- Structs.
- Interfaces.
- Traits.
- Enums.
- Functions.
- Methods.
- Constructors.
- Variables and constants.
- Imports and exports.
- Type aliases.
- Implementations and extensions.
- Inheritance and interface implementations.
- Function call expressions.
- Attribute and decorator declarations.
- References to symbols.
- Relevant language-specific constructs.

Not every construct exists in every language. Each adapter must map language-specific syntax into common concepts without discarding important metadata.

Each extracted symbol should include:

- A stable identifier.
- A symbol kind.
- A name.
- A qualified name when available.
- A repository-relative file path.
- A source range.
- A language identifier.
- Optional parent symbol.
- Optional signature and type metadata.
- Export or visibility information where applicable.

Use byte offsets internally when required by parser APIs, and define a consistent convention for line and column positions in exported data.

### 5.3 Unified graph model

Model a repository as a directed, attributed graph.

A node represents a source-code entity.

An edge represents a relationship between entities.

Support at least these node kinds:

- `repository`
- `file`
- `module`
- `package`
- `namespace`
- `class`
- `struct`
- `interface`
- `trait`
- `enum`
- `function`
- `method`
- `constructor`
- `variable`
- `constant`
- `type`
- `property`
- `parameter`
- `external_symbol`
- `unknown`

Support at least these relationship kinds:

- `contains`
- `imports`
- `exports`
- `calls`
- `references`
- `inherits`
- `implements`
- `uses_type`
- `defines`
- `overrides`
- `depends_on`

Use a consistent naming convention in the actual Rust types and JSON schema.

The graph model must support adding new node and edge kinds without requiring a redesign of the engine.

### 5.4 Stable identity

Do not use transient array indices as public symbol identifiers.

Create stable identifiers using a documented strategy based on repository-relative paths, qualified names, symbol kinds, and disambiguating information such as source positions or signatures.

Requirements:

- Identifiers must remain deterministic for unchanged source and configuration.
- Distinguish overloads and same-named local symbols.
- Handle anonymous functions and closures.
- Avoid collisions between files and modules.
- Avoid machine-specific absolute paths in portable graph exports.
- Document which edits can cause identifiers to change.
- Keep display labels separate from canonical IDs.

Do not promise perfectly stable identity when code is moved, renamed, or structurally changed unless a robust identity mechanism actually supports it.

### 5.5 Symbol resolution

Implement a layered symbol-resolution strategy.

Resolution should consider:

1. Lexical scope.
2. Local declarations.
3. Imports and exports.
4. Module and package boundaries.
5. Qualified names.
6. Type information when available.
7. Language-specific rules.
8. External dependencies when their metadata is available.

Distinguish at least:

- Resolved internal relationships.
- Resolved external relationships.
- Unresolved relationships.
- Ambiguous relationships.
- Syntax-only inferences.

Each relationship should record its origin and resolution status where relevant.

Do not connect two same-named functions merely because their names match.

When a relationship cannot be resolved, preserve useful information without inventing a target symbol.

### 5.6 Function call graph

Build a call graph containing functions and methods and the relationships between them.

Requirements:

- Identify direct function calls.
- Identify method calls where resolvable.
- Handle qualified calls.
- Handle constructors.
- Handle imported functions.
- Handle calls across files and modules.
- Preserve source locations of call sites.
- Support multiple call sites between the same pair of symbols.
- Distinguish known targets from possible or unresolved targets.
- Support recursive calls.
- Support mutually recursive functions.
- Avoid claiming complete call coverage for dynamic dispatch or reflective calls that cannot be resolved statically.

Where multiple call sites exist, represent them consistently. Either aggregate them into a single edge with multiple source locations or preserve individual call-site relationships in a dedicated structure. Document the choice and ensure the JSON schema can represent it.

For dynamic dispatch, function pointers, callbacks, reflection, dynamic imports, and other difficult cases, provide conservative results and explicit uncertainty.

### 5.7 Dependency graph

Analyse:

- File-to-file dependencies.
- Module imports.
- Package dependencies.
- External package dependencies when metadata is available.
- Type dependencies.
- Cross-module references.
- Inheritance and implementation relationships.

Support dependency traversal, cycle detection, and identification of highly connected modules.

Distinguish direct dependencies from transitive dependencies.

Do not report a package dependency as a function call.

### 5.8 Graph queries

Implement a reusable query API in the core engine.

Support:

- Find a node by ID.
- Find nodes by name or qualified name.
- Find callers of a function.
- Find callees of a function.
- Find all references to a symbol.
- Find direct dependencies of a file or module.
- Find transitive dependencies.
- Find reverse dependencies.
- Find paths between two nodes.
- Find nodes reachable from a given node.
- Detect strongly connected components and dependency cycles.
- Find symbols that have no incoming call relationships, with clearly documented limitations.
- Filter nodes and edges by language, kind, file, relationship type, and resolution status.

Queries must operate on indexed graph data where appropriate. Avoid scanning every node for common lookups.

Provide depth limits and result limits for expensive traversals.

### 5.9 Impact analysis

Implement a basic impact-analysis command.

Given a file or symbol, identify:

- Direct dependents.
- Transitive dependents.
- Callers affected by changes.
- Related types and interfaces.
- Dependency paths explaining why a node is included.

Clearly separate graph reachability from claims about actual runtime impact.

The first version should provide static impact analysis only. Do not claim to know whether a change will break production behaviour.

### 5.10 Incremental analysis

Design for incremental analysis from the beginning, but implement it in stages.

First implement reliable full scans.

Then add caching based on:

- File content hashes.
- Parser and analyzer versions.
- Configuration.
- Relevant dependency information.
- Semantic-analysis environment where necessary.

When a file changes, invalidate affected syntax data and all dependent semantic results that may no longer be valid.

Do not use modification timestamps as the sole correctness mechanism.

Ensure cached and uncached analyses produce equivalent graph results.

If complete incremental semantic invalidation is not yet reliable, prefer a conservative rescan over incorrect cached results.

### 5.11 Diagnostics

Provide structured diagnostics for:

- Parse errors.
- Unsupported syntax.
- Missing language tooling.
- Unresolved imports.
- Ambiguous symbols.
- Failed external analyzer invocations.
- Invalid configuration.
- Unreadable files.
- Partial analysis.

Diagnostics must include relevant file paths and source positions when available.

A partially analysed repository should still produce useful output when safe.

The CLI must clearly distinguish successful complete analysis, successful partial analysis, and fatal failures.

## 6. Language Adapter Contract

Implement a common language adapter interface.

The interface should expose capabilities such as:

- Language detection.
- Syntax parsing.
- Symbol extraction.
- Import and export extraction.
- Call-site extraction.
- Reference extraction.
- Optional semantic resolution.
- Diagnostics.
- Incremental invalidation dependencies.

Use Rust traits and typed data structures instead of a collection of unstructured JSON objects inside the core engine.

The common contract must not force every language to implement capabilities that it cannot support.

Track capabilities separately, for example:

- Syntax parsing available.
- Symbol extraction available.
- Import resolution available.
- Type-aware resolution available.
- Call resolution available.
- Cross-file references available.

Expose those capabilities through a CLI command or analysis metadata.

### TypeScript

Implement support for:

- `.ts` and `.tsx`.
- JavaScript companion files where configured.
- ES module imports and exports.
- Named and default exports.
- Re-exports.
- Aliases.
- Classes, interfaces, functions, methods, and types.
- Type aliases and type references.
- Direct function calls.
- Imported function calls.
- Method calls where semantic resolution permits.
- Project references and path aliases where supported.
- `tsconfig.json` and relevant module-resolution rules.
- Package boundaries and workspace layouts.

Use Tree-sitter for syntax extraction and the TypeScript Compiler API for semantic analysis.

Keep the Rust engine as the source of truth for the unified graph. The TypeScript integration should return structured semantic facts, not implement a competing graph database.

### Python

Implement support for:

- Functions and methods.
- Classes and inheritance.
- Imports and aliases.
- Relative imports.
- Package and module relationships.
- Direct calls.
- Decorators.
- Type annotations.
- References that can be resolved statically.

Use Python AST facilities where appropriate for semantic enrichment. If a Python helper is needed, make its invocation explicit, version-aware, and optional for basic syntax parsing.

Handle dynamic imports, monkey-patching, decorators, and dynamic attribute access conservatively.

### Go

Implement support for:

- Packages and modules.
- Imports and aliases.
- Functions and methods.
- Structs, interfaces, and types.
- Method sets and interface implementations where resolvable.
- Function calls.
- Receiver methods.
- Type references.
- Workspace and module boundaries.

Use Go's parser and type-checking facilities when available.

Do not require Go tooling for basic syntax parsing.

### Rust

Implement support for:

- Modules.
- Crates.
- Functions and methods.
- Structs, enums, traits, and implementations.
- `use` declarations.
- Re-exports.
- Traits and trait implementations.
- Generic types and functions.
- Method calls where resolvable.
- Macro-related syntax to the extent supported.
- Workspace relationships and crate dependencies.

Use Tree-sitter for initial parsing and consider `syn` for syntax-level enrichment.

Investigate rust-analyzer or compiler-assisted semantic analysis for reliable symbol resolution.

Do not claim full macro expansion or type-aware resolution without implementing it.

### Kotlin

Implement support for:

- Packages and imports.
- Classes, objects, interfaces, and data classes.
- Functions and methods.
- Properties.
- Constructors.
- Inheritance and interface implementations.
- Extension functions.
- Type aliases.
- Function calls and references.
- Gradle project and module relationships where practical.

Use Tree-sitter for syntax-level analysis and integrate Kotlin compiler or IDE analysis facilities for semantic resolution where feasible.

Handle overloads, extension functions, and JVM interoperability conservatively.

### Swift

Implement support for:

- Modules and imports.
- Classes, structs, enums, protocols, and extensions.
- Functions and methods.
- Initializers.
- Properties and computed properties.
- Protocol conformance.
- Type references.
- Function calls and method calls.
- Swift package relationships.
- Relevant access-control declarations.

Use Tree-sitter for syntax-level analysis and SwiftSyntax, SourceKit, or compiler-assisted tooling for deeper semantic resolution.

Handle overloads, protocol dispatch, extensions, and dynamic dispatch without inventing targets.

### Java

Implement support for:

- Packages and single-type / on-demand imports (`import package.*`, `import static`).
- Classes, abstract classes, interfaces, and enums.
- Methods, constructors, and field declarations (constants and properties).
- Inheritance (`extends`) and interface implementations (`implements`).
- Method invocations, direct calls, and object instantiations (`new`).
- Parameter and return type references.
- Visibility modifiers (`public`, `protected`, `private`, package-private).

Use Tree-sitter for syntax-level analysis and Java compiler / symbol table tools for semantic resolution where feasible.

### C#

Implement support for:

- Namespaces (file-scoped and block-scoped) and `using` directives (including static and alias usings).
- Classes, structs, interfaces, and enums.
- Methods, constructors, properties, and fields.
- Base class inheritance and interface implementations.
- Method invocations and object creations.
- Parameter and return type references.
- Visibility and modifier extraction (`public`, `internal`, `private`, `protected`).

Use Tree-sitter for syntax-level analysis and Roslyn-based analysis facilities where feasible.

### PHP

Implement support for:

- Namespaces and `use` declarations (including aliased imports).
- Classes, abstract classes, interfaces, traits, and enums.
- Methods, functions, properties, and constants.
- Class inheritance (`extends`) and interface implementations (`implements`).
- Method calls, function invocations, and static calls.
- Type references in parameters and return types.

Use Tree-sitter for syntax-level analysis and static PHP AST/reflection facilities where feasible.

### Ruby

Implement support for:

- Modules and classes.
- Methods (instance methods and singleton/class methods `def self.foo`).
- Constants and global references.
- Module inclusions and class inheritance (`class Foo < Bar`).
- Method calls, command calls, and `require` / `require_relative` dependencies.
- Dynamic method definition handled conservatively without fabricating ungrounded targets.

Use Tree-sitter for syntax-level analysis and Prism / Ruby AST facilities where feasible.

## 7. CLI Specification

Create a CLI named `trace`.

Implement the following command structure:

    trace scan [PATH]
    trace graph [PATH]
    trace callers <SYMBOL>
    trace callees <SYMBOL>
    trace refs <SYMBOL>
    trace deps <TARGET>
    trace impact <TARGET>
    trace path <SOURCE> <TARGET>
    trace cycles [PATH]
    trace stats [PATH]
    trace languages
    trace check [PATH]
    trace validate [PATH]

Exact syntax can be adjusted to avoid ambiguity, but equivalent functionality must exist.

### `trace scan`

Analyse a repository and generate a graph.

Support options such as:

- `--format json` (output format for stdout: json or dot)
- `--output PATH` (write output files `.json`, `.dot`, and `.html` to file or directory)
- `--include GLOB`
- `--exclude GLOB`
- `--language LANGUAGE`
- `--no-cache`
- `--jobs N`
- `--verbose`
- `--quiet`
- `--config PATH`
- `--write-trace-dir` (generate `.trace/graph.json`, `.trace/graph.dot`, and `.trace/graph.html`; enabled by default)
- `--no-trace-dir` (disable writing the `.trace` directory)

Default to a useful, predictable output format. Do not flood standard output with progress messages when JSON is being written to standard output.

### `trace graph`

Load or construct a graph and provide filtered graph output.

### `trace callers` and `trace callees`

Display relationships with source locations and resolution status.

Support depth and result limits.

### `trace refs`

List references to a symbol.

### `trace deps`

Display direct or transitive dependencies.

### `trace impact`

Display affected nodes and the dependency or call paths explaining their inclusion.

### `trace path`

Find a path between two symbols or graph nodes.

Clearly report when no path exists or when the query is ambiguous.

### `trace cycles`

Detect dependency cycles and report the involved nodes.

### `trace stats`

Report counts for files, languages, symbols, edges, relationship types, unresolved references, and analysis diagnostics.

### `trace languages`

Report supported languages, detected parser versions, and available semantic-analysis capabilities.

### `trace check`

Validate configuration, source discovery, parser availability, and optional semantic-analysis tools without performing unnecessary work.

### `trace validate`

Validate graph integrity: every edge must reference an existing node. Fails if any dangling edge targets are found.

Implement useful `--help` output, meaningful exit codes, and actionable error messages for every command.

## 8. JSON Graph Format

Define a versioned JSON graph schema.

Include:

- Schema version.
- Trace version.
- Repository metadata.
- Analysis configuration metadata.
- Supported language and analyzer information.
- Nodes.
- Edges.
- Source locations.
- Diagnostics.
- Resolution status.
- Optional analysis statistics.

Use repository-relative paths.

Represent optional data explicitly and document whether omitted fields, null values, or empty arrays are used.

Add a JSON Schema file and validate exported graphs in automated tests.

Provide a compatibility policy for schema changes.

Ensure that output ordering is deterministic. Sort nodes and edges by stable identifiers and relationship types, and sort source locations where necessary.

Do not include timestamps, absolute paths, machine-specific details, or nondeterministic iteration order in canonical output unless explicitly requested.

## 9. Performance Requirements

Build for real-world repositories rather than synthetic demonstrations alone.

Priorities:

- Avoid reading the same file repeatedly during a scan.
- Parse independent files concurrently.
- Avoid unnecessary allocations in hot paths.
- Index nodes by ID, qualified name, and other useful lookup keys.
- Avoid repeatedly resolving the same symbol.
- Cache expensive analysis results safely.
- Bound memory growth for very large repositories.
- Provide progress reporting for long-running scans.
- Measure performance before implementing low-level optimisations.
- Keep deterministic output despite parallel execution.

Create benchmark fixtures at multiple repository sizes.

Measure:

- File discovery time.
- Parsing throughput.
- Symbol extraction time.
- Resolution time.
- Graph construction time.
- Query latency.
- Peak memory usage.
- Cache-hit performance.

Do not promise specific speedups without benchmark evidence.

## 10. Testing Strategy

Create meaningful automated tests for the engine and every language adapter.

### Unit tests

Test:

- Stable ID generation.
- Node and edge construction.
- Graph indexing.
- Graph validation.
- Relationship resolution.
- Graph traversal.
- Dependency cycles.
- Path queries.
- Impact analysis.
- Serialization and schema compatibility.
- Configuration parsing.
- Cache invalidation.

### Language fixtures

For each of the ten languages, create small realistic projects containing:

- Multiple files.
- Cross-file imports.
- Several functions and classes.
- Nested declarations where relevant.
- Direct function calls.
- Method calls.
- Type references.
- Inheritance or interface implementation where applicable.
- Unresolved symbols.
- Syntax errors.
- Overloaded functions or methods where supported.
- External dependencies.
- Edge cases specific to the language.

Each fixture must have expected graph results or explicit assertions.

Do not use snapshots as the only test of semantic correctness.

### Integration tests

Verify:

- The CLI works from a clean build.
- Scanning a repository generates valid JSON.
- The same repository produces deterministic output.
- Query commands return correct results.
- Partial failures produce useful diagnostics.
- Cache-enabled and cache-disabled results match.
- Missing optional tooling does not break syntax-only analysis.
- Exclusion patterns are respected.
- Large fixtures complete within reasonable test budgets.

### Quality checks

Run and fix failures from:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- Documentation tests.
- JSON Schema validation.
- Frontend checks if the frontend has been implemented.

Use additional checks only when their maintenance cost is justified.

## 11. Security and Reliability

Treat repositories as untrusted input.

- Never execute scanned source code.
- Do not automatically install project dependencies.
- Do not execute arbitrary project build scripts.
- Avoid shell interpolation when invoking external tools.
- Use explicit executable paths and argument arrays.
- Apply timeouts to external analyzers.
- Limit diagnostic output and protect against oversized files.
- Handle symlinks and path traversal carefully.
- Do not upload repository content.
- Avoid logging secrets or source-code contents unnecessarily.
- Validate configuration and imported graph data.
- Handle malformed files without panicking.
- Prevent one malformed file from corrupting unrelated graph results.

Any optional semantic-analysis helper must follow the same security constraints.

## 12. Configuration

Support an optional project-level configuration file, for example `trace.toml`.

Allow configuration of:

- Included and excluded paths.
- Enabled languages.
- Analysis depth or scope where relevant.
- Semantic-analysis modes.
- External tool paths.
- Cache location and behaviour.
- Parallelism.
- Output format.
- Diagnostic verbosity.
- Generated-code handling.

Provide documented defaults.

Command-line arguments should override configuration values predictably.

Do not require a configuration file for a basic scan.

## 13. Documentation and Developer Experience

Create:

- A comprehensive README.
- Installation and build instructions.
- Quick-start examples.
- CLI reference.
- Architecture overview.
- Language support matrix.
- Graph schema documentation.
- Analysis limitations.
- Configuration reference.
- Development and testing instructions.
- Contribution guidelines.
- Troubleshooting guide.
- Performance benchmarking instructions.

The README must explain the difference between syntax-level relationships and semantically resolved relationships.

Provide realistic example output from actual fixtures.

Do not claim a feature is supported until it is implemented and tested.

## 14. Continuous Integration and Release

Set up CI to:

- Build the Rust workspace.
- Run formatting checks.
- Run Clippy.
- Run unit and integration tests.
- Validate JSON schemas.
- Test supported operating systems where practical.
- Build release binaries for supported targets.

Prioritise macOS on Apple Silicon and common Linux environments, while keeping the implementation portable.

Add Windows support when dependencies and external tool integrations permit it.

Pin toolchain and dependency versions appropriately. Avoid unnecessary nightly Rust features.

Provide a reproducible release process and useful version output.

## 15. Implementation Milestones

Implement in the following order.

### Milestone 1: Working Rust foundation

- Initialise the project.
- Establish the module architecture.
- Implement the CLI.
- Implement repository discovery.
- Define graph nodes, edges, IDs, and source locations.
- Implement JSON export.
- Add unit tests and CI.

Deliverable: Trace scans a repository and emits a valid, deterministic graph containing files.

### Milestone 2: Syntax extraction across all ten languages

- Integrate Tree-sitter grammars.
- Implement language detection.
- Extract declarations, imports, and call expressions.
- Record syntax-level relationships and diagnostics.
- Add realistic fixtures for all languages.

Deliverable: Trace produces useful syntax-level graphs for TypeScript, Python, Go, Rust, Kotlin, Swift, Java, C#, PHP, and Ruby.

### Milestone 3: Dependency analysis

- Resolve straightforward relative imports.
- Analyse modules and packages.
- Add dependency traversal and cycle detection.
- Implement graph statistics and query APIs.

Deliverable: Trace can answer basic repository dependency questions.

### Milestone 4: Semantic analysis

- Integrate TypeScript Compiler API.
- Integrate Go type information.
- Improve Python symbol and import resolution.
- Improve Rust resolution using suitable compiler or IDE tooling.
- Investigate Kotlin and Swift semantic integrations.
- Add explicit resolution status and confidence metadata.

Deliverable: Trace resolves a documented subset of cross-file calls and references accurately for each language.

### Milestone 5: Call graphs and impact analysis

- Implement callers and callees.
- Add call-site metadata.
- Implement graph paths and reachability.
- Implement reverse dependencies and impact analysis.
- Add ambiguity and unresolved-call handling.

Deliverable: Developers can investigate call chains and potential change impact from the CLI.

### Milestone 6: Incremental analysis

- Implement content hashing.
- Cache syntax extraction.
- Add semantic cache invalidation.
- Compare cached results against clean scans.
- Benchmark representative repositories.

Deliverable: Repeated scans avoid unnecessary work while preserving correctness.

### Milestone 7: Visualisation and release readiness

- Finalise the graph schema.
- Implement the standalone Obsidian-style visual graph explorer using D3.js and HTML5 Canvas (`graph_template.html`).
- Add interactive navigation, ego-network highlighting, search, and filtering.
- Implement graph validation (`trace validate`) to enforce edge-node referential integrity.
- Improve packaging and release automation.
- Complete documentation and performance testing.

Deliverable: Trace is usable as a high-performance CLI with full query capabilities and an integrated, standalone visual code-graph explorer.

## 16. Acceptance Criteria

The project is not complete merely because it compiles.

The implementation must satisfy these criteria:

1. Trace builds successfully using the documented stable Rust toolchain.
2. A user can scan a repository without a mandatory external service.
3. All ten target languages have functioning syntax adapters.
4. Supported declarations, imports, and call expressions are represented in the common graph.
5. Cross-file dependencies are resolved where the implementation has sufficient information.
6. Semantic-analysis coverage and limitations are reported honestly.
7. Callers, callees, dependency, path, cycle, and impact queries work on tested fixtures.
8. Exported JSON conforms to the documented schema.
9. Repeated scans produce deterministic canonical output.
10. Malformed files and missing optional tooling produce useful diagnostics.
11. Tests cover each language and the core graph algorithms.
12. Formatting, linting, tests, and CI pass.
13. Documentation reflects actual behaviour.
14. The core engine does not execute repository code or upload source files.
15. No major feature is represented by a placeholder or fabricated result.

## 17. Coding-Agent Workflow

Before editing:

1. Inspect the current repository and existing implementation.
2. Identify the current architecture, build system, coding conventions, and existing tests.
3. Preserve working functionality unless a change is justified.
4. Verify current library APIs and compatibility using official documentation or local dependency metadata.
5. Identify the smallest useful implementation sequence.

During implementation:

1. Create or modify actual project files.
2. Implement complete modules instead of leaving TODOs for essential functionality.
3. Add tests alongside each meaningful feature.
4. Run formatting, linting, and relevant tests after changes.
5. Fix regressions before proceeding.
6. Keep the graph schema and documentation in sync with the implementation.
7. Record any features that cannot yet be completed accurately and explain the blocker.
8. Prefer complete, coherent files and maintainable changes over disconnected snippets.

At completion:

1. Summarise the implemented architecture.
2. List the files created or changed.
3. List working features by language.
4. Clearly distinguish syntax support from semantic-resolution support.
5. Report tests and checks actually executed, including failures.
6. Provide exact build and usage commands.
7. Identify remaining limitations and the next highest-value tasks.

Do not claim to have executed tests or benchmarks unless they were actually run.

## 18. Framework Awareness

Trace is framework aware: it detects which application frameworks are used in a repository and models framework-specific constructs (components, controllers, services, models, routes, hooks, views, middleware, etc.) as first-class graph nodes and edges.

### 18.1 Design goals

- Detect frameworks from import/require statements, package manifests (package.json, Cargo.toml, go.mod, requirements.txt, Gemfile, composer.json, etc.), decorators/annotations, base-class inheritance, and naming conventions.
- Represent each detected framework as a `Framework` node with the framework name, family, version hint, and detection confidence.
- Represent framework-specific artefacts (React component, Django model, Spring controller, Express route, etc.) as typed nodes linked to their framework.
- Add framework-specific edges: `uses_framework`, `implements_pattern`, `extends_framework`, `routes_to`, `injects`, `renders`.
- Never invent framework usage: report only what imports, decorators, inheritance, or manifests actually indicate. Use `ResolutionStatus` and `ResolutionOrigin` to record confidence.
- Keep framework detection separate from syntax parsing: it runs as a post-index, pre-graph-build pass over imports, decorators, heritage clauses, and manifest files.

### 18.2 Detected frameworks (initial set)

| Language              | Framework    | Detection signals                                                                                             |
| --------------------- | ------------ | ------------------------------------------------------------------------------------------------------------- |
| TypeScript/JavaScript | React        | `react` import, `React.Component`, `React.FC`, hooks                                                          |
| TypeScript/JavaScript | Vue          | `vue` import, `defineComponent`, `<script setup>`, Vue options API                                            |
| TypeScript/JavaScript | Angular      | `@angular/core` import, `@Component`, `@Injectable`, `@NgModule`                                              |
| TypeScript/JavaScript | Express      | `express` import, `express()`, `app.get`/`app.post`, middleware                                               |
| TypeScript/JavaScript | Next.js      | `next/navigation`, `next/link`, `next/route` imports                                                          |
| TypeScript/JavaScript | NestJS       | `@nestjs/core`, `@nestjs/common` imports, `@Controller`, `@Get`, `@Post`, `@Injectable`, `@Module` decorators |
| Python                | Click        | `click` import, `click.command`, `click.group` calls                                                          |
| Python                | Typer        | `typer` import, `typer.Typer` call                                                                            |
| Go                    | Cobra        | `github.com/spf13/cobra` import, `cobra.Command` call                                                         |
| Go                    | Gin          | `gin-gonic/gin` import, `gin.Default()`, `router.GET`                                                         |
| Go                    | Echo         | `labstack/echo` import, `echo.New()`, `router.GET`                                                            |
| Rust                  | clap.rs      | `clap` import, `Command` call                                                                                 |
| Rust                  | Actix-web    | `actix-web` import, `#[get]`, `#[post]`, `HttpServer`                                                         |
| Rust                  | Axum         | `axum` import, `Router::new()`, `route()`                                                                     |
| Kotlin                | Ktor         | `io.ktor` import, `embeddedServer` call                                                                       |
| Kotlin                | Spring Boot  | `org.springframework` import, `@SpringBootApplication`, `@RestController`                                     |
| Java                  | Spring Boot  | `org.springframework` import, `@SpringBootApplication`, `@RestController`                                     |
| Java                  | Jakarta EE   | `jakarta.*` import, `@WebServlet`, `@Stateless`                                                               |
| C#                    | ASP.NET Core | `Microsoft.AspNetCore` import, `[ApiController]`, `ControllerBase`                                            |
| PHP                   | Laravel      | `Illuminate` import, `Route::get`, `extends Controller`                                                       |
| PHP                   | Symfony      | `symfony` import, `@Route`, `extends AbstractController`                                                      |
| Ruby                  | Rails        | `rails` import, `ApplicationController`, `ActiveRecord::Base`                                                 |

### 18.3 Graph model additions

**Node kind:** `Framework` (`trace-core/src/node.rs`).

**Node metadata:** `framework`, `family`, `version`, `confidence`, `detection_signal`.

**Edge kinds** (`trace-core/src/edge.rs`):

- `UsesFramework` — file/module/symbol depends on a framework package.
- `ImplementsPattern` — symbol implements a framework pattern (component, controller, route, model, etc.).
- `ExtendsFramework` — class extends a framework base class.
- `RoutesTo` — symbol declares a framework route/endpoint.
- `Injects` — dependency injection into a framework-managed symbol.

**ID format** (`trace-core/src/id.rs`): `framework:<name>[:<version>]`.

### 18.4 Implementation milestones

#### Milestone A: Core types and detection engine — COMPLETE

- Add `NodeKind::Framework` to `trace-core/src/node.rs`.
- Add framework edge kinds to `trace-core/src/edge.rs`.
- Add `framework_id()` to `trace-core/src/id.rs`.
- Create `trace-analysis/src/framework/` with `types.rs`, `detector.rs`, `lookup.rs`, `signatures.rs`.
- Define framework signature database (import prefixes, decorator names, base class names, manifest files).
- Detect frameworks from parsed imports, decorators, heritage clauses, and discovered manifest files.
- Unit tests for each detector (detector.rs tests).

#### Milestone B: Graph construction integration — COMPLETE

- Run framework detection after index/import resolution and before graph build in `graph_builder.rs`.
- Add `Framework` nodes to the graph for each detected framework.
- Add `UsesFramework` edges (repository → framework). `ImplementsPattern`, `ExtendsFramework`, `RoutesTo`, `Injects` wired via symbol-level edge detection (not yet materialized in fixtures).
- Wire framework detection into `build_repository_with_options` and `build_from_parsed`.

#### Milestone C: Stats, export, and CLI — COMPLETE

- Add framework counts to `trace-core/src/graph.rs` `Stats` (`by_framework`).
- Add framework information to `trace-export` `GraphDocument`/`AnalysisMetadata` and DOT export colors.
- Add `trace frameworks` CLI command reporting detected frameworks with signals and confidence (`crates/trace-cli/src/cli/validate.rs:cmd_frameworks`, registered in `commands.rs`, wired in `main.rs`).
- Add framework breakdown to `trace stats` and `trace languages`.

#### Milestone D: Tests and fixtures — PARTIAL

- Added framework fixtures: react, django, express, actix-web.
- Added integration tests in `crates/trace-analysis/tests/framework_tests.rs` (5 tests, all passing).
- Integration test verifying framework nodes appear in exported JSON (pending).
- Schema validation for framework nodes and edges (pending).

### 18.5 Acceptance criteria — STATUS

1. `cargo build` succeeds and `trace frameworks <path>` runs without error. — PASS
2. Detected frameworks appear as `Framework` nodes in exported JSON with name, family, and confidence. — PASS (via scan.rs FrameworkInfo mapping)
3. Framework-specific symbols linked via `ImplementsPattern`/`ExtendsFramework` edges. — PARTIAL (edges added to model, not yet populated from symbol detection)
4. No framework node is created without an import, decorator, inheritance, or manifest signal. — PASS (detector is signal-driven)
5. `trace stats` reports per-framework node and edge counts. — PARTIAL (`by_framework` stat exists; CLI output not yet extended)
6. Unit and integration tests pass; `cargo fmt --check` and `cargo clippy` are clean. — PASS

## Final Instruction

Implement Trace as a real, extensible, multi-language code intelligence engine, prioritising correctness, clear architectural boundaries, and a useful CLI.

Start with repository inspection, then implement the smallest complete vertical slice. Continue through the milestones in order, validating each stage before moving on.

Do not stop after generating the project structure, writing a plan, or producing example code. Make actual changes to the repository and continue until the available implementation scope is complete. If the full specification exceeds one execution session, leave the repository in a working state, document precise progress, and identify the next concrete implementation step.
