# Dreamtides Draft Simulator — Technical Design Document

## 1. Goal and Scope

### 1.1 Primary Goal

Build a batch simulator that runs many synthetic Dreamtides drafts under
configurable parameters and outputs quantitative metrics measuring draft
experience quality. The simulator enables parameter sweeps to evaluate design
goals: choice richness, non-forceability, convergence after commitment, early
openness, splashability, and difficulty-dependent signal-reading impact.

### 1.2 Technology

The simulator is implemented in Python, type-checked with Pyre. No test suite is
included. Code under scripts/draft_simulator/. High quality, modular.

### 1.3 Non-Goals

- No post-draft gameplay or match simulation.
- No rules-text parsing or integration with the Dreamtides rules engine.
- No heavy manual card tagging beyond the minimal attributes defined in section
  4\.

### 1.4 Determinism

Given identical configuration and RNG seed, the simulator produces identical
results including per-pick traces and aggregate metrics. All random operations
consume from a single seeded PRNG instance (or a deterministic tree of child
PRNGs) in a fixed, documented order. No operation may use a non-deterministic
entropy source.

______________________________________________________________________

## 2. Draft Model

### 2.1 Table Configuration

| Parameter     | Description                   | Default |
| ------------- | ----------------------------- | ------- |
| `seat_count`  | Total seats (human + AI)      | 6       |
| `human_seats` | Number of human-modeled seats | 1       |

Seats are arranged in a fixed circular order. Packs pass to the left (seat index
\+ 1, wrapping around) by default. The human seat is always seat index 0.

| Parameter             | Description                        | Default |
| --------------------- | ---------------------------------- | ------- |
| `alternate_direction` | Alternate pass direction per round | false   |

When `alternate_direction` is true, odd-numbered rounds (1, 3, ...) pass to the
right (seat index - 1, wrapping). When false, all rounds pass left.

### 2.2 Total Picks

Each seat drafts exactly 30 cards total. The drafted pool IS the final deck;
there is no cut or sideboard construction step.

### 2.3 Round Structure

| Parameter         | Description                 | Default      |
| ----------------- | --------------------------- | ------------ |
| `round_count`     | Number of rounds            | 3            |
| `picks_per_round` | Ordered list of pick counts | [10, 10, 10] |

Constraint: `sum(picks_per_round)` must equal 30. The list length must equal
`round_count`. Each entry `picks_per_round[r]` specifies how many picks occur in
round `r`.

### 2.4 Pack Configuration

| Parameter   | Description                | Default |
| ----------- | -------------------------- | ------- |
| `pack_size` | Cards per pack at creation | 15      |

At the start of each round, one pack is generated per seat (see section 3).
Packs carry a persistent unique identity within a round, enabling tracking as
they circulate.

### 2.5 Pick Loop (Within a Round)

For each pick index `k` in `0 .. picks_per_round[r] - 1`:

1. Each seat has exactly one pack in front of it.
2. The seat selects one card from its available choices. Observability rules
   (section 5) restrict which cards the human seat may choose from.
3. The selected card is removed from the pack and appended to that seat's
   drafted pool.
4. All packs rotate one position to the left (seat `s` passes its pack to seat
   `(s + 1) % seat_count`).
5. If the active refill policy is not "no refill," each pack receives exactly
   one card according to the refill strategy (section 3.4).

### 2.6 End of Round

After all picks in a round complete, remaining pack contents are discarded. They
are not returned to the cube. The next round begins with fresh pack generation.

### 2.7 Constraints: picks_per_round vs pack_size

- **No-refill mode**: `picks_per_round[r] <= pack_size` for every round `r`. The
  simulator rejects configurations that violate this constraint.
- **Refill mode**: `picks_per_round[r]` may exceed `pack_size`. The simulator
  emits a warning (not an error) if any round's pick count exceeds
  `2 * pack_size`, as extreme ratios risk degenerate pack composition drift.

______________________________________________________________________

## 3. Cube, Duplicates, Pack Generation, and Refill

### 3.1 Cube Definition

The cube is the master card pool from which all packs and refills draw. It is
constructed from a set of distinct card designs, each with a configurable copy
count that encodes rarity through multiplicity.

| Parameter         | Description                                       | Default |
| ----------------- | ------------------------------------------------- | ------- |
| `distinct_cards`  | Number of unique card designs                     | 360     |
| `copies_per_card` | Per-card copy count (uniform int or per-card map) | 1       |

When specified as an integer, every card design receives that many copies. When
specified as a map (`{card_id: count}`), each card receives its own copy count;
cards not present in the map default to 1. The physical cube size is
`sum(copies_per_card[c] for all cards c)`. A flat cube (1 copy each) has size
equal to `distinct_cards`.

