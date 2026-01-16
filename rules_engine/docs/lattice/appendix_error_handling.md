# Appendix: Error Handling

This appendix documents the error handling philosophy and implementation.
See [Lattice Design](lattice_design.md#error-handling) for an overview.

## Core Philosophy

Lattice distinguishes errors based on *ownership*: who is responsible?

**Expected Errors (User's Fault):** Problems caused by user input or external
systems the user manages. Produce helpful guidance. Exit codes 2-4.

**System Errors (Lattice's Fault):** Internal invariant violations and
"impossible" code paths. Trigger panics for immediate visibility. Exit code 1.

Key question: "Could a reasonable user have prevented this?" If yes, expected
error. If no, system error. When uncertain, default to panic—better to crash
loudly than silently corrupt data.

## Expected Errors

| Category | Examples | Exit Code |
|----------|----------|-----------|
| Validation | Invalid frontmatter, malformed ID, circular deps | 2 |
| User Input | Invalid arguments, unknown flags, bad path | 3 |
| Not Found | Unknown ID, missing file, no results | 4 |

Use `thiserror` crate with `LatticeError` enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum LatticeError {
    #[error("Document {id} not found")]
    DocumentNotFound { id: LatticeId },

    #[error("Invalid frontmatter in {path}: {reason}")]
    InvalidFrontmatter { path: PathBuf, reason: String },

    #[error("Reference to unknown ID {target} in {source} at line {line}")]
    BrokenReference { source: LatticeId, target: LatticeId, line: usize },
    // ...
}
```

### Error Message Requirements

Every error message must include:
1. **What happened:** Clear description
2. **Where:** File path, line number, document ID
3. **How to fix:** Actionable suggestion or command

### Structured Output

With `--json`, errors include machine-readable fields:

```json
{
  "error_code": "E002",
  "category": "validation",
  "message": "Reference to nonexistent ID",
  "affected_documents": ["LXXXXX"],
  "location": {"path": "auth/docs/login.md", "line": 42, "column": 15},
  "suggestion": "Create the target document or correct the ID",
  "fix_command": "lat create auth/ \"Target document description\""
}
```

**Required:** `error_code`, `category`, `message`

**Optional:** `affected_documents`, `location`, `context`, `suggestion`,
`fix_command`

## System Errors

| Category | Examples |
|----------|----------|
| Invariant Violation | Index has ID not in filesystem |
| Corruption | Malformed index, invalid internal state |
| Resource Exhaustion | Out of memory, disk full |
| Git Failures | Git operation fails when state should be valid |
| Impossible States | Match arm that should be unreachable |

Use `panic!` with descriptive messages. Use `human-panic` crate for
user-friendly crash output. The [Appendix: Chaos Monkey](appendix_chaos_monkey.md)
tests for panics and other system errors.

### When to Panic

**Do panic:**
- Index state doesn't match filesystem after Lattice modified it
- Internal data structures in impossible states
- Git operations fail when Lattice ensured valid preconditions

**Don't panic:**
- User provides invalid input (return LatticeError)
- External file missing or malformed (return LatticeError)
- Git repository in unusual state (handle gracefully or return error)

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | System error (panic) |
| 2 | Validation error |
| 3 | User input error |
| 4 | Not found |

## Logging

Operations log to `.lattice/logs.jsonl` (JSONL format):

```json
{"timestamp":"2026-01-15T10:30:00Z","level":"info","category":"command","message":"lat show","duration_us":5000,"details":{"id":"LXXXXX"}}
{"timestamp":"2026-01-15T10:30:01Z","level":"error","category":"observation","message":"Validation error E002","details":{"code":"E002","path":"docs/a.md"}}
```

Levels: `error`, `warn`, `info`, `debug` (--verbose), `trace` (dev only).

See [Appendix: Code Review](appendix_code_review.md) for implementation checklist.
