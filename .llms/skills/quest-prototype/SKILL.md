---
name: quest-prototype
description: Use when debugging bugs in the quest simulator, adding features to the quest prototype, analyzing quest JSONL logs, or running quest simulator tests and typechecking. Triggers on quest simulator, quest prototype, quest sim, quest log, quest bug.
---

# Quest Prototype Development

Working with the interactive quest simulator at
`scripts/quest_simulator/`.

## The Golden Rule

**Every change MUST be manually tested by running the simulator on the
command line.** Acceptance criteria for all changes include running the
tool interactively and verifying correct behavior. Automated tests are
necessary but never sufficient.

## Running the Simulator

```sh
# Interactive run (random seed)
python3 scripts/quest_simulator/quest_sim.py

# Reproducible run with a specific seed
python3 scripts/quest_simulator/quest_sim.py --seed 42
```

Run from the project root. Arrow keys navigate menus, Enter confirms,
Space toggles multi-select, Ctrl+C quits.

## Key Files

| File | Purpose |
|------|---------|
| `quest_sim.py` | CLI entry point, arg parsing, initialization |
| `flow.py` | Main quest loop: atlas nav, site visits, battle, victory |
| `quest_state.py` | Mutable quest state (deck, essence, dreamsigns, pool) |
| `models.py` | All data models (Card, Site, Dreamcaller, Dreamsign, etc.) |
| `site_dispatch.py` | Routes site types to handler modules |
| `input_handler.py` | Raw terminal input (arrow keys, menus, confirm/decline) |
| `algorithm.py` | Resonance-weighted card selection algorithm |
| `pool.py` | Draft pool generation and variance |
| `atlas.py` | Dream atlas graph generation and node management |
| `jsonl_log.py` | JSONL session logger |
| `render.py`, `render_*.py` | Terminal UI rendering |
| `sites_*.py` | Individual site type implementations |
| `data/config.toml` | Quest economy config (essence, limits, draft params) |
| `data/*.toml` | Dreamcallers, dreamsigns, journeys, offers, banes, bosses |

## Design Documents

Read these to understand quest mechanics before making changes:

- `docs/plans/quests/quests.md` — Master design doc (sites, atlas,
  resonance, drafting, all game systems)
- `docs/plans/quests/current_prototype.md` — Technical reference for
  current prototype implementation and gaps
- `docs/plans/quests/resonance_and_tags.md` — Resonance weighting and
  card tagging details
- `docs/plans/quests/bosses.md` — Boss dreamcaller design
- `docs/plans/quests/meta_progression.md` — Meta progression unlocks

## Running Tests

Quest simulator tests live alongside source files as `test_*.py`.
They are **not** included in `just python-test`. Run them directly:

```sh
# All quest simulator tests
cd scripts/quest_simulator && python3 -m unittest discover -s . -p "test_*.py"

# Verbose output
cd scripts/quest_simulator && python3 -m unittest discover -s . -p "test_*.py" -v

# Single test file
cd scripts/quest_simulator && python3 -m unittest test_sites_draft -v

# Single test case
cd scripts/quest_simulator && python3 -m unittest test_sites_draft.TestDraftSite.test_basic_draft -v
```

## Typechecking with Pyre

```sh
# Quick check (suppressed output on success)
just pyre-check

# Verbose output
just pyre-check-verbose

# Or directly
uvx --from pyre-check pyre check
```

Pyre IS included in `just review` for `.py` file changes.

## Debugging with JSONL Logs

Every interactive run writes a JSONL log to `.logs/` at the project
root. One file per session, named
`quest_{timestamp}_{ns}_seed{seed}.jsonl`.

**Find the most recent log:**
```sh
ls -t .logs/quest_*.jsonl | head -1
```

**Event types in logs:** `session_start`, `dreamscape_enter`,
`site_visit`, `draft_pick`, `shop_purchase`, `battle_complete`,
`session_end`, `error`.

**Useful jq queries:**

```sh
LOG="$(ls -t .logs/quest_*.jsonl | head -1)"

# List all events
jq -c '.event' "$LOG"

# Session config
jq 'select(.event == "session_start")' "$LOG"

# All draft picks with resonance profile evolution
jq 'select(.event == "draft_pick") | {picked: .picked.name, resonances: .picked.resonances, profile: .profile_after}' "$LOG"

# Draft weights (what was offered, selection probability)
jq 'select(.event == "draft_pick") | .offered | map({name, weight}) | sort_by(-.weight)' "$LOG"

# Site visits and choices
jq 'select(.event == "site_visit") | {site: .site_type, enhanced: .is_enhanced, choice: .choice_made}' "$LOG"

# Shop purchases
jq 'select(.event == "shop_purchase") | {bought: [.items_bought[].name], spent: .essence_spent}' "$LOG"

# Battle results
jq 'select(.event == "battle_complete")' "$LOG"

# Final deck summary
jq 'select(.event == "session_end")' "$LOG"

# Errors
jq 'select(.event == "error")' "$LOG"
```

**Reproduce a bug:** Extract the seed from the log's filename or
`session_start` event, then re-run with `--seed <N>`. Behavior is
deterministic for a given seed.

## Debugging Workflow

1. Reproduce the bug with a known seed
2. Run interactively and observe the problem
3. Check the JSONL log for the state at the point of failure
4. Read the relevant `sites_*.py` handler and `flow.py` loop
5. Fix the issue
6. Run automated tests: `cd scripts/quest_simulator && python3 -m unittest discover -s . -p "test_*.py"`
7. Run pyre: `just pyre-check`
8. **Manually run the simulator** with the original seed to confirm the fix
9. Run `just fmt` then `just review`

## Adding Features

1. Read the relevant section of `docs/plans/quests/quests.md`
2. Identify which files need changes (usually a `sites_*.py` +
   `models.py` + `site_dispatch.py` + possibly `flow.py`)
3. Write tests in the corresponding `test_*.py` file
4. Implement the feature
5. Add JSONL logging for new events if applicable
6. Run automated tests
7. Run pyre
8. **Manually run the simulator end-to-end** and verify the feature
   works correctly in context
9. Run `just fmt` then `just review`

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Skipping manual QA | Always run the simulator interactively before committing |
| Running `just python-test` for quest sim tests | Quest sim tests are separate; run directly with unittest |
| Forgetting to log new events | Add `logger.log_*` calls for any new site behavior |
| Modifying `data/*.toml` without testing | Re-run simulator to verify economy balance |
| Not testing with a fixed seed | Use `--seed` for reproducible debugging |
