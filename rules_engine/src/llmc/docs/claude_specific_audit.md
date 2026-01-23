# LLMC Claude Code Audit Report

## Scope

- Reviewed `rules_engine/docs/llmc.md` for design assumptions.
- Reviewed LLMC implementation under `rules_engine/src/llmc/src`.
- Referenced Claude-specific appendix and hook docs for additional assumptions:
  - `rules_engine/docs/llmc2-appendix-claude-state.md`
  - `rules_engine/docs/llmc_hooks.md`

## Claude-Specific Touchpoints

### CLI binary and flags

- Hardcoded `claude` command invocation in multiple places:
  - `rules_engine/src/llmc/src/tmux/session.rs`
  - `rules_engine/src/llmc/src/commands/console.rs`
  - `rules_engine/src/llmc/src/overseer_mode/overseer_session.rs`
- Hardcoded flags:
  - `--model <model>` and `--dangerously-skip-permissions` are always appended in command builders.
  - `defaults.skip_permissions` exists in config but is not consulted by these builders.
- `llmc doctor` requires a `claude` binary in PATH:
  - `rules_engine/src/llmc/src/commands/doctor.rs`

### Claude model and tool assumptions

- Model validation is limited to Claude models: `haiku`, `sonnet`, `opus`:
  - `rules_engine/src/llmc/src/config.rs`
- Default config template uses Claude model names and Claude tool list:
  - `rules_engine/src/llmc/src/commands/init.rs`
  - `rules_engine/docs/llmc.md`
- Allowed tools list maps directly to Claude tool names:
  - `rules_engine/src/llmc/src/config.rs`

### Claude hook configuration and event semantics

- `.claude/settings.json` is created and expected in each worktree:
  - `rules_engine/src/llmc/src/commands/add.rs`
  - `rules_engine/src/llmc/src/commands/up.rs`
  - `rules_engine/src/llmc/src/auto_mode/auto_workers.rs`
  - `rules_engine/src/llmc/src/overseer_mode/overseer_session.rs`
- The hook payload schema is Claude-specific:
  - `rules_engine/src/llmc/src/ipc/messages.rs`
  - `rules_engine/src/llmc/src/commands/hook.rs`
- Hook event names are Claude-specific and drive state transitions:
  - `Stop`, `SessionStart`, `SessionEnd` are relied on in `rules_engine/src/llmc/src/patrol.rs`.
- Claude-specific hook semantics are assumed:
  - `SessionEnd` with `reason="clear"` is ignored because `/clear` restarts Claude.
  - `rules_engine/src/llmc/src/patrol.rs`
- Claude Code hook API reference (baseline):
  - <https://code.claude.com/docs/en/hooks>

### Hook usage in `rules_engine/docs/llmc.md`

- Session startup and readiness depend on hooks:
  - `SessionStart` is the signal for Offline -> Idle, so startup depends on hooks firing.
  - `llmc up` regenerates `.claude/settings.json` if missing because Claude may clear it.
  - [`rules_engine/docs/llmc.md`](../../../docs/llmc.md) (Session Startup + Hook Configuration sections)
- State detection is defined as hook-driven:
  - `SessionStart` = ready; `SessionEnd` = exit/crash (ignore `reason="clear"`); `Stop` = task completion.
  - [`rules_engine/docs/llmc.md`](../../../docs/llmc.md) (State Detection section)
- Fallback detection assumes hooks can fail and relies on commit polling:
  - If Stop does not fire within 5 minutes of commits being detected, patrol forces recovery.
  - Missing `.claude/settings.json` is logged as the most common failure.
  - [`rules_engine/docs/llmc.md`](../../../docs/llmc.md) (Fallback Detection section)
- Patrol behavior is explicitly described as hook-driven:
  - Task completion transitions are handled by hooks rather than polling.
  - [`rules_engine/docs/llmc.md`](../../../docs/llmc.md) (Patrol System section)

### Hook usage in LLMC runtime code (key files)

- Hook config generation (Claude-specific `.claude/settings.json`):
  - [`rules_engine/src/llmc/src/commands/add.rs`](../src/commands/add.rs) (`create_claude_hook_settings*`)
  - [`rules_engine/src/llmc/src/overseer_mode/overseer_session.rs`](../src/overseer_mode/overseer_session.rs) (overseer hook config)
- Hook CLI entrypoints (reads stdin JSON, emits IPC events):
  - [`rules_engine/src/llmc/src/commands/hook.rs`](../src/commands/hook.rs) (Stop/SessionStart/SessionEnd handlers)
- Hook event schema and IPC transport:
  - [`rules_engine/src/llmc/src/ipc/messages.rs`](../src/ipc/messages.rs) (`HookEvent`, `ClaudeHookInput`)
  - [`rules_engine/src/llmc/src/ipc/socket.rs`](../src/ipc/socket.rs) (Unix socket listener + send)
