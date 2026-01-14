# Appendix: Hierarchy and Relationships

## Filesystem Hierarchy

### Directory Structure Commands

**Tree View:**
```
lat tree [path] [options]
```

Options:
- `--depth N`: Maximum depth to display (default: unlimited)
- `--counts`: Show document counts per directory
- `--issues-only`: Show only directories with issues
- `--docs-only`: Show only directories with knowledge base docs

Example output:
```
project/
├── !overview.md (LROOT)
├── auth/ (4 docs, 3 issues)
│   ├── !authentication.md (LAUTH)
│   ├── login-flow.md
│   ├── oauth-design.md
│   └── issues/
│       ├── login-bug.md [bug/P1/open]
│       ├── oauth-feature.md [feature/P2/open]
│       └── session-timeout.md [bug/P3/closed]
├── api/ (6 docs, 2 issues)
│   ├── !api-design.md (LAPI)
│   └── ...
└── docs/ (12 docs, 0 issues)
    └── ...
```

**Root Documents:**
```
lat roots [options]
```

Lists all root documents (`!*.md`) with their paths and child counts.

```
Root documents:

LROOT  /!overview.md              12 children
LAUTH  /auth/!authentication.md    7 children
LAPI   /api/!api-design.md         8 children
LDOCS  /docs/!index.md            15 children
```

**Children Query:**
```
lat children <root-id> [options]
```

Lists documents under a root document's directory.

Options:
- `--recursive`: Include nested directories
- `--issues`: Show only issues
- `--docs`: Show only knowledge base documents

```
Children of LAUTH (auth/!authentication.md):

Documents:
  LOAUTH  oauth-design.md
  LLOGIN  login-flow.md
  LSESS   session-management.md

Issues:
  LBUG1   login-bug.md          [bug/P1/open]
  LFEAT1  oauth-feature.md      [feature/P2/in_progress]
  LBUG2   session-timeout.md    [bug/P3/closed]
```

### Root Document Rules

1. **Single Root**: Each directory should have at most one root document
2. **Naming**: Root documents start with `!` (e.g., `!overview.md`)
3. **Epic Role**: Root documents act as the "epic" for sibling issues
4. **Context Inheritance**: Children inherit context from ancestor roots

**Multiple Root Detection:**

If a directory contains multiple `!*.md` files, `lat check` warns:
```
Warning: Directory auth/ has multiple root documents:
  !authentication.md
  !auth-overview.md
Consider consolidating into a single root.
```

**Missing Root Handling:**

Directories without root documents:
- Use parent directory's root as implicit parent
- `lat tree` shows "(no root)" indicator
- `lat check --strict` warns about missing roots

## Issue Tracking Fields

### Extended Fields

Beyond the basic fields, Lattice supports richer issue tracking:

**Acceptance Criteria:**
```yaml
---
lattice-id: LXXXX
issue-type: feature
acceptance:
  - User can log in with OAuth
  - Session persists across browser refresh
  - Logout properly clears session
---
```

**Design Notes:**
```yaml
---
design-notes: |
  Using OAuth 2.0 PKCE flow for security.
  Token refresh handled by background job.
  See LOAUTH for full design.
---
```

**Progress Tracking:**
```yaml
---
progress: 60  # Percentage complete (0-100)
estimate: 3d  # Time estimate
---
```

### Querying Extended Fields

```
lat list --has-acceptance        # Issues with acceptance criteria
lat list --progress-below 50     # Less than 50% complete
lat list --has-estimate          # Issues with time estimates
```

### Issue Metadata Display

When showing issues, `lat show` renders metadata in human-readable format:

```
# implement-oauth
─────────────────────────────────────────
Type:       feature
Status:     in_progress (●)
Priority:   P1
Labels:     auth, security, q1-2024
Blocked by: LDESIGN (oauth-design) [open]
Progress:   60%

## Acceptance Criteria
☐ User can log in with OAuth
☐ Session persists across browser refresh
☐ Logout properly clears session

## Design Notes
Using OAuth 2.0 PKCE flow for security.
Token refresh handled by background job.
See LOAUTH for full design.
─────────────────────────────────────────

[Issue body content follows...]
```

## Relationship Queries

### Link Analysis

**Forward Links:**
```
lat links-from <id>
```

Shows all documents/sections this document links to, categorized by
link location (body vs frontmatter).

**Backlinks:**
```
lat links-to <id>
```

Shows all documents that link to this document, with context snippets.

**Path Finding:**
```
lat path <id1> <id2>
```

Finds shortest path between two documents through the link graph.
Useful for understanding how documents relate.

### Orphan Detection

```
lat orphans [options]
```

Finds documents with no incoming links (potentially disconnected).

Options:
- `--exclude-roots`: Don't report root documents as orphans
- `--path <prefix>`: Check only under path

```
Orphan documents (no incoming links):

LXXXX  misc/old-notes.md          created 2023-06-15
LYYYY  experiments/failed-idea.md created 2023-09-20

Consider:
- Linking from relevant documents
- Moving to appropriate directory
- Deleting if no longer needed
```

### Impact Analysis

```
lat impact <id>
```

Shows what would be affected by changes to a document:

```
Impact analysis for LXXXX (authentication-design):

Direct dependents (documents linking to this):
  LYYYY  oauth-implementation.md  (3 links)
  LZZZZ  login-flow.md            (1 link)
  LWWWW  security-audit.md        (2 links)

Blocking (issues blocked by this):
  LAAAA  implement-oauth [feature/P1/blocked]

Total documents affected: 4
```

## Statistics

```
lat stats [options]
```

Repository-wide statistics:

```
Lattice Statistics
──────────────────────────────────────

Documents:     234
  Knowledge:   156
  Issues:       78

Issues by Status:
  Open:         23  ████████░░░░░░░░
  In Progress:   8  ███░░░░░░░░░░░░░
  Blocked:       4  ██░░░░░░░░░░░░░░
  Closed:       43  ██████████████░░

Issues by Priority:
  P0:            2
  P1:           12
  P2:           28
  P3:           21
  P4:           15

Links:
  Total:       567
  Avg/doc:     2.4
  Most linked: LROOT (45 backlinks)

Root Documents: 12
Orphans:         3
```

### Link Statistics

```
lat stats links
```

Detailed link analysis:

```
Link Statistics
───────────────────────────────────────

Total links:        567
Body links:         489 (86%)
Frontmatter links:   78 (14%)

Most linked documents:
  LROOT   overview.md            45 backlinks
  LAUTH   authentication.md      32 backlinks
  LAPI    api-reference.md       28 backlinks

Most linking documents:
  LINDEX  documentation-index.md 23 outlinks
  LARCH   architecture.md        18 outlinks

Broken links:       0
Self-references:    2 (warnings)
```
