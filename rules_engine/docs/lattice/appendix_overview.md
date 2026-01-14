# Appendix: Overview Command

This appendix documents the `lat overview` command for repository-level context.
See [Lattice Design](lattice_design.md#workflow-commands) for how this fits
into the workflow command set.

## Purpose

The `lat overview` command provides AI agents with a curated list of the most
critical documents in a repository, reducing the need for exhaustive exploration.
It ranks documents using local view tracking, recency, and priority signals.

## View Tracking

Lattice tracks document views locally in `.lattice/views.json`:

```json
{
  "LXXXX": {"count": 15, "last_viewed": "2026-01-14T10:30:00Z"},
  "LYYYY": {"count": 8, "last_viewed": "2026-01-13T14:00:00Z"}
}
```

Views are recorded when:
- `lat show <id>` is executed
- `lat overview` includes a document in output (with lower weight)

This file is local-only (not in git) and persists across sessions.

## Ranking Algorithm

Documents are scored using a weighted combination:

```
score = (view_weight * view_score) +
        (recency_weight * recency_score) +
        (priority_weight * priority_score) +
        (root_weight * is_root_document)
```

**Default weights:**
- `view_weight`: 0.4
- `recency_weight`: 0.3
- `priority_weight`: 0.2
- `root_weight`: 0.1

**Score components:**
- `view_score`: Normalized view count (0-1), with logarithmic scaling
- `recency_score`: Decay function based on days since last view
- `priority_score`: 1.0 for P0, decreasing to 0.2 for P4
- `is_root_document`: 1.0 if filename starts with `00_`, else 0.0

## Command Usage

```
lat overview [OPTIONS]

OPTIONS:
  --limit <N>         Maximum documents (default 10)
  --type <type>       Filter by issue type or 'doc' for knowledge base
  --path <prefix>     Filter to path prefix
  --include-closed    Include closed issues
  --reset-views       Clear view history
  --json              Structured output
```

### Default Output

```
$ lat overview
Repository Overview (10 most critical documents):

1. [doc] LROOT: project-overview - High-level project architecture (15 views)
2. [epic] LAUTH: authentication-system - Auth module epic (12 views)
3. [P0] LBUG1: fix-login-crash - Critical login bug (8 views)
...

View history: 47 documents tracked, 156 total views
Run 'lat overview --reset-views' to clear history
```

### JSON Output

```json
{
  "documents": [
    {
      "id": "LROOT",
      "name": "project-overview",
      "description": "High-level project architecture",
      "path": "docs/00_overview.md",
      "type": "doc",
      "score": 0.92,
      "view_count": 15,
      "last_viewed": "2026-01-14T10:30:00Z"
    }
  ],
  "view_stats": {
    "tracked_documents": 47,
    "total_views": 156
  }
}
```

## Configuration

Weights can be customized in `.lattice/config.toml`:

```toml
[overview]
limit = 10
view_weight = 0.4
recency_weight = 0.3
priority_weight = 0.2
root_weight = 0.1
recency_half_life_days = 7
```

## Use Cases

**New session orientation:** Run `lat overview` at session start to understand
which documents have been most relevant to recent work.

**Onboarding:** New agents can quickly identify the most-referenced documents
without traversing the entire repository.

**Context recovery:** After conversation compaction, `lat overview` restores
awareness of frequently-accessed documents.
