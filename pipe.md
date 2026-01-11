# Tmux pipe-pane Usage in llmc

## Summary

The llmc crate **does not use tmux `pipe-pane`** functionality.

## Analysis

A comprehensive search of the llmc crate revealed no uses of the tmux `pipe-pane` command. This was verified by:

1. Searching all Rust source files in `rules_engine/src/llmc/`
2. Searching git history for any past uses
3. Checking related tmux functionality

## Actual Tmux Pane Operations Used

Instead of `pipe-pane`, the llmc crate uses the following tmux operations to interact with panes:

### 1. `capture-pane` (via `session::capture_pane()`)
- **Location**: `rules_engine/src/llmc/src/tmux/session.rs:97-109`
- **Purpose**: Captures recent terminal output from a session's pane
- **Used by**: State detection, output monitoring, and command verification
- **Usage examples**:
  - Detecting Claude's state (ready, processing, error, etc.)
  - Checking for permission prompts
  - Verifying bypass warnings
  - Detecting partial message sends

### 2. `display-message #{pane_current_command}` (via `session::get_pane_command()`)
- **Location**: `rules_engine/src/llmc/src/tmux/session.rs:82-94`
- **Purpose**: Gets the command currently running in a pane
- **Used by**: Process health checks
- **Usage examples**:
  - Verifying Claude process is running vs shell
  - Detecting when processes have exited

### 3. `display-message #{pane_dead_status}` (via `get_pane_exit_code()`)
- **Location**: `rules_engine/src/llmc/src/tmux/monitor.rs:391-398`
- **Purpose**: Gets the exit code of a terminated process in a pane
- **Used by**: Exit type classification
- **Usage examples**:
  - Distinguishing between user-initiated exits and crashes
  - Determining if a process was killed by signal

## Why No pipe-pane?

The `pipe-pane` command is typically used for:
- Continuous logging of all pane output to a file
- Real-time streaming of terminal content

The llmc crate doesn't need this because it:
1. Uses on-demand `capture-pane` calls to read output when needed
2. Relies on structured logging via `tracing` for application logs
3. Only needs snapshots of terminal state, not continuous streaming

## Alternative: Structured Logging

Instead of piping terminal output to files, llmc uses structured logging:

- **Log file**: Size-rotating JSON log via `SizeRotatingWriter`
- **Location**: `rules_engine/src/llmc/src/logging/writer.rs`
- **Format**: Structured JSON logs with tracing
- **Benefits**: Searchable, parseable, and includes context beyond just terminal output
