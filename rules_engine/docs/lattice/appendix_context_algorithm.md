# Appendix: Context Algorithm

## Overview

The context algorithm determines which related documents are included when
viewing a document via `lat show`. It operates under a character budget and
uses a greedy inclusion strategy.

## Parameters

- `--context N`: Maximum characters for related documents (default 5000)
- `--references N`: Maximum characters for reference listing (default 500)

The target document itself is always shown in full, regardless of budget.

## Candidate Generation

Related documents are gathered from multiple sources in priority order:

### Source 1: doc-context-for Matches

Documents declaring `doc-context-for` labels that match any label on the
target document. These are globally searched across the entire repository.

Example: If target has `labels: [authentication]`, find all documents with
`doc-context-for: [authentication]` or similar overlapping labels.

### Source 2: Body Links

Links appearing in the target document's body text, in document order.
Both document links and section links qualify. Section links include
only the referenced section, not the full parent document.

### Source 3: Directory Roots

Root documents (`!*.md`) from the target's directory up to repository root.
Ordered nearest-first (target directory's root before parent's root).

### Source 4: Frontmatter Links

Any document IDs appearing in YAML frontmatter (in `blocking`, `blocked-by`,
`discovered-from`, or other ID-valued fields). Order matches field order
in the YAML.

## Sorting

Within each source category, documents are sorted by `doc-priority`:
- Higher values first (priority 3 before priority 1)
- Default priority is 0
- Negative priorities are valid (appear last within category)

## Inclusion Algorithm

```
function select_context(target, budget):
    candidates = gather_candidates(target)  # From all sources
    candidates = sort_by_priority_within_category(candidates)

    included = []
    remaining_budget = budget

    for doc in candidates:
        if doc.id == target.id:
            continue  # Skip self-reference
        if doc in included:
            continue  # Skip duplicates

        content = get_display_content(doc)
        if len(content) <= remaining_budget:
            included.append(doc)
            remaining_budget -= len(content)

    return included
```

The algorithm is greedy: it takes documents in order until one doesn't fit,
then continues checking subsequent documents (which may be smaller).

## Content Extraction

### For Documents

Display content includes:
- Document name as level-1 header: `# document-name`
- Full body text (markdown content after frontmatter)
- YAML frontmatter is excluded from context documents

### For Sections

When a link targets a section ID:
- Header line of the section
- All content until the next header of equal or higher level
- Nested subsections are included

## Output Ordering

The `doc-position` field controls final output order:

- Position 0 (default): Normal position after target
- Negative positions: Appear before target document
- Positive positions: Explicitly after other context

Within the same position, documents appear in their candidate order.

### Output Structure

```
[Context documents with doc-position < 0]

# target-document-name
---
[target frontmatter]
---
[target body]

# context-doc-1
[context doc 1 body]

# context-doc-2
[context doc 2 body]

[Context documents with doc-position > 0]

# References
- **other-doc-name** (LXXXX): Description text...
- **another-doc** (LYYYY): Description text...
```

## References Section

Documents that qualified as candidates but didn't fit the context budget
are listed in the References section:

- Format: `- **name** (ID): description...`
- Sorted by their original candidate order
- Truncated to fit `--references` budget
- Shows as many entries as fit; partial entries are not shown

## Issue-Specific Behavior

When the target is an issue document:

### Metadata Display

Issue frontmatter fields are rendered human-readably:
- Status with color indicator (green for open, yellow for in_progress, etc.)
- Priority with P0-P4 label
- Labels as comma-separated list
- Blocking/blocked-by as ID lists

### Ready Status

If showing an issue via `lat ready`, additional context:
- Blockers are automatically included (from blocked-by field)
- Priority sorting is already applied by the ready command

## Edge Cases

### Circular References

If A links to B and B links to A:
- Showing A includes B in context
- B's links are not recursively followed
- Only direct references from target are considered

### Self-Reference

A document linking to itself is skipped in context gathering.

### Empty Context

If no related documents exist or budget is 0:
- Only the target document is shown
- References section is omitted if empty

### Large Documents

If a single context document exceeds the entire budget:
- It is skipped (not truncated)
- Listed in References section instead

## Performance Considerations

### Index Usage

Context gathering uses the index for:
- `doc-context-for` label lookup (indexed query)
- Link target resolution (links table)
- Root document discovery (path prefix query)

### Lazy Content Loading

Document bodies are only loaded when:
1. The document fits the remaining budget
2. The document is selected for inclusion

This prevents loading large documents that won't be shown.

### Caching

Repeated `lat show` commands don't cache context results. Each invocation
re-runs the algorithm. For repeated queries, consider shell caching or
tooling integration.
