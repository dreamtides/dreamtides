# Draft Simulator

The draft simulator at `scripts/draft_simulator/` is a standalone Python batch
tool that runs synthetic Dreamtides drafts under configurable parameters and
measures draft experience quality. It validates design goals: choice richness,
non-forceability, early openness, convergence after commitment, splashability,
and difficulty-dependent signal-reading impact — through quantitative metrics
and parameter sweeps.

No external dependencies. Stdlib-only. Requires Python 3.11+ for TOML config
support (JSON works on 3.9+).

## Design Goals

The simulator enables parameter sweeps to evaluate six core design goals:

- **Choice richness**: drafters should have multiple viable options at each
  pick, not be "on rails" toward a single obvious choice.
- **Non-forceability**: no single archetype should be reliably forceable — an
  adaptive strategy that reads signals should outperform a forcing strategy.
- **Early openness**: the first several picks should expose drafters to many
  archetypes before requiring commitment.
- **Convergence after commitment**: once a drafter commits to an archetype,
  packs should continue delivering on-plan cards at a reasonable rate.
- **Splashability**: post-commitment picks should still offer viable off-plan
  cards worth including for raw strength or flexibility.
- **Signal-reading impact**: under hard difficulty, reading supply signals from
  pack contents should meaningfully improve outcomes over ignoring them.

### Non-Goals

- No post-draft gameplay or match simulation.
- No rules-text parsing or integration with the Dreamtides rules engine.
- No heavy manual card tagging beyond minimal card attributes (fitness vector,
  power, commit, flex).

### Determinism

Given identical configuration and RNG seed, the simulator produces identical
results including per-pick traces and aggregate metrics. All random operations
consume from a single seeded PRNG instance in a fixed order. No operation may
use a non-deterministic entropy source.

## Running the Simulator

Run from the project root. The entry point is `draft_simulator.py`:

```
cd scripts/draft_simulator
python3 draft_simulator.py [mode] [options]
```

Modes:

- `single` (default): one draft, per-seat summary and metrics printed to stdout
- `trace`: one draft with per-pick JSON output to `--output-dir`
- `sweep`: batch parameter experiment across a Cartesian product of values,
  writes CSV files to `--output-dir`
- `demo`: component demonstrations for each subsystem (legacy)

Common options:

- `--seed N` / `-s N`: base RNG seed (default: 42)
- `--config PATH` / `-c PATH`: TOML or JSON config file
- `--param KEY=VALUE`: dot-notation override, repeatable (example:
  `--param draft.seat_count=8 --param agents.policy=greedy`)
- `--runs N`: number of runs for sweep mode (default: 1000)
- `--output-dir PATH`: output directory (default: `./draft_output/`)
- `--preset easy|hard`: difficulty preset

Presets set `ai_optimality`, `ai_signal_weight`, and `seat_count` as a bundle.

Tests and type-checking run through the project's standard gates:

```
just python-test   # runs unittest discover across scripts/
just pyre-check    # Pyre type checker (flat namespace across all simulators)
```

The draft simulator tests are in `test_*.py` files inside
`scripts/draft_simulator/`. As of this writing, `python-test` only discovers
tests in `scripts/review/tests` and `scripts/abu` — run the draft simulator
tests directly:

```
cd scripts/draft_simulator
python3 -m unittest discover -p "test_*.py"
```

## Draft Model Overview

Seats are arranged in a circular order. Packs pass left by default (optionally
alternating direction per round). The human seat is always seat index 0.

Each seat drafts exactly 30 cards total across multiple rounds. The drafted pool
IS the final deck — there is no cut or sideboard construction step.

Each round: packs are generated (one per seat), then a pick loop runs where each
seat selects one card, packs rotate, and optional refill adds one card per pack.
At end of round, remaining pack contents are discarded.

**Observability**: AI seats see the full pack; human-modeled seats see only a
subset selected by the show-N strategy. The human also sees the current pick/
round index, pack ID, remaining card count (but not which hidden cards remain),
and their own drafted pool. The human does NOT see other seats' pools,
preferences, or picks.

## Module Layout

