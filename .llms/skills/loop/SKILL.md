---
name: loop
description: Implements parallel agentic feature loop
disable-model-invocation: true
---

Please begin parallel agentic loop implementation of project tasks
via `MAX_AGENTS` sub-agents until all tasks are completed.

## Stale Worktree Cleanup

Before starting, clean up any stale worktrees from previous runs:
```
cd /Users/dthurn/Documents/GoogleDrive/dreamtides/
git worktree list
```
For any worktrees matching `../dreamtides-task-*`, remove them:
```
git worktree remove --force ../dreamtides-task-[id]
git branch -D task-[id]
```

## Parallel Task Execution via Git Worktrees

Do not load any skills or perform any analysis yourself. Keep your own output
minimal — no summaries, no status reports, no commentary beyond what is needed
to execute the steps below.

1. Check ready tasks with TaskList

2. For each ready task up to a maximum of `MAX_AGENTS`, spawn an Opus sub-agent. Select
   tasks in creation order (oldest/lowest number first).

   **IMPORTANT**: Sub-agents do NOT have access to TaskGet. You (the orchestrator)
   must fetch task details yourself using `TaskGet` and include them in the prompt.

   For each task:
   a. Call `TaskGet` with the task ID to retrieve the full task details
   b. Create the agent with this prompt template, substituting [id] and [task_details]:

 ---
 Implement Task #[id] in a git worktree.

 **Task Details:**
 [task_details]

 Setup:
 ```
 cd /Users/dthurn/Documents/GoogleDrive/dreamtides/
 git worktree add ../dreamtides-task-[id] -b task-[id] master
 cd ../dreamtides-task-[id]
 ```

 Implementation:
 1. Read relevant documentation and code as needed
 2. Implement the task following code style in CLAUDE.md
 3. Run: just fmt && just review
 4. Commit with descriptive message

 Before Reporting Complete:
 5. Check if the main worktree's master has advanced beyond your branch point
    (other parallel tasks may have merged while you were working):
    git fetch /Users/dthurn/Documents/GoogleDrive/dreamtides master
    git log HEAD..FETCH_HEAD --oneline
 6. If there are new commits, rebase onto them:
    `git rebase FETCH_HEAD`
    (resolve any conflicts, keeping your intended changes)
    Run: `just fmt && just review`
 7. Report final commit hash only after rebase is complete (or unnecessary)
 ---

   c. Mark each task as `in_progress` via `TaskUpdate` before launching its agent.

3. Launch with: `run_in_background: true`, `model: opus`, `subagent_type: "general-purpose"`

4. **STOP and wait.** Do not send any further messages or tool calls. You will
   receive a `<task-notification>` message when agents complete. Only then
   should you proceed to process completions.

### Processing Completed Agents

Process completions one at a time, in the order notifications arrive.

5. On completion notification, **always rebase onto current master first**:
 ```bash
 cd /Users/dthurn/Documents/GoogleDrive/dreamtides-task-[id]
 git rebase master
 ```
 If there are conflicts, resolve them (keep the task's intended changes),
 then run `just fmt && just review` in the worktree.

 If the conflicts are too complex to resolve yourself, spawn a rebase agent:

 ---
 Rebase task-[id] branch onto master and resolve conflicts.

 The worktree already exists at: /Users/dthurn/Documents/GoogleDrive/dreamtides-task-[id]

 Steps:
 1. cd /Users/dthurn/Documents/GoogleDrive/dreamtides-task-[id]
 2. git rebase master
 3. Resolve any conflicts (keep the task's intended changes)
 4. git rebase --continue
 5. Run: just fmt && just review
 6. Report the new commit hash
 ---

 Launch with: `model: opus`, `subagent_type: "general-purpose"`, `run_in_background: true`

6. After rebase succeeds, fast-forward merge (should always succeed now):
 ```bash
 cd /Users/dthurn/Documents/GoogleDrive/dreamtides/
 git checkout master
 git merge --ff-only task-[id]
 ```

7. Clean up and continue:
 - `git worktree remove ../dreamtides-task-[id]`
 - `git branch -d task-[id]`
 - Update task status to completed via `TaskUpdate`
 - Process next completion, or if all completions are handled, go back to
   step 1: check TaskList for remaining ready tasks and spawn the next batch
 - Repeat until TaskList shows no pending or in-progress tasks

## Re-orientation After Context Compaction

If you are uncertain about current state (e.g. after context compaction), run
these two commands before doing anything else:

1. `git worktree list` — shows which worktrees currently exist. Any
   `dreamtides-task-*` worktree means an agent is (or was) working on that task.
2. `TaskList` — shows task statuses. Cross-reference with worktrees:
   - `in_progress` + worktree exists → agent is likely still running; wait for
     its `<task-notification>`
   - `in_progress` + no worktree → agent finished but wasn't merged; check the
     branch with `git branch --list 'task-*'` and merge if the branch exists,
     or reset the task to `pending` if the branch is gone
   - `pending` + worktree exists → task was spawned but not marked in_progress
     (treat as in_progress)

Do NOT spawn new agents until you've confirmed the current number of active
worktrees is below 3.

## Critical Rules

### Waiting for agents (MOST IMPORTANT)

After launching background agents, you MUST **stop generating immediately**.
Do not send any tool calls, do not write any messages, do not attempt to check
on the agents. Simply end your turn. The system will deliver a
`<task-notification>` message to you when each agent finishes. That notification
is your signal to process the completion.

**Specifically, do NOT do any of the following while waiting:**
- Do NOT call `Bash` with `sleep` + `tail` to poll the output file
- Do NOT call `Read` on the agent's output file to check progress
- Do NOT call `TaskOutput` (with `block: true` or `block: false`) - this WILL
  KILL all running background agents when interrupted
- Do NOT write a loop, polling mechanism, or repeated checks of any kind
- Do NOT send any message at all — just stop and wait for the notification

The correct pattern after launching background agents is:
1. Launch agents with `run_in_background: true`
2. Optionally send a short status message to the user
3. **End your turn. Do nothing else. Wait for `<task-notification>`.**

### Other rules

- NEVER operate more than `MAX_AGENTS` worktrees at a time
- **ALL Task tool invocations MUST use `run_in_background: true`**
- NEVER use the "resume" parameter to continue agents - always spawn fresh agents
- NEVER read large documentation files yourself - let agents read what they need
- Always fetch task details with TaskGet and include them in the spawn prompt (sub-agents cannot access TaskGet)
- Launch multiple agents in parallel when possible (single message, multiple Task calls)
- Prefer doing simple rebases yourself rather than spawning agents for them
- Only spawn a rebase agent if conflicts are too complex to resolve manually
