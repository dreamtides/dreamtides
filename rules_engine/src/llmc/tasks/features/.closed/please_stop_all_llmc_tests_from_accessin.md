---
lattice-id: LD7WQN
name: please-stop-all-llmc-tests-from-accessin
description: 'Please stop all llmc tests from accessing global environment variables like LLMCROOT this is receipe disaster We should '
parent-id: LBYWQN
task-type: feature
priority: 1
labels:
- llmc-auto
created-at: 2026-01-22T14:32:05.335373Z
updated-at: 2026-01-22T14:56:49.977449Z
closed-at: 2026-01-22T14:56:49.977448Z
---

Please stop all llmc tests from accessing global environment variables like `LLMC_ROOT`, this is a receipe for disaster. We should figure out some kind of workaround like using a fake/dependency injection strategy
