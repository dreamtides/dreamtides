# LLMC v2 Design Document

## Overview

- Manages multiple Claude Code CLI sessions running in parallel git worktrees
- Uses TMUX for persistent session management
- Enables coordinated development work across multiple workers
- Maintains a clean single-commit workflow on the master branch

### Design Philosophy

**Deterministic signals only**: LLMC relies exclusively on observable, unambiguous events:
- Git commits (not "it looks done")
- Process exit codes (not "it seems stuck")
- Explicit user commands (not inferred intent)

**No heuristic inference**: Previous systems attempted to "understand" agent output through regex patterns to detect questions, completion, errors, or stuck states. This proved unreliable and complex. LLMC intentionally avoids this.

**Manual intervention over automation**: When issues arise, LLMC surfaces them to the user rather than attempting automatic recovery. Users can always attach directly to workers via `llmc attach`.

**Future evolution**: In a later phase, a dedicated coordinator agent may analyze output and provide intelligent categorization, but the current system deliberately keeps this simple.

### Key Differences from V1

- **TMUX-based sessions**: Workers are persistent Claude Code sessions in TMUX,
  not transient subprocesses
- **Interactive control**: Full interactive access to worker sessions via
  `llmc attach`
- **Persistent daemon**: `llmc up` runs continuously, monitoring workers and
  orchestrating state transitions
- **Patrol system**: Background process that detects commits and maintains rebase health
- **Simplified state detection**: No regex-based output analysis or heuristics

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

Each worker progresses through a well-defined state machine driven by **deterministic signals only** (git commits, process state, explicit user commands). LLMC does not attempt to infer worker intent from output analysis.

```
                      ┌─────────────────────────────────────────┐
                      │                                         │
                      ▼                                         │
┌─────────┐      ┌──────────┐      ┌───────────────┐           │
│  IDLE   │─────▶│ WORKING  │─────▶│ NEEDS_REVIEW  │───────────┤
└─────────┘ start└──────────┘commit└───────────────┘   accept  │
     ▲                                    │                     │
     │                                    │ reject              │
     │                                    ▼                     │
     │                              ┌──────────┐                │
     │                              │ REJECTED │────────────────┘
     │                              └──────────┘    completes
     │                                    │
     │                                    │ completes
     │                                    │
     └────────────────────────────────────┘

Special States:
- REBASING: Transitional state during rebase operations
- ERROR: Worker encountered an unrecoverable error
- OFFLINE: TMUX session not running (needs `llmc up`)
```

### State Definitions

| State | Description |
|-------|-------------|
| `idle` | Worker has no active task, ready to receive work |
| `working` | Worker is actively implementing a task |
| `needs_review` | Worker completed work and committed, awaiting human review |
| `rejected` | Work was rejected with feedback, worker is implementing changes |
| `rebasing` | Worker is resolving merge conflicts after a rebase |
| `error` | Worker is in an error state requiring manual intervention |
| `offline` | TMUX session is not running |

### State Transition Philosophy

LLMC uses **deterministic signals only** for state transitions:
- **Git commits**: Worker transitions from `working` → `needs_review` when patrol detects a new commit
- **Exit codes**: Process crashes detected via TMUX pane exit codes
- **User commands**: Explicit transitions via `llmc start`, `llmc accept`, `llmc reject`, etc.

**No heuristic analysis**: LLMC does not attempt to detect questions, interpret output for completion indicators, or guess if a worker is "stuck". The user can always attach to a worker via `llmc attach` to interact directly.

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
configured flags, waits for Claude to initialize (one-time check for the ">" prompt),
accepts the bypass permissions warning if shown, sends `/clear`, and marks the
worker as idle.

**Note**: The ">" prompt check is used only during initial startup to verify Claude
is ready. LLMC does not use prompt detection for ongoing state management.

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

**Philosophy**: LLMC relies exclusively on deterministic, observable signals. No regex-based output analysis, no heuristics for "inferring" agent state.

**What LLMC monitors:**
1. **Process health**: Is the Claude process running or has it exited? (via TMUX pane process inspection)
2. **Exit codes**: Non-zero exit codes (except 130 for SIGINT) indicate crashes
3. **Git commits**: New commits detected via `git log` trigger state transitions
4. **Rebase state**: Presence of `.git/rebase-merge` or conflict markers

