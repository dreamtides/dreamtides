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

Find ready work (tasks not in `.closed/`, with all blockers closed). Options:
`--parent {id}`, `--priority {n}`, `--type {type}`, `--label {list}`,
`--label-any {list}`, `--limit {n}`, `--include-backlog`, `--include-claimed`,
`--pretty`, `--sort {policy}` (hybrid/priority/oldest).

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

### lat track {path} "{description}" [options]

Add Lattice tracking to existing markdown file by generating a new Lattice ID
and adding frontmatter with the required fields.

**Arguments:**
- `{path}`: Path to the markdown file to track
- `{description}`: Human-readable description (required)

**Behavior:**
- Derives `name` from the filename (underscores → hyphens, lowercase)
- Generates a new unique Lattice ID
- Adds frontmatter with `lattice-id`, `name`, `description`, and timestamps
- If the file already has frontmatter with a Lattice ID, requires `--force`

**Options:**
- `--force`: Regenerate ID even if document already has one (for resolving duplicates)

**Example:**
```bash
lat track docs/api_design.md "API design document for the authentication service"
```

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
state transitions, and template inheritance.

### lat create {parent} "{description}" [options]

Create new document with convention-based placement and auto-generated filename.

**Auto-placement:** The `-t` flag determines the subdirectory:
- With `-t {type}`: creates `{parent}/tasks/{filename}.md`
- Without `-t`: creates `{parent}/docs/{filename}.md`

**Auto-naming:** Filename is generated from description (lowercase, underscores,
significant words, ~40 char max). Numeric suffix on collision.

Options: `-t, --type {type}`, `-p, --priority {n}`, `--body-file {path}`,
`-l, --labels {list}`, `--deps {spec}`.

Examples:
```bash
# Task - auto-placed in auth/tasks/, filename from description
lat create auth/ "Fix login after password reset" -t bug -p 1
# → auth/tasks/fix_login_after_password_reset.md

# Knowledge base - auto-placed in auth/docs/
lat create auth/ "OAuth 2.0 implementation design"
# → auth/docs/oauth_implementation_design.md

# Root document - explicit path required (typically NOT a task)
lat create auth/auth.md "Authentication system"

# Explicit filename when you want control
lat create auth/tasks/oauth_bug.md "Fix OAuth token validation" -t bug
```

### lat update {id} [id...] [options]

Modify existing tasks. To change task state, use `lat close` or `lat reopen`.

**Options:**
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

Close tasks by moving them to `.closed/` subdirectory. Updates all links to
point to the new path (like `lat mv`). Sets `closed-at` timestamp.

**Options:**
- `--reason {text}`: Closure reason (appended to document body)
- `--dry-run`: Preview without moving

### lat reopen {id} [id...]

Reopen closed tasks by moving them from `.closed/` back to their original
parent directory. Updates all links to the restored path.

**Options:**
- `--dry-run`: Preview without moving

### lat prune {path} [options]
### lat prune --all [options]

Permanently delete closed tasks. Requires either a path argument or `--all`.

**Behavior:**
- Removes tasks from `.closed/` directories
- Removes references from `blocking`, `blocked-by`, `discovered-from` fields
- Inline markdown links to pruned tasks produce an error unless `--force`

**Options:**
- `--all`: Prune all closed tasks in the repository (required if no path given)
- `--force`: Convert inline links to plain text instead of erroring
- `--dry-run`: Preview without deleting

## Query Commands

### lat list [options]

Search and filter documents. By default excludes closed tasks.

**Filter Options:**
- `--state {state}`: Filter by state (open/blocked/closed)
- `--include-closed`: Include tasks in `.closed/` directories
- `--closed-only`: Show only closed tasks
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
- `--roots-only`: List only root documents (files whose name matches their
  containing directory, e.g., `api/api.md`)

**Output Options:**
- `--limit {n}`: Maximum results
- `--sort {field}`: Sort by priority/created/updated/name
- `--reverse`: Reverse sort order
- `--format rich|compact|oneline`: Output format

**Output Formats:**

Default format shows rich metadata with name and description:
```
LXXXXX [bug/P1] login-failure - Users cannot log in after password reset
LYYYYY [feature/P2] oauth-support - Add OAuth 2.0 authentication support
LZZZZZ [doc] authentication-design - OAuth 2.0 implementation design
```

