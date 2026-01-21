---
lattice-id: LCRWQN
name: test-scenario-mcp-tool-integration
description: 'Test Scenario: MCP Tool Integration'
parent-id: LCEWQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
created-at: 2026-01-20T06:17:15.352136Z
updated-at: 2026-01-21T22:32:22.891003Z
---

# Test Scenario: MCP Tool Integration

See [Agent Manual Testing Guide](../../../docs/agent_manual_testing.md#LCBWQN)
for
general testing instructions.

## Objective

Verify that MCP tools (`lattice_create_task`, `lattice_create_document`) work
correctly via the `lat mcp` command, and that `lat setup claude` configures
integration properly.

## Important Note

This test involves MCP (Model Context Protocol) server operations. The `lat mcp`
command reads JSON-RPC requests from stdin and writes responses to stdout.

## Setup

```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"
mkdir -p .lattice project/tasks project/docs

lat create project/project.md "Test Project"
ROOT_ID=$(grep "lattice-id:" project/project.md | cut -d' ' -f2)

git add .
git commit -m "Initial setup"
```

## Test Sequence

### Part 1: MCP Server Protocol

**Step 1.1**: Test tools/list method.

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | lat mcp --project "$TEST_DIR"
```

**Verify**:

- Valid JSON-RPC response
- Contains `lattice_create_task` tool
- Contains `lattice_create_document` tool
- Each tool has name, description, inputSchema

### Part 2: lattice_create_task Tool

**Step 2.1**: Create task via MCP.

```bash
cat << 'EOF' | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_task",
    "arguments": {
      "directory": "project/",
      "description": "Fix critical bug in authentication",
      "task_type": "bug",
      "body": "Users are unable to log in after password reset.\n\n## Steps to Reproduce\n\n1. Request password reset\n2. Complete reset flow\n3. Try to log in",
      "priority": 1,
      "labels": ["auth", "urgent"]
    }
  }
}
EOF
```

**Verify response**:

- `result` contains `lattice_id`, `path`, `name`
- `path` is under `project/tasks/`
- Filename includes lattice ID suffix

**Verify file**:
```bash
ls project/tasks/
cat project/tasks/*.md | head -20
```

**Verify**:

- File created with correct content
- Frontmatter has all fields
- Labels included
- Priority set to 1
- task-type is bug

**Step 2.2**: Create task with all fields.

```bash
cat << 'EOF' | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_task",
    "arguments": {
      "directory": "project/",
      "description": "Add new feature with dependencies",
      "task_type": "feature",
      "body": "Implement OAuth 2.0 support.",
      "priority": 2,
      "labels": ["oauth", "feature"],
      "blocked_by": [],
      "discovered_from": []
    }
  }
}
EOF
```

**Verify**:

- Task created successfully
- All fields populated

**Step 2.3**: Create task with dependencies.

```bash
# Get ID of first task
FIRST_TASK_ID=$(ls project/tasks/*.md | head -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

cat << EOF | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_task",
    "arguments": {
      "directory": "project/",
      "description": "Dependent task",
      "task_type": "task",
      "body": "This depends on another task.",
      "blocked_by": ["$FIRST_TASK_ID"]
    }
  }
}
EOF
```

**Verify**:

- Task created with blocked-by field set
- First task has blocking field updated

### Part 3: lattice_create_document Tool

**Step 3.1**: Create knowledge base document.

```bash
cat << 'EOF' | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_document",
    "arguments": {
      "directory": "project/",
      "name": "authentication_design",
      "description": "Design document for the authentication system",
      "body": "# Authentication Design\n\nThis document describes our authentication architecture.\n\n## Overview\n\nWe use OAuth 2.0 with PKCE flow."
    }
  }
}
EOF
```

**Verify response**:

- `result` contains `lattice_id`, `path`, `name`
- `path` is `project/docs/authentication_design.md`
- Filename does NOT include lattice ID

**Verify file**:
```bash
cat project/docs/authentication_design.md
```

**Verify**:

- File created at correct path
- name field is "authentication-design" (hyphens)
- No task-type or priority fields
- Body content preserved

**Step 3.2**: Create document with labels.

```bash
cat << 'EOF' | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_document",
    "arguments": {
      "directory": "project/",
      "name": "security_guidelines",
      "description": "Security best practices and guidelines",
      "body": "# Security Guidelines\n\nFollow these practices.",
      "labels": ["security", "documentation"]
    }
  }
}
EOF
```

**Verify**:

- Labels included in frontmatter

### Part 4: Error Handling

**Step 4.1**: Invalid directory.

```bash
cat << 'EOF' | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_task",
    "arguments": {
      "directory": "nonexistent/",
      "description": "Task in bad directory",
      "task_type": "task",
      "body": "Content"
    }
  }
}
EOF
```

**Verify**:

- Error response with INVALID_DIRECTORY code
- Clear error message

**Step 4.2**: Invalid task type.

```bash
cat << 'EOF' | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_task",
    "arguments": {
      "directory": "project/",
      "description": "Task with bad type",
      "task_type": "invalid_type",
      "body": "Content"
    }
  }
}
EOF
```

**Verify**:

- Error response with INVALID_TASK_TYPE code

**Step 4.3**: Invalid priority.

```bash
cat << 'EOF' | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_task",
    "arguments": {
      "directory": "project/",
      "description": "Task with bad priority",
      "task_type": "task",
      "body": "Content",
      "priority": 99
    }
  }
}
EOF
```

**Verify**:

- Error response with INVALID_PRIORITY code

**Step 4.4**: Name too long.

```bash
LONG_NAME=$(printf 'a%.0s' {1..100})
cat << EOF | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_document",
    "arguments": {
      "directory": "project/",
      "name": "$LONG_NAME",
      "description": "Doc with long name",
      "body": "Content"
    }
  }
}
EOF
```

**Verify**:

- Error response with NAME_TOO_LONG code

### Part 5: Setup Claude Command

**Step 5.1**: Check installation status.

```bash
lat setup claude --check
```

**Verify**:

- Reports whether Lattice is installed
- Exit code 0 if installed, 2 if not

**Step 5.2**: Dry run installation.

```bash
lat setup claude --dry-run
```

**Verify**:

- Shows what would be configured
- Does NOT modify any files

**Step 5.3**: Project-local installation.

```bash
lat setup claude --project
```

**Verify**:

- Creates or modifies `.mcp.json` in project root
- Contains lattice entry

```bash
cat .mcp.json
```

**Verify**:

- Valid JSON
- Has `mcpServers.lattice` entry
- Command points to `lat mcp`

**Step 5.4**: Verify installation.

```bash
lat setup claude --check
```

**Verify**:

- Reports installation valid
- Exit code 0

**Step 5.5**: Remove installation.

```bash
lat setup claude --remove
```

**Verify**:

- Lattice entry removed from config
- Other MCP servers preserved (if any)

### Part 6: MCP with Parent-ID

**Step 6.1**: Create task and verify parent.

```bash
cat << 'EOF' | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 11,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_task",
    "arguments": {
      "directory": "project/",
      "description": "Task with parent verification",
      "task_type": "task",
      "body": "Check parent-id is set"
    }
  }
}
EOF
```

**Verify created file**:
```bash
cat project/tasks/*parent*.md 2>/dev/null || cat project/tasks/*.md | tail -1
```

**Verify**:

- parent-id field references the project root document

### Part 7: Filename Generation

**Step 7.1**: Task filename includes ID.

```bash
ls project/tasks/
```

**Verify**:

- Task filenames include lattice ID suffix (e.g., `fix_bug_LXXXXX.md`)

**Step 7.2**: Document filename does not include ID.

```bash
ls project/docs/
```

**Verify**:

- Document filenames are just the name (e.g., `authentication_design.md`)
- No lattice ID in filename

### Part 8: Special Characters in Body

**Step 8.1**: Create task with special content.

```bash
cat << 'EOF' | lat mcp --project "$TEST_DIR"
{
  "jsonrpc": "2.0",
  "id": 12,
  "method": "tools/call",
  "params": {
    "name": "lattice_create_task",
    "arguments": {
      "directory": "project/",
      "description": "Task with special characters",
      "task_type": "task",
      "body": "Code example:\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n\nJSON: {\"key\": \"value\"}\n\nQuotes: \"double\" and 'single'\n\nBackslashes: C:\\Users\\test"
    }
  }
}
EOF
```

**Verify file content**:
```bash
grep -A 10 "Code example" project/tasks/*special*.md
```

**Verify**:

- All special characters preserved
- Code blocks intact
- JSON not mangled
- Backslashes preserved

### Part 9: Concurrent Operations

**Step 9.1**: Create multiple tasks quickly.

```bash
for i in 1 2 3 4 5; do
  cat << EOF | lat mcp --project "$TEST_DIR" &
{
  "jsonrpc": "2.0",
  "id": $((100 + i)),
  "method": "tools/call",
  "params": {
    "name": "lattice_create_task",
    "arguments": {
      "directory": "project/",
      "description": "Concurrent task $i",
      "task_type": "task",
      "body": "Created concurrently"
    }
  }
}
EOF
done
wait
```

**Verify**:

- All 5 tasks created
- No ID collisions
- No file conflicts

```bash
ls project/tasks/ | wc -l
```

## Cleanup

```bash
rm -f .mcp.json
cd /
rm -rf "$TEST_DIR"
```

## Expected Issues to Report

1. MCP server not responding to JSON-RPC
2. tools/list not returning correct schema
3. Task not created in correct location
4. Document filename including ID (should not)
5. Task filename missing ID (should have)
6. Parent-id not set automatically
7. Labels not saved
8. Dependencies not established
9. Special characters mangled
10. Error responses missing codes
11. Setup claude not creating config
12. Any panics in MCP operations
