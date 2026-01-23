# Task Management Tools

This document describes the four task management tools available in Claude Code for tracking work during a coding session.

## Overview

| Tool | Purpose |
|------|---------|
| **TaskCreate** | Create a new task to track |
| **TaskGet** | Retrieve full details of a specific task |
| **TaskUpdate** | Modify a task's status, details, or dependencies |
| **TaskList** | List all tasks and their current state |

---

## TaskCreate

Creates a structured task for tracking work in the current coding session. Use this to organize complex, multi-step work.

### When to Use

- Complex multi-step tasks requiring 3 or more distinct steps
- When the user provides multiple tasks (numbered or comma-separated)
- When in plan mode to track planned work
- When the user explicitly requests a todo list

### When NOT to Use

- Single, straightforward tasks
- Trivial tasks that can be completed in fewer than 3 steps
- Purely conversational or informational requests

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `subject` | string | **Yes** | A brief title for the task in imperative form (e.g., "Fix authentication bug in login flow") |
| `description` | string | **Yes** | A detailed description of what needs to be done, including context and acceptance criteria |
| `activeForm` | string | No | Present continuous form shown in the spinner when the task is `in_progress` (e.g., "Running tests"). Should match the subject but in present continuous tense. |
| `metadata` | object | No | Arbitrary key-value metadata to attach to the task |

### Behavior

- All tasks are created with status `pending`
- Tasks have no owner when created
- Task IDs are automatically generated

### Example

```json
{
  "subject": "Implement user authentication",
  "description": "Add JWT-based authentication to the API endpoints. Include login, logout, and token refresh functionality.",
  "activeForm": "Implementing user authentication"
}
```

---

## TaskGet

Retrieves the full details of a task by its ID.

### When to Use

- When you need the full description and context before starting work on a task
- To understand task dependencies (what it blocks, what blocks it)
- After being assigned a task, to get complete requirements

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `taskId` | string | **Yes** | The ID of the task to retrieve |

### Response Fields

| Field | Description |
|-------|-------------|
| `id` | The task's unique identifier |
| `subject` | The task title |
| `description` | Detailed requirements and context |
| `status` | Current status: `pending`, `in_progress`, or `completed` |
| `owner` | Agent ID if assigned, empty if available |
| `blocks` | List of task IDs that are waiting on this task to complete |
| `blockedBy` | List of task IDs that must complete before this task can start |
| `activeForm` | The spinner text shown when task is in progress |
| `metadata` | Any attached metadata |

### Example

```json
{
  "taskId": "task-123"
}
```

---

## TaskUpdate

Updates an existing task's status, details, or dependencies.

### When to Use

- **Mark tasks as in_progress**: When starting work on a task
- **Mark tasks as completed**: When you have fully accomplished the task
- **Update task details**: When requirements change or become clearer
- **Establish dependencies**: When tasks need to be sequenced

### Important Notes

- Only mark a task as `completed` when it is **fully** accomplished
- If you encounter errors, blockers, or cannot finish, keep the task as `in_progress`
- When blocked, create a new task describing what needs to be resolved
- Never mark a task as completed if:
  - Tests are failing
  - Implementation is partial
  - You encountered unresolved errors
  - You couldn't find necessary files or dependencies

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `taskId` | string | **Yes** | The ID of the task to update |
| `status` | string | No | New status: `pending`, `in_progress`, or `completed` |
| `subject` | string | No | New title for the task (imperative form) |
| `description` | string | No | New description for the task |
| `activeForm` | string | No | New spinner text (present continuous form) |
| `owner` | string | No | New owner for the task (agent name) |
| `metadata` | object | No | Metadata keys to merge into the task. Set a key to `null` to delete it. |
| `addBlocks` | string[] | No | Task IDs that this task blocks (cannot start until this completes) |
| `addBlockedBy` | string[] | No | Task IDs that block this task (must complete before this can start) |

### Status Workflow

```
pending  -->  in_progress  -->  completed
```

### Examples

**Mark task as in progress when starting work:**
```json
{
  "taskId": "task-123",
  "status": "in_progress"
}
```

**Mark task as completed after finishing:**
```json
{
  "taskId": "task-123",
  "status": "completed"
}
```

**Claim a task by setting owner:**
```json
{
  "taskId": "task-123",
  "owner": "agent-1"
}
```

**Set up task dependencies:**
```json
{
  "taskId": "task-456",
  "addBlockedBy": ["task-123"]
}
```

**Update multiple fields:**
```json
{
  "taskId": "task-123",
  "status": "in_progress",
  "description": "Updated requirements: also include password reset flow",
  "addBlocks": ["task-789"]
}
```

---

## TaskList

Lists all tasks in the task list, providing a summary view of the project's work items.

### When to Use

- To see what tasks are available to work on (status: `pending`, no owner, not blocked)
- To check overall progress on the project
- To find tasks that are blocked and need dependencies resolved
- Before assigning tasks to teammates, to see what's available
- After completing a task, to check for newly unblocked work or claim the next available task

### Parameters

This tool takes no parameters.

### Response Fields

For each task, returns:

| Field | Description |
|-------|-------------|
| `id` | Task identifier (use with TaskGet, TaskUpdate, or assignTask) |
| `subject` | Brief description of the task |
| `status` | `pending`, `in_progress`, or `completed` |
| `owner` | Agent ID if assigned, empty if available |
| `blockedBy` | List of open task IDs that must be resolved first |

### Finding Available Work

Tasks are available to claim when they have:
- Status: `pending`
- No owner assigned
- Empty `blockedBy` list (no blocking dependencies)

### Example Usage

After completing a task:
1. Call `TaskList` to see all tasks
2. Look for tasks with status `pending`, no owner, and empty `blockedBy`
3. Use `TaskUpdate` to claim the task by setting yourself as owner
4. Use `TaskUpdate` to set status to `in_progress`
5. Work on the task
6. Use `TaskUpdate` to set status to `completed`
7. Repeat

---

## Best Practices

1. **Create tasks with clear, specific subjects** that describe the outcome
2. **Include enough detail in the description** for another agent to understand and complete the task
3. **Always provide `activeForm`** when creating tasks - it improves the user experience
4. **Use dependencies** (`addBlocks`/`addBlockedBy`) when tasks must be completed in a specific order
5. **Read a task's latest state** using `TaskGet` before updating it to avoid stale data issues
6. **Mark tasks `in_progress` before starting** so others know the task is being worked on
7. **Only mark tasks `completed` when fully done** - partial completion should remain `in_progress`
