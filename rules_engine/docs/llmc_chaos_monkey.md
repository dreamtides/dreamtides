# LLMC Chaos Monkey Design Document

## Overview

LLMC Chaos Monkey is a Python testing framework for validating the reliability
and self-healing capabilities of LLMC. It performs controlled, reproducible
sequences of operations designed to uncover edge cases, race conditions, and
recovery failures across the LLMC system.

### Goals

1. **Validate self-healing**: Confirm LLMC recovers gracefully from crashes in
   the daemon, TMUX, and Claude Code sessions
2. **Reproducible failures**: Every failure should be reproducible via a minimal
   sequence of logged operations
3. **Distinguish expected from unexpected**: Clearly separate "correct
   rejections" (e.g., messaging nonexistent worker) from "bugs" (e.g., crash
   during accept)
4. **Single-error analysis**: Detect ONE error, stop immediately, and provide
   sufficient context for Claude Sonnet to diagnose and fix the issue

### Non-Goals

- Purely random fuzzing (would rarely hit interesting states)
- Performance testing or benchmarking
- Testing Claude Code itself (only LLMC orchestration)
- Long-running soak tests (focus on rapid iteration)

## Architecture

```
chaos_monkey/
├── chaos_monkey.py           # Main entry point
├── config.py                 # Test configuration
├── operations/               # Individual operation implementations
│   ├── __init__.py
│   ├── daemon.py            # up, down operations
│   ├── workers.py           # add, nuke, start, message operations
│   ├── review.py            # review, accept, reject operations
│   ├── maintenance.py       # doctor, rebase, reset operations
│   └── chaos.py             # Crash injection operations
├── scenarios/               # Predefined test scenarios
│   ├── __init__.py
│   ├── merge_conflicts.py   # Conflict generation scenarios
│   ├── crash_recovery.py    # Crash and recovery scenarios
│   └── lifecycle.py         # Full worker lifecycle scenarios
├── state_machine.py         # Expected state tracking
├── oracle.py                # Result validation
├── logger.py                # Structured logging for replay
├── reset.py                 # Clean reset operations
└── replay.py                # Replay logged operations
```

## Expected vs Unexpected Errors

This is the critical distinction for meaningful testing. An "expected error" is
LLMC correctly rejecting an invalid operation. An "unexpected error" is a bug.

### Expected Errors (Not Bugs)

These are operations LLMC should reject with a clear error message:

| Operation | Expected Error Condition |
|-----------|-------------------------|
| `llmc start --worker X` | Worker X does not exist |
| `llmc start --worker X` | Worker X is not idle |
| `llmc message X "..."` | Worker X does not exist |
| `llmc accept X` | Worker X not in `needs_review` state |
| `llmc reject "..."` | No worker has been reviewed recently |
| `llmc rebase X` | Worker X not in `working` or `needs_review` state |
| `llmc attach X` | Worker X does not exist |
| `llmc add X` | Worker X already exists |
| `llmc nuke X` | Worker X does not exist |
| `llmc up` | Daemon already running |
| `llmc review X` | Worker X not in `needs_review` state |

### Unexpected Errors (Bugs)

These indicate LLMC failures that should never occur:

| Category | Examples |
|----------|----------|
| **Crashes** | Daemon crashes, segfaults, panics |
| **State corruption** | Invalid JSON, missing fields, inconsistent state |
| **Stuck workers** | Worker stays in `working` forever despite completion |
| **Lost work** | Commits disappear, worktrees corrupted |
| **Recovery failures** | System doesn't recover after crash injection |
| **Race conditions** | Operations fail intermittently |
| **Zombie processes** | TMUX sessions orphaned, Claude processes leaked |
| **Git corruption** | Worktree in invalid state, failed rebases |

### Classification Protocol

```python
class ErrorClassification(Enum):
    EXPECTED_REJECTION = "expected"     # LLMC correctly rejected invalid operation
    UNEXPECTED_FAILURE = "bug"          # LLMC failed unexpectedly
    RECOVERY_FAILURE = "recovery_bug"   # LLMC failed to recover from injected crash
    TIMEOUT = "timeout"                 # Operation exceeded time limit
```

**Classification Rules:**

1. **Exit code + known error pattern → EXPECTED**: If `llmc` exits non-zero AND
   stderr matches a known rejection pattern, classify as expected
