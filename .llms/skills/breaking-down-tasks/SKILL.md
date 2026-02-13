---
name: breaking-down-tasks
description: Break a technical design document into well-structured implementation tasks using TaskCreate. Use when handed a design doc, spec, or project plan that needs to be decomposed into actionable tasks. Triggers on mentions of task breakdown, project planning, creating tasks from a design doc, or decomposing work.
---

# Breaking Down a Design Document into Tasks

Given a technical design document, decompose it into well-structured, independent
implementation tasks using the `TaskCreate` tool. Each task should be completable
in a single Claude session without overflowing the context window.

Each task should link to the technical design document in its initial task
description.

## Workflow

### Phase 1: Research (Parallel)

Spawn 2-4 **Haiku Explore** agents in parallel (`run_in_background: true`) to
gather context. Tailor prompts to the design doc. Common research questions:

- **Codebase structure**: Which existing files/modules will be modified or extended?
- **Patterns & conventions**: How does the codebase handle similar features today?
- **Dependencies**: What existing types, traits, functions will tasks need to use?
- **Test patterns**: How are similar features tested? Where do tests live?

Each agent should return a structured list of file paths with one-line explanations.

**Wait for all research agents to complete** before proceeding to Phase 2.

### Phase 2: Task Design

Using research results and the design doc, design the task breakdown:

1. Identify the logical units of work
2. Order them foundationally (see "Task Ordering" below)
3. Determine dependencies (see "Dependency Strategy" below)
4. Write detailed task descriptions (see "Task Quality" below)

### Phase 3: Task Creation

Create tasks using `TaskCreate`. Tasks must be created in parallel via subagents.
After creation, use `TaskUpdate` to set `addBlockedBy` dependencies.

## Task Ordering

Tasks execute oldest-first (lowest ID first) by default. Use this to your
advantage as a **soft dependency mechanism**:

1. **First tasks**: Foundational types, traits, enums, data structures
2. **Middle tasks**: Core logic, implementations that build on foundations
3. **Later tasks**: Integration points, higher-level features using core logic
4. **Final tasks**: Tests-only tasks, cleanup, polish, documentation

This natural ordering reduces merge conflicts even without explicit dependencies
because foundational code gets merged first, and later tasks build on a stable base.

**Example ordering for a new game mechanic:**
1. Add new enum variants and data types
2. Implement core logic functions
3. Add parser/serializer support
4. Wire into game loop / event handling
5. Add integration tests

## Dependency Strategy

Use `TaskUpdate` with `addBlockedBy` to create explicit dependencies.

### When to add a dependency

- Two tasks modify the **same file** — add a dependency to prevent merge conflicts
- Task B **calls functions** or **uses types** introduced by Task A
- Task B **extends behavior** that Task A implements (e.g., A adds the enum, B matches on it)

### When NOT to add a dependency

- Tasks touch **different files** with no shared interfaces
- Tasks are **parallel features** that happen to be part of the same project
- The only relationship is **conceptual** (both relate to "authentication") but code doesn't overlap

### Balancing parallelism vs. safety

More dependencies = fewer merge conflicts but slower execution (less parallelism).

**Default to fewer dependencies.** Rely on task ordering (soft dependencies) to
handle most cases. Only add explicit dependencies when:
- File overlap is certain and significant
- The compile would fail without the dependency's output
- Merge conflicts would be complex to resolve (not just import additions)

**Rule of thumb**: If two tasks edit different files, no dependency needed. If they
edit the same file in different sections, consider a dependency. If they edit the
same functions or data structures, a dependency is mandatory.

## Task Quality

Every task description must contain ALL of the following sections. The implementing
agent has **no prior knowledge** of the project or problem domain.

### Template

Use this structure for every task description:

```
## Context

[2-3 sentences explaining the project, the feature being built, and where this
task fits in the larger plan. Include domain terminology definitions if needed.]

## Objective

[1-2 sentences stating exactly what this task accomplishes.]

## Key Files

Read these files for context before starting:
- `path/to/relevant/file.rs` - [why to read it]
- `path/to/another/file.rs` - [why to read it]
- `path/to/pattern/example.rs` - [example of the pattern to follow]

## Requirements

1. [Specific, concrete requirement]
2. [Another requirement]
3. [...]

## Acceptance Criteria

- [ ] [Verifiable criterion, e.g., "New enum variant `Foo` exists in `bar.rs`"]
- [ ] [Another criterion]
- [ ] Code compiles: `just check`
- [ ] Formatting applied: `just fmt`
- [ ] All lints and tests pass: `just review`
- [ ] Changes committed with descriptive message via `git commit`
```

### Description guidelines

- **Context section**: Always explain the project domain briefly. Define game-specific
  terms (e.g., "materializing means playing a character card"). Reference the design
  doc's goals.
- **Key Files**: List 3-7 files. Include both files to modify AND files that serve as
  examples of the pattern to follow. Use exact paths from research results.
- **Requirements**: Numbered list of concrete changes. Each requirement should map to
  a small, verifiable code change. Avoid vague instructions like "implement the feature."
- **Acceptance Criteria**: Always end with the three standard criteria (check, fmt, review,
  commit). Add 2-5 task-specific criteria that are objectively verifiable.

### Sizing guidelines

Each task should be completable in **one Claude session** without exhausting the
context window. Rules of thumb:

- **1-3 files modified** per task (ideal)
- **~200 lines of new/changed code** maximum per task
- If a task touches more than 5 files, split it
- If describing the task requires more than ~400 words, split it
- Prefer more smaller tasks over fewer large tasks
- A task that adds types/structs and a task that implements logic using them
  should generally be separate tasks

### Subject and activeForm

- **subject**: Imperative form, specific. E.g., "Add DamageEffect enum variant and parser"
- **activeForm**: Present continuous. E.g., "Adding DamageEffect enum variant and parser"

Avoid generic subjects like "Implement feature" or "Update code."
