Please follow all of the "code style" and "validating changes" rules below at all times.

Please use `just` commands instead of `cargo`, e.g. `just fmt`, `just check`,
`just-clippy`, `just-test`, `just parser-test`

# CODE STYLE


- Prefer writing code inline (when possible) to creating new variables via "let" statements
- DO NOT add inline comments to code. Add a short doc comment to top-level public functions.
- DO NOT delete existing inline comments
- DO NOT fully-qualify names in code ever
- Function calls and enum values have exactly one qualifier, struct names and enum types have
  zero qualifiers:
  - WRONG FUNCTION CALL: crate::zone_mutations::move_card::to_destination_zone()
  - WRONG FUNCTION CALL: to_destination_zone()
  - WRONG ENUM VALUE: battle_state::battle_cards::zone::Zone::Battlefield
  - WRONG STRUCT NAME: battle_state::BattleState
  - CORRECT ENUM VALUE: Zone::Battlefield
  - CORRECT FUNCTION CALL: move_card::to_destination_zone()
  - CORRECT STRUCT NAME: BattleState
- Public functions and types go at the TOP of the file, then private ones.
- Don't add "pub use" for anything.
- Keep Cargo.toml dependencies alphabetized in two lists, internal then external dependencies.
- Use modern Rust features such as let-else statements and "{inline:?}" variable formatting
- Do not add code to `mod.rs` or `lib.rs` files except for module declarations
- Do not add `use` declarations within function bodies, only place them at the top of files
- Qualify imports via `crate::`, not via `super::`
- Do not write inline `mod tests {` tests, place them in the `/tests/` directory

# VALIDATING CHANGES


- After completing work, please always run "just fmt" to apply rustfmt
  formatting rules
- Run `just check` to type check code
- Run `just clippy` to check for lint warnings
- After writing a test, use `just battle-test <TEST NAME>` to run it
- After completing work, please ALWAYS run `just review` to validate changes
- Do not print a summary of changes after completing work.
- Prefer the `just` commands over `cargo` commands since they have project-specific rules


# PROJECT CONTEXT


This project is an implementation of a rules engine for the 'Dreamtides'
card game. Dreamtides is a game similar to TCGs like 'Magic: the Gathering'.
Players put 'character' cards into play and use one-time 'event' cards to affect
the game. Cards are played using a resource called 'energy'. Characters have a
value called 'spark' that lets them generate victory 'points'. Playing a character
is called 'materializing' it and destroying it is called 'dissolving' it.
Dissolved characters and played events go the discard pile, which is called the
'void'. Playing one game or match of Dreamtides is known as a 'battle'.

A few relevant keywords:

- Reclaim: Play from void (discard pile)
- Kindle: Add spark to your leftmost character
- Foresee: Look at top N of deck and reorder or put in void
- Prevent: Stop a card from resolving (counterspell)


# ADDITIONAL DOCUMENTATION

More task-specification documentation is available:

- Development environment setup: `rules_engine/docs/environment_setup.md`
- Adding new battle effects: `rules_engine/docs/adding_new_effects.md`
- Adding new trigger conditions: `rules_engine/docs/adding_new_triggers.md`
- Running benchmarks: `rules_engine/docs/benchmarks.md`


# CODE STRUCTURE

The code is structured as a series of Rust crates using the cargo "workspace" feature.

Dreamtides:
  - `justfile`
  - `rules_engine/`
    - 'Cargo.toml`
  - `client/

Rules engine Rust source code lives in the `rules_engine/` directory.
Client source code lives in the `client/` directory.

Your current working directory is `rules_engine/`. The `justfile` and similar are located in the parent directory.

Card data lives in `rules_engine/tabula/cards.toml`. Do NOT read this file directly, it is much too large.

# TASK TRACKING

When ending a session or when you discover work outside of the scope of the current session,
please track it via the `bd` task software. If a prompt references something with a `dr-`
prefix, like `dr-l3x`, this is a `bd` isssue id. When asked to work on a task, please
close it when complete.

## Core Rules
- Track strategic work in beads (multi-session, dependencies, discovered work)
- Use `bd create` for issues, TodoWrite for simple single-session execution
- When in doubt, prefer bd—persistence you don't need beats lost context
- Session management: check `bd ready` for available work

## Essential Commands

### Finding Work
- `bd list --status=open` - All open issues
- `bd list --status=in_progress` - Your active work
- `bd show <id>` - Detailed issue view with dependencies

### Creating & Updating
- `bd create --title="..." --type=task|bug|feature --priority=2` - New issue
  - Priority: 0-4 or P0-P4 (0=critical, 2=medium, 4=backlog). NOT "high"/"medium"/"low"
- `bd update <id> --status=in_progress` - Claim work
- `bd update <id> --assignee=username` - Assign to someone
- `bd close <id>` - Mark complete
- `bd close <id1> <id2> ...` - Close multiple issues at once (more efficient)
- `bd close <id> --reason="explanation"` - Close with reason
- **Tip**: When creating multiple issues/tasks/epics, use parallel subagents for efficiency

### Dependencies & Blocking
- `bd dep add <issue> <depends-on>` - Add dependency (issue depends on depends-on)
- `bd blocked` - Show all blocked issues
- `bd show <id>` - See what's blocking/blocked by this issue

### Sync & Collaboration

DO NOT RUN `bd sync` ever. We do not use this workflow.

### Project Health
- `bd stats` - Project statistics (open/closed/blocked counts)
- `bd doctor` - Check for issues (sync problems, missing hooks)

## Common Workflows

**Starting work:**
```bash
bd show <id>       # Review issue details
bd update <id> --status=in_progress  # Claim it
```

**Completing work:**
```bash
bd close <id1> <id2> ...    # Close all completed issues at once
```

**Creating dependent work:**
```bash
# Run bd create commands in parallel (use subagents for many items)
bd create --title="Implement feature X" --type=feature
bd create --title="Write tests for X" --type=task
bd dep add beads-yyy beads-xxx  # Tests depend on Feature (Feature blocks tests)
```
