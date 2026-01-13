# Appendix: Markdown Linter Integration

## Overview

This appendix evaluates existing markdown linters for potential integration with
Lattice's `lat check` command and recommends an implementation strategy.

## Linters Evaluated

### markdownlint (DavidAnson/markdownlint)

**Repository:** https://github.com/DavidAnson/markdownlint

**Architecture:** Monolithic Node.js library with ~60 built-in rules. Uses
micromark parser and honors CommonMark specification with GFM support.

**Key Characteristics:**
- Rules identified by numeric IDs (MD001-MD060) and descriptive aliases
- Configuration via `.markdownlint.json` or `markdownlint.yaml`
- Supports auto-fixing via `--fix` flag
- Custom rules possible via npm packages
- CLI available via `markdownlint-cli` and `markdownlint-cli2`

**Notable Rules:**
| Rule | Description | Lattice Alignment |
|------|-------------|-------------------|
| MD003 | Heading style consistency | W007 |
| MD004 | Unordered list style | W008 |
| MD009 | Trailing spaces | Formatting |
| MD013 | Line length | Token efficiency |
| MD024 | Duplicate headings | Document structure |
| MD025 | Single top-level heading | Document structure |
| MD033 | Inline HTML | S003 (no XML) |
| MD034 | Bare URLs | W009 |
| MD041 | First line heading | Document structure |
| MD047 | Final newline | File format |
| MD051 | Link fragment validity | E002 |
| MD052 | Reference label definition | E002 |

### remark-lint

**Repository:** https://github.com/remarkjs/remark-lint

**Architecture:** Plugin-based system built on unified/remark ecosystem. Each
rule is a separate npm package (~80 official rules).

**Key Characteristics:**
- Granular: install only rules you need
- Built on AST manipulation (mdast)
- Preset packages for common configurations
- Highly extensible custom rule support
- Configuration via `.remarkrc` or `package.json`

**Notable Rules:**
| Rule | Description | Lattice Alignment |
|------|-------------|-------------------|
| heading-increment | Heading levels advance by one | Document structure |
| heading-style | Consistent heading syntax | W007 |
| unordered-list-marker-style | Consistent list markers | W008 |
| no-literal-urls | URLs need angle brackets | W009 |
| no-html | No raw HTML elements | S003 |
| no-duplicate-headings | Unique headings | Document structure |
| maximum-line-length | Line length limit | Token efficiency |
| final-newline | File ends with newline | File format |
| no-undefined-references | References must exist | E002 |

**Presets:**
- `remark-preset-lint-recommended`: Prevents common mistakes
- `remark-preset-lint-consistent`: Enforces consistency
- `remark-preset-lint-markdown-style-guide`: Stricter style rules

## Claude Skill Validation Requirements

From Claude's official skill authoring documentation, these rules are
mechanically verifiable:

### Required Validations (Lattice implements as errors)

| Requirement | Field | Rule |
|-------------|-------|------|
| Maximum 64 characters | `name` | W004 → promote to error for skills |
| Lowercase letters, numbers, hyphens only | `name` | W006 → promote to error |
| Non-empty | `description` | S002 |
| Maximum 1024 characters | `description` | W005 → promote to error |
| No XML tags (`<`, `>`) | `name`, `description` | S003 |
| No reserved words | `name` | S001 |

### Recommended Validations (Lattice implements as warnings)

| Requirement | Rationale |
|-------------|-----------|
| Body under 500 lines | Context window efficiency |
| Third person descriptions | Discovery consistency |
| Gerund naming (verb-ing) | Naming convention |
| Forward slashes in paths | Cross-platform compatibility |
| No time-sensitive content | Document longevity |

## Integration Strategy Recommendation

### Rationale for Native Implementation

Given Lattice's design principles, we recommend implementing lint rules natively
in Rust rather than depending on external linters:

