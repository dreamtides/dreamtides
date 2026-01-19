# Appendix: AI Integration

This appendix documents AI agent integration. See
[Lattice Design](lattice_design.md#skill-integration) for Lattice's general
approach to AI compatibility and [Appendix: Workflow](appendix_workflow.md) for
command specifications.

## Design Goals

AI agents benefit from using Lattice commands rather than direct file reads:

- **`lat show`** provides composed template content, dependencies, and related
  documents in a single call
- **`lat ready`** surfaces actionable work without manual filtering
- **`lat overview`** provides curated context for session orientation
- **View tracking** improves `lat overview` recommendations over time

Direct file reads miss template composition and don't contribute to view
tracking.

## Skill Documents

Documents with `skill: true` in frontmatter become Claude Skills via automatic
symlink generation. See [Appendix: Startup Operations](appendix_startup_operations.md)
for sync timing.

**Requirements for skill documents:**

- `name`: Max 64 characters, no "anthropic" or "claude" substrings
- `description`: Max 1024 characters, non-empty
- No XML-like tags in name

**Symlink location:** `.claude/skills/<name>.md` → actual document path

## Recommended AI Workflows

**Session start:**
```bash
lat prime             # Workflow reminders
lat overview          # Get critical document context
lat ready             # See available work
```

**Working on a task:**
```bash
lat show LXXXXX       # Full context with templates and dependencies
lat claim LXXXXX      # Mark as in-progress locally
# ... do work ...
lat close LXXXXX      # Mark complete (auto-releases claim)
```

**Before committing:**
```bash
lat check             # Validate all documents
lat fmt               # Normalize links
```

**Context recovery after compaction:**
```bash
lat prime             # Workflow reminders
lat overview          # Most-viewed documents
```