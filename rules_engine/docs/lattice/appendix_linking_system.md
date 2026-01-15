# Appendix: Linking System

This appendix documents the complete link format specification and edge cases.
See [Lattice Design](lattice_design.md#linking-system) for an overview of the
linking system.

## Link Syntax

Lattice uses standard markdown link syntax with relative file paths and Lattice
ID fragments for stable cross-references.

### Document Links

The canonical link format combines file path with Lattice ID fragment:

```markdown
See the [design document](../design/system_overview.md#LJCQ2X) for details.
```

**Format components:**
- Standard markdown link: `[link text](url)`
- Relative file path: `../design/system_overview.md`
- URL fragment with Lattice ID: `#LJCQ2X`

### Link Authoring (Recommended Workflow)

The recommended workflow for AI agents and humans is to write links using
**shorthand ID-only format**, then run `lat fmt` to expand them:

**Step 1: Write shorthand links:**
```markdown
See the [design document](LJCQ2X) for architecture details.
```

**Step 2: Run `lat fmt`** to expand to canonical format:
```markdown
See the [design document](../design/system_overview.md#LJCQ2X) for architecture details.
```

This avoids needing to look up or remember file paths when authoring.

**Alternative: File path only** (also normalized by `lat fmt`):
```markdown
[design document](../design/system_overview.md)
```
Becomes:
```markdown
[design document](../design/system_overview.md#LJCQ2X)
```

### Path Requirements

All links must use **relative file system paths** from the linking document's
location. Absolute paths and URLs are not supported.

**Valid examples:**
- `[doc](sibling.md#LK3DTX)` - same directory
- `[doc](../parent/file.md#LK3DTX)` - parent directory
- `[doc](subdir/nested/file.md#LK3DTX)` - nested subdirectory

**Invalid examples:**
- `[doc](/absolute/path.md#LK3DTX)` - absolute path
- `[doc](file.md)` - missing Lattice ID fragment (warning)
- `[doc](https://example.com)` - external URL (not a Lattice link)

## Link Storage

### Index Representation

The `links` table stores:

| Column | Description |
|--------|-------------|
| source_id | Document containing the link |
| target_id | Referenced document |
| link_type | 'body' or 'frontmatter' |
| position | Order within source (0-indexed) |

### Bidirectional Queries

The index supports both:
- Forward: "What does document X link to?"
- Reverse: "What documents link to X?"

Reverse queries power impact analysis and orphan detection.

## Link Normalization

### The `lat fmt` Command

The formatter handles link normalization and maintenance:

```bash
lat fmt                    # Normalize all links
```

**Document rename/move detection:**

When documents are renamed or moved, `lat fmt` uses the Lattice ID to find and
update all links pointing to the old path:

```
Before move: [doc](../old/location.md#LJCQ2X)
After move:  [doc](../new/location.md#LJCQ2X)
```

The formatter queries the index to find the current path for each Lattice ID
and rewrites links accordingly. This ensures links remain valid through
repository reorganization.

### Normalization Algorithm

For each link in each document:

1. **Extract components:** Parse link into text, path, and optional fragment
2. **Validate fragment:** If fragment present, verify it's a valid Lattice ID
3. **Resolve target:** Look up document by ID in index
4. **Check path:** Compare link path to actual document path
5. **Update if needed:** Rewrite link if path has changed
6. **Add missing fragment:** If link has path but no fragment, add ID fragment
7. **Add path:** If link has only ID, prepend relative path

### Path Resolution

Relative paths are computed from the linking document's directory:

```
Linker: docs/features/auth.md
Target: docs/design/system.md
Result: ../design/system.md#LJCQ2X
```

The formatter uses standard filesystem path resolution (`.` and `..`
normalization) to compute minimal relative paths.

## Link Validation

### Missing Targets

The `lat check` command validates all link targets exist:

```
Error: Document LXXXXX links to unknown ID LYYYYY at line 42
```

This validates the Lattice ID in the fragment, not the file path. Links are
resolved by ID first, path second.

### Path Mismatches

If a link's file path doesn't match the target document's actual path:

```
Warning: Document LXXXXX has stale link at line 42
  Expected: ../design/current.md#LYYYYY
  Found:    ../design/old.md#LYYYYY
  Run: lat fmt to fix
```

### Missing Fragments

Links with file paths but no Lattice ID fragment generate warnings:

```
Warning: Link missing Lattice ID at line 42: [text](../doc.md)
  Run: lat fmt to add fragment
```

### Circular References

Circular links (A→B→C→A) are valid. They only affect context display,
which doesn't recursively follow links.

### Self-References

A document linking to itself is invalid and produces a warning:

```
Warning: Document LXXXXX contains self-reference at line 15
```

## Frontmatter Links

### Recognized Fields

IDs in these frontmatter fields are indexed as links:

- `blocking`: Tasks this blocks
- `blocked-by`: Tasks blocking this
- `discovered-from`: Parent tasks
- Any custom field with `*-id` or `*-ids` suffix

### Example

```yaml
---
lattice-id: LXXXXX
blocking: [LYYYYY, LZZZZZ]
related-ids: [LWWWWW]
---
```

All IDs in these fields become links with type 'frontmatter'.

## Link Maintenance

### Broken Link Detection

```
$ lat check
Error: Broken link in LXXXXX: target LYYYYY does not exist
```

### Orphan Detection

Documents with no incoming links can be found:

```
$ lat orphans
$ lat orphans --exclude-roots  # Don't report root documents
$ lat orphans --path docs/     # Check only under path
```

This helps identify disconnected documents.

### Index Updates

When a document changes:
1. Delete all existing links from that source
2. Re-extract links from new content
3. Insert new link records

This is simpler and faster than diffing.

## Closed Task Link Handling

When tasks are closed via `lat close`, they move to a `.closed/` subdirectory.
All links pointing to the task are automatically updated to the new path:

```
Before close: [task](../auth/fix_login.md#LXXXXX)
After close:  [task](../auth/.closed/fix_login.md#LXXXXX)
```

This rewriting happens automatically during `lat close`, similar to `lat mv`.
The Lattice ID in the fragment remains unchanged, only the path updates.

### Reopening Tasks

When `lat reopen` moves a task back from `.closed/`, links are rewritten again:

```
Before reopen: [task](../auth/.closed/fix_login.md#LXXXXX)
After reopen:  [task](../auth/fix_login.md#LXXXXX)
```

### Pruning Tasks

When `lat prune <path>` or `lat prune --all` permanently deletes closed tasks:

1. **YAML frontmatter references** (`blocking`, `blocked-by`, `discovered-from`)
   are automatically removed from all documents
2. **Inline markdown links** produce an error by default
3. With `--force`, inline links are converted to plain text:
   ```
   Before prune: See [the old task](../auth/.closed/fix_login.md#LXXXXX) for context.
   After prune:  See the old task for context.
   ```

The index is updated to remove all references to pruned documents.