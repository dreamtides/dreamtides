---
lattice-id: LDAWQN
name: autolog-task-poollog-files-are-empty-des
description: auto.log and task_pool.log files are empty despite auto mode operation
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- auto-mode
- bug
- llmc-auto
- logging
created-at: 2026-01-22T03:06:01.330065Z
updated-at: 2026-01-22T06:12:35.856479Z
closed-at: 2026-01-22T06:12:35.856478Z
---

# Bug: auto.log and task_pool.log Files Empty

## Summary

During auto mode operation, the `auto.log` and `task_pool.log` files in `$LLMC_ROOT/logs/` remain empty despite successful task assignment, execution, and auto-accept completion.

## Observation

After a complete auto mode lifecycle (task assigned, worker completed, auto-accept merged):

```
$ wc -c logs/auto.log logs/task_pool.log
0 logs/auto.log
0 logs/task_pool.log
```

Meanwhile, `llmc.jsonl` has 60 lines of structured logs including all the expected events.

## Expected Behavior

According to the test scenario expectations:
- `auto.log` should contain task assignment and completion events
- `task_pool.log` should show task pool command invocations and returned task descriptions

## Evidence

`llmc.jsonl` shows the events happened:
```json
{"message":"Processing completed worker","worker":"auto-1"}
{"message":"Starting auto accept workflow","worker":"auto-1"}
{"message":"Successfully merged to master","worker":"auto-1","commit":"0a9d732"}
```

But `auto.log` has no content.

## Possible Causes

1. The AutoLogger may not be writing to auto.log correctly
2. The task pool logging may not be implemented
3. Log buffering issue (though unlikely given daemon ran for several minutes)
4. Log file initialization issue

## Impact

Low priority - the structured logs in llmc.jsonl contain all necessary information for debugging. However, the human-readable auto.log is more convenient for quick inspection and the test scenario expects these files to have content.