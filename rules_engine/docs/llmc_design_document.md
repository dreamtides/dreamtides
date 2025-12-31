# LLMC Design Document

## Top-Level Design

### Purpose
LLMC is a Rust CLI that coordinates multiple command line AI agents working in parallel worktrees. It should create isolated worktrees per agent, run one agent per worktree in headless mode, resolve conflicts (including rerere reuse), and merge cleanly back onto the local master branch. The workflow must default to minimal manual setup and use existing agent CLIs (`claude`, `codex`, `gemini`, `cursor`).

### Requirements and Constraints
- All LLMC code lives in `rules_engine/src/llmc` as a new workspace member crate.
- Avoid modifying other paths unless required for workspace membership or runtime state.
- The canonical repo root is the top-level checkout containing `client/` and `rules_engine/`.
- `llmc setup` creates a new checkout at `~/Documents/llmc` and does not modify the existing `~/Documents/GoogleDrive/dreamtides` checkout.
- Git LFS is required and must be installed and pulled in the new checkout.
- Worktrees live under `<repo>/.worktrees/agent-<id>` and each agent runs with `WORKTREE` set and `cwd` equal to that path.
- Rerere must be enabled once per checkout: `rerere.enabled=true` and `rerere.autoupdate=true`.
- `.llmc/` must be ignored in the repo `.gitignore`; `llmc setup` validates this and fails if missing.
- Agents must follow `AGENTS.md` conventions, including running `just fmt`, `just check`, `just clippy`, and `just review`.

### Repository and Runtime Layout
- Source checkout path: `~/Documents/GoogleDrive/dreamtides` (existing).
- Target checkout path: `~/Documents/llmc` (new, created by `llmc setup`).
- Repo root detection: `git rev-parse --show-toplevel`.
- Worktrees: `<repo>/.worktrees/agent-<id>`.
- LLMC state directory: `<repo>/.llmc/` containing:
  - `state.json` (agent registry and full prompt text)
  - `logs/` (per-run logs, optional)

### Architecture Overview
LLMC is a single binary crate (`rules_engine/src/llmc`) with explicit modules that keep CLI, git operations, state, and runtime execution separate.

Proposed module layout inside `rules_engine/src/llmc/src/`:
- `main.rs` (CLI entrypoint and top-level dispatch)
- `cli.rs` (clap definitions and argument parsing)
- `config.rs` (defaults and derived paths)
- `state.rs` (JSON state load/save, agent registry)
- `worktree.rs` (worktree creation, cleanup, and branch naming)
- `git_ops.rs` (git/LFS/rerere operations via `Command`)
- `prompt.rs` (prompt assembly from flags and files)
- `runtime.rs` (agent runtime command builders)
- `review.rs` (forgejo/diff/difftastic/vscode handlers)
- `notify.rs` (osascript notifications)

### Data Model and State
State is stored in `.llmc/state.json` for machine-friendly lookup by agent name. The file is authoritative for agent lifecycle.

`AgentRecord` fields:
- `agent_id` (string, unique lowercase English noun, used in branch and worktree names)
- `branch` (string, `agent/<agent_id>`)
- `worktree_path` (absolute path)
- `runtime` (`claude | codex | gemini | cursor`)
- `prompt` (full prompt text used for the run)
- `created_at_unix` (u64)
- `last_run_unix` (u64)
- `status` (`idle | running | rebasing | needs_review | accepted | rejected`)
- `last_pid` (optional u32)

State is updated atomically using a temp file and rename.

### CLI Surface
The binary is `llmc` with subcommands: `setup`, `start`, `rebase`, `review`, `reject`, `accept`.

Common flags:
- `--repo <path>`: override repo root detection
- `--agent <id>`: agent identifier (required for `rebase`, `review`, `reject`, `accept`; optional for `start`)
- `--runtime <claude|codex|gemini|cursor>`: runtime for `start`

#### llmc setup
Purpose: create a new repo checkout, enable rerere, configure LFS, and initialize LLMC directories.