- Daemon wiring and event ingestion:
  - [`rules_engine/src/llmc/src/commands/up.rs`](../src/commands/up.rs) (IPC listener startup before workers)
  - [`rules_engine/src/llmc/src/auto_mode/auto_orchestrator.rs`](../src/auto_mode/auto_orchestrator.rs) (hook event queue + processing)
- State transitions and fallback logic:
  - [`rules_engine/src/llmc/src/patrol.rs`](../src/patrol.rs) (primary hook handling and fallback recovery)
  - [`rules_engine/src/llmc/src/tmux/session.rs`](../src/tmux/session.rs) (SessionStart readiness expectation)

### Transcript parsing and API error handling

- The transcript reader expects Claude transcript line JSON with `isApiErrorMessage`:
  - `rules_engine/src/llmc/src/transcript_reader.rs`
- Hook events include a Claude transcript path to locate those logs:
  - `rules_engine/src/llmc/src/ipc/messages.rs`
  - `rules_engine/src/llmc/src/patrol.rs` uses `transcript_path` for API error heuristics.

### Commit attribution cleanup

- Commit message cleanup removes Claude-specific attribution:
  - `rules_engine/src/llmc/src/git.rs` strips "Generated with [Claude Code]" and "Co-Authored-By: Claude".

### Claude state detection assumptions (documentation)

- The appendix documents Claude-specific process names, prompts, and UI patterns:
  - `rules_engine/docs/llmc2-appendix-claude-state.md`
- Hooks document assumes Claude hook API and `.claude/settings.json` location:
  - `rules_engine/docs/llmc_hooks.md`

### Legacy LLMC (`llmc_old`) Claude assumptions

- CLI flags are explicitly Claude-branded and assume Claude CLI behavior:
  - `rules_engine/src/llmc_old/src/cli.rs` defines `--claude-model`, `--claude-no-thinking`,
    `--claude-sandbox`, `--claude-skip-permissions`, `--claude-allowed-tools`,
    `--claude-mcp-config`, and `--claude-interactive`.
- Runtime execution hardcodes Claude CLI flags and streaming output format:
  - `rules_engine/src/llmc_old/src/runtime.rs` uses `--output-format stream-json`,
    `--include-partial-messages`, `--permission-mode bypassPermissions`,
    `--dangerously-skip-permissions`, `--allowedTools`, and `--mcp-config`.
- Stream parser assumes Claude-specific event names and tool names:
  - `rules_engine/src/llmc_old/src/runtime.rs` parses `stream_event`, `tool_result`,
    and tool names like `Read`, `Edit`, `Write`, `Bash`, `Glob`, `Grep`, `TodoWrite`.
- Setup validates the Claude CLI is installed:
  - `rules_engine/src/llmc_old/src/setup.rs` requires `claude` in `REQUIRED_BINARIES`.

### Tests and documentation anchored to Claude

- Tests assert Claude-specific hook config and attribution cleanup:
  - `rules_engine/tests/llmc_tests/tests/hook_settings_tests.rs` requires
    `.claude/settings.json` and hook names `Stop`, `SessionStart`, `SessionEnd`.
  - `rules_engine/tests/llmc_tests/tests/git_tests.rs` expects Claude commit
    attribution strings.
- Overseer design doc assumes a Claude remediation session and `/clear` behavior:
  - `rules_engine/src/llmc/docs/auto_overseer_design.md` references Claude Code
    sessions, hooks, and transcript expectations.
- Unified architecture docs define workers in terms of Claude sessions:
  - `rules_engine/src/llmc/docs/unified_architecture/docs/rollback_plan.md`

## Generalization Strategies

### 1) Introduce a runtime abstraction layer

Define a runtime trait or interface with explicit capabilities and behaviors:

- `RuntimeAdapter::build_command(worker_config, defaults) -> String`
- `RuntimeAdapter::hook_config(worktree_path, worker_name, llmc_root) -> Option<HookConfig>`
- `RuntimeAdapter::parse_hook_input(stdin) -> HookInput`
- `RuntimeAdapter::map_hook_event(raw) -> HookEvent`
- `RuntimeAdapter::transcript_reader() -> Option<TranscriptReader>`
- `RuntimeAdapter::strip_commit_attribution(message) -> String`
- `RuntimeAdapter::capabilities() -> RuntimeCapabilities`

This concentrates all CLI-specific details into `runtime/claude.rs`, enabling
`runtime/codex.rs` or other backends without modifying core state logic.

### 2) Make hooks optional and capability-driven

Not all CLIs provide hooks. Add a fallback to polling-based detection when
`capabilities.hooks == false`:

- Use TMUX output parsing or process inspection to detect ready, working, and
  exit states (as documented in the Claude appendix, but with runtime-specific
  patterns).
- Keep current hook pipeline for runtimes that support it.

### 3) Decouple transcript parsing from the core patrol logic

Abstract the transcript format and error patterns:

- `TranscriptReader::scan(path) -> ApiErrorInfo` per runtime.
- Only run API error heuristics when the runtime declares transcript support.

### 4) Normalize config to support multiple runtimes