Seventeen modules with clear ownership boundaries:

| Module               | Role                                                                        |
| -------------------- | --------------------------------------------------------------------------- |
| `draft_simulator.py` | Entry point: CLI parsing, mode dispatch                                     |
| `config.py`          | `SimulatorConfig` dataclass tree; TOML/JSON loading; dot-notation overrides |
| `draft_models.py`    | Core data types: `CardDesign`, `CardInstance`, `Pack`, enums                |
| `card_generator.py`  | Synthetic card pool generation with fitness vectors                         |
| `cube_manager.py`    | `CubeManager`: card supply tracking, with/without replacement               |
| `pack_generator.py`  | Pack generation strategies: uniform, rarity_weighted, seeded_themed         |
| `refill.py`          | Pack refill strategies: no_refill, uniform_refill, constrained_refill       |
| `agents.py`          | `AgentState`; five pick policies; openness estimation                       |
| `show_n.py`          | Show-N selection strategies for the human seat                              |
| `commitment.py`      | Commitment detection: concentration-based and entropy-based                 |
| `deck_scorer.py`     | `deck_value()`: power + coherence + focus bonus                             |
| `draft_runner.py`    | `run_draft()`: main simulation loop; produces `DraftResult`                 |
| `metrics.py`         | Six metric families computed from `DraftResult`                             |
| `sweep.py`           | Parameter sweep orchestration; Cartesian product; CSV output                |
| `output.py`          | File writing: trace JSON, run-level CSV, aggregate CSV, config metadata     |
| `validation.py`      | Post-sweep calibration checks with numeric pass/fail bounds                 |
| `utils.py`           | Shared utilities (argmax, etc.)                                             |

Module dependency direction: `draft_simulator.py` → all others. Internal modules
import from `config`, `draft_models`, and `utils` freely. No circular imports.

## Configuration System

`SimulatorConfig` is a tree of frozen dataclasses with ten sections:

| Section           | Key Parameters                                                             |
| ----------------- | -------------------------------------------------------------------------- |
| `draft`           | `seat_count`, `round_count`, `picks_per_round`, `pack_size`, `human_seats` |
| `cube`            | `distinct_cards`, `copies_per_card`, `consumption_mode`                    |
| `pack_generation` | `strategy`, `archetype_target_count`, `primary_density`                    |
| `refill`          | `strategy`, `fingerprint_source`, `fidelity`, `commit_bias`                |
| `cards`           | `source`, `archetype_count`, `cards_per_archetype`, `bridge_fraction`      |
| `agents`          | `policy`, `show_n`, `show_n_strategy`, `ai_optimality`, `learning_rate`    |
| `scoring`         | `weight_power`, `weight_coherence`, `weight_focus`                         |
| `commitment`      | `commitment_threshold`, `stability_window`, `entropy_threshold`            |
| `metrics`         | `richness_gap`, `tau`, `on_plan_threshold`, `splash_power_threshold`       |
| `sweep`           | `runs_per_point`, `base_seed`, `axes`                                      |

Load order: defaults → config file → `--param` overrides → `--preset`. CLI
`--seed` overwrites `sweep.base_seed` after loading.

Dot-notation overrides use the form `section.param=value`. Types are inferred
from the current default: `int`, `float`, `bool`, or `str`. Lists and dicts
accept JSON encoding.

## Card Representation

Each card carries a fitness vector (per-archetype affinity, length
`archetype_count`, values in [0,1]), a `power` score (archetype-agnostic raw
strength), a `commit` score (archetype lock-in tendency), and a `flex` score
(archetype-agnostic flexibility). If `flex` is not provided, it is derived from
the fitness vector's Gini coefficient: `flex = 1.0 - gini(fitness)`.

Cards can be synthetically generated (configurable archetype count, bridge
fraction, power/commit distributions) or loaded from a JSON/TOML file. A
"bridge" card has high fitness in two or more archetypes; a "primary" card has
high fitness in exactly one.

## Pick Policies

Five policies govern card selection. All support epsilon-greedy noise via
`ai_optimality` (probability of picking optimally; at `0.8`, 20% of picks are
random):

