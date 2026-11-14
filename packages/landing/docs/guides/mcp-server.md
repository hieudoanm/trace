# MCP server

`tracers` ships a [Model Context Protocol](https://modelcontextprotocol.io)
server over stdio. Any MCP-compatible client — Claude Desktop, Cursor,
Windsurf, or your own agent — can query the code graph directly, without
shelling out per command.

## Run

```sh
tracers mcp
```

The server reads newline-delimited JSON-RPC on stdin and writes responses on
stdout. It delegates every tool call to the `tracers` binary itself, so MCP
behaviour always matches the CLI.

## Client configuration

```json
{
  "mcpServers": {
    "tracers": {
      "command": "tracers",
      "args": ["mcp"]
    }
  }
}
```

To analyse a specific repository, point the `path` argument at it (it defaults
to the current directory). Passing `graph` loads a previously exported
`graph.json` instead of rescanning.

## Tools

| Tool            | Description                                                        |
| --------------- | ------------------------------------------------------------------ |
| `stats`         | Graph statistics for a repository                                  |
| `languages`     | List supported programming languages                               |
| `frameworks`    | Report detected frameworks                                         |
| `validate`      | Validate a scan and report diagnostics                             |
| `cycles`        | Detect dependency cycles                                           |
| `roots`         | Callable symbols with no incoming call edges                       |
| `graph_export`  | Export the graph as JSON or DOT                                    |
| `callers`       | Callers of a symbol                                                |
| `callees`       | Callees of a symbol                                                |
| `refs`          | References to a symbol                                             |
| `deps`          | Dependencies of a target (`transitive` for the full closure)       |
| `impact`        | Impact analysis — what depends on a target                         |
| `dependents`    | Reverse dependencies of a target                                   |
| `path`          | Shortest path between two symbols                                  |

Symbol queries accept a symbol `id`, qualified name, or short name — the same
selectors as the CLI. See the [CLI reference](cli-reference.md) for details.