**What LLMC does NOT do:**
- ❌ Detect if Claude is "asking a question"
- ❌ Infer completion from output patterns like "done", "finished", etc.
- ❌ Detect permission prompts or tool confirmations
- ❌ Parse output for error messages or "stuck" indicators
- ❌ Send automatic "nudges" to idle workers

**State detection hierarchy**: `Exited` > `Processing` > `Unknown`

**Rationale**: Complex regex-based state inference proved unreliable and added unnecessary complexity. In a future iteration, a dedicated coordinator agent may be introduced to analyze recent output and categorize it intelligently, but the current system intentionally avoids this.

## The Patrol System

The Patrol is a background process that runs periodically during `llmc up` to
maintain system health. It performs three main operations:

1. **Check session health**: Verify TMUX sessions exist and match state file
2. **Detect state transitions**: Find workers that have committed and transition them to `needs_review`
3. **Rebase pending reviews**: Keep `needs_review` workers rebased on master

The patrol runs on a configurable interval (default: 60 seconds) unless another
patrol is already running or the system is shutting down.

**Detection approach**: State transitions are detected via deterministic signals:
- Git commit detection: `git log -1 --format=%H` to check for new commits
- Process health: TMUX pane process inspection
- Rebase status: Checking for `.git/rebase-merge` or conflict markers

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
| Persistent conflicts | Rebase state persists across patrol runs | User attaches via `llmc attach` to assist |
| Validation failure | `just check` fails post-rebase | Worker output shows errors, worker fixes or user attaches |

During rebasing, the worker cannot receive new tasks and cannot be reviewed. The user can attach at any time via `llmc attach` to provide direct assistance.

## Failure Handling

LLMC's approach to failure handling is minimal and deterministic.

### Crash Detection

Worker crashes are detected via TMUX pane exit codes:
- Exit code 0 or 130 (SIGINT): Normal termination
- Any other exit code: Crash

When a crash is detected, patrol marks the worker as having crashed and increments the crash counter. After 24 hours without a crash, the counter resets.

### State File Integrity

The `state.json` file is protected by:

- **Atomic writes**: All updates use temp file + rename pattern
- **Pre-write validation**: Schema validation before any write
- **Automatic backups**: Previous state preserved as `state.json.bak`
- **Recovery command**: `llmc doctor --repair` attempts automatic repair
- **Rebuild command**: `llmc doctor --rebuild` reconstructs state from filesystem

### Manual Intervention

LLMC intentionally avoids automatic recovery attempts. When issues occur:
1. **Session crashes**: User inspects logs and restarts manually
2. **Workers appear inactive**: User attaches via `llmc attach` to interact directly
3. **State corruption**: User runs `llmc doctor --rebuild`

**Rationale**: Automatic recovery and "smart" inference of worker state led to unpredictable behavior. The current approach prefers transparency and manual control.

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
ctrlc for signal handling, tracing for logging.

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
- Bypass dialog: Down + Enter sequence
- Session identity: environment variables
- Crash detection: tmux pane-died hook
- Many configuration steps are best-effort (non-fatal)

### Evolution from Gastown

LLMC v2 **simplifies** Gastown's approach:
- **Removed**: Complex output analysis, nudge system, heuristic "stuck" detection
- **Removed**: Protocol-based message classification (Witness system)
- **Kept**: TMUX session management, debounce logic, basic process health checks
- **Philosophy shift**: From "smart inference" to "deterministic signals only"

Gastown attempted to infer agent state through regex patterns and output analysis. This proved brittle and unpredictable. LLMC v2 instead relies on git commits, exit codes, and explicit user commands.

## Security Considerations

1. **Skip permissions mode**: Workers use `--dangerously-skip-permissions`.
   This is acceptable because workers operate in isolated worktrees, have no
   network access to remotes, and require human review before merge.

2. **Prompt injection**: Prompts sent via tmux literal mode (`-l` flag) to
   prevent special character interpretation.

3. **State file**: Contains prompts and paths but should not contain secrets.

## Appendices

- `llmc2-appendix-tmux.md`: TMUX integration details, debounce timing research
