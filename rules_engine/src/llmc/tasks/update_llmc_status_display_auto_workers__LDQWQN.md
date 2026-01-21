---
lattice-id: LDQWQN
name: update-llmc-status-display-auto-workers-
description: Update llmc status to display auto workers and overseer state
task-type: feature
priority: 2
labels:
- auto-overseer
- integration
- ui
blocking:
- LDRWQN
blocked-by:
- LDEWQN
- LDPWQN
- LDSWQN
created-at: 2026-01-21T04:02:15.180960Z
updated-at: 2026-01-21T04:09:15.106358Z
---

## Overview

Update the `llmc status` command to display auto workers in a separate section and show overseer state when active.

## Implementation Steps

1. **Auto workers section**:
   - Add new section header: "Auto Workers" (separate from regular workers)
   - Display each auto worker with: name, status, current task (truncated), time in state
   - Add summary line: "Auto Mode: N workers, M tasks completed"
   - Only show this section when auto mode is active

2. **Overseer section**:
   - Add new section header: "Overseer" (when overseer is active)
   - Display overseer state: Running, Remediating, or not shown if inactive
   - Show daemon PID and uptime
   - Show current remediation attempt number if in remediation
   - Show last few status events (optional)

3. **State detection**:
   - Read `auto_mode` flag from state to determine if auto section needed
   - Check for `overseer.json` registration file to detect active overseer
   - Handle cases where files are missing or stale

4. **Update JSON output**:
   - Extend JSON schema for `llmc status --json`
   - Add `auto_workers` array
   - Add `auto_mode_summary` object
   - Add `overseer` object with state info

5. **Visual formatting**:
   - Clear visual separation between regular workers, auto workers, and overseer
   - Consistent with existing status output style
   - Color coding if terminal supports it (errors in red, etc.)

## Acceptance Criteria

- Auto workers displayed in separate section
- Overseer state displayed when active
- JSON output includes new fields
- Clear visual hierarchy in terminal output
- Graceful handling of missing state
