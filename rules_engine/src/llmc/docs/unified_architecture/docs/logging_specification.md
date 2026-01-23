---
lattice-id: LB7WQN
name: logging-specification
description: |-
  Log levels, structured log fields, and key operations to log for unified
  architecture.
parent-id: LRMWQN
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-21T22:38:24.875705Z
---

# Logging Specification

## Log Levels

| Level | Usage |
|-------|-------|
| ERROR | Operation failed, requires attention |
| WARN | Unexpected state, auto-recovered or degraded |
| INFO | Significant state changes, operation completion |
| DEBUG | Detailed operation flow, useful for debugging |
| TRACE | Very verbose, git command output |

## Structured Log Fields

All log entries should include:

```rust
tracing::info!(
    operation = "accept",
    worker = %worker_name,
    source_repo = %config.source_repo().display(),
    worktree = %worktree_path.display(),
    duration_ms = elapsed.as_millis(),
    result = "success",
    commit_sha = %new_sha,
    "Accepted worker changes"
);
```

## Key Operations to Log

| Operation | Level | Fields |
|-----------|-------|--------|
| Init | INFO | source_repo, metadata_dir |
| Add worker | INFO | worker, branch, worktree_path |
| Nuke worker | INFO | worker, had_uncommitted_changes |
| Start task | INFO | worker, prompt_length, self_review |
| Accept | INFO | worker, commit_sha, rebase_needed |
| Accept fail | ERROR | worker, reason, source_repo_state |
| Rebase | INFO | worker, base_ref, conflicts |
| Patrol run | DEBUG | workers_checked, transitions, errors |
| Self-heal | WARN | worker, issue, action_taken |
| Git operation | DEBUG | operation_type, repo, duration_ms, result |
| Git operation fail | ERROR | operation_type, repo, error, stderr |

## Transcript Archival

Claude Code transcripts are archived for deep-dive analysis of worker sessions:

**Storage Location**: `$LLMC_ROOT/logs/transcripts/<worker>/<timestamp>_<session_id>.jsonl`

**When Transcripts Are Archived**:
- On task completion (worker transitions to Idle or NeedsReview via Stop hook)
- On worker stall detection (session crash, timeout, or error)

**WorkerRecord Fields**:
- `transcript_session_id`: Claude session ID captured from SessionStart hook
- `transcript_path`: Path to Claude's transcript file for the current session

**Hook Data Flow**:
1. `SessionStart` hook includes `transcript_path` from Claude Code
2. Worker stores `transcript_session_id` and `transcript_path` when task begins
3. `Stop` hook may include updated `transcript_path` (if Claude ran /clear)
4. On completion, transcript is copied to archive location
5. Fields are cleared for next task

**Log Events**:
| Operation | Level | Fields |
|-----------|-------|--------|
| Archive transcript | INFO | worker, source, dest, bytes |
| Archive fail | ERROR | worker, source, error |
| No transcript path | DEBUG | worker |

## Log File Rotation

```rust
// In logging/config.rs
pub fn configure_logging() -> Result<()> {
    let log_dir = config::get_llmc_root().join("logs");

    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .max_log_files(7)  // Keep 1 week
        .filename_prefix("llmc")
        .filename_suffix("log")
        .build(&log_dir)?;

    // ... rest of setup
}
```
