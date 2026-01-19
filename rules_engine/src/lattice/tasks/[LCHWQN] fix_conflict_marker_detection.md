---
lattice-id: LCHWQN
name: fix-conflict-marker-detection
description: Conflict marker detection flags decorative equals signs in code
task-type: bug
priority: 3
parent-id: LCEWQN
created-at: 2026-01-19T05:15:00.000000Z
updated-at: 2026-01-19T05:15:00.000000Z
---

# Conflict marker detection is too aggressive

## Problem

The conflict marker detection incorrectly flags lines containing decorative
equals signs as git conflict markers.

## Steps to Reproduce

1. Create a document containing code like:
   ```rust
   println!("==== decorative ====");
   ```
2. Run any `lat` command that processes the file
3. Warning appears: `Skipping file with conflict markers`

## Expected Behavior

Only actual git conflict markers should be detected. These have a specific
format:

- Seven or more `<` at start of line (less-than signs)
- Seven or more `=` at start of line (equals signs)
- Seven or more `>` at start of line (greater-than signs)

A line like `println!("==== decorative ====");` should not trigger this warning.

## Actual Behavior

The detection appears to match any line containing many consecutive equals
signs, even within strings or code.

## Suggested Fix

Update the conflict marker regex to require:

1. The marker characters at the start of the line (after optional whitespace)
2. Exactly 7 or more consecutive characters
3. Not inside a code block or string literal

Example regex: `^[<>=]{7,}`