2. **Exit code + unknown error → UNEXPECTED**: Non-zero exit with unknown error
   is a bug
3. **Crash/signal → UNEXPECTED**: Process killed by signal is always a bug
4. **State validation failure → UNEXPECTED**: Post-operation state invariants
   violated is a bug
5. **Timeout → TIMEOUT**: Special handling, may or may not be a bug depending on
   context

```python
EXPECTED_ERROR_PATTERNS = [
    r"Worker '.*' does not exist",
    r"Worker '.*' is not idle",
    r"Worker '.*' is not in needs_review state",
    r"Worker '.*' already exists",
    r"No worker has been reviewed recently",
    r"Daemon is already running",
    r"Daemon is not running",
    r"Cannot .* while worker is in .* state",
]
```

## Test Scenarios

Chaos Monkey uses **scenarios** rather than pure randomness. Each scenario is a
directed sequence of operations designed to reach interesting states.

### Scenario 1: Basic Lifecycle

Tests the happy path: create worker → assign work → review → accept.

```python
scenario_basic_lifecycle = [
    Reset(),
    Init(),
    Up(),
    Add(worker="alpha", model="haiku"),
    WaitForIdle("alpha"),
    Start(worker="alpha", prompt="Create a file named test.txt with content 'hello'"),
    WaitForNeedsReview("alpha"),
    Review(worker="alpha"),
    Accept(worker="alpha"),
    AssertState("alpha", WorkerStatus.IDLE),
    Down(),
]
```

### Scenario 2: Rejection Cycle

Tests reject → rework → accept flow.

```python
scenario_rejection_cycle = [
    Reset(),
    Init(),
    Up(),
    Add(worker="alpha", model="haiku"),
    WaitForIdle("alpha"),
    Start(worker="alpha", prompt="Create test.txt with 'hello'"),
    WaitForNeedsReview("alpha"),
    Review(worker="alpha"),
    Reject(message="Please also create test2.txt"),
    WaitForNeedsReview("alpha"),  # Back to needs_review after rework
    Review(worker="alpha"),
    Accept(worker="alpha"),
    Down(),
]
```

### Scenario 3: Merge Conflict

Tests conflict detection and resolution.

```python
scenario_merge_conflict = [
    Reset(),
    Init(),
    Up(),
    Add(worker="alpha", model="haiku"),
    Add(worker="beta", model="haiku"),
    WaitForIdle("alpha"),
    WaitForIdle("beta"),

    # Both workers modify same file
    Start(worker="alpha", prompt="Create src/shared.rs with fn hello() { println!(\"alpha\"); }"),
    Start(worker="beta", prompt="Create src/shared.rs with fn hello() { println!(\"beta\"); }"),

    WaitForNeedsReview("alpha"),
    WaitForNeedsReview("beta"),

    # Accept alpha first
    Review(worker="alpha"),
    Accept(worker="alpha"),

    # Beta should need rebase
    Review(worker="beta"),  # Should trigger rebase
    WaitForState("beta", WorkerStatus.REBASING),
    WaitForNeedsReview("beta"),  # After conflict resolution

    Accept(worker="beta"),
    Down(),
]
```

### Scenario 4: Daemon Crash Recovery

```python
scenario_daemon_crash = [
    Reset(),
    Init(),
    Up(),
    Add(worker="alpha", model="haiku"),
    WaitForIdle("alpha"),
    Start(worker="alpha", prompt="Create test.txt"),

    # Kill daemon while worker is active
    Sleep(seconds=2),  # Let work start
    KillDaemon(),
    Sleep(seconds=1),

    # Restart and verify recovery
    Up(),
    AssertWorkerExists("alpha"),
    AssertSessionAlive("alpha"),

    WaitForNeedsReview("alpha"),
    Accept(worker="alpha"),
    Down(),
]
```

### Scenario 5: TMUX Session Crash

```python
scenario_tmux_crash = [
    Reset(),
    Init(),
    Up(),
    Add(worker="alpha", model="haiku"),
    WaitForIdle("alpha"),
    Start(worker="alpha", prompt="Create test.txt"),

    # Kill TMUX session mid-work
    Sleep(seconds=2),
    KillTmuxSession("llmc-alpha"),

    # Wait for patrol to detect and recover
    Sleep(seconds=120),  # Wait for patrol cycle

    # Verify recovery
    Doctor(),
    AssertWorkerExists("alpha"),
    Down(),
]
```

