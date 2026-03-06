# Quest Simulator

The quest simulator at `scripts/quest_simulator/` is an interactive
terminal roguelike deckbuilder. Players navigate a dream atlas (a graph
of dreamscape nodes), visiting sites that offer cards, dreamsigns,
dreamcallers, essence, transfiguration, and battles. A full quest spans
roughly 7 battles across 7+ dreamscapes.

## The Core Invariant

**Quest mode IS the draft simulator's 6-seat loop.** Card selection is
not "inspired by" or "adapted from" the draft simulator — it runs the
*identical* round/pick/seat/rotation structure from
`draft_simulator/draft_runner.py` (lines 112-203), extended to an
indefinite number of rounds.

Quest sites are hooks *within* that loop at the human seat's pick slot.
They are **not** drivers of independent pack generation. When a Draft
site consumes 5 picks, those are 5 consecutive human-seat pick steps
inside the continuous draft loop — AI bots pick and packs rotate between
each one, exactly as in a real draft.

Any design or implementation that generates packs per-site or outside
this loop is wrong.

## Execution Model

The draft loop runs as follows, mirroring `draft_runner.py` lines
112-203:

**Round start**: Generate one pack of 20 cards per seat (6 packs total)
via `pack_generator.generate_pack()` using the `seeded_themed` strategy.
Each pack is drawn from the shared `CubeManager`.

**Pick loop** (10 picks per round): For each pick step, iterate seats
0-5:

- AI seats (1-5): each AI agent sees its full pack, picks one card via
  `agents.pick_card()` with `adaptive` policy and `ai_optimality=0.80`.
  `update_agent_after_pick()` is called after each pick.
- Human seat (0): the quest site handler that owns this pick presents
  cards filtered by `show_n.select_cards()` with `show_n=4` and
  `sharpened_preference` strategy. The player picks one card.
  `update_agent_after_pick()` is called on the human agent.

**Pack rotation**: after all 6 seats have picked, packs rotate left.
The pack that was at seat 0 moves to seat 1, etc.

**Round end**: after 10 pick steps, a new round starts from step 1.
Quest mode extends this indefinitely — there is no fixed round count.

The quest does **not** call `run_draft()`. That function runs a fixed
batch draft and returns a `DraftResult`. The quest builds its own
incremental loop via `round_manager.py`, which calls the same
sub-functions in the same order.

## Module Layout

Entry point: `quest_sim.py` — CLI parsing, draft engine init, quest
flow launch.

| Module               | Role                                                        |
| -------------------- | ----------------------------------------------------------- |
| `quest_sim.py`       | Entry point: CLI, initialization, launches `flow.run_quest` |
| `quest_state.py`     | `QuestState` dataclass: all mutable state across the quest  |
| `round_manager.py`   | Incremental draft loop: advance to human pick, complete it  |
| `flow.py`            | Atlas loop, dreamscape loop, site dispatch, progression     |
| `site_dispatch.py`   | Routes site types to handler modules; `SiteData` bundle     |
| `atlas.py`           | Dreamscape graph generation and navigation                  |
| `data_loader.py`     | TOML data loading: config, dreamcallers, dreamsigns, etc.   |
| `models.py`          | Quest-specific types: `DeckCard`, `Dreamsign`, `Dreamcaller`|
| `render.py`          | Terminal rendering utilities and color palette shims        |
| `render_cards.py`    | Card display formatting                                     |
| `render_status.py`   | Victory screen, archetype preference footer                 |
| `render_atlas.py`    | Atlas/dreamscape map display                                |
| `input_handler.py`   | Terminal input and keyboard handling                        |
| `jsonl_log.py`       | Session logging to JSONL format                             |
| `validate_data.py`   | Data file validation at startup                             |

**Card-offering site handlers**: `sites_draft.py`, `sites_shop.py`,
`sites_discovery.py`, `sites_battle.py`, `sites_journey.py`,
`sites_misc.py`.

**Non-card site handlers**: `sites_dreamsign.py`, `sites_dreamcaller.py`,
`sites_purge.py`, `sites_transfig.py`, `sites_essence.py`.

## Cross-Simulator Import

`quest_sim.py` adds `scripts/draft_simulator/` to `sys.path` before
importing draft modules. Both simulators use flat namespace directories
with no `__init__.py`. Module names are disjoint — the draft simulator
uses the `draft_` prefix on its model file (`draft_models.py`) to avoid
colliding with `quest_simulator/models.py`. Any new module added to
either simulator must have a unique name.

Pyre's `search_path` flattens all directories under `scripts/` into one
namespace, so name collisions cause type errors project-wide.

## QuestState Fields

Draft-engine fields on `QuestState` (initialized in `quest_sim.py`):

- `human_agent: AgentState` — the human seat's agent, persists across
  the quest. Created via `agents.create_agent(archetype_count=8)`.
