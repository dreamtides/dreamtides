---
name: qs-analyze
description: Use when analyzing quest simulator logs, investigating draft behavior, reviewing AI bot decisions, or answering questions about a quest session. Triggers on quest log analysis, draft analysis, AI agent picks, archetype commitment, session replay, qs-analyze, analyze quest, what happened in the draft.
---

# Quest Simulator Log Analysis

Analyze JSONL logs from quest simulator sessions to answer questions about human and AI drafter behavior.

## Step 1: Find the Most Recent Session

Use filesystem modification time, NOT filename sorting (filenames embed creation date which may not reflect the most recent run):

```bash
stat -f "%Sm %N" .logs/quest_*.jsonl | sort -r | head -5
```

Pick the file with the most recent modification timestamp. The filename encodes: `quest_{DATE}_{NANOS}_seed{SEED}.jsonl`

## Step 2: Load and Parse Events

Read the JSONL file. Each line is a JSON object with an `"event"` field. Key event types:

| Event | Key Fields | Use For |
|-------|-----------|---------|
| `session_start` | `seed`, `draft_config` | Session parameters |
| `round_start` | `round_index`, `global_pick_index`, `pack_card_count` | Draft round boundaries |
| `ai_pick` | `seat_index`, `global_pick_index`, `chosen`, `chosen_score`, `agent_w_top3`, `was_random` | AI bot decisions |
| `show_n_filter` | `shown_cards_with_scores`, `filtered_out_top3`, `strategy` | What human was shown vs filtered |
| `draft_pick` | `offered`, `picked`, `human_w_top3`, `global_pick_index` | Human picks |
| `preference_snapshot` | `preference_vector`, `top_archetype_index`, `concentration` | Human archetype evolution |
| `site_visit` | `site_type`, `choices_offered`, `choice_made` | Non-draft site interactions |
| `battle_complete` | `opponent_name`, `essence_reward`, `rare_pick` | Battle outcomes |
| `session_end` | `deck`, `preference_vector`, `completion_level` | Final deck and state |

## Step 3: Archetype Index Mapping

Map archetype indices to names using this table:

| Index | Name | Resonance |
|-------|------|-----------|
| 0 | Flash | Thunder/Tide |
| 1 | Awaken | Thunder/Flame |
| 2 | Flicker | Flame/Thunder |
| 3 | Ignite | Flame/Stone |
| 4 | Shatter | Stone/Flame |
| 5 | Endure | Stone/Tide |
| 6 | Submerge | Tide/Stone |
| 7 | Surge | Tide/Thunder |

Only a subset of archetypes are active per session. Check `session_start.draft_config` for which are selected.

## Step 4: Answer Questions

### AI Agent Archetype Commitment

AI agents commit to an archetype after picking `ai_resonance_commit_pick` cards (default ~9). To find when agent N committed:

1. Filter `ai_pick` events where `seat_index == N`
2. Track `agent_w_top3` across picks — the top archetype stabilizing indicates commitment
3. The commit happens at the pick where `len(drafted) >= ai_resonance_commit_pick`

### Human Drafter Behavior

Track via `draft_pick` events:
- `human_w_top3` shows evolving archetype preferences
- `offered` vs `picked` shows decision-making
- `preference_snapshot` events show concentration (commitment strength)

### Draft Flow Reconstruction

Use `round_start` events to segment picks into rounds. Within each round, `ai_pick` events for each seat precede the `draft_pick` (human pick). The `global_pick_index` links all events to a unified timeline.

## Analysis Script

For complex queries, write a Python script inline:

```python
import json, sys
events = [json.loads(line) for line in open(LOG_PATH)]
# Filter by event type
ai_picks = [e for e in events if e["event"] == "ai_pick"]
# Group by seat
from collections import defaultdict
by_seat = defaultdict(list)
for p in ai_picks:
    by_seat[p["seat_index"]].append(p)
```

## Common Questions

- **"What did AI agent N pick?"** — Filter `ai_pick` where `seat_index == N`, list `chosen.name` fields
- **"When did agent N commit?"** — Find the `ai_pick` for seat N where `agent_w_top3[0]` stabilizes and pick count crosses commit threshold
- **"What was filtered from the human?"** — Check `show_n_filter` events, compare `shown_cards_with_scores` vs `filtered_out_top3`
- **"What archetype did the human end up in?"** — Check `session_end.preference_vector` or last `preference_snapshot`
- **"What was the final deck?"** — Read `session_end.deck` array

## When Logs Cannot Answer the Question

If the existing log events lack the data needed to answer a question, add new logging rather than guessing.

### Procedure

1. **Identify the gap.** State exactly what data is missing and which event type would logically carry it (existing or new).
2. **Check if an existing event can be enriched.** Prefer adding fields to an existing event over creating a new event type. For example, if `ai_pick` lacks a field you need, add it there.
3. **Add logging in the source code.** All logging goes through `SessionLogger` in `scripts/quest_simulator/jsonl_log.py`:
   - To enrich an existing event: add a parameter to the relevant `log_*()` method and include it in the `_write()` dict.
   - To add a new event type: add a new `log_*()` method following the existing pattern.
4. **Wire it up at the call site.** Find where the `log_*()` method is called (typically in `round_manager.py`, `draft_strategy.py`, or `sites_*.py`) and pass the new data.
5. **Update the test.** Add a test case in `scripts/quest_simulator/test_jsonl_log.py` that verifies the new field/event appears in output.
6. **Re-run the simulator** to generate a fresh log with the new data:
   ```bash
   python3 scripts/quest_simulator/quest_sim.py --seed 42
   ```
7. **Update this skill.** Add the new event/field to the event table in Step 2 above so future analysis sessions know it exists.

### Key files for adding logging

| File | Role |
|------|------|
| `scripts/quest_simulator/jsonl_log.py` | `SessionLogger` — all log methods defined here |
| `scripts/quest_simulator/log_helpers.py` | Shared scoring/serialization helpers |
| `scripts/quest_simulator/round_manager.py` | AI pick loop — calls `log_ai_pick`, `log_round_start` |
| `scripts/quest_simulator/draft_strategy.py` | Show-N filtering — calls `log_show_n_filter` |
| `scripts/quest_simulator/sites_*.py` | Site handlers — call `log_draft_pick`, `log_site_visit`, etc. |
| `scripts/quest_simulator/test_jsonl_log.py` | Tests for all log methods |

### Rules

- Never invent data or guess values that aren't in the logs. If it's not logged, say so and offer to add logging.
- Follow the `qs` skill acceptance criteria after any code changes: run the simulator manually, run tests, `just fmt`, `just review`.
