# tracers documentation

**tracers** is a high-performance, multi-language code intelligence and graph
analysis tool. It analyses source repositories and builds a unified graph of
their internal structure and relationships — function call chains, dependencies,
symbol references, and change impact.

## Sections

- **Getting started** — install `tracers` and scan your first repository.
- **Concepts** — the graph schema and analysis architecture.
- **Guides** — the CLI reference, visual explorer, configuration, and MCP
  server.

## Highlights

- 23 languages via Tree-sitter, plus framework detection (React, Vue, Angular,
  Express, Next.js, NestJS, Svelte, Nuxt, Hono, tRPC, Prisma, Drizzle).
- Built-in Obsidian-style visual graph explorer (`.tracers/graph.html`).
- Graph queries: callers, callees, refs, deps, dependents, reach, path,
  cycles, impact.
- MCP server (`tracers mcp`) — the whole graph is available to MCP clients
  like Claude Desktop and Cursor.
- Versioned JSON graph output with resolved/unresolved edge tracking.
