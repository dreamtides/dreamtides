---
lattice-id: LD4WQN
name: post-accept-command-not-executed-after-a
description: post_accept_command not executed after auto-accept
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
- auto-mode
- post-accept
blocking:
- LH5WQN
created-at: 2026-01-22T06:39:47.701805Z
updated-at: 2026-01-22T07:21:14.892636Z
closed-at: 2026-01-22T07:21:14.892635Z
---

# Bug: post_accept_command Not Executed After Auto-Accept

## Summary

When `post_accept_command` is configured in the `[auto]` section of config.toml, the command is never executed after a worker's changes are merged to master in auto mode.

## Expected Behavior

Per the design doc (LDBWQN):
> `post_accept_command` (optional): Shell command invoked after successfully rebasing a worker's changes onto master. Stdout logged to `logs/post_accept.log`.

The daemon should:
1. Complete the accept workflow (rebase, squash, merge)
2. Execute the configured `post_accept_command`
3. Block until the command completes
4. Log output to `logs/post_accept.log`
5. On non-zero exit, trigger graceful shutdown

## Actual Behavior

The post_accept_command is completely ignored:
1. The command is never executed
2. `logs/post_accept.log` remains empty (0 bytes)
3. No log entry for attempting to execute the command
4. Worker immediately transitions to Idle after merge

## Reproduction Steps

```bash
export TEST_DIR="/tmp/llmc-post-accept-test-$$"
export LLMC_ROOT="$TEST_DIR"

llmc init --source ~/Documents/GoogleDrive/dreamtides --target "$TEST_DIR"

# Create task pool script
mkdir -p $LLMC_ROOT/test_scripts
cat > $LLMC_ROOT/test_scripts/pool.sh << 'EOF'
#!/bin/bash
echo "Create test.txt with content 'test'"
exit 0
EOF
chmod +x $LLMC_ROOT/test_scripts/pool.sh

# Create post-accept script
cat > $LLMC_ROOT/test_scripts/post_accept.sh << 'EOF'
#!/bin/bash
echo "Post-accept executed at $(date)"
EOF
chmod +x $LLMC_ROOT/test_scripts/post_accept.sh

# Configure
cat >> $LLMC_ROOT/config.toml << EOF
[auto]
task_pool_command = "$LLMC_ROOT/test_scripts/pool.sh"
post_accept_command = "$LLMC_ROOT/test_scripts/post_accept.sh"
concurrency = 1
EOF

llmc up --auto &
# Wait for task completion...

# Check - post_accept.log will be empty
cat $LLMC_ROOT/logs/post_accept.log
```

## Evidence From Test

Log entries show the accept workflow completing without any post-accept execution:

```
06:38:33.245709 - Successfully merged to master (worker=auto-1, commit=059ed57)
06:38:37.963077 - Fetched from origin
06:38:38.836487 - Created git worktree (worktree_create)
06:38:38.837443 - Worker changes accepted (worker=auto-1)
```

Missing expected log entries:
- "Executing post_accept_command" 
- "Post_accept_command completed"
- Any content in post_accept.log

## Impact

- Post-accept validation/tests are never run
- Deployment automation won't trigger
- System behaves as if post_accept_command doesn't exist

## Likely Location

The bug is likely in `auto_mode/auto_accept.rs` or `auto_mode/auto_orchestrator.rs` - the post_accept_command execution logic is either missing or not being called after the merge completes.