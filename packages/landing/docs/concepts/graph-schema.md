# Graph schema

`tracers scan` emits a versioned JSON graph:

- **Repository metadata** — id, name, root path, file/language counts
- **Analysis metadata** — format, cache status, semantic mode, languages
- **Nodes** — source entities: `repository`, `file`, `module`, `package`,
  `namespace`, `class`, `struct`, `interface`, `trait`, `enum`, `function`,
  `method`, `constructor`, `variable`, `constant`, `type`, `property`,
  `parameter`, `external_symbol`, `unknown`
- **Edges** — `contains`, `imports`, `exports`, `calls`, `references`,
  `inherits`, `implements`, `uses_type`, `defines`, `overrides`, `depends_on`
- **Diagnostics** — parse errors, unresolved imports, ambiguous symbols
- **Statistics** — counts by node kind, edge kind, language, resolution status

## Resolution status

- `resolved_internal` — resolved inside the repository
- `resolved_external` — resolved to an external dependency (with
  `include_external`)
- `unresolved` — could not be resolved; materialises as
  `unresolved:<language>:<name>` placeholder nodes so the graph stays
  target-complete
- `ambiguous` — multiple possible targets

Multiple call sites between the same pair of symbols aggregate into one edge
whose `sites` array records every source location.
