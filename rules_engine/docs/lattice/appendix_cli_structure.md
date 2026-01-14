# Appendix: CLI Structure

## Command Overview

All commands support `--json` for structured output and `--verbose` for
detailed logging. The `lat` binary is the single entry point.

## Global Options

All commands support:

- `--json`: Output in JSON format
- `--ai`: AI-optimized output (minimal decoration, rich metadata)
- `--verbose`: Show detailed operation log
- `--quiet`: Suppress non-error output
- `--help`: Show command help
- `--version`: Show version information

## Document Commands

### lat show \<id\> [options]

Display document with context. Primary viewing interface.

See [Appendix: Context Retrieval](appendix_context_retrieval.md) for full
details on context options.

**Context Options:**
- `--context N`: Character budget for context (default 5000, 0 to disable)
- `--references N`: Character budget for references (default 500)
- `--no-context`: Equivalent to `--context 0 --references 0`
- `--max-context`: Use large budget (100000 chars)
- `--brief`: Task briefing mode (30000 char budget, comprehensive)

**Loading Options:**
- `--peek`: Show only YAML frontmatter
- `--raw`: Output without formatting

**Intent Options:**
- `--intent=implement`: Design docs, API refs, examples
- `--intent=bug-fix`: Related bugs, error docs, tests
- `--intent=review`: Change history, test coverage
- `--intent=understand`: Overview, glossary, architecture
- `--intent=document`: Style guides, examples

### lat track \<path\> [options]

Add Lattice tracking to existing markdown file.

**Options:**
- `--name <name>`: Set document name
- `--description <desc>`: Set description

### lat generate-ids [options]

Pre-allocate IDs for authoring.

**Options:**
- `-n N`: Number of IDs (default 10)

### lat split \<path\> [options]

Split document by top-level sections.

**Options:**
- `--output-dir <dir>`: Directory for split files
- `--dry-run`: Preview without writing

## Issue Commands

### lat create \<name\> [options]

Create new issue document.

**Required:**
- `--path <dir>`: Directory for issue file

**Options:**
- `-t, --type <type>`: bug/feature/task/epic/chore (default: task)
- `-p, --priority <N>`: 0-4, 0 highest (default: 2)
- `-d, --description <text>`: Issue description
- `--body-file <path>`: Read description from file
- `-l, --labels <list>`: Comma-separated labels
- `--deps <spec>`: Dependency (e.g., `discovered-from:LK1DT`)

### lat update \<id\> [id...] [options]

Modify existing issues.

**Options:**
- `--status <status>`: Change status
- `--priority <N>`: Change priority
- `--type <type>`: Change type
- `--add-labels <list>`: Add labels
- `--remove-labels <list>`: Remove labels
- `--progress <N>`: Set progress percentage (0-100)

### lat edit \<id\> [options]

Open issue in editor. Human-only.

**Options:**
- `--name`: Edit name only
- `--description`: Edit description
- `--body`: Edit full body
- `--acceptance`: Edit acceptance criteria
- `--design-notes`: Edit design notes

### lat close \<id\> [id...] [options]

Mark issues as closed.

**Options:**
- `--reason <text>`: Closure reason

### lat reopen \<id\> [id...]

Change closed issues to open.

## Query Commands

### lat list [options]

Search and filter documents.

**Filter Options:**
- `--status <status>`: Filter by status
- `--priority <N>`: Exact priority
- `--priority-min <N>`: Minimum priority
- `--priority-max <N>`: Maximum priority
- `--type <type>`: Filter by type
- `--label <list>`: Must have ALL labels
- `--label-any <list>`: Must have ANY label
- `--name-contains <text>`: Substring match
- `--path <prefix>`: Path prefix filter
- `--created-after <date>`: Created after date
- `--created-before <date>`: Created before date
- `--updated-after <date>`: Updated after date
- `--updated-before <date>`: Updated before date
- `--has-acceptance`: Has acceptance criteria
- `--progress-below <N>`: Progress under N%

**Output Options:**
- `--limit N`: Maximum results
- `--sort <field>`: Sort by priority/created/updated/name
- `--reverse`: Reverse sort order
- `--format rich|compact|oneline`: Output format

**Output Formats:**

Default format shows rich metadata inline:
```
LXXXX [bug/P1/open] login-authentication-failure
LYYYY [feature/P2/in_progress] oauth-implementation
LZZZZ [doc] authentication-design
```

Compact format:
```
LXXXX  login-authentication-failure
LYYYY  oauth-implementation
```