### Scenario 6: Multi-Worker Chaos

```python
scenario_multi_worker_chaos = [
    Reset(),
    Init(),
    Up(),

    # Add 3 workers
    Add(worker="alpha", model="haiku"),
    Add(worker="beta", model="haiku"),
    Add(worker="gamma", model="haiku"),

    # Start all on different tasks
    Start(worker="alpha", prompt="Create alpha.txt"),
    Start(worker="beta", prompt="Create beta.txt"),
    Start(worker="gamma", prompt="Create gamma.txt"),

    # Random operations while working
    Choice([
        Message(worker="alpha", message="Use uppercase"),
        Message(worker="beta", message="Add a header comment"),
        KillTmuxSession("llmc-gamma"),
    ]),

    # Wait and accept in different order
    WaitForNeedsReview("beta"),
    Accept(worker="beta"),

    WaitForNeedsReview("alpha"),
    Reject(message="Also add gamma.txt"),

    # Continue chaos...
    Sleep(seconds=30),
    Doctor(repair=True, yes=True),

    Down(),
]
```

### Scenario 7: Invalid Operation Sequence

Tests that LLMC correctly rejects invalid operations (expected errors).

```python
scenario_invalid_operations = [
    Reset(),
    Init(),
    Up(),
    Add(worker="alpha", model="haiku"),

    # These should all fail with expected errors
    ExpectError(Start(worker="nonexistent", prompt="test")),
    ExpectError(Accept(worker="alpha")),  # Not in needs_review
    ExpectError(Message(worker="nonexistent", message="test")),
    ExpectError(Add(worker="alpha")),  # Already exists

    WaitForIdle("alpha"),
    Start(worker="alpha", prompt="Create test.txt"),

    ExpectError(Start(worker="alpha", prompt="Another task")),  # Not idle

    Down(),
]
```

## Logging and Replay

### Structured Log Format

Every operation produces a log entry enabling exact replay:

```json
{
    "sequence_id": 42,
    "timestamp_unix": 1705123456.789,
    "operation": {
        "type": "Start",
        "args": {
            "worker": "alpha",
            "prompt": "Create test.txt with content 'hello'"
        }
    },
    "pre_state": {
        "workers": {
            "alpha": {"status": "idle", "branch": "llmc/alpha"}
        },
        "daemon_running": true
    },
    "result": {
        "exit_code": 0,
        "stdout": "Started worker alpha on task...",
        "stderr": "",
        "duration_ms": 1250
    },
    "post_state": {
        "workers": {
            "alpha": {"status": "working", "branch": "llmc/alpha"}
        },
        "daemon_running": true
    },
    "classification": "success"
}
```

### Log Files

```
chaos_runs/
├── run_20250112_143052/
│   ├── operations.jsonl      # Structured log of all operations
│   ├── state_snapshots/      # Full state.json copies at key points
│   ├── llmc_logs/            # Copied from ~/llmc/logs/
│   ├── daemon.log            # Daemon stdout/stderr
│   ├── tmux_captures/        # Captured pane contents at failure
│   └── summary.json          # Run summary with error details
└── latest -> run_20250112_143052
```

### Minimal Reproduction

When an error is detected, the system generates a minimal reproduction:

```python
def generate_minimal_replay(operations_log: str, error_index: int) -> list[Operation]:
    """
    Given a log and the index where error occurred, attempt to find
    a minimal prefix that reproduces the error.

    Uses binary search: try half the operations, if error doesn't reproduce,
    try last 75%, etc.
    """
    operations = load_operations(operations_log)
    prefix = operations[:error_index + 1]

    # Binary search for minimal prefix
    # ... implementation details ...

    return minimal_prefix
```

## Reset Protocol

Clean resets are essential for reproducibility. The reset operation:

```python
def reset():
    """
    Reset to known neutral state.

    1. Kill any running llmc daemon
    2. Kill all llmc-* TMUX sessions
    3. Kill any orphaned Claude processes
    4. Remove ~/llmc directory entirely
    5. Verify no leftover git worktrees
    6. Verify no leftover TMUX sessions
    """

    # Kill daemon if running
    subprocess.run(["pkill", "-f", "llmc up"], check=False)

    # Kill TMUX sessions
    sessions = get_tmux_sessions()
    for session in sessions:
        if session.startswith("llmc-"):
            subprocess.run(["tmux", "kill-session", "-t", session])

    # Remove directory
    llmc_dir = Path.home() / "llmc"
    if llmc_dir.exists():
        shutil.rmtree(llmc_dir)

    # Verify cleanup
    assert not llmc_dir.exists()
    assert not any(s.startswith("llmc-") for s in get_tmux_sessions())
```

