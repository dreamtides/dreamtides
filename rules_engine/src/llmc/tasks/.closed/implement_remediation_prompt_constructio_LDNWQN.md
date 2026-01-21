---
lattice-id: LDNWQN
name: implement-remediation-prompt-constructio
description: Implement remediation prompt construction
task-type: feature
priority: 1
labels:
- auto-overseer
- core
- overseer
blocking:
- LDOWQN
blocked-by:
- LDLWQN
- LDMWQN
created-at: 2026-01-21T04:01:29.580102Z
updated-at: 2026-01-21T15:02:44.928641Z
closed-at: 2026-01-21T15:02:44.928640Z
---

## Overview

Implement the logic that constructs the remediation prompt sent to the overseer's Claude Code session when a daemon failure is detected.

## Implementation Steps

1. **Create remediation_prompt.rs** under `src/overseer_mode/`:
   - `build_remediation_prompt(failure: &HealthStatus, config: &Config) -> String`
   - Assembles all context needed for Claude to diagnose and fix the issue

2. **Prompt structure**:
   - Start with user-configured `remediation_prompt` from TOML
   - Append structured error context section
   - Append recovery instructions section

3. **Error context gathering**:
   - Failure type (heartbeat stale, process death, log error, stall, identity mismatch)
   - Daemon registration info (PID, start time, instance ID) if available
   - Last N lines of daemon log (e.g., 100 lines)
   - Last N lines of task_pool.log
   - Last N lines of post_accept.log
   - Current worker states from state.json
   - Git status summary of main repo

4. **Recovery instructions**:
   - "After fixing the issue, exit normally. The overseer will restart the daemon."
   - "If the issue cannot be fixed, create a file `.llmc/manual_intervention_needed_<timestamp>.txt` with explanation."
   - Include current timestamp for the filename

5. **Context size management**:
   - Truncate log excerpts if too long
   - Prioritize most recent entries
   - Include clear markers for truncation

## Acceptance Criteria

- Prompt includes user's remediation instructions
- All relevant log files included
- Worker states and git status included
- Recovery instructions are clear
- Prompt is well-structured for Claude to parse