Steps:
1. Validate required binaries: `git`, `git-lfs`, `claude`, `codex`, `gemini`, `cursor`, `forgejo`, `difft`, `code`, `osascript`.
2. Create target directory if missing; fail if non-empty.
3. Clone from source checkout using local clone to preserve objects:
   - `git clone --local <source> <target>`
4. Validate `<target>/.gitignore` contains a `.llmc/` entry; fail if missing.
5. `git -C <target> config rerere.enabled true`
6. `git -C <target> config rerere.autoupdate true`
7. `git -C <target> lfs install`
8. `git -C <target> lfs pull`
9. Create `<target>/.worktrees` and `<target>/.llmc`.

#### llmc start
Purpose: create a new agent worktree, build prompt, run agent headlessly, and record results.

Steps:
1. Resolve repo root and ensure `.worktrees` exists.
2. Determine `agent_id`:
   - Use `--agent` if provided (must be a lowercase English noun).
   - Otherwise select an unused noun from a built-in list and use it as the agent id.
3. Create branch and worktree:
   - `git -C <root> worktree add -b agent/<agent_id> <root>/.worktrees/agent-<agent_id> master`
4. Build prompt:
   - Concatenate `--prompt` and `--prompt-file` in order, separated by two newlines.
   - Prepend a fixed LLMC preamble (see Prompt Composition).
   - Store the full prompt in `.llmc/state.json` for the agent record.
5. Spawn runtime with `WORKTREE` set and `cwd` set to the worktree path.
6. Stream stdout/stderr to console unless `--background` is set.
7. If `--notify` (default), run:
   - `osascript -e 'display notification "Task finished" with title "LLMC"'`

#### llmc rebase
Purpose: rebase agent branch onto the latest master and resolve conflicts.

Steps:
1. `git -C <root> fetch origin master`.
2. `git -C <worktree> rebase origin/master`.
3. If conflicts occur, re-run the agent with a conflict-resolution prompt and `git status --porcelain` output.
4. Require the agent to complete `git rebase --continue`, then run `just check` and `just clippy`.

#### llmc review
Purpose: present diffs for a specific agent.

Interfaces:
- `diff`: `git -C <worktree> diff master...agent/<agent_id>`
- `difftastic`: `git -C <worktree> -c diff.external=difft diff master...agent/<agent_id>`
- `vscode`: `code --reuse-window --wait <worktree>`
- `forgejo`: open `http://localhost:3000/<owner>/<repo>/compare/master...agent/<agent_id>`

Forgejo slug extraction:
- Parse `git -C <root> config --get remote.origin.url` and normalize to `<owner>/<repo>`.

#### llmc reject
Purpose: send reviewer notes and the current diff back to the agent for fixes.

Steps:
1. Append notes (from `--notes` or `--notes-file`) to the original prompt.
2. Append `git diff master...agent/<agent_id>` output to provide precise context.
3. Update the stored prompt in `.llmc/state.json` and re-run the agent in the existing worktree using the stored runtime.

#### llmc accept
Purpose: finalize agent work, rebase onto master, fast-forward merge, and clean up.

Steps:
1. Ensure clean working tree and exactly one commit ahead of master.
2. Run `llmc rebase --agent <id>`.
3. `git -C <root> checkout master`.
4. `git -C <root> merge --ff-only agent/<agent_id>`.
5. Remove worktree and branch:
   - `git -C <root> worktree remove <worktree>`
   - `git -C <root> branch -d agent/<agent_id>`
6. Remove agent entry from `.llmc/state.json` and archive logs if present.

### Prompt Composition
LLMC builds a deterministic prompt wrapper around the user prompt:
1. Fixed preamble:
   - Repo root path and worktree path
   - Reminder to follow `AGENTS.md`
   - Hard requirement: only change files under `rules_engine/src/llmc`
   - Required validations: `just fmt`, `just check`, `just clippy`, `just review`
2. User prompt (concatenated from flags/files)
3. Persist the full prompt string in `.llmc/state.json` for future rebase or reject runs.

### Agent Runtime Commands
All agents run headlessly with `WORKTREE` set and `cwd` equal to the worktree path.

- Claude Code:
  - `claude -p <PROMPT> --allowedTools "Bash,Edit,Read" --output-format stream-json`
