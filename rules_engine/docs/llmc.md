# LLMC v2 Design Document

## Overview

LLMC v2 is a complete rewrite of the LLMC agent coordination system that manages multiple Claude Code CLI sessions running in parallel git worktrees. The system uses TMUX for persistent session management, enabling coordinated development work across multiple workers while maintaining a clean single-commit workflow on the master branch.

### Key Differences from V1

LLMC v2 introduces several architectural improvements over the original version. The system offers full interactive control through the `llmc attach` command, allowing direct access to worker sessions when needed. A persistent daemon started via `llmc up` runs continuously in the background, monitoring workers and orchestrating state transitions throughout their lifecycle. Additionally, a patrol system operates as a background process that maintains system health and facilitates automatic rebasing of worker branches.

## Repository Layout

```
~/llmc/                           # LLMC root directory
├── config.toml                   # Global configuration
├── state.json                    # Worker registry and state
├── logs/                         # Per-worker logs
│   └── <worker>.log
├── .worktrees/                   # Git worktrees (one per worker)
│   ├── adam/                     # Worker "adam"'s worktree
│   ├── baker/                    # Worker "baker"'s worktree
│   └── ...
└── .git/                         # Git repository (local clone of master repo)

~/Documents/GoogleDrive/dreamtides/   # Master repository (source of truth)
```

## Worker State Machine

Each worker progresses through a well-defined state machine:

```
                      ┌─────────────────────────────────────────┐
                      │                                         │
                      ▼                                         │
┌─────────┐      ┌──────────┐      ┌───────────────┐           │
│  IDLE   │─────▶│ WORKING  │─────▶│ NEEDS_REVIEW  │───────────┤
└─────────┘ start└──────────┘commit└───────────────┘   accept  │
     ▲               │                    │                     │
     │               │ no commit          │ reject              │
     │               ▼                    ▼                     │
     │        ┌─────────────┐      ┌──────────┐                │
     │        │ NEEDS_INPUT │      │ REJECTED │────────────────┘
     │        └─────────────┘      └──────────┘    completes
     │               │                    │
     │               │ message            │ completes
     │               ▼                    │
     └───────────────────────────────────-┘

Special States:
- ERROR: Worker encountered an unrecoverable error
- OFFLINE: TMUX session not running (needs `llmc up`)
```

### State Definitions

| State | Description |
|-------|-------------|
| `idle` | Worker has no active task, ready to receive work |
| `working` | Worker is actively implementing a task |
| `needs_input` | Worker stopped without committing, likely waiting for clarification |
| `needs_review` | Worker completed work and committed, awaiting human review |
| `rejected` | Work was rejected with feedback, worker is implementing changes |
| `rebasing` | Worker is resolving merge conflicts after a rebase |
| `error` | Worker is in an error state requiring manual intervention |
| `offline` | TMUX session is not running |

## Configuration

Configuration is stored in `~/llmc/config.toml`:

