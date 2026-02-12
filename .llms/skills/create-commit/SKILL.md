---
description: Create a git commit with a detailed summary of changes. Use when work is complete and changes need to be committed.
user_invocable: true
---

# Creating Commits

Create a git commit summarizing completed work.

## Steps

1. Run `git status` to see all changed files.
2. Run `git diff --staged` and `git diff` to review the actual changes.
3. Stage all relevant files with `git add` (prefer naming specific files over `git add -A`).
4. Write a commit message that:
   - Uses a short imperative subject line (< 72 chars)
   - Describes *what* changed and *why* in the body
   - References specific files or components affected
5. Create the commit.
