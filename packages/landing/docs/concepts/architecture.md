# Architecture

Tracers is a Rust workspace. The pipeline:

1. **Discovery** — walk the repository, apply `include`/`exclude` globs and
   the language filter.
2. **Parsing** — Tree-sitter grammars produce syntax trees for each supported
   language; framework detection recognises React, Vue, Angular, Express,
   Next.js, NestJS, Svelte/SvelteKit, Nuxt, Hono, tRPC, Prisma, and Drizzle.
3. **Graph construction** — nodes and edges are built per the
   [graph schema](graph-schema.md); a syntax cache (`.tracers-cache`) speeds up
   repeat scans.
4. **Output** — JSON graph, Graphviz DOT, or the interactive HTML explorer.

Queries (`callers`, `deps`, `impact`, `cycles`, …) run over the in-memory or
exported graph without rescanning when `--graph` points at a saved file.

Optional **semantic analysis** (`semantic = "auto"` in `tracers.toml`) falls
back to syntax analysis when semantic data is unavailable.
