# Appendix: AI Integration

This appendix documents AI agent integration. See
[Lattice Design](lattice_design.md#skill-integration) for Lattice's general
approach to AI compatibility and [Appendix: Workflow](appendix_workflow.md) for
command specifications.

## MCP Tools

Lattice provides Model Context Protocol (MCP) tools for direct integration with
AI coding agents. These tools enable document creation with structured input,
avoiding the shell escaping issues that arise when passing multi-line markdown
via CLI.

### Installation

```bash
lat setup claude              # Install MCP in Claude Code
lat setup claude --check      # Verify installation
lat setup claude --remove     # Remove Claude integration
```

The `lat setup claude` command modifies the Claude Code MCP configuration
(typically `~/.claude/settings.json` or project-local `.claude/settings.json`)
to register Lattice as an MCP tool provider.

**Installation behavior:**
- Detects project root via git repository
- Adds `lattice` entry to the `mcpServers` configuration
- Configures the command to run `lat mcp` with the project path
- Creates any missing directories

**Check behavior (`--check`):**
- Verifies MCP configuration exists and is valid JSON
- Confirms `lattice` entry is present
- Validates the command points to the correct `lat` binary
- Reports version mismatch if `lat` was updated since installation

**Removal behavior (`--remove`):**
- Removes the `lattice` entry from MCP configuration
- Preserves other MCP servers in the configuration
- Exits cleanly if Lattice was not installed

### MCP Protocol

Lattice implements the standard MCP (Model Context Protocol) server protocol.
When invoked, `lat mcp` runs as a continuous server process:

```
Claude Code → spawns `lat mcp` → server loop: reads JSON-RPC requests from stdin
                                            → executes operation
                                            → writes JSON-RPC response to stdout
                                → exits when stdin closes
```

**Supported MCP methods:**
- `initialize` - Protocol handshake, returns server capabilities
- `tools/list` - Returns available tools and their schemas
- `tools/call` - Executes a tool with the provided arguments

The server uses newline-delimited JSON-RPC 2.0 over stdin/stdout. It is invoked
automatically by Claude Code and not intended for direct user invocation.

### Tool Design Philosophy

Lattice exposes **two separate tools** for document creation rather than a
single unified tool. This design choice prioritizes AI usability:

1. **Clear intent**: Each tool has a single, unambiguous purpose
2. **Distinct parameters**: Tasks and KB documents have different required
   fields (tasks need `task_type`, KB documents need explicit `name`)
3. **Different conventions**: Task filenames include the Lattice ID suffix;
   KB document filenames do not
4. **Better validation**: Each tool validates only the parameters relevant to
   its document type
5. **Reduced cognitive load**: AI agents see two focused tools rather than one
   tool with complex conditional behavior

### Tool: lattice_create_task

Creates a task document with auto-generated filename.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `directory` | string | Yes | Parent directory path (e.g., `"api/"`) |
| `description` | string | Yes | Human-readable task title |
| `task_type` | string | Yes | One of: `bug`, `feature`, `task`, `chore` |
| `body` | string | Yes | Markdown body content |
| `priority` | integer | No | 0-4 (default: 2) |
| `labels` | string[] | No | List of labels |
| `blocked_by` | string[] | No | List of blocking task IDs |
| `discovered_from` | string[] | No | List of parent task IDs for provenance |

**Behavior:**

1. Generates filename from description (lowercase, underscores, significant
   words, ~40 char max)
2. Appends Lattice ID to filename: `fix_login_bug_LCGWQN.md`
3. Places file in `{directory}/tasks/`
4. Auto-populates `parent-id` from directory's root document
5. Sets `created-at` to current timestamp
6. Derives `name` field from filename (underscores → hyphens)

**Return value:**

```json
{
  "lattice_id": "LCGWQN",
  "path": "api/tasks/fix_login_bug_LCGWQN.md",
  "name": "fix-login-bug-lcgwqn"
}
```

**Example:**

```json
{
  "directory": "auth/",
  "description": "Fix login after password reset",
  "task_type": "bug",
  "body": "Users receive 401 errors when logging in after password reset.\n\n## Steps to reproduce\n\n1. Request password reset\n2. Complete reset flow\n3. Attempt login with new password",
  "priority": 1,
  "labels": ["auth", "urgent"]
}
```

Creates: `auth/tasks/fix_login_after_password_reset_LXXXXX.md`

### Tool: lattice_create_document

Creates a knowledge base document with explicit filename.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `directory` | string | Yes | Parent directory path (e.g., `"api/"`) |
| `name` | string | Yes | Document name (becomes filename, max 64 chars) |
| `description` | string | Yes | Human-readable description (max 1024 chars) |
| `body` | string | Yes | Markdown body content |
| `labels` | string[] | No | List of labels |

**Behavior:**

1. Uses provided `name` for filename: `{name}.md`
2. Places file in `{directory}/docs/`
3. Auto-populates `parent-id` from directory's root document
4. Sets `created-at` to current timestamp
5. Lattice ID is assigned but NOT included in filename

**Return value:**

```json
{
  "lattice_id": "LDTWQN",
  "path": "api/docs/authentication_design.md",
  "name": "authentication-design"
}
```

**Example:**

```json
{
  "directory": "auth/",
  "name": "oauth_implementation",
  "description": "OAuth 2.0 implementation design and security considerations",
  "body": "# OAuth 2.0 Implementation\n\nThis document describes our OAuth 2.0 integration...\n\n## Supported Flows\n\n- Authorization Code with PKCE\n- Client Credentials"
}
```

Creates: `auth/docs/oauth_implementation.md`

### Error Handling

MCP tools return structured errors following the MCP protocol:

```json
{
  "error": {
    "code": "INVALID_DIRECTORY",
    "message": "Directory 'nonexistent/' does not exist",
    "suggestion": "Create the directory or use an existing path"
  }
}
```

Common error codes:

| Code | Description |
|------|-------------|
| `INVALID_DIRECTORY` | Specified directory does not exist |
| `INVALID_TASK_TYPE` | Task type not one of: bug, feature, task, chore |
| `INVALID_PRIORITY` | Priority not in range 0-4 |
| `NAME_TOO_LONG` | Name exceeds 64 characters |
| `DESCRIPTION_TOO_LONG` | Description exceeds 1024 characters |
| `FILE_EXISTS` | A file with the generated name already exists |
| `MISSING_ROOT` | Directory has no root document for parent-id |
| `ID_COLLISION` | Generated ID collides (extremely rare) |

### Why Lattice ID in Task Filenames?

Task filenames include the Lattice ID suffix (e.g., `fix_bug_LXXXXX.md`) for
several reasons:

1. **Guaranteed uniqueness**: Multiple tasks with similar descriptions won't
   collide
2. **Stable references**: The filename contains the ID, so file renames don't
   break mental associations
3. **Quick identification**: Humans and AI can identify the task from the
   filename alone
4. **Search-friendly**: Searching for an ID finds both the file and references

Knowledge base documents don't include the ID because they typically have
carefully chosen, stable names that serve as their primary identifier.

### Future MCP Tools

The stateless architecture supports adding additional tools. Planned tools
include:

- `lattice_close_task`: Close one or more tasks
- `lattice_show`: Retrieve document details
- `lattice_list`: Query documents with filters
- `lattice_update`: Modify task metadata
- `lattice_add_dependency`: Add blocking/blocked-by relationships

## Skill Documents

Documents with `skill: true` in frontmatter become Claude Skills via automatic
symlink generation. See [Appendix: Startup Operations](appendix_startup_operations.md)
for sync timing.

**Requirements for skill documents:**

- `name`: Max 64 characters, no "anthropic" or "claude" substrings
- `description`: Max 1024 characters, non-empty
- No XML-like tags in name

**Symlink location:** `.claude/skills/<name>.md` → actual document path

## Recommended AI Workflows

**Session start:**
```bash
lat prime             # Workflow reminders
lat overview          # Get critical document context
lat ready             # See available work
```

**Working on a task:**
```bash
lat show LXXXXX       # Full context with templates and dependencies
lat claim LXXXXX      # Mark as in-progress locally
# ... do work ...
lat close LXXXXX      # Mark complete (auto-releases claim)
```

**Before committing:**
```bash
lat check             # Validate all documents
lat fmt               # Normalize links
```

**Context recovery after compaction:**
```bash
lat prime             # Workflow reminders
lat overview          # Most-viewed documents
```

## MCP vs CLI Workflows

AI agents can use either MCP tools or CLI commands. General guidance:

| Operation | Preferred Method | Reason |
|-----------|------------------|--------|
| Create document | MCP tool | Structured input, no shell escaping |
| Close task | CLI (`lat close`) | Simple, single argument |
| Query documents | CLI (`lat list`, `lat show`) | Flexible output formats |
| Validate | CLI (`lat check`) | Human-readable output |

MCP tools are particularly valuable when the agent needs to pass multi-line
content (like document bodies) that would require complex shell escaping via
CLI.