```toml
[defaults]
model = "opus"
skip_permissions = true
allowed_tools = ["Bash", "Edit", "Read", "Write", "Glob", "Grep"]
patrol_interval_secs = 60
sound_on_review = true

[repo]
source = "~/Documents/GoogleDrive/dreamtides"

[workers.adam]
model = "opus"
role_prompt = """
You are Adam, a senior engineer focused on backend systems.
You prefer simple, direct solutions over complex abstractions.
"""
excluded_from_pool = false

[workers.baker]
model = "sonnet"
role_prompt = """
You are Baker, focused on UI and user experience.
"""
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `model` | string | `"opus"` | Claude model to use |
| `skip_permissions` | bool | `true` | Use `--dangerously-skip-permissions` |
| `allowed_tools` | string[] | see above | Tools to allow via `--allowedTools` |
| `patrol_interval_secs` | u32 | `60` | Seconds between patrol runs |
| `sound_on_review` | bool | `true` | Play terminal bell when work needs review |
| `role_prompt` | string | `""` | Additional context for the worker |
| `excluded_from_pool` | bool | `false` | Exclude from automatic task assignment |

## Data Model

### State File (`~/llmc/state.json`)

The state file tracks all workers and their current status. Each worker record
contains: name, worktree path, branch name, current status, active prompt text,
timestamps for creation and last activity, commit SHA if awaiting review, and
TMUX session identifier.

### Worker Record Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Unique worker identifier |
| `worktree_path` | string | Absolute path to git worktree |
| `branch` | string | Git branch name (`llmc/<name>`) |
| `status` | Status | Current worker state |
| `current_prompt` | string | Full prompt text for current task |
| `created_at_unix` | u64 | Unix timestamp of worker creation |
| `last_activity_unix` | u64 | Unix timestamp of last state change |
| `commit_sha` | Option<string> | SHA of commit awaiting review |
| `session_id` | string | TMUX session identifier |

## TMUX Integration

LLMC uses TMUX for persistent Claude Code sessions with session names following
the pattern `llmc-<worker>` (e.g., `llmc-adam`). Sessions are created detached
with the worker's worktree as the working directory.

### Session Startup

When starting a worker session, the system creates a detached TMUX session,
sets environment variables for worker identification, launches Claude with
configured flags, waits for Claude to initialize (polling for the ">" prompt),
accepts the bypass permissions warning if shown, sends `/clear`, and marks the
worker as idle.

### Reliable Communication

The most critical component is reliable message delivery. TMUX `send-keys` can
have race conditions where Enter arrives before text is fully pasted. The system
uses a multi-phase approach: send text in literal mode, calculate a debounce
delay based on message size (500ms base + 100ms per KB, capped at 2000ms), then
send Enter with retry logic.

For large prompts (>1KB), use TMUX's `load-buffer` from a temporary file,
then `paste-buffer`, to avoid shell command-line limits.

**Important**: Create sessions with wide terminals (`-x 500`) to prevent message
truncation.

See `llmc2-appendix-tmux.md` for detailed implementation notes on debounce
timing, failure recovery, and research results.

### State Detection

Detecting Claude's state requires parsing terminal output. The system checks
process health (is Claude running?), looks for the ">" prompt indicating
readiness, detects permission prompts and questions, and identifies error
states. The detection hierarchy is: crash > confirmation > question > ready >
processing.

See `llmc2-appendix-claude-state.md` for detailed heuristics and pattern
matching code.

## The Patrol System

The Patrol is a background process that runs periodically during `llmc up` to
maintain system health. It performs four main operations:

1. **Check session health**: Verify TMUX sessions exist and match state file
2. **Detect state transitions**: Find workers that have finished but haven't
   been processed yet
3. **Rebase pending reviews**: Keep `needs_review` workers rebased on master
4. **Detect stuck workers**: Find workers that appear stuck

The patrol runs on a configurable interval (default: 60 seconds) unless another
patrol is already running or the system is shutting down.

## Command Reference

### `llmc init`

Initializes a new LLMC project directory.

```bash
llmc init [--source <path>] [--target <path>]
```

Clones source repository with `--local`, configures git rerere, installs LFS,
creates directory structure, initializes state files, and copies `Tabula.xlsm`
if present.

### `llmc up`

Starts the LLMC daemon, bringing up all worker sessions.

```bash
llmc up [--no-patrol]
```

Loads configuration, starts TMUX server, creates worktrees and sessions for
each worker, enters main loop monitoring for state changes and running patrol.
Handles graceful shutdown on Ctrl-C by sending Ctrl-C to each session and
saving final state.

### `llmc down`

Stops all worker sessions.

```bash
llmc down [--force]
```

### `llmc add <name>`

Adds a new worker to the system.

```bash
llmc add <name> [--model <model>] [--role-prompt <prompt>]
```

Creates git worktree on branch `llmc/<name>`, copies `Tabula.xlsm`, adds worker
to state, and signals `llmc up` to create the session.

### `llmc nuke <name>`

Permanently removes a worker.

```bash
llmc nuke <name>
llmc nuke --all
```

Kills TMUX session, removes worktree, deletes branch, removes from state.

### `llmc status`

Displays status of all workers.

```bash
llmc status [--json]
```

### `llmc start`

Assigns a task to an idle worker.

```bash
llmc start --prompt "Implement feature X"
llmc start --prompt-file task.md
llmc start --worker adam --prompt "..."
```

Selects worker (specified or first idle from pool), verifies idle state, pulls
latest master into worktree, copies `Tabula.xlsm`, builds full prompt with
preamble, sends `/clear` and prompt, updates state to `working`.

The prompt preamble includes worktree location, repository root, instructions
to follow AGENTS.md conventions, run validation commands, create a single
commit, and not push to remote.

### `llmc message <worker> <message>`

Sends a message to a worker.

```bash
llmc message adam "Use the existing auth helper instead"
```

### `llmc attach <worker>`

Attaches to a worker's TMUX session for interactive use.

```bash
llmc attach adam
```

### `llmc review [worker]`

Shows diff for a worker awaiting review. Triggers rebase via patrol if master
has advanced.

```bash
llmc review          # Reviews oldest pending worker
llmc review baker    # Reviews specific worker
llmc review --interface difftastic  # Default
llmc review --interface vscode      # Open in VS Code
```

### `llmc reject [message]`

Sends feedback to the most recently reviewed worker.

```bash
llmc reject "Please add error handling for the API call"
```

Does NOT send `/clear` (preserves context), sends reject message with original
diff context, updates state to `rejected`.

### `llmc accept [worker]`

Accepts a worker's changes and merges to master.

```bash
llmc accept        # Accepts most recently reviewed worker
llmc accept baker
```

Verifies `needs_review` state, ensures clean worktree, rebases onto master,
squashes to single commit, strips agent attribution ("Generated with",
"Co-Authored-By"), fast-forward merges to master, removes worktree and branch,
resets worker to `idle` with new worktree, triggers background rebase for other
`needs_review` workers.

### `llmc rebase <worker>`

Manually triggers a rebase for a worker.

```bash
llmc rebase adam
```

Fetches latest master, attempts rebase. If conflicts occur, marks worker as
`rebasing` and sends conflict resolution prompt to worker.

### `llmc doctor`

Runs health checks on the system.

```bash
llmc doctor
llmc doctor --repair
llmc doctor --repair --yes
llmc doctor --rebuild
```

Checks: required binaries present, state file valid, TMUX sessions match
workers, worktrees exist and are clean, git configuration correct, no orphaned
branches.

#### Flags

- `--repair`: Automatically fix detected issues including:
  - Kill orphaned TMUX sessions (sessions that exist but have no configured worker)
  - Mark workers as offline when TMUX sessions are missing
  - Reset workers in error state to clean idle state
  - Fix state inconsistencies (missing commit_sha, empty prompts, future timestamps)
  - Reset daemon crash flag
  - Clean up orphaned git rebase state

- `--yes`: Skip confirmation prompts when used with `--repair`. Useful for automated workflows.

- `--rebuild`: Rebuild state.json from filesystem by scanning worktrees and TMUX sessions. Use when state file is corrupted or lost.

## Merge Conflict Resolution

Merge conflicts can occur during patrol rebases, manual rebases, or accept
operations. The system detects conflicts by checking for `.git/rebase-merge` or
`.git/rebase-apply` directories and running `git diff --name-only --diff-filter=U`.

### Conflict Types

- **Content**: Both sides modified the same lines
- **ModifyDelete**: File modified on one side, deleted on other
- **AddAdd**: Both sides added a file with same name
- **RenameRename**: File renamed differently on each side

### Conflict Presentation

When presenting conflicts to a worker, provide a high-level summary (why
rebasing, how many files), a file list with conflict types and marker counts,
conflict details showing actual conflicted regions with context, and clear
step-by-step resolution instructions.

The conflict resolution prompt template explains the situation, lists conflicts,
provides resolution steps (examine markers, decide resolution, remove markers,
stage files, continue rebase, run validation), and includes notes about viewing
original versions via `git show :2:<file>` and `:3:<file>`.

### Detecting Resolution Completion

Resolution is complete when: no conflict markers remain, all conflicted files
are staged, `git rebase --continue` succeeds, and validation passes. The system
monitors for progress patterns ("Resolved conflicts", "git add", "Staged"),
completion patterns ("Successfully rebased"), and failure patterns ("rebase
--abort", "Cannot continue").

### Failure Handling

| Failure Mode | Detection | Recovery |
|--------------|-----------|----------|
| Worker aborts | `git rebase --abort` in output | Reset to `needs_review`, notify user |
| Worker stuck | Stuck indicators + timeout | Send help prompt, escalate after 5 min |
| Persistent conflicts | Multiple continue failures | Send detailed help, escalate after 3 attempts |
| Validation failure | `just check` fails post-rebase | Send error output, worker fixes |

During rebasing, the worker cannot receive new tasks, cannot be reviewed, is
monitored by patrol for resolution, and timeouts trigger escalation.

## Failure Recovery

The system must handle several failure modes gracefully. This section summarizes
the recovery strategies; see `llmc2-appendix-error-recovery.md` for detailed
decision trees.

### Summary Table

| Failure Mode | Detection | Retries | Escalation |
|--------------|-----------|---------|------------|
| Lost input | Ready state persists after send | 3 attempts | Error state, notify user |
| Session crash | Pane command is shell | Auto-restart | Manual after 3rd crash |
| Stuck processing | `working` state >30 min | Nudge, then alert | Error after 2nd nudge |
| Partial send | Incomplete text in pane | Clear and resend | Error after 3 failures |
| State corruption | JSON parse error | Backup restore | Manual intervention |

### Key Principles

1. **Auto-recover when possible**: Most transient failures (lost input, partial
   sends) should be automatically retried before involving the user.

2. **Preserve work in progress**: When a crash occurs during `working` state,
   the system attempts to restore context and resume rather than losing progress.

3. **Escalate with context**: When manual intervention is required, provide
   diagnostic information (logs, pane output, state history) to help the user
   understand what went wrong.

4. **Fail fast on corruption**: State file corruption requires immediate user
   attention since recovery involves potentially destructive operations.

### State File Integrity

The `state.json` file is protected by:

- **Atomic writes**: All updates use temp file + rename pattern
- **Pre-write validation**: Schema validation before any write
- **Automatic backups**: Previous state preserved as `state.json.bak`
- **Recovery command**: `llmc doctor --repair` attempts automatic repair

## Architecture

### Module Layout

```
rules_engine/src/llmc/
├── Cargo.toml
└── src/
    ├── main.rs           # CLI entrypoint
    ├── cli.rs            # clap command definitions
    ├── config.rs         # Configuration loading
    ├── state.rs          # State file operations
    ├── tmux/
    │   ├── mod.rs
    │   ├── session.rs    # Session management
    │   ├── sender.rs     # Reliable input sending
    │   └── monitor.rs    # Output monitoring
    ├── patrol.rs         # Patrol system
    ├── git.rs            # Git operations
    ├── worker.rs         # Worker lifecycle
    ├── commands/
    │   ├── mod.rs
    │   ├── init.rs
    │   ├── up.rs
    │   ├── down.rs
    │   ├── add.rs
    │   ├── nuke.rs
    │   ├── status.rs
    │   ├── start.rs
    │   ├── message.rs
    │   ├── attach.rs
    │   ├── review.rs
    │   ├── reject.rs
    │   ├── accept.rs
    │   ├── rebase.rs
    │   └── doctor.rs
    └── sound.rs          # Terminal bell/sounds
