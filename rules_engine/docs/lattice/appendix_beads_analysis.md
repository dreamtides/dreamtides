# Appendix: Beads CLI Analysis

This appendix documents the detailed analysis of beads' CLI behavior for
`bd show`, `bd ready`, and `bd list` commands, identifying patterns to
preserve and patterns to adapt for Lattice's filesystem-centric model.

For the Lattice implementations of these commands, see:
- [Appendix: Workflow](appendix_workflow.md) for show, ready, prime, and claim
- [Appendix: CLI Structure](appendix_cli_structure.md) for the complete command reference

## Command Analysis: bd show

### Core Behavior

The `bd show <id>` command displays complete issue details including:

1. **Header line**: ID, title
2. **Metadata block**: Status, priority, type, timestamps, creator
3. **Description**: Full markdown body
4. **Children section**: List of child issues with status indicators

Example output structure:
```
dr-ulj: Tabula V2: Complete Card Data Loading Rewrite
Status: open
Priority: P1
Type: epic
Created: 2026-01-09 18:16
Created by: dthurn
Updated: 2026-01-09 18:16

Description:
[full markdown content]

Children (21):
  -> dr-ulj.1: Convert strings.toml... [P1 - open]
  -> dr-ulj.2: Update TOML files... [P1 - closed]
```

### Key Flags

| Flag | Behavior |
|------|----------|
| `--short` | Single-line output: `id [status] priority type: title` |
| `--refs` | Reverse lookup showing what references this issue |
| `--thread` | Full conversation thread for messages |
| `--json` | Structured JSON output with all fields |

### Edge Cases

**Ambiguous ID Resolution**: When an ID prefix matches multiple issues,
beads returns an error with suggestions:
```
ambiguous ID "dr-2" matches 8 issues: [dr-2z0 dr-2pp ...]
Use more characters to disambiguate
```

**Lattice Adaptation**: Lattice should implement similar prefix matching
with clear disambiguation messages. The error should list matching IDs
sorted by relevance (most recently updated first).

**Child Display**: Beads shows children inline with status indicators
`[P1 - open]` and `[P1 - closed]`. This inline status is essential for
quick triage without expanding each child.

**Lattice Adaptation**: For epics (directory roots), Lattice should show
sibling documents in the same directory with similar status indicators.

### Patterns to Preserve

1. **Metadata-first layout**: Status/priority/type appear before description
2. **Status indicators in child lists**: `[P1 - open]` format
3. **JSON output structure**: All fields exposed for programmatic access
4. **Reverse reference lookup**: `--refs` essential for impact analysis

### Patterns to Adapt

1. **Children section**: Replace with "Siblings" showing directory contents
2. **Epic association**: Implicit via directory root rather than parent field
3. **Thread support**: Not applicable to filesystem documents

## Command Analysis: bd ready

### Core Behavior

The `bd ready` command shows work available to start:
- Status is `open` or `pinned`
- No blocking dependencies (all `blocked-by` issues are closed)
- Default excludes P4 (backlog) priorities

Default output:
```
Ready work (3 issues with no blockers):

1. [P1] [epic] dr-ulj: Tabula V2: Complete Card Data Loading Rewrite
2. [P1] [task] dr-ulj.1: Convert strings.toml...
3. [P1] [epic] dr-67z: LLMC v2: Agent Coordination System
```

### Sort Policies

| Policy | Behavior |
|--------|----------|
| `hybrid` | Default. Priority first, then creation date |
| `priority` | Strict priority ordering (P0 > P1 > P2...) |
| `oldest` | Creation date ascending |

### Key Flags

| Flag | Behavior |
|------|----------|
| `--limit N` | Maximum issues (default 10) |
| `--type T` | Filter by issue type |
| `--pretty` | Tree format with status symbols |
| `--parent ID` | Filter to descendants of an epic |
| `--assignee` | Filter by assignee |
| `--unassigned` | Only unassigned issues |

### Pretty Format

The `--pretty` flag provides a visual tree with status symbols:
```
o [orange] dr-ulj - [EPIC] Tabula V2...
|-- o [orange] dr-ulj.1 - Convert strings.toml...

o [orange] dr-67z - [EPIC] LLMC v2...

Legend: o open | (x) blocked | [red] P0 | [orange] P1...
```

### Edge Cases

**Empty Results**: When no issues are ready:
```
No ready work found (all issues have blocking dependencies)
```

**Lattice Adaptation**: Lattice should provide the same clear empty-state
message rather than silent output.

**Molecule Filtering**: Beads supports `--mol` to filter by molecule (work
template). Lattice does not have molecules, but the `--path` filter provides
equivalent functionality for directory-scoped queries.

### Patterns to Preserve

1. **Hybrid sort default**: Priority then age prevents starvation
2. **Pretty tree format**: Visual hierarchy with symbols
3. **Status symbol legend**: Self-documenting output
4. **Limit default**: 10 prevents overwhelming output
5. **Empty state messaging**: Clear explanation when nothing ready

### Patterns to Adapt

1. **--parent filter**: Replace with `--path` for directory filtering
2. **Molecule filtering**: Not applicable; use directory structure
3. **Assignee tracking**: Not applicable; use `lat claim` for work tracking

## Command Analysis: bd list

### Core Behavior

The `bd list` command provides flexible issue querying with extensive
filtering. Default shows open issues sorted by priority.

Default output:
```
dr-5dl [P0] [bug] open - Fix crash count not being incremented
dr-t0z [P0] [epic] open - Fix LLMC v2 code review issues
dr-2z0 [P1] [bug] open - Fix stuck worker nudging
```

### Filter Categories

**Status Filters**:
- `--status=open|blocked|deferred|closed`
- `--all` includes closed issues