### lat ready [options]

Find ready work (open status, no blockers).

**Options:** Same as `lat list`.

### lat stale [options]

Find issues not updated recently.

**Options:**
- `--days N`: Staleness threshold (default 30)
- Additional `lat list` options

### lat search \<query\> [options]

Semantic and keyword search.

See [Appendix: Semantic Search](appendix_semantic_search.md) for details.

**Options:**
- `--mode keyword|semantic|hybrid`: Search mode (default: hybrid)
- `--limit N`: Maximum results
- `--threshold F`: Minimum similarity (0.0-1.0)
- `--path <prefix>`: Restrict to path
- `--type <type>`: Filter by type

### lat similar \<id\> [options]

Find semantically similar documents.

**Options:**
- `--limit N`: Maximum results (default 10)
- `--threshold F`: Minimum similarity

### lat dep tree \<id\>

Display dependency tree with status indicators.

### lat changes [options]

Show documents changed since a point in time.

**Options:**
- `--since <date>`: Since date/time
- `--since <commit>`: Since git commit
- (No option): Since last session

## Relationship Commands

### lat links-from \<id\>

Show documents this document links to.

### lat links-to \<id\>

Show documents that link to this document.

### lat path \<id1\> \<id2\>

Find shortest path between documents.

### lat orphans [options]

Find documents with no incoming links.

**Options:**
- `--exclude-roots`: Don't report root documents
- `--path <prefix>`: Check only under path

### lat impact \<id\>

Analyze what would be affected by changes to document.

## Hierarchy Commands

### lat tree [path] [options]

Display directory structure with documents.

**Options:**
- `--depth N`: Maximum depth
- `--counts`: Show document counts
- `--issues-only`: Only show issue directories
- `--docs-only`: Only show documentation directories

### lat roots

List all root documents with child counts.

### lat children \<root-id\> [options]

List documents under a root's directory.

**Options:**
- `--recursive`: Include nested directories
- `--issues`: Only issues
- `--docs`: Only knowledge base documents

### lat stats [options]

Repository-wide statistics.

**Options:**
- `links`: Detailed link statistics
- `issues`: Issue statistics by status/priority

## Session Commands

### lat prime [options]

Output session-start context. Called by hooks.

See [Appendix: Session Management](appendix_session_management.md).

**Options:**
- `--format text|json|minimal`: Output format
- `--stealth`: Skip sync operations
- `--no-ready`: Omit ready work list
- `--no-recent`: Omit recent changes

### lat session start \<id\>

Set current working issue.

### lat session end

Clear current working issue.

### lat session status

Show session state.

### lat session note \<text\>

Add working note to current session.

### lat session notes

List session notes.

### lat session clear

Reset all session state.

## Setup Commands

### lat setup claude [options]

Install Claude Code hooks.

**Options:**
- `--check`: Check installation status
- `--remove`: Remove hooks
- `--project`: Install for project only
- `--stealth`: Use stealth mode for hooks

### lat setup cursor

Install Cursor IDE rules.

### lat setup aider

Install Aider configuration.

## Label Commands

### lat label add \<id\> [id...] \<label\>

Add label to documents.

### lat label remove \<id\> [id...] \<label\>

Remove label from documents.

### lat label list \<id\>

List labels on document.

### lat label list-all

List all labels with counts.

## Maintenance Commands

### lat check [options]

Validate documents and repository.

**Options:**
- `--path <prefix>`: Check under path
- `--errors-only`: Suppress warnings
- `--fix`: Auto-fix where possible
- `--staged-only`: Check staged files only
- `--rebuild-index`: Force index rebuild

### lat fmt [options]

Format documents.

**Options:**
- `--path <prefix>`: Format under path
- `--check`: Report without modifying
- `--line-width N`: Wrap width (default 80)

### lat chaosmonkey [options]

Run fuzz testing.

**Options:**
- `--seed N`: Random seed
- `--max-ops N`: Maximum operations
- `--operations <list>`: Restrict operation types
- `--stop-before-last`: Stop before failing operation

## Exit Codes

- 0: Success
- 1: System error (internal failure)
- 2: Validation error (document issues)
- 3: User error (invalid arguments)
- 4: Not found (ID doesn't exist)

## Environment Variables

- `LATTICE_LOG_LEVEL`: error/warn/info/debug/trace
- `LATTICE_NO_COLOR`: Disable colored output
- `LATTICE_DEFAULT_CONTEXT`: Default context budget
- `EDITOR`: Editor for `lat edit`
