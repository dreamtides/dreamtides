# Appendix: Testing Strategy

This appendix documents the testing architecture. See
[Lattice Design](lattice_design.md#testing-architecture) for an overview.
For fuzz testing, see [Appendix: Chaos Monkey](appendix_chaos_monkey.md).

## Philosophy

Tests invoke `lat` commands via inline function calls to `main()`.

Tests are black-box: they exercise the CLI interface, not internal APIs.

## Fake vs Real Analysis

| Dependency | Decision | Rationale |
|------------|----------|-----------|
| Git | **Fake** | Real git is slow (~100ms per repo init). For 1000+ tests, this dominates runtime. |
| Filesystem | **Real** | tmpdir operations are fast (<1ms). Real filesystem catches encoding, permission, and path edge cases that fakes miss. |
| SQLite | **Real** | In-memory mode (`:memory:`) is as fast as any fake. Real SQLite catches query edge cases. |
| Time | **Real** | Timestamps are recorded but not used for logic. Determinism rarely needed. |

## GitOps Trait

All git operations go through the `GitOps` trait:

```rust
trait GitOps {
    fn ls_files(&self, pattern: &str) -> Result<Vec<PathBuf>>;
    fn diff(&self, from: &str, to: &str, pattern: &str) -> Result<Vec<PathBuf>>;
    fn status(&self, pattern: &str) -> Result<Vec<StatusEntry>>;
    fn rev_parse(&self, refname: &str) -> Result<String>;
}
```

Production code uses `RealGit` which shells out to git. Tests inject `FakeGit`
which maintains in-memory state:

```rust
struct FakeGit {
    files: HashMap<PathBuf, FileState>,  // Working tree
    commits: Vec<Commit>,                 // History
    head: String,                         // Current HEAD
}
```

The trait is threaded through the codebase via a context parameter. Functions
that need git access receive `&dyn GitOps`.

## Test Structure

```
tests/lattice/
├── commands/     # Per-command tests (create, show, list, etc.)
├── index/        # Reconciliation and query tests
└── integration/  # Multi-command workflows
```

Each test:
1. Creates a temp directory
2. Runs `lat` commands with `FakeGit` injected
3. Asserts on stdout, stderr, exit code, and file state

## Test Setup

Use `lat` commands directly for setup—no separate builder API:

```rust
#[test]
fn show_displays_blocking_tasks() {
    let env = TestEnv::new();
    lat(LatCommand.Create, env, &["tasks/a.md", "-d", "Task A"]);
    lat(LatCommand.Create, env, &["tasks/b.md", "-d", "Task B", "--deps", "blocked-by:LXXXXX"]);

    let result = lat(LatCommand.Show, env, &["LYYYYY"]);
    assert!(result.stdout.contains("Blocked by"));
}
```

This tests the actual CLI interface and serves as executable documentation.

## Test Categories

- **Happy path**: Normal operation with valid inputs
- **User errors**: Invalid inputs producing clear error messages
- **System errors**: Index corruption, simulated git failures
- **Edge cases**: Empty repos, unicode, deep nesting, circular links