Extend `config.toml` to specify a runtime per worker or default runtime:

- `[defaults] runtime = "claude"`
- `[workers.adam] runtime = "codex"`

Move `model`, `skip_permissions`, and `allowed_tools` under runtime-specific
sections or support per-runtime validation.

### 5) Externalize CLI command templates

Allow runtime command templates to be configured, so switching CLIs does not
require code changes:

- `claude_command = "claude --model {model} --dangerously-skip-permissions"`
- `codex_command = "codex --model {model}"`

### 6) Standardize hook event schemas

Define an internal LLMC hook event format and write adapters to normalize
incoming CLI hook payloads. For CLIs without hooks, emit synthetic events from
polling logic.

### 7) Replace Claude-specific commit attribution logic

Introduce a per-runtime list of attribution patterns to strip from commit
messages, configured in adapter-specific code or config.

### 8) Replace Claude hooks with Codex event sources (notify + streams)

Codex does not expose a Claude-style hook configuration file. The current Codex
surface area for eventing is:

- `notify = [...]` in `~/.codex/config.toml` runs an external script for
  `agent-turn-complete` events and passes a single JSON argument
  (includes `type`, `thread-id`, `turn-id`, `cwd`, `input-messages`,
  `last-assistant-message`). See
  <https://developers.openai.com/codex/config-advanced/>.
- `codex exec --json` streams JSONL events (`thread.started`, `turn.started`,
  `turn.completed`, `turn.failed`, `item.*`, etc.). See
  <https://developers.openai.com/codex/noninteractive>.
- Codex App Server emits `thread/started`, `turn/started`, `turn/completed`,
  and `item/*` notifications over its protocol. See
  <https://developers.openai.com/codex/app-server>.
- Codex SDK provides programmatic control of local Codex agents (start/resume
  threads and run prompts). This is a possible runtime adapter surface, but it
  does not advertise hook callbacks. See
  <https://developers.openai.com/codex/sdk>.
- Agents SDK workflows can invoke Codex as an MCP server, exposing `codex` and
  `codex-reply` tools with thread IDs. This is another integration surface if
  we want to own the orchestration layer. See
  <https://developers.openai.com/codex/guides/agents-sdk>.
- Codex CLI is open source and documented in the CLI overview. See
  <https://developers.openai.com/codex/cli/> and
  <https://github.com/openai/codex>.
- Community requests for first-class event hooks are tracked publicly. See
  <https://github.com/openai/codex/issues/2109>.

Replacement plan to approximate Claude hook goals:

1. Stop hook replacement (task completion)
   - Use `notify` for `agent-turn-complete` to call a small adapter script.
   - The adapter can map `cwd` to the worker worktree and send a synthetic
     `HookEvent::Stop` into LLMC (or a new Codex-specific event type).
   - This preserves the "evaluate git state on completion" behavior, but note:
     `agent-turn-complete` fires every turn, so the handler must be idempotent.

2. SessionStart replacement (readiness)
   - Use `codex exec --json` or the App Server protocol to detect
     `thread.started` / `turn.started` as the first readiness signal.
   - Emit a synthetic SessionStart when the first turn begins.
   - If LLMC continues to rely on TMUX, a fallback can still detect readiness
     via process/session existence when events are not available.

3. SessionEnd replacement (exit/crash)
   - For non-interactive runs, treat `turn.failed` or process exit as a
     SessionEnd equivalent.
   - For interactive CLI sessions, detect TMUX session termination and emit a
     synthetic SessionEnd with a best-effort reason.

4. Hook payload normalization
   - Extend the runtime adapter to accept Codex event payloads from `notify`,
     JSONL streams, or App Server notifications and map them into LLMC's
     internal hook event format.

Key limitation to document: `notify` currently only supports
`agent-turn-complete`, so it can cover Stop-equivalent logic but does not
replace SessionStart/SessionEnd without additional event streams or polling.

## Suggested Integration Plan

1. Create `rules_engine/src/llmc/src/runtime` with a `RuntimeAdapter` trait and
   a `ClaudeAdapter` implementation that wraps the current behavior.
2. Refactor command builders and hook config creation to call the adapter.
3. Add `runtime` field to config defaults and workers, with adapter lookup.
4. Move Claude-only transcript parsing and commit attribution stripping into
   the Claude adapter.
5. Add a `NoHooks` fallback path that polls TMUX output for runtimes without
   hooks. Start with basic ready/exit detection and expand as needed.
6. Add a `CodexAdapter` skeleton with TODOs for:
   - CLI binary and flags
   - Model naming
   - Permission handling
   - Hook or polling support
   - Transcript or error signal support

## Open Questions For Alternate CLIs

- Do alternate CLIs provide hooks or a stable event protocol similar to Claude?
- Are there transcript or log files suitable for API error detection?
- What are the equivalents for model naming, tool permissions, and session reset?
- Can they be started reliably from a non-interactive TMUX send, or do they
  require a different session bootstrap?
