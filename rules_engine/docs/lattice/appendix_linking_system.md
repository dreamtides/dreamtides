# Appendix: Linking System

## Link Syntax

Lattice extends standard markdown link syntax to support ID-based references.

### Document Links

```markdown
See the [design document](LJCQ2) for details.
```

The URL portion contains only a Lattice ID. The link text is arbitrary.

### Section Links

```markdown
Refer to the [error handling](LK1DT) section.
```

Section IDs and document IDs share the same namespace. Resolution checks
the sections table if the ID isn't found in documents.

### Placeholder Links

```markdown
See the [Error Handling](LATTICE) section.
```

The literal string `LATTICE` marks a link for automatic resolution by
`lat annotate`. The link text is used to find a matching header.

## Section ID Assignment

### Header Annotation

Section IDs appear in brackets before the header text:

```markdown
# [LK1DT] Error Handling
## [LK2AB] Common Errors
### [LK3CD] Network Failures
```

### Assignment Depth

The `lat annotate` command assigns IDs based on header depth:

- `--depth 1` (default): Only level-1 headers (`#`)
- `--depth 2`: Levels 1 and 2 (`#`, `##`)
- `--all`: All header levels

Headers already having IDs are not re-assigned.

### ID Placement

IDs always appear immediately after the `#` symbols and before the text:

```markdown
# [ID] Header Text
```

This placement ensures consistent parsing and avoids conflicts with
trailing ID patterns sometimes used in markdown.

## Placeholder Resolution

### Matching Algorithm

For a placeholder link `[Link Text](LATTICE)`:

1. Normalize link text: lowercase, collapse whitespace
2. Scan all section headers in the repository
3. For each header, normalize similarly
4. If normalized link text is a substring of normalized header: match

### Uniqueness Requirement

Resolution requires exactly one match:
- Zero matches: Error "No matching header for 'Link Text'"
- One match: Replace LATTICE with the section ID
- Multiple matches: Warning "Ambiguous: 'Link Text' matches N headers"

### Scope Options

By default, resolution searches the entire repository. Options:

- `--local`: Only search the same file
- `--path <prefix>`: Only search files under path

## Link Storage

### Index Representation

The `links` table stores:

| Column | Description |
|--------|-------------|
| source_id | Document containing the link |
| target_id | Referenced document or section |
| link_type | 'body' or 'frontmatter' |
| position | Order within source (0-indexed) |

### Bidirectional Queries

The index supports both:
- Forward: "What does document X link to?"
- Reverse: "What documents link to X?"

Reverse queries power impact analysis and orphan detection.

## Link Validation

### Missing Targets

The `lat check` command validates all link targets exist:

```
Error: Document LXXXX links to unknown ID LYYYY at line 42
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
├── blocks: LYYYY (in_progress)
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
- Body links: `\[([^\]]+)\]\(([A-Z0-9]+)\)`
- Section IDs: `^#{1,6}\s+\[([A-Z0-9]+)\]`

### Index Updates

When a document changes:
1. Delete all existing links from that source
2. Re-extract links from new content
3. Insert new link records

This is simpler and faster than diffing.
