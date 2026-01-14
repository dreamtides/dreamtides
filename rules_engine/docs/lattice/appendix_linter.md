# Appendix: Linter

This appendix documents the complete rule set for the `lat check` command.
See [Lattice Design](lattice_design.md#linter-and-formatter) for how the
linter fits into the overall system.

## Overview

The `lat check` command validates documents and repository state. It
distinguishes between errors (blocking issues) and warnings (advisory).

## Error-Level Rules

These prevent `lat` operations and must be fixed.

### E001: Duplicate Lattice ID

Two or more documents share the same Lattice ID.

**Detection:** Query documents table grouped by ID, flag groups > 1.

**Message:** `Error [E001]: Duplicate Lattice ID LXXXX found in: path1.md, path2.md`

**Fix:** Regenerate ID for one document using `lat track --force`.

### E002: Missing Reference Target

A link references an ID that doesn't exist.

**Detection:** Compare link targets against documents table.

**Message:** `Error [E002]: path.md:42 links to unknown ID LYYYY`

**Fix:** Create the target document or correct the ID.

### E003: Invalid Frontmatter Key

YAML frontmatter contains an unrecognized key.

**Detection:** Compare keys against allowed key list.

**Message:** `Error [E003]: path.md has invalid frontmatter key 'priorty' (did you mean 'priority'?)`

**Fix:** Correct the typo or remove the invalid key.

### E004: Missing Required Field

Issue document lacks required fields.

**Detection:** If `issue-type` present, require `status` and `priority`.

**Message:** `Error [E004]: path.md is an issue but missing 'status' field`

**Fix:** Add the required field.

### E005: Invalid Field Value

A field contains an invalid value.

**Detection:** Validate against allowed values for each field type.

**Message:** `Error [E005]: path.md has invalid status 'doing' (allowed: open, blocked, deferred, closed, tombstone, pinned)`

**Fix:** Use a valid value.

### E006: Circular Blocking

Blocking dependencies form a cycle.

**Detection:** DFS on blocking graph, detect back edges.

**Message:** `Error [E006]: Circular blocking dependency: LXXXX → LYYYY → LZZZZ → LXXXX`

**Fix:** Remove one blocking relationship.

### E007: Invalid ID Format

A Lattice ID doesn't match the expected format.

**Detection:** Regex validation on ID fields.

**Message:** `Error [E007]: path.md has malformed lattice-id 'L12'`

**Fix:** Use a properly formatted ID.

## Warning-Level Rules

These are advisory and don't block operations.

### W001: Document Too Large

Document exceeds the recommended 500 line limit.

**Detection:** Count lines in document body.

**Message:** `Warning [W001]: path.md has 750 lines (recommended max: 500)`

**Fix:** Split using `lat split` or manually divide content.

### W002: Missing Name

Knowledge base document lacks a `name` field.

**Detection:** If no `issue-type` and no `name`, warn.

**Message:** `Warning [W002]: path.md is missing 'name' field`

**Fix:** Add name to frontmatter or run `lat fmt`.

### W003: Missing Description

Knowledge base document lacks a `description` field.

**Detection:** If no `issue-type` and no `description`, warn.

**Message:** `Warning [W003]: path.md is missing 'description' field`

**Fix:** Add description explaining the document's purpose.

### W004: Name Too Long

Document name exceeds 64 characters.

**Detection:** Validate name field length.

**Message:** `Warning [W004]: path.md name is 78 characters (max: 64)`

**Fix:** Shorten the name.

### W005: Description Too Long

Description exceeds 1024 characters.

**Detection:** Validate description field length.

**Message:** `Warning [W005]: path.md description is 1200 characters (max: 1024)`

**Fix:** Shorten the description.

### W006: Invalid Name Characters

Name contains characters other than lowercase letters, numbers, hyphens.

**Detection:** Regex validation on name field.

**Message:** `Warning [W006]: path.md name 'My_Document' contains invalid characters (use lowercase-hyphen-format)`

**Fix:** Rename using only lowercase letters, numbers, and hyphens.

### W007: Inconsistent Header Style

Document mixes ATX (`#`) and setext (underline) headers.

**Detection:** Parse header styles, flag if both present.

**Message:** `Warning [W007]: path.md mixes header styles (use ATX # headers consistently)`

**Fix:** Convert all headers to ATX style.

### W008: Inconsistent List Markers

Document mixes list markers (`-`, `*`, `+`).

**Detection:** Parse list markers, flag if multiple types.

**Message:** `Warning [W008]: path.md mixes list markers (use - consistently)`

**Fix:** Convert all list markers to dashes.

### W009: Bare URL

URL appears without markdown link syntax.

**Detection:** Regex for URLs not inside `[]()` or `<>`.

**Message:** `Warning [W009]: path.md:15 has bare URL (use [text](url) format)`

**Fix:** Convert to markdown link or angle-bracket URL.

### W010: Self-Reference

Document contains a link to itself.

**Detection:** Check if any link target equals document's own ID.

**Message:** `Warning [W010]: path.md contains self-reference at line 23`

**Fix:** Remove the self-reference.

### W011: Backslash in Path

Path contains backslashes (Windows-style).

**Detection:** Check paths in links for backslash characters.

**Message:** `Warning [W011]: path.md:30 uses backslash in path (use forward slashes)`

**Fix:** Replace backslashes with forward slashes.

### W012: Time-Sensitive Content

Document contains date-specific language.

**Detection:** Regex for patterns like "after August 2025", "by Q4 2024".

**Message:** `Warning [W012]: path.md:45 contains time-sensitive content 'after August 2025'`

**Fix:** Reword to be time-independent or move to "old patterns" section.

## Skill-Specific Rules

When `skill: true` is present, additional rules apply:

### S001: Name Contains Reserved Word

Name contains "anthropic" or "claude".

**Detection:** Check name for reserved substrings.

**Message:** `Error [S001]: skill name cannot contain 'claude'`

### S002: Description Empty

Skill has empty or missing description.

**Detection:** Check description is non-empty string.

**Message:** `Error [S002]: skill must have non-empty description`

### S003: Name Contains XML

Name contains XML-like tags.

**Detection:** Regex for `<` or `>` in name.

**Message:** `Error [S003]: skill name cannot contain XML tags`

## Output Format

### Default Output

```
Checking 234 documents...

path/to/doc1.md:
  Error [E002]: Line 42 links to unknown ID LYYYY
  Warning [W001]: 750 lines (recommended max: 500)

path/to/doc2.md:
  Warning [W003]: Missing 'description' field

Found 1 error, 2 warnings in 2 documents.
```

### JSON Output

```json
{
  "documents_checked": 234,
  "errors": [
    {
      "code": "E002",
      "path": "path/to/doc1.md",
      "line": 42,
      "message": "links to unknown ID LYYYY"
    }
  ],
  "warnings": [
    {
      "code": "W001",
      "path": "path/to/doc1.md",
      "message": "750 lines (recommended max: 500)"
    },
    {
      "code": "W003",
      "path": "path/to/doc2.md",
      "message": "missing 'description' field"
    }
  ],
  "summary": {
    "error_count": 1,
    "warning_count": 2,
    "affected_documents": 2
  }
}
```

## Additional Formatting Rules

These rules enforce consistent markdown formatting.

### W013: Trailing Whitespace

Lines end with unnecessary whitespace.

**Detection:** Regex for spaces/tabs at line end.

**Message:** `Warning [W013]: path.md:20 has trailing whitespace`

**Fix:** Run `lat fmt` to strip automatically.

### W014: Multiple Blank Lines

More than one consecutive blank line in document.

**Detection:** Count consecutive newlines > 2.

**Message:** `Warning [W014]: path.md:45 has 3 consecutive blank lines (max: 1)`

**Fix:** Run `lat fmt` to collapse automatically.

### W015: Missing Final Newline

File does not end with a newline character.

**Detection:** Check last byte of file.

**Message:** `Warning [W015]: path.md does not end with newline`

**Fix:** Run `lat fmt` to add automatically.

### W016: Heading Without Blank Lines

Heading not surrounded by blank lines.

**Detection:** Parse heading and check adjacent lines.

**Message:** `Warning [W016]: path.md:30 heading should have blank line before/after`

**Fix:** Run `lat fmt` to insert blank lines.

### W017: List Without Blank Lines

List not surrounded by blank lines.

**Detection:** Parse list and check adjacent lines.

**Message:** `Warning [W017]: path.md:55 list should have blank line before/after`

**Fix:** Run `lat fmt` to insert blank lines.

## Command Options

- `--errors-only`: Suppress warnings
- `--path <prefix>`: Check only files under path
- `--fix`: Auto-fix where possible (W002, W006, W007, W008, W013-W017)
- `--staged-only`: Check only staged files
- `--rebuild-index`: Force full index rebuild before checking
