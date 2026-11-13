# Configuration

Optional project-level `tracers.toml`:

```toml
[analysis]
max_file_size = 1048576        # bytes
follow_symlinks = false
include_generated = false
languages = ["typescript", "python", "go", "rust"]
include = []                   # globs limiting discovered files
exclude = []
jobs = 4                       # parallel parse jobs (default: CPUs)
semantic = "off"               # "auto" falls back to syntax analysis

[cache]
enabled = true
directory = ".tracers-cache"
max_age_days = 30              # 0 disables pruning

[output]
format = "json"                # default output format
```

**Priority:** CLI arguments > `tracers.toml` > defaults.
