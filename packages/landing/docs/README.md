# tracers documentation

**tracers** is a high-performance, multi-language code intelligence and graph
analysis tool. It analyses source repositories and builds a unified graph of
their internal structure and relationships — function call chains, dependencies,
symbol references, and change impact.

## Sections

- **Getting started** — install `tracers` and scan your first repository.
- **Concepts** — the graph schema and analysis architecture.
- **Guides** — the CLI reference, visual explorer, and configuration.

## Highlights

- 23 languages via Tree-sitter, plus framework detection (React, Vue, Angular,
  Express, Next.js, NestJS, Svelte, Nuxt, Hono, tRPC, Prisma, Drizzle).
- Built-in Obsidian-style visual graph explorer (`.tracers/graph.html`).
- Graph queries: callers, callees, refs, deps, dependents, reach, path,
  cycles, impact.
- Versioned JSON graph output with resolved/unresolved edge tracking.