**Reset frequency**: Reset before each scenario to ensure isolation.

## Oracle: State Validation

The oracle validates that LLMC state is consistent after each operation.

### Invariants

```python
INVARIANTS = [
    # State file consistency
    "state.json is valid JSON",
    "all workers in state have existing worktrees",
    "all workers in state have matching TMUX sessions (if not offline)",
    "worker status matches TMUX session state",

    # Worker state machine
    "idle workers have no current_prompt",
    "working workers have current_prompt set",
    "needs_review workers have commit_sha set",
    "rebasing workers are in rebase state in git",

    # Git state
    "each worktree is on its expected branch",
    "no worktree has uncommitted changes (except during work)",
    "master has no llmc-prefixed branches (after accept)",

    # Process state
    "daemon_running matches actual daemon process",
    "each worker session has Claude running (if not offline)",
]
```

### Validation Implementation

```python
class Oracle:
    def validate(self, expected_state: ExpectedState) -> list[Violation]:
        violations = []

        # Load actual state
        actual = self.load_actual_state()

        # Check state file
        if not self.is_valid_json(actual.state_file):
            violations.append(InvalidStateFile(actual.state_file))

        # Check each worker
        for name, worker in actual.workers.items():
            # Worktree exists
            if not worker.worktree_path.exists():
                violations.append(MissingWorktree(name))

            # Session matches status
            session_alive = self.tmux_session_exists(worker.session_id)
            if worker.status != WorkerStatus.OFFLINE and not session_alive:
                violations.append(MissingSession(name))

            # Status-specific checks
            if worker.status == WorkerStatus.NEEDS_REVIEW:
                if not worker.commit_sha:
                    violations.append(MissingCommitSha(name))

        return violations
```

## Error Detection Protocol

This is the key protocol for Claude Sonnet analysis.

### Detection Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        Run Operation                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Check Exit Code                               │
│  - Zero → proceed to validation                                  │
│  - Non-zero → classify as expected or unexpected                 │
│  - Signal → definitely unexpected                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Run Oracle Validation                         │
│  - Check all invariants                                          │
│  - Compare actual vs expected state                              │
│  - Record all violations                                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Generate Error Report                         │
│  (only if unexpected error detected)                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    STOP IMMEDIATELY                              │
│  Do not run more operations - preserve state for analysis        │
└─────────────────────────────────────────────────────────────────┘
```

### Error Report Structure

When an unexpected error is detected, generate this report:

```markdown
# Chaos Monkey Error Report

## Summary
- **Error Type**: StateCorruption
- **Operation**: Accept(worker="alpha")
- **Sequence ID**: 47
- **Timestamp**: 2025-01-12T14:30:52Z

## Error Details
Worker "alpha" was in needs_review state but commit_sha was None after accept operation.

## Context

### Last 5 Operations
| Seq | Operation | Result |
|-----|-----------|--------|
| 43 | Start(worker="alpha", ...) | success |
| 44 | WaitForNeedsReview("alpha") | success |
| 45 | Review(worker="alpha") | success |
| 46 | KillDaemon() | success |
| 47 | Accept(worker="alpha") | **FAILED** |

### Pre-Operation State
```json
{
    "workers": {
        "alpha": {
            "status": "needs_review",
            "commit_sha": "abc123",
            ...
        }
    }
}
```

### Post-Operation State
```json
{
    "workers": {
        "alpha": {
            "status": "needs_review",
            "commit_sha": null,
            ...
        }
    }
}
```

### LLMC Command Output
```
$ llmc accept alpha
Error: Failed to accept worker alpha: ...
```

### Relevant Log Excerpts

#### Daemon Log (last 50 lines)
```
[14:30:51] Accepting worker alpha
[14:30:51] Rebasing onto master...
[14:30:52] PANIC: unwrap on None value at state.rs:142
```

#### Worker Session Capture
```
... (last 100 lines of TMUX pane content)
```

