# Appendix: Task Templates

This appendix documents the task template system. See
[Lattice Design](lattice_design.md#task-templates) for an overview.

## Core Concept

Task templates provide reusable context and acceptance criteria through the
existing directory hierarchy. Rather than explicit template references, Lattice
leverages root documents (documents whose filename matches their directory name,
e.g., `api/api.md`) to automatically compose template content for all tasks in
that directory and its subdirectories.

This design requires no additional frontmatter fields. The filesystem hierarchy
IS the template structure. When a template changes, all tasks in that subtree
immediately reflect the update at display time.

## Template Sections in Root Documents

Root documents (filename matches directory name) can include two specially-marked
sections that serve as templates for descendant tasks:

```yaml
---
lattice-id: LXXXXX
task-type: epic
priority: 1
name: api
description: API subsystem tasks
---

# API Subsystem Epic

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
├── project.md           # Project-wide context and acceptance
├── api/
│   ├── api.md           # API context and acceptance
│   ├── docs/
│   │   └── api_spec.md  # Knowledge base document
│   └── tasks/
│       └── create/
│           ├── create.md    # Create-endpoint context and acceptance
│           └── tasks/
│               └── fix_bug.md   # Task inherits from ancestors
```

For `fix_bug.md`, the composition is:

**Context order (general → specific):**
1. `project.md` (project root) [Lattice] Context section
2. `api/api.md` [Lattice] Context section
3. `api/tasks/create/create.md` [Lattice] Context section
4. `fix_bug.md` body

**Acceptance order (specific → general):**
1. `api/tasks/create/create.md` [Lattice] Acceptance Criteria section
2. `api/api.md` [Lattice] Acceptance Criteria section
3. `project.md` (project root) [Lattice] Acceptance Criteria section

This ordering ensures tasks receive appropriately scoped context up front
(broad project context first, then domain-specific details), while universal
requirements anchor the acceptance criteria at the end.

## Display Rendering

The `lat show` command composes ancestor templates with task content:

```
$ lat show LZZZZZ
LZZZZZ: Fix validation bug in create endpoint
State: open
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
  LAAAAA: create-endpoint [epic]
...
```

The `--raw` flag skips template composition, showing only the task's own
content without ancestor context or acceptance criteria.

## Root Document Requirements

For a document to provide template content to descendants:

1. **Must be a root document:** Filename (without `.md`) matches the containing
   directory name (e.g., `api/api.md`, `create/create.md`)
2. **Must have marked sections:** Include `[Lattice] Context` and/or
   `[Lattice] Acceptance Criteria` headings
3. **Sections are optional:** A root without these sections simply provides
   no template content (descendants still inherit from higher ancestors)

Non-root documents (those whose filename does not match the directory name)
never provide template content, even if they contain `[Lattice]` sections.

## Skipping Levels

Not every directory needs a root document. Missing roots are simply skipped
in the ancestry chain:

```
project/
├── project.md           # Has [Lattice] Context
├── api/
│   └── tasks/
│       └── create/
│           ├── create.md    # Has [Lattice] Context (no api/api.md exists)
│           └── tasks/
│               └── implement_endpoint.md
```

The task inherits from `project.md` and `create.md` directly. The missing
`api/api.md` creates no gap—the chain simply doesn't include that level.

## Linter Rules

`lat check` enforces W016: template sections in non-root documents. See
[Appendix: Linter](appendix_linter.md#w016-template-section-in-non-root)
for details.

## JSON Output

Template information appears in `lat show --json` output:

```json
{
  "id": "LZZZZZ",
  "description": "Fix validation bug in create endpoint",
  "ancestors": [
    {"id": "LPROJX", "name": "project", "path": "project.md"},
    {"id": "LAPIXX", "name": "api", "path": "api/api.md"},
    {"id": "LCREAX", "name": "create", "path": "api/tasks/create/create.md"}
  ],
  "composed_context": "Project context...\nAPI context...\nCreate context...",
  "composed_acceptance": "- [ ] Create checks\n- [ ] API checks\n- [ ] Project checks",
  "body": "The validation logic incorrectly..."
}
```

The `ancestors` array lists root documents in hierarchy order. The
`composed_context` and `composed_acceptance` fields contain fully resolved
content after walking the ancestor chain.
