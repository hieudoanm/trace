# Installation

## Prebuilt binary

Prebuilt binaries for Linux and macOS (x86_64 and aarch64) are attached to
GitHub releases:

```bash
curl -fsSL https://raw.githubusercontent.com/hieudoanm/tracers/main/scripts/install.sh | sh
```

The script detects your platform and installs `tracers-<os>-<arch>` to
`~/.local/bin`.

## From a local checkout

```bash
./scripts/install-local.sh
```

## From source

```bash
git clone https://github.com/hieudoanm/tracers.git tracers
cd tracers
cargo build --release     # binary at target/release/tracers
```

## Requirements

- Rust 1.85 or later
- Cargo (comes with Rust)