### 3.2 Cube Consumption Modes

| Mode                  | Behavior                            |
| --------------------- | ----------------------------------- |
| `without_replacement` | Cards drawn are removed from supply |
| `with_replacement`    | Cards are sampled without depleting |

**Exhaustion policy (without_replacement only):** The simulator validates the
configuration before execution. It computes the maximum possible card demand:

```
max_demand = seat_count * pack_size * round_count
             + total_refills
```

where `total_refills` equals the total number of refill events across all rounds
(one per seat per pick, for rounds where refill is active). If `max_demand`
exceeds the physical cube size, the configuration is rejected with a diagnostic
message identifying the shortfall. There is no fallback to replacement mode; the
operator must adjust parameters to ensure supply sufficiency.

### 3.3 Pack Generation Strategies

At the start of each round, one pack per seat is generated using the active
strategy. The strategy is selected per experiment via configuration.

**Strategy: Uniform** Sample `pack_size` cards from the cube with uniform
probability over physical copies (each copy equally likely). Respects the active
consumption mode.

**Strategy: Rarity-weighted** Sample `pack_size` cards with per-card weights
beyond physical multiplicity.

| Parameter        | Description                                     | Default     |
| ---------------- | ----------------------------------------------- | ----------- |
| `rarity_weights` | Map from card_id to sampling weight (float > 0) | 1.0 for all |

When all weights are 1.0, this is equivalent to uniform sampling over copies.

**Strategy: Seeded/Themed** Collated generation that controls archetype density
within packs to reduce variance and ensure each pack carries a readable
archetype profile.

| Parameter                | Description                                | Default |
| ------------------------ | ------------------------------------------ | ------- |
| `archetype_target_count` | Target archetypes represented per pack     | 4       |
| `primary_density`        | Fraction of pack for top 1-2 archetypes    | 0.5     |
| `bridge_density`         | Fraction of pack for multi-archetype cards | 0.15    |
| `variance`               | Randomness in archetype selection (0-1)    | 0.3     |

Algorithm outline (implementation may refine details):

1. Select `archetype_target_count` archetypes for this pack. Selection is
   weighted by cube availability; `variance` controls how much randomness is
   added (0 = deterministic top-N by supply, 1 = fully random).
2. Designate the first 1-2 selected archetypes as primary. Fill
   `floor(pack_size * primary_density)` slots with cards whose highest fitness
   aligns with a primary archetype, preferring higher fitness values.
3. Fill `floor(pack_size * bridge_density)` slots with bridge cards (fitness
   above 0.5 in at least two of the selected archetypes).
4. Fill remaining slots by uniform sampling from the cube.

The implementation agent may choose the exact weighting and tie-breaking
mechanics. The key invariant is that each generated pack has a coherent
archetype "fingerprint" such that a signal-reading agent can infer which
archetypes are well-represented.

New pack generation strategies are added by implementing the pack generator
interface (section 10) without modifying the core simulator.

### 3.4 Refill Strategies

After packs are passed at each pick step, the active refill strategy adds
exactly one card to each pack (if enabled). The refill strategy is selected per
experiment via configuration.

**Strategy: No refill (classic depletion)** No card is added. Packs shrink by
one card per pick.

**Strategy: Uniform refill** Add one card sampled uniformly from cube supply,
respecting the active consumption mode.

**Strategy: Constrained refill** Add one card sampled from a distribution
conditioned on the pack's identity. The purpose of constrained refill is to
preserve signal readability: as a pack circulates, its archetype composition
remains statistically similar to its original profile, so downstream seats can
infer archetype openness from pack contents without the signal being diluted by
random refills.

| Parameter            | Description                             | Default       |
| -------------------- | --------------------------------------- | ------------- |
| `fingerprint_source` | Conditioning signal source              | `pack_origin` |
| `fidelity`           | Strength of conditioning (0.0-1.0)      | 0.7           |
| `commit_bias`        | Bias toward high/low commit cards (0-1) | 0.3           |

`fingerprint_source` options:

- `pack_origin`: condition on the pack's archetype profile snapshot taken at
  creation time.
- `round_environment`: condition on the round's global archetype distribution
  across all generated packs.

The sampling weight for each candidate card `c` is computed as:

```
similarity = dot(c.fitness, signal) / (norm(c.fitness) * norm(signal) + eps)
weight = (1 - fidelity) + fidelity * similarity
weight *= (1 - commit_bias) + commit_bias * c.commit
```

