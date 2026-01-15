# Appendix: Overview Command

This appendix documents the `lat overview` command for repository-level context.
See [Lattice Design](lattice_design.md#workflow-commands) for how this fits
into the workflow command set.

## Purpose

The `lat overview` command provides AI agents with a curated list of the most
critical documents in a repository, reducing the need for exhaustive exploration.
It ranks documents using local view tracking, recency, and filename priority.

## View Tracking

Lattice tracks document views in the SQLite index (`views` table) for
concurrent safety. Views are recorded when:

- `lat show <id>` is executed
- `lat overview` includes a document in output (with lower weight)

View data is local-only (index is gitignored) and persists across sessions.
See [Appendix: Indexing](appendix_indexing.md#views) for schema.

## Ranking Algorithm

Documents are scored using a weighted combination:

```
score = (view_weight * view_score) +
        (recency_weight * recency_score) +
        (filename_priority_weight * filename_priority_score)
```

**Default weights:**
- `view_weight`: 0.5
- `recency_weight`: 0.3
- `filename_priority_weight`: 0.2

**Score components:**
- `view_score`: Normalized view count (0-1), with logarithmic scaling
- `recency_score`: Decay function based on days since last view
- `filename_priority_score`: Based on filename prefix—`README.md` and `00_*` score
  1.0, `01_*` scores 0.9, `02_*` scores 0.8, and so on; unprefixed files score 0.5

## Command Usage

```
lat overview [OPTIONS]

OPTIONS:
  --limit <N>         Maximum documents (default 10)
  --type <type>       Filter by task type or 'doc' for knowledge base
  --path <prefix>     Filter to path prefix
  --include-closed    Include tasks in .closed/ directories
  --reset-views       Clear view history
  --json              Structured output
```

By default, `lat overview` excludes tasks in `.closed/` directories. Use
`--include-closed` to see recently-viewed closed tasks in the overview.

### Default Output

```
$ lat overview
Repository Overview (10 most critical documents):

1. [doc] LROOTX: project-overview - High-level project architecture (15 views)
2. [epic] LAUTHX: authentication-system - Auth module epic (12 views)
3. [P0] LBUGBX: fix-login-crash - Critical login bug (8 views)
...

View history: 47 documents tracked, 156 total views
Run 'lat overview --reset-views' to clear history
```

### JSON Output

```json
{
  "documents": [
    {
      "id": "LROOTX",
      "name": "project-overview",
      "description": "High-level project architecture",
      "path": "docs/README.md",
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

Field names match the YAML frontmatter for consistency across all JSON outputs.

## Configuration

Weights can be customized in `.lattice/config.toml`:

```toml
[overview]
limit = 10
view_weight = 0.5
recency_weight = 0.3
filename_priority_weight = 0.2
recency_half_life_days = 7
```

## Contextual Overview

`lat overview <id>` shows documents relevant to a specific task. This answers:
"What do I need to know to work on this task?"

### Relevance Model

Context is determined by graph distance from the target task:

**Distance 0** (the task itself):
- Always included first

**Distance 1** (directly connected):
- Parent epic (directory root document)
- `blocked-by` tasks (must understand what's blocking)
- `blocking` tasks (understand downstream impact)
- Documents linked in the task body

**Distance 2** (one hop away):
- Sibling tasks (same directory, open only)
- Documents linked from blocked-by tasks
- Parent's parent (grandparent epic)

### Ranking

Within each distance tier, documents are ranked by:

1. **Link type weight:**
   - `blocked-by`: 1.0 (critical path)
   - `blocking`: 0.8 (impact awareness)
   - `parent`: 0.7 (context)
   - `body-link`: 0.6 (referenced material)
   - `sibling`: 0.4 (related work)

2. **Filename priority:** Documents with `README.md` or `00_*` filenames rank
   highest, followed by `01_*`, `02_*`, etc. Unprefixed files rank lowest.

3. **For body links:** Position in document (earlier = higher)

4. **For siblings:** Filename priority, then recency

### Output

```
$ lat overview LB234X
Context for LB234X: Fix login timeout bug

Parent:
  LAA42X: [epic] Authentication System

Blocked by (1):
  LCCCCC: [P1] Refactor session handling

Blocks (2):
  LDDDDD: [P0] Release 2.0 checklist
  LEEEEE: [P1] User acceptance testing

Referenced docs (2):
  LFFFFF: auth-design - Authentication architecture
  LGGGGG: session-spec - Session management specification

Siblings (3 of 7 open):
  LHHHHH: [P1] Add OAuth support
  LJJJJJ: [P2] Improve error messages
  LKKKKK: [P2] Add rate limiting
```

Tasks in `.closed/` directories show `[P<N>/closed]` indicator when displayed
via `--include-closed`.

### Limits

Default limits prevent overwhelming output:
- `blocked-by`: all (usually few)
- `blocking`: 5
- Referenced docs: 5
- Siblings: 5

Override with `--limit <N>` to show more in each category.

### JSON Output

```
$ lat overview LB234X --json
```

Returns structured data with the same categories, suitable for programmatic
consumption by AI agents.

## Use Cases

**New session orientation:** Run `lat overview` at session start to understand
which documents have been most relevant to recent work.

**Task context:** Run `lat overview <id>` before starting work on a task to
see its dependency graph, referenced documentation, and related tasks.

**Onboarding:** New agents can quickly identify the most-referenced documents
without traversing the entire repository.

**Context recovery:** After conversation compaction, `lat overview` restores
awareness of frequently-accessed documents.
