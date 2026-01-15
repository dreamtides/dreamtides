# Appendix: CLI Structure

This appendix provides a concise reference for all `lat` commands. See
[Lattice Design](lattice_design.md#command-overview) for conceptual overview.

For detailed specifications of workflow commands, see
[Appendix: Workflow](appendix_workflow.md).

## Global Options

All commands support:

- `--json`: Output in JSON format
- `--verbose`: Show detailed operation log
- `--quiet`: Suppress non-error output
- `--help`: Show command help
- `--version`: Show version information

## Workflow Commands

See [Appendix: Workflow](appendix_workflow.md) for detailed specifications
including output formats, JSON schemas, and behavioral details.

### lat show {id} [id...] [options]

Display document details. Options: `--short`, `--refs`, `--peek`, `--raw`.

### lat ready [options]

Find ready work. Options: `--parent {id}`, `--priority {n}`, `--type {type}`,
`--label {list}`, `--label-any {list}`, `--limit {n}`, `--include-backlog`,
`--include-claimed`, `--pretty`, `--sort {policy}` (hybrid/priority/oldest).

### lat prime [options]

Output AI workflow context. Options: `--full`, `--export`.

### lat overview [id] [options]

Show critical documents. Without arguments, shows repository-level overview ranked
by view frequency, recency, and filename priority. With an ID argument, shows
contextual documents relevant to that task. See
[Appendix: Overview Command](appendix_overview_command.md).

Options: `--limit {n}`, `--type {type}`, `--path {prefix}`, `--include-closed`,
`--reset-views`.

### lat claim {id} [options]

Mark task as locally in progress. Options: `--list`, `--release {id}`,
`--release-all`, `--release-worktree {path}`, `--gc`.

## Document Commands

### lat track {path} [options]

Add Lattice tracking to existing markdown file.

**Options:**
- `--name {name}`: Set document name
- `--description {desc}`: Set description
- `--force`: Regenerate ID even if document already has one (for resolving duplicates)

### lat generate-ids [options]

Pre-allocate IDs for authoring.

**Options:**
- `-n {n}`: Number of IDs (default 10)

### lat split {path} [options]

Split document by top-level sections.

**Options:**
- `--output-dir {dir}`: Directory for split files
- `--dry-run`: Preview without writing

### lat mv {id} {new-path}

Move document to a new location. Updates `parent-id` based on the new
directory, derives `name` from the new filename, and normalizes all links
pointing to the moved document.

**Options:**
- `--dry-run`: Preview without writing

## Task Commands

See [Appendix: Task Tracking](appendix_task_tracking.md) for task lifecycle,
status transitions, and template inheritance.

### lat create {path} "{description}" [options]

Create new document (task or knowledge base). The description is a required
positional argument. The `name` field is derived automatically from the
filename (underscores → hyphens).

To create a task, include `-t {type}`. Omitting `-t` creates a knowledge base
document.

Options: `-t, --type {type}`, `-p, --priority {n}`, `--body-file {path}`,
`-l, --labels {list}`, `--deps {spec}`.

Examples:
```bash
lat create docs/auth/oauth_design.md "OAuth 2.0 implementation design"
lat create tasks/auth/fix_login.md "Fix login after password reset" -t bug -p 1
```

The path specifies directory location and filename. For epics, use `README.md`
or `00_` prefix.

### lat update {id} [id...] [options]

Modify existing tasks.

**Options:**
- `--status {status}`: Change status
- `--priority {n}`: Change priority
- `--type {type}`: Change type
- `--add-labels {list}`: Add labels
- `--remove-labels {list}`: Remove labels

### lat edit {id} [options]

Open task in editor. Human-only.

**Options:**
- `--name`: Edit name only
- `--description`: Edit description
- `--body`: Edit full body

### lat close {id} [id...] [options]

Mark tasks as closed.

**Options:**
- `--reason {text}`: Closure reason

### lat reopen {id} [id...]

Change closed tasks to open.

## Query Commands

### lat list [options]

Search and filter documents.

**Filter Options:**
- `--status {status}`: Filter by status
- `--priority {n}`: Exact priority
- `--priority-min {n}`: Minimum priority
- `--priority-max {n}`: Maximum priority
- `--type {type}`: Filter by type
- `--label {list}`: Must have ALL labels
- `--label-any {list}`: Must have ANY label
- `--name-contains {text}`: Substring match
- `--path {prefix}`: Path prefix filter
- `--created-after {date}`: Created after date
- `--created-before {date}`: Created before date
- `--updated-after {date}`: Updated after date
- `--updated-before {date}`: Updated before date
- `--discovered-from {id}`: Tasks discovered from specified parent task
- `--roots-only`: List only directory root documents (files named `README.md` or
  with `00_` prefix)

**Output Options:**
- `--limit {n}`: Maximum results
- `--sort {field}`: Sort by priority/created/updated/name
- `--reverse`: Reverse sort order
- `--format rich|compact|oneline`: Output format

**Output Formats:**

Default format shows rich metadata with name and description:
```
LXXXXX [bug/P1/open] login-failure - Users cannot log in after password reset
LYYYYY [feature/P2/open] oauth-support - Add OAuth 2.0 authentication support
LZZZZZ [doc] authentication-design - OAuth 2.0 implementation design
```

Compact format (name only):
```
LXXXXX  login-failure
LYYYYY  oauth-support
```

### lat stale [options]

Find tasks not updated recently.

**Options:**
- `--days {n}`: Staleness threshold (default 30)
- Additional `lat list` options

### lat search {query} [options]

Keyword search across document content.

**Options:**
- `--limit {n}`: Maximum results
- `--path {prefix}`: Restrict to path
- `--type {type}`: Filter by type

### lat blocked [options]

Show tasks in blocked status with their blocking relationships.

**Options:**
- `--path {prefix}`: Filter to path prefix
- `--limit {n}`: Maximum results

### lat dep add {id} {depends-on-id}

Add dependency (first task depends on second). Updates `blocked-by` field.

### lat dep remove {id} {depends-on-id}

Remove dependency relationship.

### lat dep tree {id}

Display dependency tree with status indicators.

### lat changes [options]

Show documents changed since a point in time.

**Options:**
- `--since {date}`: Since date/time
- `--since {commit}`: Since git commit

### lat stats [options]

Display project statistics: document counts, open/closed/blocked by status,
priority and type breakdowns, recent activity, and health metrics.

Follows the flags and output format of `bd stats`.

**Options:**
- `--path {prefix}`: Restrict to path prefix
- `--period {days}`: Activity period (default 7)

## Relationship Commands

### lat links-from {id}

Show documents this document links to.

### lat links-to {id}

Show documents that link to this document.

### lat path {id1} {id2}

Find shortest path between documents.

### lat orphans [options]

Find documents with no incoming links.

**Options:**
- `--exclude-roots`: Don't report root documents
- `--path {prefix}`: Check only under path

### lat impact {id}

Analyze what would be affected by changes to document.

## Hierarchy Commands

### lat tree [path] [options]

Display directory structure with documents.

**Options:**
- `--depth {n}`: Maximum depth
- `--counts`: Show document counts
- `--tasks-only`: Only show task directories
- `--docs-only`: Only show documentation directories

### lat roots

List all root documents with child counts.

### lat children {root-id} [options]

List documents under a root's directory.

**Options:**
- `--recursive`: Include nested directories
- `--tasks`: Only tasks
- `--docs`: Only knowledge base documents

## Label Commands

### lat label add {id} [id...] {label}

Add label to documents.

### lat label remove {id} [id...] {label}

Remove label from documents.

### lat label list {id}

List labels on document.

### lat label list-all

List all labels with counts.

## Setup Commands

### lat setup claude [options]

Install Claude Code hooks and configuration. See
[Appendix: AI Integration](appendix_ai_integration.md) for hook details.

Options: `--check`, `--remove`, `--project`.

## Maintenance Commands

See [Appendix: Linter](appendix_linter.md) for validation rules and
[Appendix: Linking System](appendix_linking_system.md) for link format details.

### lat check [options]

Validate documents and repository. Options: `--path {prefix}`, `--errors-only`,
`--fix`, `--staged-only`, `--rebuild-index`.

### lat fmt [options]

Format documents and normalize links. Options: `--path {prefix}`, `--check`,
`--line-width {n}`.

Link normalization: adds Lattice ID fragments, expands bare ID links, updates
paths on rename/move.

### lat chaosmonkey [options]

Run fuzz testing. See [Appendix: Chaos Monkey](appendix_chaos_monkey.md).
Options: `--seed {n}`, `--max-ops {n}`, `--operations {list}`, `--stop-before-last`.

## Exit Codes

- 0: Success
- 1: System error (internal failure)
- 2: Validation error (document tasks)
- 3: User error (invalid arguments)
- 4: Not found (ID doesn't exist)

## Structured Error Output

With `--json`, all errors include structured information:

```json
{
  "error_code": "E002",
  "message": "Reference to nonexistent ID",
  "affected_documents": ["LXXXXX"],
  "location": {"path": "docs/example.md", "line": 42},
  "suggestion": "Create the target document or correct the ID",
  "fix_command": "lat create docs/target.md"
}
```

Fields vary by error type. The `fix_command` field is present when an
automated fix is available.

## Environment Variables

- `LATTICE_LOG_LEVEL`: error/warn/info/debug/trace
- `LATTICE_NO_COLOR`: Disable colored output
- `EDITOR`: Editor for `lat edit`

## Shell Completions

Generate shell completion scripts:

```bash
lat completion bash > ~/.local/share/bash-completion/completions/lat
lat completion zsh > ~/.zfunc/_lat
lat completion fish > ~/.config/fish/completions/lat.fish
```

Completions include command names, flags, and dynamic completion for Lattice
IDs (queried from index).