where `signal` is the conditioning vector (pack origin profile or round
environment profile depending on `fingerprint_source`). At `fidelity=0.0` all
candidates are equally weighted (uniform refill); at `fidelity=1.0` weight is
dominated by cosine similarity to the conditioning signal. `commit_bias` tilts
the refill toward high-commit cards when high, or toward low-commit cards when
low. The implementation agent may substitute an alternative similarity function
if cosine similarity proves inadequate, provided the `fidelity` 0-to-1
interpolation semantics are preserved.

______________________________________________________________________

## 4. Card Representation

### 4.1 Required Attributes

Each card instance carries the following attributes:

| Attribute | Type                  | Range      | Description                     |
| --------- | --------------------- | ---------- | ------------------------------- |
| `card_id` | str                   | unique     | Unique design identifier        |
| `name`    | str                   | —          | Display name                    |
| `fitness` | list of float (len 8) | [0,1] each | Archetype fitness vector        |
| `power`   | float                 | [0,1]      | Archetype-agnostic raw strength |
| `commit`  | float                 | [0,1]      | Archetype lock-in tendency      |
| `flex`    | float                 | [0,1]      | Archetype-agnostic flexibility  |

### 4.2 Flex Derivation

If `flex` is not explicitly provided for a card, it is derived from the fitness
vector's dispersion using the Gini coefficient: `flex = 1.0 - gini(fitness)`. A
Gini of 0 (perfectly uniform fitness) yields maximum flexibility (1.0); a Gini
of 1 (all fitness concentrated in one archetype) yields minimum flexibility
(0.0). Explicit overrides take precedence over derivation.

### 4.3 Copy Instance Identity

Each physical copy in the cube is a distinct instance sharing the same
card-level attributes but carrying a unique `instance_id`. This enables tracking
which specific copy was drafted by which seat and prevents the same instance
from appearing in two places simultaneously.

### 4.4 Data Sourcing

The card pool may be sourced in two ways:

**Synthetic generation**: A card pool generator component produces cards
procedurally using configurable distributions.

| Parameter             | Description                        | Default          |
| --------------------- | ---------------------------------- | ---------------- |
| `archetype_count`     | Number of archetypes               | 8                |
| `cards_per_archetype` | Primary cards per archetype        | 45               |
| `bridge_fraction`     | Fraction of cards that are bridges | 0.15             |
| `power_distribution`  | Distribution for power values      | uniform(0.2,0.9) |
| `commit_distribution` | Distribution for commit values     | beta(2, 3)       |

A bridge card has high fitness (above 0.5) in two or more archetypes. A primary
card has high fitness in exactly one archetype. The generator ensures the
high-level expectation: per archetype, roughly 45 cards with fitness above 0.5
and roughly 30 additional cards with fitness in \[0.3, 0.5), with overlap across
archetypes for bridge cards.

**File-loaded**: Read from a structured JSON or TOML file where each entry
specifies all required attributes.

### 4.5 Validation

Before simulation, all cards are validated:

- All fitness values are in [0, 1].
- `power`, `commit`, `flex` are each in [0, 1].
- Fitness vector length equals `archetype_count`.
- No card has an all-zero fitness vector.
- Warning (not error) for cards where `max(fitness) < 0.3`, indicating a card
  that is weak in every archetype.
- Warning for cards with `power < 0.1` and `commit > 0.7` (high-commitment but
  very weak, likely a data entry error).

______________________________________________________________________

## 5. Observability and Agent Interfaces

### 5.1 Observability Rules

| Seat type | Sees full pack? | Action space         |
| --------- | --------------- | -------------------- |
| AI        | Yes             | Any card in the pack |
| Human     | No              | Only the shown cards |

### 5.2 Human Seat Visible Metadata

At each pick, the human seat observes:

- The shown cards (selected by the show-N policy; default N=4).
- The current pick index within the round (0-based).
- The current round index (0-based).
- The pack's unique ID (enabling tracking across hops).
- The number of cards remaining in the pack (but NOT which hidden cards remain).
- The human seat's own drafted pool so far.

The human seat does NOT observe:

- Hidden cards in the current pack.
- Other seats' drafted pools or archetype preferences.
- Which cards other seats have picked.

### 5.3 Show-N Policy

The show-N system selects exactly N cards from the current pack to present to
the human seat. N is configurable.

| Parameter | Description                    | Default |
| --------- | ------------------------------ | ------- |
| `show_n`  | Number of cards shown to human | 4       |

Constraint: if the pack contains fewer than N cards, all remaining cards are
shown (show-N degrades gracefully).

Show-N is a deterministic function of (pack contents, human agent state,
strategy configuration, RNG state). Given the same inputs, it always produces
the same selection.

**Strategy: Uniform** Select N cards uniformly at random from the pack.

**Strategy: Power-biased** Select N cards with probability proportional to each
card's `power` value, biasing the human toward seeing stronger cards.

**Strategy: Curated** Select cards to balance experience goals:

