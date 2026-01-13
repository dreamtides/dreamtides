# Appendix: Testing Strategy

## Philosophy

Testing in Lattice follows three principles:
1. Black box: Test the CLI interface, not internal APIs
2. Fast execution: Slow tests discourage running them
3. Comprehensive coverage: Happy paths, errors, and edge cases

## Test Architecture

### CLI Testing

All tests invoke `lat` commands as external processes:

```rust
fn test_create_issue() {
    let env = TestEnv::new();
    let result = env.run(&["create", "Test issue", "--path", "tasks/"]);
    assert!(result.success());
    assert!(result.stdout.contains("Created:"));
}
```

This ensures tests validate user-visible behavior, not implementation details.

### Fake Implementations

External dependencies are replaced with in-memory fakes:

**FakeGit:** In-memory commit graph and file tree
- Supports: status, diff, log, ls-files, rev-parse
- State: HashMap of paths to content, commit history list
- No actual git repository on disk

**FakeFileSystem:** Virtual filesystem
- Supports: read, write, delete, list, exists
- State: HashMap of paths to content
- Optional hooks for failure injection

**FakeClock:** Controllable time
- Fixed or advancing timestamps
- Enables testing time-dependent behavior
- No reliance on system clock

### Test Environment

The `TestEnv` struct provides isolated test context:

```rust
struct TestEnv {
    fake_git: FakeGit,
    fake_fs: FakeFileSystem,
    index_path: PathBuf,  // Temp directory
    client_id: String,    // Fixed for reproducibility
}
```

Each test gets a fresh environment. No state leaks between tests.

## Test Categories

### Happy Path Tests

Normal operation with valid inputs:

- Create document with all fields
- Update issue status through lifecycle
- Generate and use IDs
- Query with various filters
- Show document with context
- Format and check documents

### User Error Tests

Invalid inputs that should produce clear errors:

- Malformed YAML frontmatter
- Invalid field values (status, priority, type)
- Missing required fields
- References to nonexistent IDs
- Invalid command arguments
- Path outside repository

### System Error Tests

Internal failures that should be handled gracefully:

- Index corruption (malformed SQLite)
- Git command failures
- File permission errors
- Disk full conditions
- Concurrent modification

### Edge Case Tests

Boundary conditions and unusual states:

- Empty repository (no documents)
- Single document repository
- Maximum-size documents (500 lines exactly)
- Documents with no frontmatter
- Unicode in document names and content
- Deep directory nesting
- Circular link graphs

## Fake Git Implementation

### Interface

```rust
trait GitOperations {
    fn status(&self) -> Vec<FileStatus>;
    fn diff(&self, from: &str, to: &str) -> Vec<PathBuf>;
    fn ls_files(&self, pattern: &str) -> Vec<PathBuf>;
    fn rev_parse(&self, ref_name: &str) -> Option<String>;
    fn log(&self, path: Option<&Path>, limit: usize) -> Vec<Commit>;
}
```

### In-Memory State

```rust
struct FakeGit {
    files: HashMap<PathBuf, String>,      // Working tree
    staged: HashSet<PathBuf>,             // Staged files
    commits: Vec<FakeCommit>,             // Commit history
    head: String,                         // Current HEAD
    branches: HashMap<String, String>,    // Branch -> commit
}
```

### Commit Simulation

```rust
impl FakeGit {
    fn commit(&mut self, message: &str) -> String {
        let hash = generate_hash();
        let commit = FakeCommit {
            hash: hash.clone(),
            parent: Some(self.head.clone()),
            message: message.to_string(),
            files: self.staged.clone(),
        };
        self.commits.push(commit);
        self.staged.clear();
        self.head = hash.clone();
        hash
    }
}
```

## Test Helpers

### Document Builder

Fluent API for creating test documents:

```rust
let doc = DocBuilder::new()
    .id("LTEST1")
    .name("test-doc")
    .issue_type("bug")
    .status("open")
    .priority(1)
    .body("Test content")
    .build();
```

### Assertion Helpers

Custom assertions for common patterns:

```rust
fn assert_document_exists(env: &TestEnv, id: &str);
fn assert_link_exists(env: &TestEnv, from: &str, to: &str);
fn assert_error_contains(result: &Output, substring: &str);
fn assert_warning_contains(result: &Output, substring: &str);
```

### Snapshot Testing

For complex output verification:

```rust
fn test_show_output() {
    let env = setup_complex_docs();
    let result = env.run(&["show", "LMAIN"]);
    assert_snapshot!("show_with_context", result.stdout);
}
```

Snapshots are stored in `tests/snapshots/` and reviewed on change.

## Test Organization

### Directory Structure

```
tests/
├── cli/
│   ├── create_tests.rs
│   ├── update_tests.rs
│   ├── show_tests.rs
│   └── list_tests.rs
├── index/
│   ├── reconcile_tests.rs
│   └── query_tests.rs
├── fakes/
│   ├── fake_git.rs
│   ├── fake_filesystem.rs
│   └── fake_clock.rs
├── helpers/
│   ├── doc_builder.rs
│   ├── test_env.rs
│   └── assertions.rs
└── snapshots/
    └── *.snap
```

### Naming Convention

Test functions describe the scenario:

```rust
#[test]
fn create_bug_with_all_fields_succeeds() { }

#[test]
fn create_without_path_returns_error() { }

#[test]
fn update_closed_issue_to_in_progress_fails() { }
```

## Performance Requirements

### Speed Targets

- Individual test: < 50ms
- Full test suite: < 10 seconds
- No network access
- No disk I/O (except temp SQLite)

### Parallelization

Tests run in parallel by default. Each test uses isolated state.
Shared fixtures are immutable and thread-safe.

### Profiling

Slow tests are flagged in CI:

```
Warning: test_large_context_budget took 120ms (limit: 50ms)
```

## Continuous Integration

### Pre-Commit

Fast subset of tests (~2 seconds):
- Syntax validation
- Basic CRUD operations
- Error message format

### Full Suite

All tests on every PR:
- All categories
- Multiple platforms (Linux, macOS)
- Both debug and release builds

### Coverage

Coverage targets:
- Line coverage: > 80%
- Branch coverage: > 70%
- All error paths exercised

## Property-Based Testing

For complex logic, property tests supplement examples:

```rust
#[proptest]
fn any_valid_id_roundtrips(id: LatticeId) {
    let encoded = id.to_string();
    let decoded = LatticeId::parse(&encoded).unwrap();
    assert_eq!(id, decoded);
}

#[proptest]
fn context_budget_never_exceeded(budget: u32, docs: Vec<TestDoc>) {
    let result = show_with_context(&docs, budget);
    assert!(result.len() <= budget as usize);
}
```
