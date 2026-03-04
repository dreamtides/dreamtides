# Draft Simulator

The draft simulator at `scripts/draft_simulator/` is a standalone Python batch
tool that runs synthetic Dreamtides drafts under configurable parameters and
measures draft experience quality. It validates design goals: choice richness,
non-forceability, early openness, convergence after commitment, splashability,
and difficulty-dependent signal-reading impact — through quantitative metrics
and parameter sweeps.

No external dependencies. Stdlib-only. Requires Python 3.11+ for TOML config
support (JSON works on 3.9+).

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
- `--param KEY=VALUE`: dot-notation override, repeatable
  (example: `--param draft.seat_count=8 --param agents.policy=greedy`)
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

## Module Layout

Seventeen modules with clear ownership boundaries:

| Module | Role |
|---|---|
| `draft_simulator.py` | Entry point: CLI parsing, mode dispatch |
| `config.py` | `SimulatorConfig` dataclass tree; TOML/JSON loading; dot-notation overrides |
| `draft_models.py` | Core data types: `CardDesign`, `CardInstance`, `Pack`, enums |
| `card_generator.py` | Synthetic card pool generation with fitness vectors |
| `cube_manager.py` | `CubeManager`: card supply tracking, with/without replacement |
| `pack_generator.py` | Pack generation strategies: uniform, rarity_weighted, seeded_themed |
| `refill.py` | Pack refill strategies: no_refill, uniform_refill, constrained_refill |
| `agents.py` | `AgentState`; five pick policies; openness estimation |
| `show_n.py` | Show-N selection strategies for the human seat |
| `commitment.py` | Commitment detection: concentration-based and entropy-based |
| `deck_scorer.py` | `deck_value()`: power + coherence + focus bonus |
| `draft_runner.py` | `run_draft()`: main simulation loop; produces `DraftResult` |
| `metrics.py` | Six metric families computed from `DraftResult` |
| `sweep.py` | Parameter sweep orchestration; Cartesian product; CSV output |
| `output.py` | File writing: trace JSON, run-level CSV, aggregate CSV, config metadata |
| `validation.py` | Post-sweep calibration checks with numeric pass/fail bounds |
| `utils.py` | Shared utilities (argmax, etc.) |

Module dependency direction: `draft_simulator.py` → all others. Internal
modules import from `config`, `draft_models`, and `utils` freely. No circular
imports.

## Configuration System

`SimulatorConfig` is a tree of frozen dataclasses with ten sections:

| Section | Key Parameters |
|---|---|
| `draft` | `seat_count`, `round_count`, `picks_per_round`, `pack_size`, `human_seats` |
| `cube` | `distinct_cards`, `copies_per_card`, `consumption_mode` |
| `pack_generation` | `strategy`, `archetype_target_count`, `primary_density` |
| `refill` | `strategy`, `fingerprint_source`, `fidelity`, `commit_bias` |
| `cards` | `source`, `archetype_count`, `cards_per_archetype`, `bridge_fraction` |
| `agents` | `policy`, `show_n`, `show_n_strategy`, `ai_optimality`, `learning_rate` |
| `scoring` | `weight_power`, `weight_coherence`, `weight_focus` |
| `commitment` | `commitment_threshold`, `stability_window`, `entropy_threshold` |
| `metrics` | `richness_gap`, `tau`, `on_plan_threshold`, `splash_power_threshold` |
| `sweep` | `runs_per_point`, `base_seed`, `axes` |

Load order: defaults → config file → `--param` overrides → `--preset`. CLI
`--seed` overwrites `sweep.base_seed` after loading.

Dot-notation overrides use the form `section.param=value`. Types are inferred
from the current default: `int`, `float`, `bool`, or `str`. Lists and dicts
accept JSON encoding.

## Pick Policies

Five policies govern card selection. All support epsilon-greedy noise via
`ai_optimality` (probability of picking optimally; at `0.8`, 20% of picks are
random):

- **greedy**: maximizes `deck_value(current_pool + [candidate])`; most
  expensive but holistic
- **archetype_loyal**: highest fitness for `argmax(w)`, ties broken by power
- **force**: highest fitness for a fixed `force_archetype` index; ignores `w`
  entirely; requires `agents.force_archetype` to be set
- **adaptive**: `0.3*power + 0.5*dot(fitness, normalize(w)) + ai_signal_weight
  *dot(fitness, openness)`; balances power, preference, and supply signal
- **signal_ignorant**: adaptive formula with uniform openness instead of actual
  supply signal; used as baseline for signal benefit measurement

Default policy is `adaptive`. The `force` policy requires
`agents.force_archetype` set explicitly (or `--param agents.force_archetype=N`).

## Commitment Detection

Commitment is detected per-seat from the history of preference vectors `w`.
Two parallel methods run on every draft:

**Concentration-based (primary)**: a seat commits at pick `i` when
`max(w) / sum(w) >= commitment_threshold` and stays above threshold with the
same `argmax(w)` for the next `stability_window` picks.

**Entropy-based (secondary)**: same structure, using
`shannon_entropy(normalized_w) < entropy_threshold` as the trigger condition.

Default `commitment_threshold` is `0.35`, `stability_window` is `3`. With
default parameters and `adaptive` policy, real commit picks cluster in picks
4-8. If you observe zero or near-zero commitment rates, the threshold is
too high relative to how fast `w` concentrates under your card pool and
learning rate.

**Calibration warning**: default parameters produce low commitment rates with
purely uniform synthetic cards. To verify calibration, run `sweep` mode and
check the validation output — the commitment timing check targets mean pick in
[4, 8] and uncommitted rate below 10%. If the check fails, lower
`commitment_threshold` (try 0.25) or increase `agents.learning_rate`.

## Metrics

Six metric families are computed from each draft's `DraftResult`. Most require
`trace_enabled=True` in `run_draft()`, which is automatic in `trace` and
`sweep` modes.

**Choice richness**: near-optimal count, score gap, and choice entropy for each
pick, bucketed into early (picks 0-5), mid (6-19), and late (20-29) phases.
Computed on both the full pack and the shown-N subset.

**Forceability**: `mean(force_deck_value[arch]) / mean(adaptive_deck_value)`
across runs, per archetype. Requires cross-run data; only available in `sweep`
mode.

**Signal benefit**: `(mean_aware - mean_ignorant) / mean_ignorant * 100%`
comparing adaptive vs signal-ignorant policy deck values across runs. Requires
sweep.

**Convergence**: on-plan card density in late picks after commitment. Measures
whether the pack continues delivering archetype-appropriate cards once a drafter
commits.

**Splashability**: fraction of post-commitment picks containing a viable
off-plan card (high power or flex score, low on-plan fitness).

**Early openness**: distinct archetypes exposed in picks 0-4; Shannon entropy
of `w` in picks 0-5.

## Sweep Output

Sweep mode writes three files to `--output-dir`:

- `runs_<timestamp>.csv`: one row per draft run with all per-seat metrics
- `aggregate_<timestamp>.csv`: one row per parameter combination with mean,
  std, p5, p95 for each metric
- `config_<timestamp>.json`: full config metadata and sweep axes

After writing CSVs, sweep automatically runs `validation.run_validation()` and
prints a calibration report. The report includes commitment timing, choice
richness baseline, archetype density, and metric stability checks with numeric
pass/fail bounds. A FAIL result indicates the parameter combination is outside
expected design ranges.

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
