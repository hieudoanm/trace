# CLI reference

## Global options

- `-v, --verbose` — verbose logging
- `-q, --quiet` — silence non-error output
- `-h, --help` / `-V, --version`

## `tracers scan [PATH]`

Analyse a repository and generate a graph.

| Option | Description |
| --- | --- |
| `-o, --output <PATH>` | Write `.json`/`.dot`/`.html` to PATH instead of stdout |
| `--include <GLOB>` / `--exclude <GLOB>` | File filters (repeatable) |
| `--language <LANG>` | Only analyse given language(s) |
| `--no-cache` | Disable the syntax cache |
| `--jobs <N>` | Parallel jobs |
| `--write-tracers-dir` / `--no-tracers-dir` | Control `.tracers/` output (on by default) |
| `--config <PATH>` | Path to `tracers.toml` |

## Graph queries

`stats`, `graph`, `callers`, `callees`, `refs`, `deps`, `dependents`, `reach`,
`impact`, `path`, `cycles`, `check`, `validate`, `languages`.

Common query options: `--max-depth <N>` (default 32), `--max-results <N>`
(default 1000), `--graph <PATH>` to load an exported graph.

- `deps`/`dependents` support `--transitive`.
- `graph --format json|dot` re-emits a filtered graph.