- At least 1 card that is on-plan for the human's current top archetype (fitness
  for best archetype >= 0.6), if any such card exists in the pack.
- At least 1 card that is off-plan (fitness for best archetype < 0.3) but strong
  (power >= 0.5), if any exists.
- Remaining slots filled by power-weighted sampling from the rest of the pack.
- If constraints cannot be satisfied, fall back to power-biased.

**Strategy: Signal-rich** Select cards biased toward high `commit` values,
ensuring the human sees archetype-defining signpost cards that carry information
about archetype openness. Selection probability is proportional to
`commit * 2 + power` for each card.

New show-N strategies are added by implementing the show-N strategy interface
(section 10).

### 5.4 Agent State

Each agent (AI or human model) maintains:

- `w`: archetype preference vector (float array of length `archetype_count`),
  initialized to uniform values, updated after each pick.
- `drafted`: ordered list of card instances drafted so far.
- `pick_history`: ordered list of (pick_index, round_index, card_id, pack_id)
  tuples.

### 5.5 Pick Policies

All seats (AI and human-modeled) select one card per pick according to their
assigned policy. AI seats choose from the full pack; human-modeled seats choose
from the shown-N subset only. Any policy may be assigned to any seat type; the
only difference is the candidate card set provided to the policy.

**Policy: Greedy** Pick the card that maximizes immediate improvement to
`deck_value`. For each candidate card, compute `deck_value` of
`(current_pool + [candidate])` using the agent's current `w` as the preference
vector (not the final `w`, which is unknown mid-draft). Select the candidate
with the highest `deck_value`. This is O(N) `deck_value` evaluations per pick
where N is the candidate set size.

**Policy: Archetype-loyal** Pick the card with the highest fitness for the
agent's current top archetype (`argmax(w)`), breaking ties by `power`. The agent
commits early and stays loyal.

**Policy: Force** A variant of archetype-loyal used for forceability measurement
(section 8.4). The agent is assigned a target archetype at initialization and
always picks to maximize fitness for that archetype, breaking ties by `power`.
Unlike archetype-loyal, the target never changes regardless of `w` updates.

**Policy: Adaptive** Pick using a weighted scoring function:

```
score(card) = α * card.power
            + β * dot(card.fitness, normalize(w))
            + γ * dot(card.fitness, openness)
```

where `α`, `β`, `γ` are configurable per-agent weights (defaults: `α=0.3`,
`β=0.5`, `γ=0.2`). The `γ` weight corresponds to `ai_signal_weight` from section
5.6.

**Openness estimate**: a per-archetype running signal maintained by each agent.
For each archetype `a`, after each pick the agent updates:

```
openness[a] = mean(supply_signal[a] over last W packs seen)
```

where `supply_signal[a]` for a pack is the mean fitness-for-archetype-`a` of all
cards in the pack (or shown cards, for human seats), and `W` is the
`openness_window` parameter (default: 3). All `openness[a]` values are
initialized to `1.0 / archetype_count`. The implementation agent may refine the
exact update formula (e.g., exponential moving average instead of simple window
mean) provided the directional semantics are preserved: more high-fitness cards
seen for an archetype increases its openness estimate.

**Policy: Signal-ignorant** Identical to adaptive but the openness estimate is
held constant at its initial value (uniform). This policy ignores all signal
from pack contents and serves as a control for measuring signal benefit in
metrics.

### 5.6 AI Difficulty Parameterization

Difficulty is controlled by continuous parameters, not a single enum. The
following knobs are independently configurable:

| Parameter          | Range  | Effect                                                                           |
| ------------------ | ------ | -------------------------------------------------------------------------------- |
| `ai_optimality`    | [0, 1] | 0=random picks; 1=optimal. Controls noise injected into card scoring.            |
| `ai_signal_weight` | [0, 1] | Weight of openness estimate in adaptive policy (maps to `γ`). 0=signal-ignorant. |
| `openness_window`  | int    | Number of recent packs used for openness estimate. Default: 3.                   |
| `seat_count`       | int    | More seats = more contention.                                                    |

**Easy preset**: `ai_optimality=0.4`, `ai_signal_weight=0.0`, `seat_count=5`.
AIs pick semi-randomly and ignore signals, leaving ample good cards for the
human.

**Hard preset**: `ai_optimality=0.9`, `ai_signal_weight=0.8`, `seat_count=8`.
AIs pick near-optimally and read signals, creating heavy archetype contention
where the human must also read signals to draft well.

Presets are syntactic sugar over the underlying parameters; any individual
parameter may be overridden.

______________________________________________________________________

## 6. Canonical Outcome Score

### 6.1 Purpose