With `--include-closed`, closed tasks show their state:
```
LAAAAA [task/P2/closed] old-feature - Previously completed feature
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

Show tasks with unresolved blockers (open tasks in their `blocked-by` field).

**Options:**
- `--path {prefix}`: Filter to path prefix
- `--limit {n}`: Maximum results
- `--show-blockers`: Display the blocking tasks for each result

### lat dep add {id} {depends-on-id}

Add dependency (first task depends on second). Updates `blocked-by` field of the
first task to include the second task's ID. Also updates the `blocking` field of
the second task to include the first task's ID, maintaining bidirectional
consistency.

### lat dep remove {id} {depends-on-id} [options]

Remove dependency relationship. Removes the second task from the first task's
`blocked-by` field, and removes the first task from the second task's `blocking`
field. If removing the last blocker from a blocked task, that task becomes ready.

**Options:**
- `--json`: Output as structured JSON with source_id, target_id, and became_ready
  fields

### lat dep tree {id} [options]

Display dependency tree with state indicators (open/blocked/closed). Shows both
upstream dependencies (what this task depends on) and downstream dependents
(what depends on this task).

**Options:**
- `--json`: Output as structured JSON

### lat changes [options]

Show documents changed since a point in time.

**Options:**
- `--since {date}`: Since date/time
- `--since {commit}`: Since git commit

### lat stats [options]

Display project statistics: document counts by state (open/blocked/closed),
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

## Maintenance Commands

See [Appendix: Linter](appendix_linter.md) for validation rules and
[Appendix: Linking System](appendix_linking_system.md) for link format details.

### lat check [options]

Validate documents and repository. Options: `--path {prefix}`, `--errors-only`,
`--fix`, `--staged-only`, `--rebuild-index`.

### lat fmt [options]

Format documents and normalize links.

**Options:**
- `--path {prefix}`: Only format files under this path
- `--check`: Check formatting without modifying files (exit code 1 if changes needed)
- `--line-width {n}`: Override text wrap column (default: 80)

Link normalization: adds Lattice ID fragments, expands bare ID links, updates
paths on rename/move.

### lat doctor [options]

Diagnose system health issues distinct from document validation (`lat check`).
While `lat check` validates document content (frontmatter, links, structure),
`lat doctor` validates the Lattice installation and infrastructure.

See [Appendix: Doctor Command](appendix_doctor.md) for check categories, output
format, and fixable issues.

**Options:**
- `--fix`: Automatically repair issues where possible
- `--dry-run`: Preview fixes without applying (requires `--fix`)
- `--deep`: Run additional integrity checks (slower)
- `--json`: Machine-readable output
- `--quiet`: Only show warnings and errors

### lat chaosmonkey [options]

Run fuzz testing. See [Appendix: Chaos Monkey](appendix_chaos_monkey.md).
Options: `--seed {n}`, `--max-ops {n}`, `--operations {list}`, `--stop-before-last`.

## Integration Commands

### lat setup claude [options]

Configure Lattice integration with Claude Code via MCP. See
[Appendix: AI Integration](appendix_ai_integration.md#mcp-server) for full
MCP server documentation.

**Default behavior (no flags):**
- Installs Lattice MCP server in Claude Code configuration
- Detects project root and configures server with project path
- Creates `.claude/settings.json` if needed

**Options:**
- `--check`: Verify installation without modifying configuration
- `--remove`: Remove Lattice from Claude Code configuration
- `--global`: Modify `~/.claude/settings.json` instead of project-local config
- `--dry-run`: Preview changes without writing (with `--check` or default install)

**Exit codes:**
- 0: Success (install/remove completed, or `--check` passed)
- 1: System error
- 2: Configuration error (with `--check`: installation invalid or missing)

### lat mcp

Execute a single MCP tool invocation. This command is invoked automatically by
Claude Code and is not intended for direct user invocation.

**Behavior:**
- Reads a JSON-RPC request from stdin
- Executes the requested tool (`lattice_create_task` or `lattice_create_document`)
- Writes a JSON-RPC response to stdout
- Exits immediately after processing

**Options:**
- `--project {path}`: Override project root (default: current directory)

**Example invocation (for testing):**
```bash
echo '{"method":"lattice_create_document","params":{...}}' | lat mcp
```

## Exit Codes

- 0: Success
- 1: System error (internal failure)
- 2: Validation error (invalid document content)
- 3: User error (invalid arguments)
- 4: Not found (ID doesn't exist)

## Structured Error Output

With `--json`, all errors include structured information:

```json
{
  "error_code": "E002",
  "message": "Reference to nonexistent ID",
  "affected_documents": ["LXXXXX"],
  "location": {"path": "api/docs/example.md", "line": 42},
  "suggestion": "Create the target document or correct the ID",
  "fix_command": "lat create api/ \"Target document description\""
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
