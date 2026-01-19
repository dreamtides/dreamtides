---
lattice-id: LCQWQN
name: fix-fmt-corrupts-code-blocks
description: lat fmt corrupts content inside code blocks by treating it as markdown
task-type: bug
priority: 0
parent-id: LCEWQN
created-at: 2026-01-19T05:20:00.000000Z
updated-at: 2026-01-19T05:20:00.000000Z
---

# lat fmt corrupts content inside code blocks (CRITICAL)

## Problem

The `lat fmt` command corrupts document content by applying markdown formatting
rules to content inside code blocks. This is a data corruption bug.

## Steps to Reproduce

1. Create a document with a YAML code block containing frontmatter:

```
1. Example:
   (three backticks)yaml
   ---
   name: my-document
   description: A test document
   ---
   (three backticks)
```

2. Run `lat fmt`

3. The file is corrupted to something like:

```
## (three backticks)yaml

   name: my-document

## description: A test document

   (three backticks)
```

## Observed Corruptions

1. Code fence openers get `## ` prepended (backticks treated as heading)
2. Lines with `---` inside code blocks are removed (treated as frontmatter)
3. Lines with colons get `## ` prepended (treated as headings)
4. Random blank lines inserted throughout

## Impact

This is a **critical data corruption bug**. Running `lat fmt` can destroy
document content. Users may lose work if they don't have version control.

## Root Cause

The formatter is not properly tracking code fence state before applying
transformations. All content modifications should check if the current line
is inside a code fence and skip formatting if so.

## Suggested Fix

1. Before any line transformation, check if inside a code fence
2. Track code fence open/close state while iterating lines
3. Add test cases with code blocks containing YAML, markdown, and other
   syntaxes that could be mistaken for markdown

## Priority

P0 - This should be fixed immediately as it causes data loss.