`deck_value` provides a single scalar score for a drafted 30-card pool, enabling
quantitative comparison of draft outcomes across agents, policies, and
configurations. It is the ground-truth measure used by metrics that compare
policy effectiveness.

### 6.2 Inputs

- The 30 drafted cards with their attributes (fitness, power, commit, flex).
- The agent's final archetype preference vector `w`.

### 6.3 Components

`deck_value` is a weighted combination of three components, each normalized to
\[0, 1\]:

**Component 1: Raw Power** The mean of `power` across all 30 cards. Rewards
drafting individually strong cards regardless of archetype coherence.

**Component 2: Archetype Coherence** Measures how well the pool's fitness aligns
with a focused archetype strategy. The effective archetype is determined by
`argmax(w)`. Coherence is the mean fitness of all 30 cards for the effective
archetype. To reward viable two-archetype (bridge) strategies, coherence
optionally includes the second-highest archetype's mean fitness, discounted by
`secondary_weight`.

**Component 3: Focus Bonus** Rewards archetype concentration in the pool.
Defined as the fraction of cards with fitness for the effective archetype at or
above `focus_threshold`, mapped through a diminishing-returns curve that
saturates at `focus_saturation`. A pool where 70% of cards are on-plan receives
nearly full focus bonus; a pool at 40% receives partial bonus.

### 6.4 Tunable Parameters

| Parameter          | Description                          | Default |
| ------------------ | ------------------------------------ | ------- |
| `weight_power`     | Weight of raw power component        | 0.3     |
| `weight_coherence` | Weight of archetype coherence        | 0.5     |
| `weight_focus`     | Weight of focus bonus                | 0.2     |
| `focus_threshold`  | Fitness threshold for on-plan card   | 0.5     |
| `focus_saturation` | On-plan fraction for max bonus       | 0.7     |
| `secondary_weight` | Discount for 2nd archetype coherence | 0.3     |

### 6.5 Properties

- Output range: [0, 1].
- Monotonically increasing in power when coherence is held fixed.
- Rewards focused pools over unfocused ones, but does not require perfect
  single-archetype focus (the secondary_weight allows viable bridge strategies
  to score well).
- Deterministic given the same pool and `w`.

______________________________________________________________________

## 7. Commitment Definition

### 7.1 Primary Definition: Concentration Threshold with Stability

A seat's **commitment pick** is the earliest pick index `i` such that both
conditions hold:

1. **Concentration condition**: The seat's archetype preference vector `w` at
   pick `i` has concentration `C(w) = max(w) / sum(w)` at or above
   `commitment_threshold`.

2. **Stability condition**: The concentration condition remains satisfied AND
   `argmax(w)` does not change for the next `stability_window` consecutive picks
   after `i`.

If no pick satisfies both conditions by the end of the draft, the seat is
classified as uncommitted (commitment_pick is null).

### 7.2 Preference Vector Update Rule

After each pick, the agent updates `w` by adding the picked card's fitness
vector scaled by `learning_rate`:

```
w[a] += learning_rate * picked_card.fitness[a]
```

for each archetype `a`. The vector `w` is NOT renormalized after updates;
concentration `C(w)` is computed on the raw accumulated vector. The initial
value of each `w[a]` is `1.0 / archetype_count`.

### 7.3 Tunable Parameters

| Parameter              | Description                      | Default |
| ---------------------- | -------------------------------- | ------- |
| `commitment_threshold` | C(w) required for commitment     | 0.35    |
| `stability_window`     | Consecutive confirming picks     | 3       |
| `learning_rate`        | Speed of preference accumulation | 1.0     |

### 7.4 Calibration Targets

The defaults are chosen to target:

- Mean commitment pick across all seats and runs: approximately 6.
- Standard deviation: approximately 2.5 picks (roughly 40% of the mean).
- Uncommitted rate: below 5% of seats.

These targets are verified during calibration (section 11).

### 7.5 Secondary Definition: Entropy-Based

For supplementary analysis, an alternative commitment measure uses the Shannon
entropy of the normalized `w`. Commitment occurs when
`H(normalized_w) < entropy_threshold` (default: 2.0 bits, corresponding to
roughly equal weight spread across 4 archetypes) with the same stability
requirement. This secondary definition is computed alongside the primary but is
not used for the acceptance targets defined in section 8.

______________________________________________________________________

## 8. Metrics

### 8.1 Evaluation Surfaces

Each metric is computed on one or both evaluation surfaces:

- **Full-pack (F)**: all cards in the pack at the time of the pick, measuring
  environment health and draft structure quality.
- **Shown-N (S)**: only the cards shown to the human seat, measuring actual
  human draft experience.

The applicable surface is noted in brackets after each metric.

### 8.2 Pick Phases

