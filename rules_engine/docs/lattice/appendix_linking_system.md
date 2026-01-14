# Appendix: Linking System

## Link Syntax

Lattice extends standard markdown link syntax to support ID-based references.

### Document Links

```markdown
See the [design document](LJCQ2) for details.
```

The URL portion contains only a Lattice ID. The link text is arbitrary.

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

### Index Updates

When a document changes:
1. Delete all existing links from that source
2. Re-extract links from new content
3. Insert new link records

This is simpler and faster than diffing.
