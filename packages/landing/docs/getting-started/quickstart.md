# Quickstart

## Scan a repository

```bash
tracers scan                  # current directory
tracers scan /path/to/repo
tracers scan . -o graph.json  # write output to a file
tracers scan . --language typescript
tracers scan . --exclude "node_modules/**"
```

`tracers scan . --write-tracers-dir` also emits `.tracers/graph.json`,
`graph.dot`, and the interactive `graph.html` explorer (writing the directory
is enabled by default).

## Query the graph

```bash
tracers stats                          # graph statistics
tracers callers myFunction             # who calls a symbol
tracers callees myFunction             # what it calls
tracers refs MyClass                   # references
tracers deps src/main.ts --transitive  # dependencies
tracers impact src/utils.ts            # change impact
tracers path mainFunction helperFn     # path between symbols
tracers cycles                         # dependency cycles
tracers languages                      # supported languages
tracers check                          # config + parser availability
tracers validate                       # graph integrity
```

Query commands accept `--graph .tracers/graph.json` to reuse an exported graph
instead of rescanning.
