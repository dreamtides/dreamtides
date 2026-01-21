---
lattice-id: LB2WQN
name: when-there-are-no-tasks-available-from-t
description: When there are no tasks available from task pool command all workers are idle please print short message stdout indicati
parent-id: LBYWQN
task-type: feature
priority: 1
created-at: 2026-01-21T19:24:04.689167Z
updated-at: 2026-01-21T20:09:21.635213Z
closed-at: 2026-01-21T20:09:21.635213Z
---

When there are no tasks available (from the task pool command) and all workers are idle, please print a short message to stdout indicating we are waiting for more tasks. Do not repeat this every time we check, just the first time we enter the idle state.
