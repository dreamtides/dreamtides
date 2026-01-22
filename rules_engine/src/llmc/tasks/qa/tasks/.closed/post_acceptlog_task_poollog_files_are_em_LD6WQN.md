---
lattice-id: LD6WQN
name: post-acceptlog-task-poollog-files-are-em
description: post_accept.log and task_pool.log files are empty despite commands running
parent-id: LB6WQN
task-type: bug
priority: 2
labels:
- llmc-auto
- auto-mode
- logging
blocking:
- LH5WQN
created-at: 2026-01-22T07:27:46.344818Z
updated-at: 2026-01-22T14:39:34.135130Z
closed-at: 2026-01-22T14:39:34.135130Z
---

# Bug: post_accept.log and task_pool.log files are empty despite commands running

## Summary

The design document specifies that `post_accept_command` stdout should be logged to `logs/post_accept.log` and `task_pool_command` stdout should be logged to `logs/task_pool.log`. However, both files remain empty (0 bytes) even after commands execute successfully.

## Steps to Reproduce

1. Configure auto mode with post_accept_command and task_pool_command
2. Start `llmc up --auto`
3. Let a task complete and accept
4. Check log files:
```bash
ls -la $LLMC_ROOT/logs/
cat $LLMC_ROOT/logs/post_accept.log
cat $LLMC_ROOT/logs/task_pool.log
```

## Expected Behavior

From the design document (LDBWQN):
> - `task_pool_command`: ... Stdout logged to `logs/task_pool.log`
> - `post_accept_command`: ... Stdout logged to `logs/post_accept.log`

The log files should contain the stdout output from the respective commands.

## Actual Behavior

```
$ ls -la $LLMC_ROOT/logs/post_accept.log
.rw-r--r--@ 0 dthurn 21 Jan 23:23 /tmp/llmc-post-accept-test-64110/logs/post_accept.log

$ ls -la $LLMC_ROOT/logs/task_pool.log
.rw-r--r--@ 0 dthurn 21 Jan 23:23 /tmp/llmc-post-accept-test-64110/logs/task_pool.log
```

Both files are empty (0 bytes) despite commands running successfully:
- Post-accept ran for 5305ms
- Task pool returned task content

## Log Evidence

Commands did execute:
```json
{"message":"Executing post-accept command","worker":"auto-1","commit":"...","command":"/tmp/.../post_accept.sh"}
{"message":"Post-accept command completed successfully","worker":"auto-1","duration_ms":"5305"}
{"message":"Task pool command returned a task","command":"...","duration_ms":"164","task_length":91}
```

## Impact

- Cannot debug post_accept failures without log output
- Cannot verify task_pool command behavior
- Makes troubleshooting auto mode issues difficult

## Environment

- LLMC auto mode
- Test environment: `/tmp/llmc-post-accept-test-64110`