- `ai_agents: list[AgentState]` — 5 AI seat agents.
- `cube: CubeManager` — shared card supply. 540 distinct synthetic
  cards, `WITH_REPLACEMENT` mode so cards are never exhausted.
- `draft_cfg: SimulatorConfig` — config reference (see below).
- `packs: list[Pack] | None` — the 6 current packs. `None` when no
  round is active (triggers new round on next advance).
- `round_pick_count: int` — picks completed in the current round.
- `round_index: int` — current round number.
- `global_pick_index: int` — total pick steps completed across the
  quest.

## Round Manager API

`round_manager.py` is the integration point between the draft loop and
quest site handlers. Two functions:

`advance_to_human_pick(state)` — if no round is active, generates 6
fresh packs. Then runs AI picks for seats 1-5. Returns the pack at seat
0 for the site handler to filter and present.

`complete_human_pick(state, chosen_card, shown_cards)` — removes the
chosen card from seat 0's pack, updates the human agent, rotates packs,
and increments pick counters. If the round boundary (10 picks) is
reached, resets `packs = None` so the next call to
`advance_to_human_pick` starts a new round.

`advance_pick_no_card(state)` — advances one pick step without taking a
card. Used by shop reroll and "buy nothing" cases. Packs still rotate
and counters increment; human agent is not updated.

## Site Handler Pattern

All card-offering sites call the round manager. The standard pattern:

1. Call `round_manager.advance_to_human_pick(state)` — returns the pack
   at seat 0 (AI picks already done for this step).
2. Call `show_n.select_cards()` on the pack's cards to filter to N
   shown cards.
3. Present cards to the player; receive their choice.
4. Call `round_manager.complete_human_pick(state, chosen, shown)`.

Sites that consume multiple picks (e.g., Draft site: 5 picks) repeat
this pattern. AI picks and pack rotation happen inside `complete_human_pick`
between each repeat — the site handler calls the full cycle per pick.

## Site Types and Draft Interaction

**Draft, Shop, Battle post-pick, Dream Journey ADD_CARDS, Tempting
Offer ADD_CARDS, Reward** — these consume picks from the draft loop.
Each pick at the human seat causes AI bots to pick and packs to rotate.

**Discovery Draft, Specialty Shop** — do NOT consume draft picks. They
draw cards from `CubeManager.draw()` directly, bypassing the current
packs. AI bots do not advance and packs do not rotate during these sites.

**Non-card sites** (Dreamsign, Dreamcaller, Purge, Duplication, Cleanse,
Essence, Transfiguration) — do NOT interact with the draft loop at all.
The draft state is completely frozen during these visits.

## Draft Configuration

The quest constructs `SimulatorConfig` in `quest_sim.py` without calling
`validate_config()`, because the draft validator enforces
`sum(picks_per_round) == 30` which does not apply to quest mode's
indefinite rounds.

Key values:
- `draft.seat_count = 6`, `draft.pack_size = 20`, `draft.human_seats = 1`
- `draft.alternate_direction = False`
- `agents.show_n = 4`, `agents.show_n_strategy = "sharpened_preference"`
- `agents.policy = "adaptive"`, `agents.ai_optimality = 0.80`
- `agents.learning_rate = 3.0`, `agents.openness_window = 3`
- `cards.archetype_count = 8`, `cards.source = "synthetic"`
- `cube.distinct_cards = 540`, `cube.consumption_mode = "with_replacement"`
- `refill.strategy = "no_refill"`
- `pack_generation.strategy = "seeded_themed"`

## Running the Quest Simulator

```
python3 scripts/quest_simulator/quest_sim.py [--seed N]
```

The simulator is interactive — it reads from stdin and renders to the
terminal. For non-interactive smoke testing:

```
echo "" | python3 scripts/quest_simulator/quest_sim.py --seed 42 2>&1 | head -50
```

Tests:

```
cd scripts/quest_simulator
python3 -m unittest discover -p "test_*.py"
```

Tests and type checking share the same gate as the draft simulator:

```
just python-test   # discovers tests under scripts/
just pyre-check    # flat-namespace Pyre check across all simulators
```

## Data Files

Quest content lives under `scripts/quest_simulator/data/`:

- `config.toml` — quest parameters (starting essence, deck limits,
  battle count, shop pricing, pick counts). Separate from draft config.
- `dreamcallers.toml`, `dreamsigns.toml`, `bosses.toml` — quest content.
- `journeys.toml`, `offers.toml`, `banes.toml` — event and offer tables.

Card data is synthetic (generated at runtime by the draft simulator's
`card_generator.py`). There are no card JSON files — those were removed
in the draft integration rewrite.

## Further Reading

- [draft_simulation.md](../draft_simulation/draft_simulation.md) —
  the draft simulator's architecture, configuration, pick policies,
  show-N strategies, and module reference. Read this to understand the
  components the quest simulator imports and the execution model it
  mirrors.