## Minimal Replay Sequence
```python
# To reproduce, run these operations:
Reset()
Init()
Up()
Add(worker="alpha", model="haiku")
Start(worker="alpha", prompt="Create test.txt")
WaitForNeedsReview("alpha")
Review(worker="alpha")
KillDaemon()
Accept(worker="alpha")  # <- Error occurs here
```

## Suggested Investigation
1. Check state.rs line 142 for unwrap on commit_sha
2. Verify accept handles daemon restart gracefully
3. Consider: was state not persisted before daemon death?
```

## Testing Protocol for Claude Sonnet

This section defines exactly how Claude Sonnet should use Chaos Monkey.

### Invocation

```bash
# Run until first error is detected
python chaos_monkey.py --until-error

# Run a specific scenario
python chaos_monkey.py --scenario merge_conflict --until-error

# Run with specific seed for reproducibility
python chaos_monkey.py --seed 12345 --until-error

# Run all scenarios once (for CI)
python chaos_monkey.py --all-scenarios --fail-fast
```

### Step-by-Step Protocol

**1. Run Chaos Monkey**

```bash
cd /path/to/rules_engine
python chaos_monkey/chaos_monkey.py --until-error
```

**2. On Error Detection, Read the Report**

```bash
cat chaos_runs/latest/summary.json
cat chaos_runs/latest/error_report.md
```

**3. Analyze the Failure**

Focus on these elements:
- What operation failed?
- What was the pre-operation state?
- What was the expected post-operation state?
- What was the actual post-operation state?
- What do the logs show?

**4. Reproduce Minimally**

```bash
# Use the minimal replay from error report
python chaos_monkey/replay.py chaos_runs/latest/minimal_replay.json
```

**5. Fix the Bug**

Navigate to the relevant LLMC source code and fix the issue.

**6. Verify the Fix**

```bash
# Replay the same sequence - should now pass
python chaos_monkey/replay.py chaos_runs/latest/minimal_replay.json

# Run full chaos again to find next bug
python chaos_monkey.py --until-error
```

### Output Format for Claude Sonnet

When run with `--until-error`, the output should be:

```
=== LLMC Chaos Monkey ===

Scenario: merge_conflict
Seed: 12345

[  1] Reset                           OK (0.5s)
[  2] Init                            OK (2.1s)
[  3] Up                              OK (3.4s)
[  4] Add(worker="alpha")             OK (1.2s)
[  5] Add(worker="beta")              OK (1.1s)
[  6] WaitForIdle("alpha")            OK (5.0s)
[  7] Start(worker="alpha", ...)      OK (1.5s)
[  8] Start(worker="beta", ...)       OK (1.4s)
[  9] WaitForNeedsReview("alpha")     OK (45.2s)
[ 10] Accept(worker="alpha")          OK (3.2s)
[ 11] Review(worker="beta")           UNEXPECTED FAILURE

!!! Error Detected !!!

Error Type: UnexpectedError
Operation: Review(worker="beta")
Exit Code: 1
Stderr:
  Error: Worker 'beta' worktree has uncommitted changes

Expected: Review should trigger rebase, not fail
Actual: Review failed due to dirty worktree

Full report: chaos_runs/latest/error_report.md
Minimal replay: chaos_runs/latest/minimal_replay.json

State preserved at: ~/llmc/
```

## Implementation Details

### Operation Classes

```python
@dataclass
class Operation:
    """Base class for all operations."""

    def execute(self) -> OperationResult:
        raise NotImplementedError

    def expected_errors(self) -> list[str]:
        """Patterns that indicate expected (not-bug) failures."""
        return []

@dataclass
class Start(Operation):
    worker: str
    prompt: str

    def execute(self) -> OperationResult:
        result = subprocess.run(
            ["llmc", "start", "--worker", self.worker, "--prompt", self.prompt],
            capture_output=True,
            text=True,
            timeout=30
        )
        return OperationResult(
            exit_code=result.returncode,
            stdout=result.stdout,
            stderr=result.stderr
        )

    def expected_errors(self) -> list[str]:
        return [
            f"Worker '{self.worker}' does not exist",
            f"Worker '{self.worker}' is not idle",
        ]

@dataclass
class ExpectError(Operation):
    """Wrapper that expects the inner operation to fail with a known error."""
    inner: Operation

    def execute(self) -> OperationResult:
        result = self.inner.execute()
        if result.exit_code == 0:
            raise UnexpectedSuccess(f"{self.inner} should have failed but succeeded")
        if not self._is_expected_error(result.stderr):
            raise UnexpectedError(f"{self.inner} failed with unexpected error: {result.stderr}")
        return result