```

### Dependencies

Core dependencies: clap for CLI, tokio for async, serde/serde_json/toml for
serialization, anyhow/thiserror for error handling, tmux_interface for TMUX,
regex for output parsing, ctrlc for signal handling, tracing for logging.

### Error Handling

All fallible operations use `anyhow::Result` with context. User-facing errors
include remediation hints (e.g., "Run 'llmc up' to start workers").

## Sound Notifications

When a worker enters `needs_review` state, LLMC plays a terminal bell (`\x07`)
if `sound_on_review` is enabled.

## Learnings from V1 and Gastown

### From LLMC V1

- Atomic state updates via temp file + rename
- Strip "Generated with" and "Co-Authored-By" from commits
- Copy gitignored `Tabula.xlsm` to worktrees
- Squash to single commit before merge

### From Gastown

- Debounce: 500ms base + 100ms/KB, capped at 2000ms
- Enter key: 3 retry attempts with 200ms delays
- Claude process detection: check for "node", "claude", or semver
- Prompt readiness: poll for ">" at line start
- Bypass dialog: Down + Enter sequence
- Session identity: environment variables
- Crash detection: tmux pane-died hook
- Many configuration steps are best-effort (non-fatal)

Key reference files in Gastown (`~/gastown`):
- `internal/tmux/tmux.go`: SendKeysDebounced, NudgeSession, WaitForClaudeReady
- `internal/polecat/session_manager.go`: Session lifecycle
- `internal/agent/state.go`: State management patterns
- `internal/witness/protocol.go`: Protocol-based message classification

## Security Considerations

1. **Skip permissions mode**: Workers use `--dangerously-skip-permissions`.
   This is acceptable because workers operate in isolated worktrees, have no
   network access to remotes, and require human review before merge.

2. **Prompt injection**: Prompts sent via tmux literal mode (`-l` flag) to
   prevent special character interpretation.

3. **State file**: Contains prompts and paths but should not contain secrets.

## Appendices

- `llmc2-appendix-tmux.md`: TMUX integration details, debounce timing research
- `llmc2-appendix-claude-state.md`: Claude state detection heuristics
- `llmc2-appendix-error-recovery.md`: Detailed error recovery decision trees
