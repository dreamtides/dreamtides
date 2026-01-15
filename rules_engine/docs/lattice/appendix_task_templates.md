# Appendix: Task Templates

This appendix documents the task template system. See
[Lattice Design](lattice_design.md#task-templates) for an overview.

## Core Concept

Task templates provide reusable context and acceptance criteria through the
existing directory hierarchy. Rather than explicit template references, Lattice
leverages directory root documents (`00_*.md` files) to automatically compose
template content for all tasks in that directory and its subdirectories.

This design requires no additional frontmatter fields. The filesystem hierarchy
IS the template structure. When a template changes, all tasks in that subtree
immediately reflect the update at display time.

## Template Sections in Root Documents

Directory root documents can include two specially-marked sections that serve
as templates for descendant tasks:

```yaml
---
lattice-id: LXXXX
task-type: epic
status: open
priority: 1
name: api-design
description: API subsystem tasks
---

# API Design Epic

Overview of the API subsystem work.

## [Lattice] Context

The API layer handles all external requests. Key considerations:
- REST conventions following OpenAPI 3.0
- Authentication via JWT tokens
- Rate limiting at 1000 req/min per client

## [Lattice] Acceptance Criteria

- [ ] Update API documentation
- [ ] Ensure backwards compatibility
- [ ] Add integration tests
```

The `[Lattice] Context` section prepends to descendant task bodies when
displayed. The `[Lattice] Acceptance Criteria` section appends. Other content
in the root document (like the overview above) is not composed into descendants.

## Section Detection

Template sections are identified by the `[Lattice]` prefix in headings:

- `[Lattice] Context` - provides context for descendant tasks
- `[Lattice] Acceptance Criteria` - provides acceptance criteria for descendants

Any heading level works (`#` through `######`). The section extends from the
heading to the next heading of the same or higher level, or end of document.

## Hierarchy Composition

When displaying a task, Lattice walks up the directory tree collecting all
ancestor root documents. Template content composes in hierarchy order:

```
project/
├── 00_project.md        # Project-wide context and acceptance
├── api/
│   ├── 00_api.md        # API context and acceptance
│   └── create/
│       ├── 00_create.md # Create-endpoint context and acceptance
│       └── fix_bug.md   # Task inherits from all three ancestors
```

For `fix_bug.md`, the composition is:

**Context order (general → specific):**
1. `00_project.md` [Lattice] Context section
2. `api/00_api.md` [Lattice] Context section
3. `api/create/00_create.md` [Lattice] Context section
4. `fix_bug.md` body

**Acceptance order (specific → general):**
1. `api/create/00_create.md` [Lattice] Acceptance Criteria section
2. `api/00_api.md` [Lattice] Acceptance Criteria section
3. `00_project.md` [Lattice] Acceptance Criteria section

This ordering ensures tasks receive appropriately scoped context up front
(broad project context first, then domain-specific details), while universal
requirements anchor the acceptance criteria at the end.

## Display Rendering

The `lat show` command composes ancestor templates with task content:

```
$ lat show LZZZZ
LZZZZ: Fix validation bug in create endpoint
Status: open
Priority: P2
Type: task

Context:
  [Project context]
  [API context]
  [Create-endpoint context]

Description:
  The validation logic incorrectly rejects valid input when...

Acceptance Criteria:
  - [ ] Create-endpoint specific checks
  - [ ] API-wide checks (documentation, backwards compat)
  - [ ] Project-wide checks (git commit, run tests)

Parent:
  LAAAA: create-endpoint [epic]
...
```

The `--raw` flag skips template composition, showing only the task's own
content without ancestor context or acceptance criteria.

## Root Document Requirements

For a document to provide template content to descendants:

1. **Must be a directory root:** Filename starts with `00_` prefix
2. **Must have marked sections:** Include `[Lattice] Context` and/or
   `[Lattice] Acceptance Criteria` headings
3. **Sections are optional:** A root without these sections simply provides
   no template content (descendants still inherit from higher ancestors)

Non-root documents (`01_*.md`, `02_*.md`, or unprefixed files) never provide
template content, even if they contain `[Lattice]` sections.

## Skipping Levels

Not every directory needs a root document. Missing roots are simply skipped
in the ancestry chain:

```
project/
├── 00_project.md        # Has [Lattice] Context
├── api/
│   └── create/
│       ├── 00_create.md # Has [Lattice] Context (no 00_api.md exists)
│       └── task.md
```

Task inherits from `00_project.md` and `00_create.md` directly. The missing
`api/00_api.md` creates no gap—the chain simply doesn't include that level.

## Linter Rules

### W023: Template Section in Non-Root

Non-root document contains `[Lattice]` template sections.

**Detection:** Check for `[Lattice]` headings in non-`00_` prefixed files.

**Message:** `Warning [W023]: path/task.md has [Lattice] sections but is not a root`

**Fix:** Move content to the directory's root document or remove sections.

## JSON Output

Template information appears in `lat show --json` output:

```json
{
  "id": "LZZZZ",
  "title": "Fix validation bug in create endpoint",
  "ancestors": [
    {"id": "LPROJ", "name": "project-overview", "path": "00_project.md"},
    {"id": "LCREA", "name": "create-endpoint", "path": "api/create/00_create.md"}
  ],
  "composed_context": "Project context...\nAPI context...\nCreate context...",
  "composed_acceptance": "- [ ] Create checks\n- [ ] API checks\n- [ ] Project checks",
  "body": "The validation logic incorrectly..."
}
```

The `ancestors` array lists root documents in hierarchy order. The
`composed_context` and `composed_acceptance` fields contain fully resolved
content after walking the ancestor chain.
