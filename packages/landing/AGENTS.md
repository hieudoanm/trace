# AGENTS.md - packages/landing

## Overview

Static marketing/landing page for tracers. A single `index.html` +
`styles.css` — no framework, no build step, no JavaScript except a tiny
year-injection snippet. Deployed to GitHub Pages as-is.

## Conventions

- Keep it dependency-free: edit `index.html`/`styles.css` directly.
- Asset URLs are relative (`logo.svg`, `favicon.svg`, `og-image.png`) so the
  page works under the `/tracers/` Pages base path and from `file://`.
  The directory is flat — no asset subdirectories.
- Light theme only; terminal mockups stay dark via their own colour tokens —
  no theme toggle, no JS theming.
- Design follows the "AI unslop" rules: no filler badges, monospace only for
  code/commands, one accent colour (navy), consistent radii/spacing, real CLI
  output instead of illustrations.
- Keep it a landing page — no API routes, no upload UI.
- Terminal output in demos must reflect real `tracers` output — regenerate it
  from an actual scan if the CLI output format changes.

## Commands

There are none — serve statically:

```bash
python3 -m http.server 8000 -d packages/landing
```