```

### Wait Operations

Wait operations poll state until condition is met or timeout:

```python
@dataclass
class WaitForNeedsReview(Operation):
    worker: str
    timeout_seconds: int = 300  # 5 minutes default
    poll_interval: float = 5.0

    def execute(self) -> OperationResult:
        start = time.time()
        while time.time() - start < self.timeout_seconds:
            status = get_worker_status(self.worker)
            if status == WorkerStatus.NEEDS_REVIEW:
                return OperationResult(exit_code=0, stdout="Ready for review")
            if status == WorkerStatus.ERROR:
                raise UnexpectedError(f"Worker entered error state while waiting")
            time.sleep(self.poll_interval)
        raise TimeoutError(f"Worker did not reach needs_review within {self.timeout_seconds}s")
```

### Chaos Operations

```python
@dataclass
class KillDaemon(Operation):
    """Simulate daemon crash by killing the process."""

    def execute(self) -> OperationResult:
        result = subprocess.run(
            ["pkill", "-KILL", "-f", "llmc up"],
            capture_output=True
        )
        return OperationResult(
            exit_code=0,  # Success means we killed it
            stdout="Daemon killed",
            stderr=""
        )

@dataclass
class KillTmuxSession(Operation):
    """Simulate TMUX session crash."""
    session: str

    def execute(self) -> OperationResult:
        result = subprocess.run(
            ["tmux", "kill-session", "-t", self.session],
            capture_output=True
        )
        return OperationResult(
            exit_code=result.returncode,
            stdout=result.stdout.decode() if result.stdout else "",
            stderr=result.stderr.decode() if result.stderr else ""
        )
```

### State Tracking

```python
class StateMachine:
    """Track expected state based on operations."""

    def __init__(self):
        self.workers: dict[str, ExpectedWorkerState] = {}
        self.daemon_running: bool = False

    def apply(self, operation: Operation, result: OperationResult):
        """Update expected state based on operation and result."""
        if isinstance(operation, Init):
            self.workers = {}
            self.daemon_running = False

        elif isinstance(operation, Up) and result.success:
            self.daemon_running = True
            for name in self.workers:
                self.workers[name].status = WorkerStatus.IDLE

        elif isinstance(operation, Add) and result.success:
            self.workers[operation.worker] = ExpectedWorkerState(
                name=operation.worker,
                status=WorkerStatus.OFFLINE  # Until daemon brings it up
            )

        elif isinstance(operation, Start) and result.success:
            self.workers[operation.worker].status = WorkerStatus.WORKING
            self.workers[operation.worker].current_prompt = operation.prompt

        # ... etc for all operations
```

## Configuration

```python
# config.py

# Timeouts (seconds)
OPERATION_TIMEOUT = 30
WAIT_FOR_IDLE_TIMEOUT = 60
WAIT_FOR_NEEDS_REVIEW_TIMEOUT = 300
WAIT_FOR_REBASE_TIMEOUT = 120

# Polling
POLL_INTERVAL = 5.0

# Retry
MAX_RETRIES = 3
RETRY_DELAY = 2.0

# Model
WORKER_MODEL = "haiku"  # Always use haiku for chaos testing

# Workers
MAX_WORKERS = 3

# Randomness
DEFAULT_SEED = None  # Use current time if None
```

## Known Limitations

1. **Haiku-only**: Tests use Haiku model exclusively for speed and cost. This
   may miss bugs specific to other models.

2. **Simple prompts**: Test prompts are minimal file creation tasks. Complex
   code changes aren't tested.

3. **No network testing**: Network failures, Claude API issues, etc. are not
   simulated.

4. **Single machine**: Doesn't test multi-machine or distributed scenarios.

5. **Time-dependent**: Some tests depend on timing (e.g., waiting for patrol).
   May have flakiness.

## Future Enhancements

1. **Property-based testing**: Generate random operation sequences that maintain
   invariants
2. **Coverage tracking**: Track which LLMC code paths are exercised
3. **Regression suite**: Store and replay known-failing sequences
4. **Performance mode**: Track operation timing regressions
5. **Network chaos**: Inject network failures to Claude API
6. **Resource exhaustion**: Test behavior under disk/memory pressure
