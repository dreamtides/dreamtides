---
name: stress-test
description: Run AI vs AI stress test loop, find crashes, dispatch subagents to fix them
---

Run the AI stress test to find and fix crashes in the rules engine.

## Arguments

Optional arguments after `/stress-test`:
- AI config override: e.g. `'{"monteCarlo":1}' '{"monteCarlo":1}'`
- `--seed <N>`: base seed
- `--deck <choice>`: vanilla, starting-five, benchmark1, core11

Default: `just stress-test` (FirstAvailableAction vs FirstAvailableAction, core11 deck)

## Workflow

### Step 1: Run the stress test

Run the stress test using the Bash tool. Use the default command unless the user
specified custom arguments:

```
just stress-test
```

Or with custom AI configs:
```
just matchup '<ai1>' '<ai2>' --stress --deck core11
```

Run this in the foreground. The stress test runs matches with incrementing seeds
until a crash is found, then exits with code 1 and prints crash details.

If the stress test runs many matches without crashing (the user interrupts or it
runs for a long time), report success and stop.

### Step 2: Capture crash info

When the stress test crashes, its output contains:
- The seed that caused the crash
- The panic message and backtrace
- A reproduce command (`just matchup ... --seed <N>`)

Extract all of this from the output.

### Step 3: Dispatch a fix subagent

Launch an Agent (general-purpose) in a worktree (`isolation: "worktree"`) with
the following prompt structure:

```
A crash was found in the Dreamtides rules engine during AI stress testing.

## Crash Details
Seed: <seed>
Panic message: <panic_message>
Backtrace: <backtrace>

## Reproduce
<reproduce_command>

## Instructions
1. Read the CLAUDE.md file for project conventions
2. Investigate the root cause of the crash by reading the relevant source files
3. Fix the bug — do NOT suppress the crash by catching/ignoring it. Fix the
   actual logic error.
4. Run the reproduce command to verify the fix: <reproduce_command>
5. Run `just fmt` then `just review` to ensure no regressions
6. Commit with a descriptive message explaining the root cause and fix
```

### Step 4: Verify and repeat

After the subagent completes:
1. Check if the subagent's worktree has changes (it will report back)
2. If the fix looks good, merge the worktree branch
3. Re-run the stress test from Step 1 to find the next crash
4. Repeat until the stress test runs without crashes

### Step 5: Report results

After each cycle, briefly report:
- What crash was found
- What the fix was
- Whether the fix was verified