- **greedy**: maximizes `deck_value(current_pool + [candidate])`; most expensive
  but holistic
- **archetype_loyal**: highest fitness for `argmax(w)`, ties broken by power
- **force**: highest fitness for a fixed `force_archetype` index; ignores `w`
  entirely; requires `agents.force_archetype` to be set
- **adaptive**:
  `0.3*power + 0.5*dot(fitness, normalize(w)) + ai_signal_weight *dot(fitness, openness)`;
  balances power, preference, and supply signal
- **signal_ignorant**: adaptive formula with uniform openness instead of actual
  supply signal; used as baseline for signal benefit measurement

Default policy is `adaptive`. The `force` policy requires
`agents.force_archetype` set explicitly (or `--param agents.force_archetype=N`).

### AI Difficulty

Difficulty is controlled by continuous parameters rather than a single enum:

| Parameter          | Effect                                                             |
| ------------------ | ------------------------------------------------------------------ |
| `ai_optimality`    | 0=random picks, 1=optimal. Controls noise in card scoring.         |
| `ai_signal_weight` | Weight of openness estimate in adaptive policy. 0=signal-ignorant. |
| `openness_window`  | Number of recent packs used for openness estimate.                 |
| `seat_count`       | More seats = more contention.                                      |

**Easy preset**: `ai_optimality=0.4`, `ai_signal_weight=0.0`, `seat_count=5`.
AIs pick semi-randomly and ignore signals, leaving ample good cards for the
human.

**Hard preset**: `ai_optimality=0.9`, `ai_signal_weight=0.8`, `seat_count=8`.
AIs pick near-optimally and read signals, creating heavy archetype contention
where the human must also read signals to draft well.

## Commitment Detection

Commitment is detected per-seat from the history of preference vectors `w`. Two
parallel methods run on every draft:

**Concentration-based (primary)**: a seat commits at pick `i` when
`max(w) / sum(w) >= commitment_threshold` and stays above threshold with the
same `argmax(w)` for the next `stability_window` picks.

**Entropy-based (secondary)**: same structure, using
`shannon_entropy(normalized_w) < entropy_threshold` as the trigger condition.

Default `commitment_threshold` is `0.35`, `stability_window` is `3`. With
default parameters and `adaptive` policy, real commit picks cluster in picks
4-8. If you observe zero or near-zero commitment rates, the threshold is too
high relative to how fast `w` concentrates under your card pool and learning
rate.

**Calibration warning**: default parameters produce low commitment rates with
purely uniform synthetic cards. To verify calibration, run `sweep` mode and
check the validation output — the commitment timing check targets mean pick in
[4, 8] and uncommitted rate below 10%. If the check fails, lower
`commitment_threshold` (try 0.25) or increase `agents.learning_rate`.

## Metrics

Six metric families are computed from each draft's `DraftResult`. Most require
`trace_enabled=True` in `run_draft()`, which is automatic in `trace` and `sweep`
modes.

Metrics are evaluated on two surfaces: **full-pack** (all cards in the pack,
measuring environment health) and **shown-N** (cards shown to the human seat,
measuring actual human experience). Results are bucketed into early (picks 0-5),
mid (6-19), and late (20-29) phases.

**Choice richness**: near-optimal count, score gap, and choice entropy for each
pick. Measures whether drafters have multiple viable options or are "on rails."

**Forceability**: `mean(force_deck_value[arch]) / mean(adaptive_deck_value)`
across runs, per archetype. Values near or above 1.0 indicate an archetype is
trivially forceable. Target: no archetype above 0.95 under the hard preset.
Requires cross-run data; only available in `sweep` mode.

**Signal benefit**: `(mean_aware - mean_ignorant) / mean_ignorant * 100%`
comparing adaptive vs signal-ignorant policy deck values across runs. Target:
below 2% at easy difficulty, 5-15% at hard difficulty. Requires sweep.

**Convergence**: on-plan card density in late picks after commitment. Measures
whether the pack continues delivering archetype-appropriate cards once a drafter
commits. Target: mean on-plan count >= 2.0 in shown-N during late phase.