**Advantages:**
1. **No runtime dependencies**: Aligns with "no daemon" philosophy
2. **Performance**: Direct parsing avoids subprocess overhead
3. **Unified error handling**: Consistent with Lattice's error format
4. **Selective rules**: Only implement rules relevant to Lattice documents
5. **Custom rules**: Easy to add Lattice-specific rules (ID format, links)

**Disadvantages:**
1. Initial implementation effort
2. Missing edge cases external linters have solved
3. Maintenance burden for rule updates

### Recommended Rule Set

Implement these rules natively, organized by priority:

**Phase 1: Core Rules (Required)**

Rules that directly support Lattice's AI-first document architecture:

| Lattice Code | Based On | Description |
|--------------|----------|-------------|
| W001 | Custom | Document exceeds 500 lines |
| W007 | MD003/heading-style | Inconsistent header styles |
| W008 | MD004/unordered-list-marker-style | Inconsistent list markers |
| W009 | MD034/no-literal-urls | Bare URLs |
| W011 | Custom | Backslash in paths |
| W012 | Custom | Time-sensitive content |

**Phase 2: Formatting Rules (Recommended)**

Rules for consistent document appearance:

| Rule | Based On | Description |
|------|----------|-------------|
| W013 | MD009 | Trailing whitespace |
| W014 | MD012/no-consecutive-blank-lines | Multiple blank lines |
| W015 | MD047/final-newline | Missing final newline |
| W016 | MD022 | Heading without blank lines |
| W017 | MD032 | List without blank lines |

**Phase 3: Structure Rules (Optional)**

Rules for document organization:

| Rule | Based On | Description |
|------|----------|-------------|
| W018 | MD001/heading-increment | Heading level skip |
| W019 | MD025/no-multiple-toplevel-headings | Multiple H1 headings |
| W020 | MD024/no-duplicate-headings | Duplicate heading text |
| W021 | MD036/no-emphasis-as-heading | Emphasis as heading |

### Optional External Linter Integration

For users wanting stricter validation, provide optional integration:

```bash
# Use external linter if available
lat check --external-linter=markdownlint

# Run both native and external
lat check --external-linter=remark
```

**Implementation:**
1. Detect if `markdownlint` or `remark` CLI is available
2. Run native checks first (always)
3. Run external linter if requested and available
4. Merge results, deduplicating overlapping rules
5. Report external linter issues with their original codes

### Configuration

Allow rule customization via `.lattice/config.toml`:

```toml
[lint]
# Enable/disable specific rules
disable = ["W013", "W014"]

# Adjust thresholds
max_document_lines = 750  # Override 500 default
max_line_length = 100     # For W020 if implemented

# External integration
external_linter = "markdownlint"  # or "remark" or "none"
external_config = ".markdownlint.json"  # Pass through to external
```

## Implementation Notes

### Parsing Strategy

Use Rust's `pulldown-cmark` for markdown parsing:
- Well-maintained, CommonMark compliant
- Provides event-based streaming parser
- Sufficient for all Phase 1-2 rules

For rules requiring AST manipulation (Phase 3), consider `markdown-rs` which
provides a full AST.

### Auto-Fix Implementation

The `lat fmt` command should auto-fix these rules:

| Rule | Fix Strategy |
|------|--------------|
| W007 | Convert setext to ATX headers |
| W008 | Normalize list markers to `-` |
| W013 | Strip trailing whitespace |
| W014 | Collapse multiple blank lines |
| W015 | Add final newline |
| W016/W017 | Insert blank lines around headings/lists |

Rules without deterministic fixes (W001, W009, W012) remain warnings only.

### Error Message Format

Follow markdownlint's clear message pattern:

```
Warning [W007]: path.md:15 mixes header styles (use ATX # headers)
  Found: setext underline header
  Expected: ATX header with # prefix
  Fix: Run `lat fmt` to convert automatically
```

## References

- [markdownlint Rules](https://github.com/DavidAnson/markdownlint/blob/main/doc/Rules.md)
- [remark-lint Rules](https://github.com/remarkjs/remark-lint)
- [Claude Skill Best Practices](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)
- [CommonMark Specification](https://spec.commonmark.org/)
