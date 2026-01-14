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
See the [design document](../design/system_overview.md#LJCQ2) for details.
```

**Format components:**
- Standard markdown link: `[link text](url)`
- Relative file path: `../design/system_overview.md`
- URL fragment with Lattice ID: `#LJCQ2`

### Link Authoring (Recommended Workflow)

The recommended workflow for AI agents and humans is to write links using
**shorthand ID-only format**, then run `lat fmt` to expand them:

**Step 1: Write shorthand links:**
```markdown
See the [design document](LJCQ2) for architecture details.
```

**Step 2: Run `lat fmt`** to expand to canonical format:
```markdown
See the [design document](../design/system_overview.md#LJCQ2) for architecture details.
```

This avoids needing to look up or remember file paths when authoring.

**Alternative: File path only** (also normalized by `lat fmt`):
```markdown
[design document](../design/system_overview.md)
```
Becomes:
```markdown
[design document](../design/system_overview.md#LJCQ2)
```

### Path Requirements

All links must use **relative file system paths** from the linking document's
location. Absolute paths and URLs are not supported.

**Valid examples:**
- `[doc](sibling.md#LK1DT)` - same directory
- `[doc](../parent/file.md#LK1DT)` - parent directory
- `[doc](subdir/nested/file.md#LK1DT)` - nested subdirectory

**Invalid examples:**
- `[doc](/absolute/path.md#LK1DT)` - absolute path
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

**Basic normalization:**
```bash
lat fmt                    # Add ID fragments to file-only links
lat fmt --add-links        # Convert ID-only links to path+fragment
```

**Document rename/move detection:**

When documents are renamed or moved, `lat fmt` uses the Lattice ID to find and
update all links pointing to the old path:

```
Before move: [doc](../old/location.md#LJCQ2)
After move:  [doc](../new/location.md#LJCQ2)
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
7. **Add path (with --add-links):** If link has only ID, prepend relative path

### Path Resolution

Relative paths are computed from the linking document's directory:

```
Linker: docs/features/auth.md
Target: docs/design/system.md
Result: ../design/system.md#LJCQ2
```

The formatter uses standard filesystem path resolution (`.` and `..`
normalization) to compute minimal relative paths.

## Link Validation

### Missing Targets

The `lat check` command validates all link targets exist:

```
Error: Document LXXXX links to unknown ID LYYYY at line 42
```

This validates the Lattice ID in the fragment, not the file path. Links are
resolved by ID first, path second.

### Path Mismatches

If a link's file path doesn't match the target document's actual path:

```
Warning: Document LXXXX has stale link at line 42
  Expected: ../design/current.md#LYYYY
  Found:    ../design/old.md#LYYYY
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

A document linking to itself is valid but produces a warning:

```
Warning: Document LXXXX contains self-reference at line 15
```

## Frontmatter Links

### Recognized Fields

IDs in these frontmatter fields are indexed as links:

- `blocking`: Issues this blocks
- `blocked-by`: Issues blocking this
- `discovered-from`: Parent issues
- Any custom field with `*-id` or `*-ids` suffix

### Example

```yaml
---
lattice-id: LXXXX
blocking: [LYYYY, LZZZZ]
related-ids: [LWWWW]
---
```

All IDs in these fields become links with type 'frontmatter'.

## Link Display

### In Document View

Links render with their target's name on hover (in supported terminals)
and navigate when clicked (in supported environments).

### In List Output

The `lat list` command shows link counts:

```
LXXXX  my-document  3 links, 2 backlinks
```

### In Dependency Tree

The `lat dep tree` command visualizes blocking relationships:

```
LXXXX (open)
├── blocks: LYYYY (open)
│   └── blocks: LZZZZ (open)
└── blocks: LWWWW (blocked)
```

## Link Maintenance

### Broken Link Detection

```
$ lat check
Error: Broken link in LXXXX: target LYYYY does not exist
```

### Orphan Detection

Documents with no incoming links can be found:

```
$ lat list --no-backlinks
```

This helps identify disconnected documents.

### Link Statistics

```
$ lat stats links
Total links: 1234
Average per document: 3.2
Most linked: LXXXX (45 backlinks)
```

## Performance Notes

### Link Extraction

During parsing, links are extracted via regex:
- Body links: `\[([^\]]+)\]\(([^)]+)\)`
- Fragment extraction: `#([A-Z0-9]+)$`

The parser extracts both file paths and Lattice ID fragments from each link.

### Index Updates

When a document changes:
1. Delete all existing links from that source
2. Re-extract links from new content
3. Insert new link records

This is simpler and faster than diffing.

### Formatter Performance

The `lat fmt` command builds a path→ID mapping from the index before processing
documents, enabling O(1) ID lookups during link normalization. For large
repositories (10,000+ documents), this provides significant speedup over
per-link index queries.
