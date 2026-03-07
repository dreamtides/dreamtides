---
name: qs
description: Use when working with the quest simulator, adding quest simulation features, fixing quest simulator bugs, or running quest simulator tests and typechecking. Triggers on quest simulator, quest sim, qs, quest bug, quest_simulator.py.
---

# Quest Simulator (QS)

Read the documentation before making changes:

- **Full documentation**: [docs/quest_simulator/quest_simulator.md](../../../docs/quest_simulator/quest_simulator.md) — architecture, draft loop integration, module layout, site handlers, configuration, and data files.

## Commands

- `just fmt` — format code. Run first before other checks.
- `just review` — full lint/test gate (~5 min, keep polling).

## Acceptance Criteria

- **Manual testing is CRITICAL.** After every change, run the simulator manually
  and verify correct behavior. Do NOT rely only on automated tests. Run at minimum:
  - `python3 scripts/quest_simulator/quest_sim.py --seed 42`
  - Navigate through at least one full dreamscape (select sites, make picks, reach battle).
  - Verify the changed behavior works correctly in context.
- Quest simulator tests are separate from `just python-test`. Run them directly:
  - `cd scripts/quest_simulator && python3 -m unittest discover -s . -p "test_*.py"`
- Run `just fmt` then `just review` after all changes.
