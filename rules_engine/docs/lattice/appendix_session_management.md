# Appendix: Session Management

## Overview

Session management helps AI agents maintain context across interactions and
track working state during task execution. Lattice provides lightweight
session support through the `lat prime` command and session state tracking.

## The `lat prime` Command

Following beads' successful pattern, `lat prime` provides session-start
context injection.

### Basic Usage

```
lat prime [options]
```

### Output Content

The prime command outputs:
1. Brief workflow instructions (~500 chars)
2. Current working issue (if any)
3. Recently modified documents (last session)
4. Issues ready for work (top 5 by priority)
5. Sync status

### Example Output

```
# Lattice Workflow

## Commands
- lat ready: Find work
- lat show <id>: View document with context
- lat create "name" --path dir/ -t bug: Create issue
- lat update <id> --status in_progress: Claim work
- lat close <id>: Complete work

## Current Session
Working on: LXXXX (implement-oauth) [P1, in_progress]
Started: 2024-01-15 10:30

## Recent Changes
- LYYYY modified 2h ago (auth-design)
- LZZZZ created 4h ago (oauth-bug-1)

## Ready Work
1. LWWWW [P0] critical-security-fix
2. LAAAA [P1] implement-oauth (current)
3. LBBBB [P2] improve-logging
```

### Options

- `--format text|json|minimal`: Output format
- `--stealth`: Skip sync operations (faster startup)
- `--no-ready`: Omit ready work list
- `--no-recent`: Omit recent changes

## Session State

### Working Issue Tracking

When an issue is marked `in_progress`, it becomes the "current" working
issue. This is tracked in `.lattice/session.toml`:

```toml
[current]
issue_id = "LXXXX"
started_at = "2024-01-15T10:30:00Z"
notes = []

[recent]
viewed = ["LYYYY", "LZZZZ"]
modified = ["LXXXX", "LWWWW"]
```

### Automatic Tracking

Session state updates automatically:
- `lat update --status in_progress`: Sets current issue
- `lat show <id>`: Adds to recently viewed
- Any document modification: Adds to recently modified
- `lat close`: Clears current issue

### Manual Session Control

```
lat session start <id>    # Explicitly set current issue
lat session end           # Clear current issue
lat session status        # Show session state
lat session clear         # Reset all session state
```

## Hook Integration

### Claude Code Hooks

Install hooks for automatic prime:

```
lat setup claude          # Install globally
lat setup claude --project # Install for project only
```

Hooks installed:
- **SessionStart**: Runs `lat prime` when Claude Code starts
- **PreCompact**: Runs `lat prime` before context compaction

### Other Editors

Similar integrations available:
- `lat setup cursor`: Cursor IDE rules
- `lat setup aider`: Aider configuration

## Change Awareness

### Recent Changes Query

Following beads' pattern, use date-based filters:

```
lat list --updated-after "yesterday"
lat list --created-after "1 week ago"
lat list --modified-since <commit>
```

### Stale Issue Detection

```
lat stale                     # Default: not updated in 30 days
lat stale --days 90           # Custom threshold
lat stale --status in_progress # Stale in-progress issues
```

### What Changed

For explicit change summaries:

```
lat changes                   # Since last session
lat changes --since "3 days"  # Since date
lat changes --since <commit>  # Since git commit
```

Output:
```
Changes since 2024-01-12:

Created (3):
  LXXXX  implement-oauth         [feature/P1/open]
  LYYYY  oauth-design-doc        [doc]
  LZZZZ  oauth-test-bug          [bug/P2/open]

Modified (2):
  LWWWW  authentication-design   status: open → in_progress
  LAAAA  api-reference           body updated

Closed (1):
  LBBBB  login-timeout-fix       closed 2024-01-13
```

## Session Lifecycle

### Starting Work

Recommended workflow:

```bash
# 1. Prime context (automatic via hooks, or manual)
lat prime

# 2. Find work
lat ready

# 3. Claim issue
lat update LXXXX --status in_progress

# 4. Get briefing
lat show LXXXX --brief
```

### During Work

```bash
# View related documents
lat show <id>

# Track discovered issues
lat create "Found bug" --path bugs/ -t bug --deps discovered-from:LXXXX

# Add working notes (stored in session)
lat session note "Tried approach A, didn't work because..."
```

### Ending Work

```bash
# Close completed issues
lat close LXXXX --reason "Implemented OAuth flow"

# Or pause work
lat update LXXXX --status open  # Back to open, not blocking others
```

## Notes Feature

Session notes provide scratchpad for working observations:

```
lat session note "Observation text"
lat session notes                    # List notes
lat session notes --clear            # Clear notes
```

Notes are:
- Associated with current working issue
- Stored in `.lattice/session.toml`
- Not persisted to documents (ephemeral)
- Useful for context that doesn't belong in issue body

## Token Efficiency

The prime command is designed for minimal token usage:
- Default output: ~1000-1500 characters
- Minimal mode: ~300 characters
- Structured to front-load most important info

Compare to alternatives:
- Loading all open issues: Variable, potentially large
- Full document dumps: 5000+ characters per doc
- Prime command: Consistent, bounded size
