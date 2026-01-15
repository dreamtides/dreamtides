# Appendix: Configuration

This appendix documents all configuration options. See
[Lattice Design](lattice_design.md#configuration) for context.

## User Configuration (~/.lattice.toml)

```toml
[clients]
"/path/to/repo" = "DT"       # Client ID for this repository
"/other/worktree" = "K2"

[defaults]
priority = 2                  # Default task priority (0-4)
line_width = 80               # Default format line width
```

## Repository Configuration (.lattice/config.toml)

### [overview]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `limit` | int | 10 | Default document count |
| `view_weight` | float | 0.5 | Weight for view count in ranking |
| `recency_weight` | float | 0.3 | Weight for last-viewed recency |
| `filename_priority_weight` | float | 0.2 | Weight for filename prefix priority |
| `recency_half_life_days` | int | 7 | Days until recency score halves |

### [prime]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `checklist` | array | See below | Commands shown in session protocol |

Default checklist:
```toml
checklist = ["lat check", "lat fmt", "git status", "git add <files>", "git commit"]
```

### [format]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `line_width` | int | 80 | Text wrap column |
| `list_marker` | string | "-" | Unordered list character |

### [check]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max_lines` | int | 500 | Line count warning threshold |
| `max_name_length` | int | 64 | Name field max characters |
| `max_description_length` | int | 1024 | Description field max characters |

### [sparse]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `warn_sparse_links` | bool | true | Warn about links to non-materialized docs |
| `auto_expand` | bool | false | Auto-expand sparse checkout for `lat show` |

### [claim]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `stale_days` | int | 7 | Days until claim considered stale |

### [logging]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `level` | string | "info" | Log level (error/warn/info/debug/trace) |
| `max_file_size_mb` | int | 10 | Max log file size before rotation |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `LATTICE_LOG_LEVEL` | Override log level |
| `LATTICE_NO_COLOR` | Disable colored output |
| `EDITOR` | Editor for `lat edit` |

## Precedence

Settings resolve in order (later wins):
1. Built-in defaults
2. `~/.lattice.toml` (user)
3. `.lattice/config.toml` (repository)
4. Environment variables
5. Command-line flags
