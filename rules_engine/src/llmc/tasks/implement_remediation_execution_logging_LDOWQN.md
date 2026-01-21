---
lattice-id: LDOWQN
name: implement-remediation-execution-logging
description: Implement remediation execution and logging
task-type: feature
priority: 2
labels:
- auto-overseer
- core
- overseer
blocking:
- LDPWQN
blocked-by:
- LDNWQN
- LDKWQN
created-at: 2026-01-21T04:02:14.576229Z
updated-at: 2026-01-21T13:56:56.746964Z
---

## Overview

Implement the execution of remediation prompts and comprehensive logging of remediation sessions.

## Implementation Steps

1. **Create remediation_executor.rs** under `src/overseer_mode/`:
   - `execute_remediation(prompt: &str, session: &OverseerSession) -> Result<RemediationOutcome>`
   - Handles sending prompt and waiting for completion

2. **Remediation execution sequence**:
   - Send `/clear` to overseer Claude Code session
   - Send constructed remediation prompt
   - Monitor for completion via hooks (same mechanism as worker completion detection)
   - Wait for Claude Code to signal task completion
   - Return outcome (success or failure indicators)

3. **Completion detection**:
   - Reuse existing hook-based completion detection from worker management
   - Hook fires when Claude completes its task
   - Handle timeout case (hook never fires) - this is a HUMAN failure case

4. **Comprehensive logging**:
   - Create `logs/remediation_<timestamp>.txt` for each remediation attempt
   - Log format:
     - Timestamp and failure type that triggered remediation
     - Full constructed prompt
     - Session transcript (all Claude output, tool calls, results)
     - Final outcome
     - Duration
   - This is in ADDITION to normal JSON logging

5. **Transcript capture**:
   - Capture full TMUX pane content after remediation completes
   - Include all tool calls and their outputs
   - Preserve for debugging and analysis

6. **Outcome determination**:
   - Check for `.llmc/manual_intervention_needed_*.txt` files
   - If present, remediation indicates unfixable issue
   - Otherwise, assume remediation attempted a fix

## Acceptance Criteria

- Remediation prompt sent to Claude session correctly
- Completion detected via hooks
- Full transcript logged to dedicated file
- Manual intervention files detected
- Timeout handling for hung remediation
