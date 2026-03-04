---
name: dds
description: Use when working with the draft simulator, adding draft simulation features, fixing draft simulator bugs, or running draft parameter sweeps. Triggers on draft simulator, draft sim, dds, draft sweep, draft metrics, draft_simulator.py.
---

# Draft Simulator (DDS)

Read the documentation before making changes:

- **Full documentation**: [docs/draft_simulation/draft_simulation.md](../../../docs/draft_simulation/draft_simulation.md) — architecture, running modes, metrics, configuration, and sweep parameters.

## Commands

- `just fmt` — format code. Run first before other checks.
- `just review` — full lint/test gate (~5 min, keep polling).

## Acceptance Criteria

- **Manual testing is CRITICAL.** After every change, run the simulator manually
  and verify output. Do NOT rely only on automated tests. Run at minimum:
  - `cd scripts/draft_simulator && python3 draft_simulator.py single`
  - `cd scripts/draft_simulator && python3 draft_simulator.py sweep --output-dir /tmp/dds_sweep`
  - Inspect the output for correctness, reasonable metric values, and no errors.
- If adding a new mode or metric, run it end-to-end and confirm the output
  makes sense before considering the task complete.
- Run `just fmt` then `just review` after all changes.