Metrics are reported per-phase as well as overall. Phases are defined by pick
index (0-based, counting across all rounds):

- **Early**: picks 0 through 5.
- **Mid**: picks 6 through 19.
- **Late**: picks 20 through 29.

### 8.3 Choice Richness — "Not On Rails"

**Near-optimal count [F, S]** At each pick, count the number of available cards
whose score (using the picking agent's own scoring function) falls within
`richness_gap` (default: 0.1) of the best available card's score. Report: mean
and distribution across picks, broken out by phase.

**Score gap [F, S]** The difference between the best and second-best card scores
at each pick. Lower gaps indicate richer choices. Report: mean, median, and 90th
percentile, broken out by phase.

**Choice entropy [S]** Shannon entropy of card scores (after softmax
normalization with temperature `tau`, default: 1.0) among the shown cards.
Higher entropy indicates more balanced options. Report: mean by phase.

### 8.4 Forceability

**Force-policy comparison [F]** Run controlled experiments comparing two
populations:

1. Force-policy agents that commit to a predetermined target archetype at pick 0
   and maximize fitness for that archetype throughout the draft.
2. Adaptive-policy agents that use the standard adaptive policy with signal
   reading.

For each, compute:

- Mean `deck_value` across runs.
- Per-archetype "force success rate": fraction of runs where the forced
  archetype yields deck_value above the population median.

**Forceability index [F]** For each archetype `a`:

```
forceability[a] = mean_deck_value(force, archetype=a)
                  / mean_deck_value(adaptive)
```

Values near or above 1.0 indicate the archetype is trivially forceable.
Acceptance target: no archetype should have forceability above 0.95 under the
hard difficulty preset.

### 8.5 Signal Benefit

**Signal-aware vs signal-ignorant comparison [F]** Run the same draft
configuration with two agent populations:

1. Signal-aware agents (adaptive policy with signal reading).
2. Signal-ignorant agents (adaptive policy without signal reading; openness
   estimate held constant).

Compute:

- `signal_benefit`: the difference in mean deck_value between signal-aware and
  signal-ignorant, expressed as a percentage of the signal-ignorant mean.

**Acceptance targets (configurable per experiment):**

| Difficulty | Signal benefit range | Force penalty                       |
| ---------- | -------------------- | ----------------------------------- |
| Easy       | below 2%             | Not required                        |
| Hard       | 5% to 15%            | Adaptive beats force by at least 5% |

These thresholds are configurable in the experiment definition and represent
default acceptance criteria.

### 8.6 Convergence (Post-Commitment)

**On-plan density [S]** After the commitment pick, for each subsequent pick,
count the number of shown cards with fitness for the committed archetype at or
above `on_plan_threshold` (default: 0.5).

Targets:

- Mean on-plan count in shown-N during the late phase: at least 2.0.
- Probability of on-plan count at least 3 during the late phase: at least 0.15.

**On-plan density [F]** Same metric computed on the full pack, measuring whether
the environment provides sufficient on-plan material regardless of the show-N
filter.

### 8.7 Splashability

**Off-plan option frequency [S]** At each post-commitment pick, check whether at
least one shown card satisfies BOTH:

- fitness for the committed archetype below 0.3 (not on-plan), AND
- either `power >= splash_power_threshold` (default: 0.5) OR
  `flex >= splash_flex_threshold` (default: 0.6).

Report the fraction of post-commitment picks where at least one such splashable
card appears.

Target: at least 0.4 (at least 40% of post-commitment picks present a viable
off-plan option).

**Off-plan option frequency [F]** Same computation on the full pack.

### 8.8 Early Openness

**Archetype exposure [S]** In picks 0 through 4, count the number of distinct
archetypes that appear among shown cards with fitness at or above
`exposure_threshold` (default: 0.5) for at least one archetype.

Target: mean distinct archetypes seen across the first 5 picks is at least 5.

**Preference entropy [F]** Mean Shannon entropy of agents' archetype preference
vectors `w` during picks 0 through 5. Higher entropy indicates agents remain
open and have not been forced into premature commitments by the card
distribution.

### 8.9 Aggregation

All metrics are computed at three levels:

1. **Per-draft**: single simulation run values.
2. **Per-configuration**: aggregated across all runs sharing the same parameter
   set. Reports: mean, standard deviation, 5th / 25th / 50th / 75th / 95th
   percentiles.
3. **Across sweep axes**: tabulated by swept parameter values for comparison.

______________________________________________________________________

## 9. Sweep Mode and Experiment Workflow

### 9.1 Experiment Definition

An experiment is a declarative specification consisting of:

**Fixed parameters**: all parameters from sections 2 through 7 that are held
constant across the sweep.

**Sweep axes**: a list of parameter names and the set of values to sweep for
each. The sweep runner computes the full Cartesian product of all axes and runs
simulations for every combination.

**Execution parameters**:

| Parameter        | Description                           | Default      |
| ---------------- | ------------------------------------- | ------------ |
| `runs_per_point` | Simulations per parameter combination | 1000         |
| `base_seed`      | Base RNG seed                         | 42           |
| `seeding_policy` | How per-run seeds are derived         | `sequential` |
| `trace_enabled`  | Whether to emit per-pick traces       | false        |

**Seeding policy options:**

- `sequential`: run `i` uses seed `base_seed + i`.
- `hashed`: run `i` uses a hash of `(base_seed, config_hash, i)`, ensuring
  different configurations do not share seed sequences even when using the same
  base_seed.

### 9.2 Configuration Schema

All parameters are organized hierarchically: `draft` (seat_count, round_count,
picks_per_round, pack_size), `cube` (distinct_cards, copies_per_card,
consumption_mode), `pack_generation` (strategy + params), `refill` (strategy +
params), `cards` (sourcing mode, generation params, file path), `agents`
(per-seat policy, AI difficulty params, show_n, show_n_strategy), `scoring`
(deck_value weights), `commitment` (threshold, stability_window, learning_rate),
`metrics` (per-metric thresholds), `sweep` (axes, runs_per_point, seeding,
output config).

Format: TOML for human-authored configs; JSON for programmatic generation. The
simulator validates against the schema before execution. A default config ships
with the simulator containing all defaults from this document; experiments
override specific values.

### 9.3 Output Schema

Each experiment produces the following outputs:

**Run-level output (one record per simulation run):**

- `run_id`: unique identifier (sequential within experiment).
- `config_hash`: deterministic hash of the full parameter set.
- `seed`: the RNG seed used for this run.
- All per-draft metric values from section 8.
- Per-seat data: commitment_pick, deck_value, final archetype (argmax of w),
  final archetype preference vector.

**Aggregate output (one record per parameter combination):**

- All fixed and swept parameter values for this combination.
- Per-metric: mean, standard deviation, 5th / 25th / 50th / 75th / 95th
  percentiles.
- Per-metric acceptance target pass/fail flags.

**Per-pick trace (emitted only when trace_enabled is true):**

- Per pick: round index, pick index, seat index, pack_id, full pack contents
  (list of card_ids), shown cards (for human seat), chosen card_id, agent `w`
  snapshot, card score.

**Output formats:**

- Parquet: default for run-level and aggregate outputs (efficient, typed,
  columnar).
- JSON: for per-pick traces (human-readable, nested structure).
- CSV: optional export target for run-level and aggregate data.

### 9.4 Reproducibility Guarantees

1. The combination of configuration and base_seed fully determines all
   simulation outputs.
2. The full configuration is embedded in every output file as metadata.
3. The simulator version string is recorded in all outputs.
4. Re-running with the same configuration and seed produces byte-identical
   Parquet outputs (deterministic serialization and deterministic key ordering).

______________________________________________________________________

## 10. System Architecture

### 10.1 Components

The system consists of the following components. Each is defined by its
interface contract (inputs, outputs, configuration).

**CardPool**: Loads or generates card definitions per section 4. Input: data
source config. Output: validated card list.

**CubeManager**: Manages the drawable card supply. Input: CardPool,
copies_per_card, consumption_mode. Operations: draw N cards (with optional
weights), query remaining count, reset. Enforces consumption vs replacement
semantics. Raises on exhaustion in without_replacement mode (pre-validated).

**PackGenerator** (strategy interface): Produces packs at round start. Input:
CubeManager, pack_size, strategy config. Output: Pack (card instances, unique
pack ID, archetype profile snapshot). Implementations: Uniform, RarityWeighted,
SeededThemed.

**RefillStrategy** (strategy interface): Adds one card to a pack after passing.
Input: Pack, CubeManager, strategy config. Output: one card instance (or nothing
for no-refill). Implementations: NoRefill, UniformRefill, ConstrainedRefill.

**RoundOrchestrator**: Runs one complete round. Input: Seats, PackGenerator,
RefillStrategy, round config. Generates packs, executes the pick loop (section
2.5), manages rotation and refills. Output: per-pick records.

**DraftRunner**: Runs a full draft. Input: draft config, RNG seed. Initializes
CubeManager, seats, agents; runs all rounds; computes metrics. Output:
DraftResult (per-seat pools, traces, metrics).

**Agent** (policy interface): Selects one card per pick. Input: observation
(full pack or shown cards + metadata), agent state. Internal state: `w`,
`drafted`, `pick_history`, `openness_estimates`. Implementations: Greedy,
ArchetypeLoyal, Force, Adaptive, SignalIgnorant.

**ShowNStrategy** (strategy interface): Selects N cards from pack for the human
seat. Input: pack contents, human state, show_n, RNG. Implementations: Uniform,
PowerBiased, Curated, SignalRich.

**DeckScorer**: Scores a 30-card pool. Input: card list, final `w`. Output:
float deck_value in [0, 1]. Config: section 6 params.

**CommitmentDetector**: Identifies commitment pick. Input: `w` history. Output:
commitment pick index (or null), committed archetype. Config: section 7 params.

**MetricsEngine**: Computes all metrics from section 8. Input: DraftResult,
metrics config. Output: per-draft metric dictionary. Computes both full-pack and
shown-N variants.

**SweepRunner**: Orchestrates batch execution. Input: experiment config (sweep
axes, fixed params, seeding). Generates parameter grid, runs DraftRunner
instances per point, aggregates results. Supports parallel execution (each run
is stateless).

**ResultsWriter**: Serializes outputs. Input: run-level and aggregate data,
output config. Output: Parquet/JSON/CSV with embedded config metadata and
version string.

### 10.2 Extension and Wiring

New strategies (pack generation, refill, show-N, agent policies) are added by
implementing the corresponding strategy interface. Strategy names are resolved
from configuration strings via a registry; no orchestration or metrics code
changes are required.

Components communicate via typed dataclasses, not direct references. DraftRunner
owns the lifecycle of all per-draft components. Dependency flow: CardPool ->
CubeManager -> PackGenerator/RefillStrategy -> RoundOrchestrator (with Agent,
ShowNStrategy) -> DraftRunner -> DeckScorer/CommitmentDetector/ MetricsEngine ->
SweepRunner -> ResultsWriter.

______________________________________________________________________

## 11. Validation and Calibration

### 11.1 Distribution Sanity Checks

Before accepting simulation results as valid, the following distribution checks
are applied:

**Archetype density in packs:** Each archetype should appear (fitness at or
above 0.5) in at least `expected_min_density` (default: 0.08, approximately 1
card per 12-card pack per archetype) fraction of pack slots. No single archetype
should exceed `expected_max_density` (default: 0.25) unless the pack generation
strategy intentionally concentrates archetypes.

**Commitment timing:**

- Mean commitment pick must fall within a configurable range (default: [4, 8]).
- Standard deviation must be within [1.5, 4.0].
- Uncommitted rate must be below 10%.

**Choice richness baseline:** Mean near-optimal count (shown-N surface) must be
at least 1.5 across all picks. The human seat should typically have at least one
genuine alternative to the apparent best shown card.

### 11.2 Cross-Configuration Comparisons

The validation suite compares pairs of configurations to ensure parameter
changes produce expected directional effects:

- **Refill vs no-refill**: refill mode should produce higher late-phase on-plan
  density (convergence) than no-refill, by at least a configurable margin.
- **More rounds (smaller packs) vs fewer rounds (larger packs)**: more rounds
  should increase early openness (fresh packs bring fresh signals each round).
  The directional effect on early openness must be positive.
- **Hard vs easy difficulty**: signal benefit must be significantly higher under
  the hard preset than under the easy preset.

### 11.3 Difficulty Knob Validation

For each difficulty preset, run the signal benefit measurement (section 8.5) and
verify:

- Easy preset: signal_benefit is below `easy_signal_ceiling` (default: 2%).
- Hard preset: signal_benefit is above `hard_signal_floor` (default: 5%).

If these checks fail, the calibration procedure in section 11.5 should be
applied.

### 11.4 Metric Stability Check

For each metric, verify that the coefficient of variation across
`runs_per_point` runs is below a stability threshold (default: 0.15 for means,
0.30 for tail percentiles). If exceeded, either increase `runs_per_point` until
stability is achieved, or flag the metric as high-variance in the output.

### 11.5 Calibration Procedure

When validation checks fail, recommended adjustments:

1. **Commitment timing off-target**: Adjust `commitment_threshold` and
   `learning_rate`.
2. **Choice richness too low**: Increase `pack_size`, enable refill, or switch
   show-N strategy to curated/signal-rich.
3. **Signal benefit out of range**: Adjust `ai_optimality`, `ai_signal_weight`,
   and `seat_count`.
4. **Forceability too high**: Review card pool archetype distribution; adjust
   synthetic generation or card data.
5. **Convergence too low**: Increase refill `fidelity` or lower
   `on_plan_threshold`.

Calibration is iterative: use sweep mode (section 9) to sweep the parameter
under adjustment, observe metric response, and select the value that hits the
target.

______________________________________________________________________

## 12. Glossary

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