**Time Filters**:
- `--created-after`, `--created-before`
- `--updated-after`, `--updated-before`
- `--closed-after`, `--closed-before`
- Supports ISO dates and relative: `+6h`, `tomorrow`, `next monday`

**Content Filters**:
- `--title`, `--title-contains`
- `--desc-contains`
- `--notes-contains`

**Label Filters**:
- `--label L1,L2` (AND: must have ALL)
- `--label-any L1,L2` (OR: must have ANY)
- `--no-labels`

**Priority Filters**:
- `--priority N` (exact)
- `--priority-min N`, `--priority-max N` (range)

**Structure Filters**:
- `--parent ID` (children of epic)
- `--type T` (bug/feature/task/epic/chore)

### Output Formats

| Format | Description |
|--------|-------------|
| Default | One-line with metadata: `id [priority] [type] status - title` |
| `--long` | Multi-line with description |
| `--pretty` | Tree with status symbols |
| `--json` | Full structured data |
| `--format digraph` | For graph tools |
| `--format dot` | Graphviz format |

### Edge Cases

**Large Result Sets**: Default limit is 50. Use `--limit 0` for unlimited.
Lattice should preserve this pagination behavior.

**Empty Label Query**: `--label nonexistent` returns empty set without
error. This is correct behavior (empty results != error).

**Combining Filters**: Filters combine with AND logic. `--status=open
--priority 0` shows open P0 issues only.

**Relative Dates**: Beads parses `+6h`, `tomorrow`, `next monday`. Lattice
should implement equivalent parsing for `--updated-after` etc.

### Patterns to Preserve

1. **Rich filtering**: Time, label, priority, content filters
2. **AND/OR label logic**: `--label` vs `--label-any`
3. **Relative date parsing**: Essential for quick queries
4. **Default limit**: 50 prevents runaway queries
5. **Multiple output formats**: JSON, pretty, long
6. **Sort flexibility**: By priority, created, updated, etc.

### Patterns to Adapt

1. **--parent filter**: Replace with `--path` prefix filter
2. **Epic type**: Implicit via root document convention
3. **assignee**: Not applicable; use `lat claim` for local work tracking

## Command Analysis: bd sync

### Core Behavior

The `bd sync` command synchronizes local state with remote:
1. Export pending changes to JSONL
2. Commit to git
3. Pull from remote
4. Import any updates
5. Push to remote

### Lattice Adaptation: No Sync Command

Lattice never performs sync operations.
This is a fundamental design difference from beads:

1. **No git push**: Lattice never pushes to remote
2. **No auto-commit**: Commits are explicit user operations
3. **Local-only claims**: Work tracking via `lat claim` instead of sync

This design supports multi-agent workflows where:
- Agents work in isolated git worktrees
- A coordinator manages merging and synchronization
- Push operations require explicit external control

### Patterns to Preserve

None. The sync model is intentionally different.

### Patterns to Adapt

1. **in_progress status**: Replace with `lat claim` local tracking
2. **Remote sync**: External coordinator responsibility
3. **Conflict detection**: Handled at coordinator level

## Additional Commands to Preserve

### bd blocked

Shows issues in blocked status with blocking relationships:
```
dr-xxx: Issue title
  Blocked by: dr-yyy (open), dr-zzz (blocked)
```

**Lattice Equivalent**: `lat blocked` showing `blocked-by` relationships.

### bd dep tree

Visualizes dependency graph for an issue:
```
Dependency tree for dr-ulj:
dr-ulj: Title [P1] (open) [READY]
  +-- dr-yyy: Child [P1] (blocked)
      +-- dr-zzz: Grandchild [P2] (open)
```

**Lattice Equivalent**: `lat dep tree` with similar visualization.

### bd prime

Outputs AI workflow context:
```
# Beads Workflow Context

> **Context Recovery**: Run `bd prime` after compaction or new session

## Session Protocol

Before completing work:
[ ] 1. bd sync
...
```

**Lattice Equivalent**: `lat prime` with modified protocol (no sync step).

## Summary: Key Adaptations for Lattice

### Structural Changes

| Beads Concept | Lattice Equivalent |
|---------------|-------------------|
| `--parent ID` | `--path <directory>` |
| Epic type | Root document (`!*.md`) |
| Parent-child | Directory siblings |
| `bd create --parent` | `lat create --path` |
| `in_progress` status | `lat claim` (local) |
| `bd sync` | Not applicable |
| Assignee field | Not applicable |

### Preserved Behaviors

1. **ID ambiguity handling**: Error with suggestions
2. **Status indicators**: `[P1 - open]` in listings
3. **Sort policies**: hybrid/priority/oldest
4. **Pretty tree output**: Status symbols and legend
5. **Empty state messages**: Clear explanations
6. **Filter combinability**: AND logic by default
7. **Relative date parsing**: `+6h`, `tomorrow`
8. **JSON output**: All commands support `--json`

### New Lattice-Specific Features

1. **Local claiming**: `lat claim` tracks work without file modification
2. **Section references**: Link to document sections, not just documents
3. **Knowledge base integration**: Issues and docs share ID space
4. **Directory roots**: `!*.md` convention for hierarchy
5. **No push operations**: Sync handled externally

## Implementation Recommendations

### Priority Order

1. **Core viewing**: `lat show` with bd-compatible output
2. **Work discovery**: `lat ready` with filters and pretty output
3. **Flexible search**: `lat list` with beads-equivalent filters
4. **Work tracking**: `lat claim` for local progress tracking

### Test Coverage Priorities

Focus tests on edge cases identified above:
- Ambiguous ID resolution
- Empty result messaging
- Filter combinations
- Relative date parsing
- Large result set pagination
- Status symbol rendering
- Claim state display and lifecycle