- Codex:
  - `codex exec --cd <WORKTREE> --ask-for-approval never --sandbox workspace-write <PROMPT>`
- Gemini:
  - `gemini -p <PROMPT> --output-format stream-json`
- Cursor:
  - `cursor --print <PROMPT> --force`

Each runtime invocation must preserve stdout/stderr streaming unless `--background` is used.

### Error Handling and Observability
- Use `anyhow` with `Context` for every external command and file IO.
- Return non-zero exit codes on missing dependencies, failed commands, or invalid state.
- Log structured events to `.llmc/logs/<agent-id>.log` if `--log` is enabled.

### Dependencies
Internal dependencies (alphabetical): none.

External dependencies (alphabetical):
- `anyhow`
- `clap`
- `ctrlc`
- `serde`
- `serde_json`
- `toml`
- `uuid`

## Milestone Documents

### Milestone 1: New LLMC Crate Skeleton
Steps:
- Create `rules_engine/src/llmc` with `Cargo.toml` and `src/main.rs`.
- Wire clap with top-level subcommands and placeholder handlers.
- Ensure the workspace member is picked up by `members = ["src/*"]` without changes to other files.
- Implement minimal `--help` output and error on unknown subcommands.

### Milestone 2: Repo Root and Path Derivation
Steps:
- Implement repo root detection using `git rev-parse --show-toplevel`.
- Add `--repo` override to all commands.
- Add path helpers for `.worktrees` and `.llmc` with validation errors if missing.
- Add unit tests for path derivation with temp dirs if allowed.

### Milestone 3: State Storage and Agent Registry
Steps:
- Define `AgentRecord` and `StateFile` structs with serde.
- Implement atomic read/write of `.llmc/state.json`.
- Add helper functions to register, update, and remove agents by id.
- Validate uniqueness of `agent_id` and branch name on creation.

### Milestone 4: Dependency Checks and Setup Workflow
Steps:
- Implement binary availability checks using `Command::new(...).arg("--version")`.
- Add `llmc setup` with `--source` and `--target` defaults.
- Implement local clone, rerere config, LFS install/pull, and directory creation.
- Add clear error messages for missing tools or non-empty target directory.

### Milestone 5: Worktree and Branch Lifecycle
Steps:
- Implement `worktree add` and `worktree remove` helpers.
- Enforce branch naming as `agent/<agent_id>`.
- Validate worktree cleanliness before starting and before accept.
- Add detection for "one commit ahead" rule using `git rev-list --count`.

### Milestone 6: Prompt Assembly and Archiving
Steps:
- Implement prompt concatenation for repeated `--prompt` and `--prompt-file` flags.
- Build fixed preamble string and prepend to user prompt.
- Store the full prompt text in `.llmc/state.json` for the agent record.
- Add tests for prompt concatenation order and spacing.

### Milestone 7: Agent Runtime Execution
Steps:
- Implement runtime enum and command builders for `claude`, `codex`, `gemini`, `cursor`.
- Ensure `cwd` is the worktree path and `WORKTREE` env var is set.
- Implement `--background` by spawning and storing PID in state.
- Add SIGINT handling using `ctrlc` to forward to child and exit cleanly.

### Milestone 8: Rebase and Conflict Resolution
Steps:
- Implement `llmc rebase` to fetch `origin/master` and rebase the worktree.
- Detect conflicts via exit code and `git status --porcelain`.
- Re-run agent with conflict context and require `git rebase --continue`.
- Verify no unmerged paths remain before marking rebase complete.

### Milestone 9: Review, Reject, Accept
Steps:
- Implement `llmc review` interfaces and forgejo URL generation.
- Implement `llmc reject` to append notes and diff to the original prompt.
- Implement `llmc accept` to rebase, fast-forward merge, and cleanup.
- Enforce clean worktree and single-commit constraint on accept.

### Milestone 10: Notifications, Logging, and Polish
Steps:
- Add `--notify/--no-notify` defaulting to notify on completion.
- Implement optional per-run log files under `.llmc/logs`.
- Add consistent error messages with actionable remediation hints.
- Run `just fmt`, `just check`, `just clippy`, and `just review` after each change set.
