# Appendix: Linter

This appendix documents the complete rule set for the `lat check` command.
See [Lattice Design](lattice_design.md#linter-and-formatter) for how the
linter fits into the overall system.

## Overview

The `lat check` command validates documents and repository state. It
distinguishes between errors (blocking problems) and warnings (advisory).

## Error-Level Rules

These prevent `lat` operations and must be fixed.

### E001: Duplicate Lattice ID

Two or more documents share the same Lattice ID.

**Detection:** Query documents table grouped by ID, flag groups > 1.

**Message:** `Error [E001]: Duplicate Lattice ID LXXXXX found in: path1.md, path2.md`

**Fix:** Regenerate ID for one document using `lat track --force`.

### E002: Missing Reference Target

A link references an ID that doesn't exist.

**Detection:** Compare link targets against documents table.

**Message:** `Error [E002]: path.md:42 links to unknown ID LYYYYY`

**Fix:** Create the target document or correct the ID.

### E003: Invalid Frontmatter Key

YAML frontmatter contains an unrecognized key.

**Detection:** Compare keys against allowed key list.

**Message:** `Error [E003]: path.md has invalid frontmatter key 'priorty' (did you mean 'priority'?)`

**Fix:** Correct the typo or remove the invalid key.

### E004: Missing Required Field

Task document lacks required fields.

**Detection:** If `task-type` present, require `priority`.

**Message:** `Error [E004]: path.md is a task but missing 'priority' field`

**Fix:** Add the required field.

### E005: Invalid Field Value

A field contains an invalid value.

**Detection:** Validate against allowed values for each field type.

**Message:** `Error [E005]: path.md has invalid priority '5' (allowed: 0-4)`

**Fix:** Use a valid value.

### E006: Circular Blocking

Blocking dependencies form a cycle.

**Detection:** DFS on blocking graph, detect back edges.

**Message:** `Error [E006]: Circular blocking dependency: LXXXXX → LYYYYY → LZZZZZ → LXXXXX`

**Fix:** Remove one blocking relationship.

### E007: Invalid ID Format

A Lattice ID doesn't match the expected format.

**Detection:** Regex validation on ID fields.

**Message:** `Error [E007]: path.md has malformed lattice-id 'L12'`

**Fix:** Use a properly formatted ID.

### E008: Name-Filename Mismatch

The `name` field doesn't match the filename.

**Detection:** Convert filename to expected name (strip `.md`, convert underscores
to hyphens, lowercase) and compare with `name` field.

**Message:** `Error [E008]: path/fix_login.md has name 'login-fix' but should be 'fix-login' (derived from filename)`

**Fix:** Correct the `name` field to match the filename, or rename the file.
Run `lat fmt` to auto-correct.

This is a core Lattice invariant: the `name` field must always match the
filename with underscores converted to hyphens.

### E009: Missing Required Field (Name)

Document lacks required `name` field.

**Detection:** Check `name` field is present.

**Message:** `Error [E009]: path.md is missing required 'name' field`

**Fix:** Add name to frontmatter or run `lat fmt` to auto-populate from filename.

### E010: Missing Required Field (Description)

Document lacks required `description` field.

**Detection:** Check `description` field is present.

**Message:** `Error [E010]: path.md is missing required 'description' field`

**Fix:** Add description explaining the document's purpose (for KB) or task title (for tasks).

### E011: Invalid Closed Directory Structure

A `.closed/` directory contains invalid structure.

**Detection:** Check that `.closed/` directories only contain task documents and
are not nested (no `.closed/.closed/`).

**Message:** `Error [E011]: path/.closed/.closed/ is a nested closed directory`

**Fix:** Move documents to parent `.closed/` or restructure.

### E012: Non-Task in Closed Directory

A knowledge base document (no `task-type`) is in a `.closed/` directory.

**Detection:** Check documents in `.closed/` directories have `task-type` field.

**Message:** `Error [E012]: path/.closed/doc.md is a knowledge base document in closed directory`

**Fix:** Move to parent directory or convert to task.

## Warning-Level Rules

These are advisory and don't block operations.

### W001: Document Too Large

Document exceeds the recommended 500 line limit.

**Detection:** Count lines in document body.

**Message:** `Warning [W001]: path.md has 750 lines (recommended max: 500)`

**Fix:** Split using `lat split` or manually divide content.

### W002: Name Too Long

Document name exceeds 64 characters.

**Detection:** Validate name field length.

**Message:** `Warning [W002]: path.md name is 78 characters (max: 64)`

**Fix:** Shorten the name.

### W003: Description Too Long

Description exceeds 1024 characters.

**Detection:** Validate description field length.

**Message:** `Warning [W003]: path.md description is 1200 characters (max: 1024)`

**Fix:** Shorten the description.

### W004: Invalid Name Characters

Name contains characters other than lowercase letters, numbers, hyphens.

**Detection:** Regex validation on name field.

**Message:** `Warning [W004]: path.md name 'My_Document' contains invalid characters (use lowercase-hyphen-format)`

**Fix:** Rename using only lowercase letters, numbers, and hyphens.

### W005: Inconsistent Header Style

Document mixes ATX (`#`) and setext (underline) headers.

**Detection:** Parse header styles, flag if both present.

**Message:** `Warning [W005]: path.md mixes header styles (use ATX # headers consistently)`

**Fix:** Convert all headers to ATX style.

### W006: Inconsistent List Markers

Document mixes list markers (`-`, `*`, `+`).

**Detection:** Parse list markers, flag if multiple types.

**Message:** `Warning [W006]: path.md mixes list markers (use - consistently)`

**Fix:** Convert all list markers to dashes.

### W007: Bare URL

URL appears without markdown link syntax.

**Detection:** Regex for URLs not inside `[]()` or `<>`.

**Message:** `Warning [W007]: path.md:15 has bare URL (use [text](url) format)`

**Fix:** Convert to markdown link or angle-bracket URL.

### W008: Self-Reference

Document contains a link to itself.

**Detection:** Check if any link target equals document's own ID.

**Message:** `Warning [W008]: path.md contains self-reference at line 23`

**Fix:** Remove the self-reference.

### W009: Backslash in Path

Path contains backslashes (Windows-style).

**Detection:** Check paths in links for backslash characters.

**Message:** `Warning [W009]: path.md:30 uses backslash in path (use forward slashes)`

**Fix:** Replace backslashes with forward slashes.

### W010: Link Path Mismatch

Link file path doesn't match the target document's actual location.

**Detection:** Compare link path against document's current path in index.

**Message:** `Warning [W010]: path.md:30 has stale link path (expected ../new/location.md#LYYYYY)`

**Fix:** Run `lat fmt` to update link paths automatically.

See [Appendix: Linking System](appendix_linking_system.md#path-mismatches) for
details on path validation.

### W010b: Missing Link Fragment

Link has file path but no Lattice ID fragment.

**Detection:** Parse links in document body, check for missing `#LXXXXX` fragment.

**Message:** `Warning [W010b]: path.md:42 link missing Lattice ID fragment: [text](../doc.md)`

**Fix:** Run `lat fmt` to add fragment automatically.

See [Appendix: Linking System](appendix_linking_system.md#missing-fragments) for
details on link normalization.

### W011: Trailing Whitespace

Lines end with unnecessary whitespace.

**Detection:** Regex for spaces/tabs at line end.

**Message:** `Warning [W011]: path.md:20 has trailing whitespace`

**Fix:** Run `lat fmt` to strip automatically.

### W012: Multiple Blank Lines

More than one consecutive blank line in document.

**Detection:** Count consecutive newlines > 2.

**Message:** `Warning [W012]: path.md:45 has 3 consecutive blank lines (max: 1)`

**Fix:** Run `lat fmt` to collapse automatically.

### W013: Missing Final Newline

File does not end with a newline character.

**Detection:** Check last byte of file.

**Message:** `Warning [W013]: path.md does not end with newline`

**Fix:** Run `lat fmt` to add automatically.

### W014: Heading Without Blank Lines

Heading not surrounded by blank lines.

**Detection:** Parse heading and check adjacent lines.

**Message:** `Warning [W014]: path.md:30 heading should have blank line before/after`

**Fix:** Run `lat fmt` to insert blank lines.

### W015: List Without Blank Lines

List not surrounded by blank lines.

**Detection:** Parse list and check adjacent lines.

**Message:** `Warning [W015]: path.md:55 list should have blank line before/after`

**Fix:** Run `lat fmt` to insert blank lines.

### W016: Template Section in Non-Root

Non-root document contains `[Lattice]` template sections.

**Detection:** Check for `[Lattice]` headings in files that are not root
documents (filename does not match containing directory name).

**Message:** `Warning [W016]: path/task.md has [Lattice] sections but is not a root`

**Fix:** Move content to the directory's root document or remove sections.

### W017: Document Not in Standard Location

Document is not a root document and is not inside a `tasks/` or `docs/`
directory.

**Detection:** Check if document is either: (a) a root document (filename
matches directory name), or (b) its path contains `/tasks/` or `/docs/`.

**Message:** `Warning [W017]: path/loose_doc.md is not in tasks/ or docs/ directory`

**Fix:** Move to appropriate subdirectory: `tasks/` for task documents,
`docs/` for knowledge base documents.

### W018: Task in docs/ Directory

A task document (has `task-type` field) is located in a `docs/` directory.

**Detection:** Check if path contains `/docs/` and document has `task-type`
frontmatter field.

**Message:** `Warning [W018]: docs/some_task.md is a task but located in docs/`

**Fix:** Move to the `tasks/` directory instead.

### W019: Knowledge Base Document in tasks/ Directory

A knowledge base document (no `task-type` field) is located in a `tasks/`
directory.

**Detection:** Check if path contains `/tasks/` and document lacks `task-type`
frontmatter field. Documents in `.closed/` subdirectories are exempt from this
warning (they may have been tasks that were closed).

**Message:** `Warning [W019]: tasks/design_doc.md is a knowledge base document but located in tasks/`

**Fix:** Move to the `docs/` directory instead.

### W020: Invalid Document Name Format

Non-root document name does not follow the required naming convention: at least
two words, underscore separated, all lowercase.

**Detection:** For non-root documents, check if filename (without `.md`)
contains at least one underscore and consists only of lowercase letters,
numbers, and underscores.

**Message:** `Warning [W020]: tasks/fixbug.md should be at least two words (e.g., fix_bug.md)`

**Fix:** Rename file to use underscore-separated lowercase words (minimum 2 words).

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
  Error [E002]: Line 42 links to unknown ID LYYYYY
  Warning [W001]: 750 lines (recommended max: 500)

path/to/doc2.md:
  Warning [W002]: name is 78 characters (max: 64)

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
      "message": "links to unknown ID LYYYYY"
    }
  ],
  "warnings": [
    {
      "code": "W001",
      "path": "path/to/doc1.md",
      "message": "750 lines (recommended max: 500)"
    },
    {
      "code": "W002",
      "path": "path/to/doc2.md",
      "message": "name is 78 characters (max: 64)"
    }
  ],
  "summary": {
    "error_count": 1,
    "warning_count": 2,
    "affected_documents": 2
  }
}
```

## Command Options

- `--errors-only`: Suppress warnings
- `--path <prefix>`: Check only files under path
- `--fix`: Auto-fix where possible (E008, W004-W006, W011-W015)
- `--staged-only`: Check only staged files
- `--rebuild-index`: Force full index rebuild before checking. Useful when the
  index may be out of sync with the filesystem.
