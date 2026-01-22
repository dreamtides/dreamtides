---
lattice-id: LDYWQN
name: test-scenario-auto-worker-stuck-offline-
description: 'Test Scenario: Auto Worker Stuck Offline Despite Running Session'
task-type: task
priority: 1
labels:
- testing
- manual-test
- scenario
- auto-mode
- llmc-auto
created-at: 2026-01-22T04:33:57.229906Z
updated-at: 2026-01-22T05:09:29.157142Z
closed-at: 2026-01-22T05:09:29.157141Z
---

# Test Scenario: Auto Worker Stuck Offline Despite Running Session

## Objective

Verify that auto workers correctly transition from `offline` to `idle` state
even when the SessionStart hook fails to fire. This tests the fallback
mechanism that detects running sessions.

## Prerequisites

- LLMC installed and configured
- No daemon currently running in your test directory

## Environment Setup

**This test MUST run in an isolated LLMC instance.** See
`../isolated_test_environment.md` for complete setup instructions.

```bash
export TEST_DIR="/tmp/llmc-stuck-offline-test-$$"
export LLMC_ROOT="$TEST_DIR"
echo "Creating isolated test environment at: $TEST_DIR"
llmc init --source ~/Documents/GoogleDrive/dreamtides --target "$TEST_DIR"
```

## Setup

```bash
# Create task pool script
mkdir -p "$LLMC_ROOT/test_scripts"
cat > "$LLMC_ROOT/test_scripts/task_pool.sh" << 'EOF'
#!/bin/bash
MARKER="$LLMC_ROOT/test_scripts/.task_issued"
if [ ! -f "$MARKER" ]; then
    touch "$MARKER"
    echo "Create a file test.txt with 'hello'"
else
    exit 0
fi
EOF
chmod +x "$LLMC_ROOT/test_scripts/task_pool.sh"

# Add auto config
cat >> "$LLMC_ROOT/config.toml" << EOF

[auto]
task_pool_command = "$LLMC_ROOT/test_scripts/task_pool.sh"
concurrency = 1
EOF
```

## Test Sequence

### Part 1: Simulate Missing SessionStart Hook

**Step 1.1**: Delete the hook settings to simulate hook not firing.

```bash
# Start daemon in background
llmc up --auto &
DAEMON_PID=$!
sleep 10

# Check initial state
llmc status
```

**Verify**:
- Worker `auto-1` exists
- TMUX session is running (can attach)
- Worker status should transition to `idle` within a few patrol cycles
  even without the SessionStart hook

**Step 1.2**: Verify fallback detection works.

```bash
# Wait and check status multiple times
for i in {1..12}; do
    STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
    echo "Iteration $i: status=$STATUS"
    if [ "$STATUS" = "idle" ] || [ "$STATUS" = "working" ]; then
        echo "SUCCESS: Worker transitioned from offline"
        break
    fi
    sleep 5
done
```

**Verify**:
- Worker transitions from `offline` to `idle` or `working`
- Transition happens within 60 seconds (patrol interval)

### Part 2: Verify Logging

```bash
grep -E "patrol|transition|session" "$LLMC_ROOT/logs/auto.log" | tail -20
```

**Verify**:
- Logs show session detection
- Logs show state transition

## Cleanup

```bash
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null
rm -rf "$TEST_DIR"
unset LLMC_ROOT
```

## Expected Behavior

The patrol loop should detect running sessions and transition workers from
`offline` to `idle` even if the SessionStart hook fails to fire. This is a
fallback mechanism to prevent workers from being stuck indefinitely.

## Related Bug

LBSWQN - Auto worker stuck in 'offline' state despite Claude Code being ready