# Appendix: Context Retrieval

## Design Philosophy

Lattice provides both "push" and "pull" context models. Users can request
automatic context expansion (push) or explicitly query for specific documents
(pull). The default behavior is configurable to suit different workflows.

## The `lat show` Command

### Basic Usage

```
lat show <id> [options]
```

### Context Control Options

**Budget Options:**
- `--context N`: Character budget for context (default 5000, 0 to disable)
- `--references N`: Character budget for references section (default 500)
- `--no-context`: Equivalent to `--context 0 --references 0`
- `--max-context`: Use very large budget (100000 chars) for comprehensive view

**Loading Options:**
- `--peek`: Show only YAML frontmatter, not body content
- `--raw`: Output without formatting or decorations

### Intent-Based Context

The `--intent` flag provides task-appropriate context selection:

```
lat show <id> --intent=<intent>
```

**Available intents:**

| Intent | Included Context |
|--------|------------------|
| `implement` | Design docs, API references, examples, tests |
| `bug-fix` | Related bugs, error docs, stack traces, test cases |
| `review` | Change history, test coverage, related PRs |
| `understand` | Overview docs, glossary, architecture |
| `document` | Style guides, existing docs, examples |

**Intent affects:**
1. Which `doc-context-for` labels are considered relevant
2. Weighting of different document types
3. Default context budget (implementation tasks get more)

### Output Modes

**Default (human-readable):**
```
# document-name
---
lattice-id: LXXXX
name: document-name
...
---

Document body text here.

# related-document
Related document body...

# References
- **other-doc** (LYYYY): Description...
```

**JSON Mode (`--json`):**
```json
{
  "document": {
    "id": "LXXXX",
    "path": "path/to/doc.md",
    "frontmatter": {...},
    "body": "..."
  },
  "context": [
    {"id": "LYYYY", "name": "...", "body": "..."}
  ],
  "references": [
    {"id": "LZZZZ", "name": "...", "description": "..."}
  ]
}
```

## Briefing Mode

For comprehensive task-start context, use the `--brief` flag:

```
lat show <issue-id> --brief
```

Briefing mode assembles:
1. The issue itself with full metadata
2. All blocking and blocked-by issues
3. The directory root document (epic context)
4. Documents linked in the issue body
5. Related design documents (inferred from path/labels)
6. Recently closed related issues (same directory, last 30 days)

**Briefing budget:** Default 30000 characters (configurable via `--context`).

### Briefing for Non-Issues

For knowledge base documents, `--brief` provides:
1. The document itself
2. Directory root documents up to repo root
3. All directly linked documents
4. Backlinks (documents that link to this one)

## Incremental Loading

### Peek Mode

Load only frontmatter:

```
lat show <id> --peek
```

Output:
```yaml
lattice-id: LXXXX
name: document-name
description: Purpose of this document
issue-type: bug
status: open
priority: 1
```

Token-efficient for scanning multiple documents.

## Context Algorithm Details

### Candidate Sources (Priority Order)

1. **doc-context-for matches**: Global label-based inclusion
2. **Body links**: In document order
3. **Directory roots**: Nearest to farthest
4. **Frontmatter links**: In field order

### Sorting Within Sources

- Primary: `doc-priority` (higher first, default 0)
- Secondary: Document order within source

### Inclusion Strategy

The algorithm is greedy but respects atomic documents:
1. Process candidates in priority order
2. If document fits budget, include it
3. If document exceeds budget, skip (don't truncate)
4. Continue until budget exhausted or candidates exhausted

### Output Ordering

The `doc-position` field controls final order:
- Negative: Before main document
- Zero (default): After main document, in candidate order
- Positive: After all default-position documents

## Relationship Queries

For explicit relationship exploration:

```
lat links-from <id>     # Documents this links to
lat links-to <id>       # Documents that link to this
lat path <id1> <id2>    # Shortest link path between documents
```

### Links-From Output

```
LXXXX links to 5 documents:

Body links (3):
  LYYYY  error-handling        "See the [error handling](../docs/error_handling.md#LYYYY) docs"
  LZZZZ  api-reference         "Refer to [API docs](api_reference.md#LZZZZ)"
  LWWWW  testing-guide         "[Testing](testing_guide.md#LWWWW) section"

Frontmatter links (2):
  LAAAA  (blocked-by)
  LBBBB  (discovered-from)
```

### Links-To Output

```
LXXXX is linked from 3 documents:

  LCCCC  overview             line 42: "See [authentication](auth/design.md#LXXXX)"
  LDDDD  security-audit       line 15: "Related: [auth design](../design/auth.md#LXXXX)"
  LEEEE  sprint-3-plan        frontmatter: blocking
```

### Path Output

```
Path from LXXXX to LYYYY (3 hops):

  LXXXX (authentication)
    → LAAAA (security-overview)
    → LBBBB (api-design)
    → LYYYY (error-handling)
```

## Performance Considerations

### Lazy Loading

Document bodies are only loaded when:
1. Document is selected for inclusion
2. Document fits remaining budget

Frontmatter is always loaded (small, needed for filtering).

### Caching and Optimization

Lattice implements a tiered caching strategy for context assembly performance:

- **L1 cache**: In-memory LRU for documents loaded within a command
- **L2 cache**: Persistent content cache for cross-command reuse
- **Budget-aware pruning**: Skip candidates that can't fit remaining budget
- **Parallel loading**: Concurrent I/O for selected context documents

See [Appendix: Context Optimization](appendix_context_optimization.md) for the
complete optimization strategy, benchmarking guidelines, and implementation
priorities.

### Budget Recommendations

| Use Case | Recommended Budget |
|----------|-------------------|
| Quick lookup | `--no-context` |
| Normal viewing | Default (5000) |
| Task briefing | `--brief` (30000) |
| Comprehensive | `--max-context` (100000) |

## Configuration

### Defaults

Set in `.lattice/config.toml`:

```toml
[show]
default_context = 5000
default_references = 500
brief_context = 30000
```

### Per-Document Overrides

Documents can suggest context behavior:

```yaml
---
lattice-id: LXXXX
doc-context-budget: 10000  # Suggest larger budget when showing this
---
```

This is advisory; command-line flags override.
