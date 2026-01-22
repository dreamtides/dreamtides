---
lattice-id: LD5WQN
name: post-accept-command-only-runs-first-acce
description: post_accept_command only runs for first accept, skipped for subsequent accepts
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
- auto-mode
- post-accept
blocking:
- LH5WQN
created-at: 2026-01-22T07:27:46.243960Z
updated-at: 2026-01-22T14:19:15.760486Z
closed-at: 2026-01-22T14:19:15.760485Z
---

# Bug: post_accept_command only runs for first accept, skipped for subsequent accepts

## Summary

In auto mode, when the `post_accept_command` is configured, it only executes after the first successful task acceptance, but is skipped for all subsequent task acceptances.

## Steps to Reproduce

1. Configure auto mode with `post_accept_command`:
```toml
[auto]
task_pool_command = "/path/to/task_pool.sh"
concurrency = 1
post_accept_command = "/path/to/post_accept.sh"
```

2. Start daemon with `llmc up --auto`
3. Let it process two tasks to completion

## Expected Behavior

The `post_accept_command` should run after EVERY successful accept:
- After first task accept: post_accept_command runs ✓
- After second task accept: post_accept_command runs (expected)

## Actual Behavior

- After first task accept: post_accept_command runs ✓
- After second task accept: post_accept_command is NOT called ✗

## Log Evidence

First task (post_accept runs):
```
{"timestamp":"2026-01-22T07:25:03.483573Z","level":"INFO","fields":{"message":"Worker changes accepted","worker":"auto-1","commit":"2e502a0276fc5b5a8b3b36301a3e4a24422d205b"},"target":"llmc::auto_mode::auto_orchestrator"}
{"timestamp":"2026-01-22T07:25:03.483575Z","level":"DEBUG","fields":{"message":"About to call execute_post_accept_command","worker":"auto-1","post_accept_command":"Some(\"/tmp/llmc-post-accept-test-64110/test_scripts/post_accept.sh\")"},"target":"llmc::auto_mode::auto_orchestrator"}
{"timestamp":"2026-01-22T07:25:03.483577Z","level":"INFO","fields":{"message":"Executing post-accept command",...}
{"timestamp":"2026-01-22T07:25:08.788768Z","level":"INFO","fields":{"message":"Post-accept command completed successfully","worker":"auto-1","duration_ms":"5305"}
```

Second task (post_accept NOT run):
```
{"timestamp":"2026-01-22T07:26:13.005307Z","level":"INFO","fields":{"message":"Successfully merged to master","worker":"auto-1","commit":"bbab3db"},"target":"llmc::auto_mode::auto_accept"}
{"timestamp":"2026-01-22T07:26:13.176441Z","level":"DEBUG","fields":{"message":"Fetched from origin",...}
```
Note: No "Worker changes accepted" and no "About to call execute_post_accept_command" for second task.

## Impact

- Post-accept validation/deployment only happens once per daemon session
- Critical for CI/CD workflows relying on post_accept_command for test/deploy

## Environment

- LLMC auto mode with concurrency=1
- Test environment: `/tmp/llmc-post-accept-test-64110`