**Splashability**: fraction of post-commitment picks containing a viable
off-plan card (high power or flex score, low on-plan fitness). Target: at least
40% of post-commitment picks present a viable off-plan option.

**Early openness**: distinct archetypes exposed in picks 0-4; Shannon entropy of
`w` in picks 0-5. Target: mean distinct archetypes >= 5 across first 5 picks.

## Sweep Output

Sweep mode writes three files to `--output-dir`:

- `runs_<timestamp>.csv`: one row per draft run with all per-seat metrics
- `aggregate_<timestamp>.csv`: one row per parameter combination with mean, std,
  p5, p95 for each metric
- `config_<timestamp>.json`: full config metadata and sweep axes

After writing CSVs, sweep automatically runs `validation.run_validation()` and
prints a calibration report. The report includes commitment timing, choice
richness baseline, archetype density, and metric stability checks with numeric
pass/fail bounds. A FAIL result indicates the parameter combination is outside
expected design ranges.

### Validation and Calibration

The validation suite checks distribution health and cross-configuration
consistency:

- **Archetype density**: each archetype should appear at a reasonable rate in
  packs (not too sparse, not too concentrated).
- **Commitment timing**: mean commit pick in [4, 8], standard deviation in
  \[1.5, 4.0\], uncommitted rate below 10%.
- **Choice richness baseline**: mean near-optimal count >= 1.5 across all picks.
- **Metric stability**: coefficient of variation across runs below threshold;
  flags high-variance metrics.

Cross-configuration comparisons verify directional effects:

- Refill mode should produce higher late-phase convergence than no-refill.
- More rounds (smaller packs) should increase early openness.
- Hard difficulty should produce higher signal benefit than easy.

When validation fails, recommended adjustments:

1. Commitment timing off-target → adjust `commitment_threshold`, `learning_rate`
2. Choice richness too low → increase `pack_size`, enable refill, or use
   curated/signal-rich show-N
3. Signal benefit out of range → adjust `ai_optimality`, `ai_signal_weight`,
   `seat_count`
4. Forceability too high → review card pool archetype distribution
5. Convergence too low → increase refill `fidelity` or lower `on_plan_threshold`

## Python Conventions

The simulator follows the same conventions as `scripts/quest_simulator/`:

- Frozen dataclasses for all data types (`@dataclass(frozen=True)`)
- Bare module imports (`import config`, not `from config import SimulatorConfig`
  at top, though `from config import SimulatorConfig` for frequently-used types
  is acceptable)
- RNG passed explicitly as `random.Random`; no global random state
- All type annotations use `Optional[X]` from `typing` (not `X | None`)
- No external dependencies; stdlib only

**Pyre namespace**: Pyre's `search_path` flattens all directories under
`scripts/` into one namespace. Any module name that collides with an existing
simulator module will cause a type error. The draft simulator uses the prefix
`draft_` on its model file (`draft_models.py`) to avoid collision with
`quest_simulator/models.py`. Any new module added to `draft_simulator/` should
use a unique name or the `draft_` prefix if the name is generic.

## Glossary

| Term            | Definition                                      |
| --------------- | ----------------------------------------------- |
| Cube            | The master card pool including all copies       |
| Pack            | A circulating set of cards within a round       |
| Seat            | A drafting position (human or AI)               |
| Pick            | Selecting one card from available choices       |
| Round           | A cycle of pack generation, picking, passing    |
| Show-N          | The N cards revealed to the human seat          |
| Commitment pick | Pick at which a seat locks into an archetype    |
| deck_value      | Canonical scalar score for a 30-card pool       |
| Fitness vector  | Per-archetype affinity scores for a card        |
| Signal reading  | Inferring archetype openness from pack contents |
| Sweep           | Running simulations across a parameter grid     |
| Forceability    | Viability of forcing one archetype repeatedly   |
| Bridge card     | Card with high fitness in multiple archetypes   |
| On-plan         | Aligned with the drafter's committed archetype  |
| Off-plan        | Not aligned with the committed archetype        |
| Splashable      | Off-plan card worth including for raw strength  |
