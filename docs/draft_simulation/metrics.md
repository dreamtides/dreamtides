# Draft Simulator Metrics Reference

## Goal Metrics

Six key metrics define a healthy Dreamtides draft. These are the primary targets
when tuning draft parameters — if these pass, the draft experience is on track.

1. **Convergence (shown-N, post-commitment): On-plan density (mid) mean >= 2.0**
   — After committing to an archetype, the draft should trend toward delivering
   on-plan cards through the mid phase.
2. **Convergence (shown-N, post-commitment): On-plan density (late) mean >=
   2.0** — On-plan delivery should sustain through the end of the draft.
3. **Choice Richness (shown-N): Near-optimal count overall >= 1.5** — Multiple
   cards should be competitive at each pick. Near 1.0 means only one real
   choice.
4. **Forceability: max < 0.95 (hard preset)** — No archetype should be blindly
   forceable to a comparable deck value as adaptive play.
5. **Splashability (shown-N): splash_fraction >= 0.40** — At least 40% of
   post-commitment picks should offer a viable off-plan card worth taking for
   raw strength or flexibility.
6. **Early Openness (shown-N): archetypes exposed >= 5.0** — The first 5 picks
   should expose the player to at least 5 distinct archetypes before requiring
   commitment.

## Metric Details

Detailed reference for the six metric families computed by `metrics.py`. Each
metric evaluates draft experience quality on two evaluation surfaces:

- **full-pack**: all cards in the pack, measuring environment health regardless
  of what the human sees.
- **shown-N**: only the N cards shown to the human seat (selected by the show-N
  strategy), measuring actual human experience.

Results are bucketed into three draft phases based on 0-indexed pick number:

| Phase | Picks | Description                                         |
| ----- | ----- | --------------------------------------------------- |
| early | 0–5   | First 6 picks; drafter is exploring archetypes      |
| mid   | 6–19  | Middle picks; drafter is building toward commitment |
| late  | 20–29 | Final 10 picks; drafter is rounding out their deck  |

Phase definitions are at `metrics.py:24-29`.

## Choice Richness

**Question**: Does the drafter have multiple viable options at each pick, or is
the correct choice obvious ("on rails")?

**Dataclass**: `ChoiceRichnessMetrics` (`metrics.py:131`)

Three sub-metrics are computed per pick for the human seat, then averaged within
each phase:

### Near-optimal count

How many cards in the shown/pack set score within `richness_gap` (default 0.1)
of the best card's score.

```
near_optimal_count = count of cards where score >= best_score - richness_gap
```

A value of 1.0 means only one card is competitive — the drafter is on rails. A
value of 3.0+ means three or more cards are close in value, giving meaningful
choice. The validation target is mean >= 1.5 across all picks (shown-N surface).

### Score gap

The difference between the best and second-best card score in the shown/pack
set.

```
score_gap = best_score - second_best_score
```

Lower is better — a small gap means the top two options are close in value.
Reported as mean, median, and p90 per phase.

### Choice entropy

Shannon entropy of softmax-normalized card scores, using temperature `tau`
(default 1.0).

```
probs = softmax(scores, tau)
entropy = -sum(p * log2(p) for p in probs)
```

Higher entropy means more evenly distributed scores (more genuine choices).
Lower entropy means one card dominates.

### Card scoring

Cards are scored using the agent's pick policy (default `adaptive`) and the
agent's state (preference vector `w`, drafted pool, openness estimate) at that
pick index. The `w` state is reconstructed from `w_history[pick_index - 1]` (the
preference vector going *into* the pick), with uniform `w` used for pick 0. See
`_score_cards_for_seat()` at `metrics.py:187`.

### Output format

```
Choice Richness (shown-N):
  Near-optimal count:  early=2.8  mid=2.1  late=1.5  overall=2.1
  Score gap:           early=0.05 mid=0.08 late=0.12 overall=0.08
  Choice entropy:      early=1.92 mid=1.64 late=1.31 overall=1.62
```

### Configurable parameters

| Parameter    | Config key             | Default | Effect                                                                |
| ------------ | ---------------------- | ------- | --------------------------------------------------------------------- |
| richness_gap | `metrics.richness_gap` | 0.1     | Threshold for near-optimal: cards within this of the best score count |
| tau          | `metrics.tau`          | 1.0     | Softmax temperature for entropy calculation                           |

## Convergence

**Question**: After the drafter commits to an archetype, does the draft keep
delivering on-plan cards?

**Dataclass**: `ConvergenceMetrics` (`metrics.py:142`)

Convergence only considers **post-commitment** picks (after the human seat's
detected commitment pick). Results are reported for the **mid** (picks 6–19) and
**late** (picks 20–29) phases separately. If the human seat never commits, all
values are 0.0.

### Calculation step by step

1. Find the human seat's `commitment_pick` and `committed_archetype` from
   `SeatResult`.
2. Iterate all trace records for the human seat where
   `pick_index > commitment_pick`.
3. For each such pick, get the card set (shown-N or full-pack surface).
4. Count cards where `card.fitness[committed_archetype] >= on_plan_threshold`
   (default 0.5). This count is the **on-plan count** for that pick.
5. Bucket the on-plan count into `mid_on_plan_counts` or `late_on_plan_counts`
   based on the pick's phase.
6. Compute mean and P(>=3) for each phase independently.

### On-plan density — mean

The arithmetic mean of per-pick on-plan counts across all eligible picks in that
phase.

```
mean = sum(on_plan_counts) / len(on_plan_counts)
```

Example: if the 10 late-phase picks had on-plan counts of
`[1, 0, 2, 1, 0, 0, 1, 0, 0, 1]`, the mean is 0.6.

**Target**: mean >= 2.0 in the shown-N surface during the late phase. A mean
below 1.0 means the drafter rarely sees even one on-plan card per pick. Mid
values are informational — they show whether on-plan density is trending down
from mid to late, which can diagnose refill or card pool exhaustion issues.

### P(>=3)

The fraction of eligible picks in that phase where the on-plan count is 3 or
more.

```
P(>=3) = count(c >= 3 for c in on_plan_counts) / len(on_plan_counts)
```

Example: if 2 of 10 late picks had 3+ on-plan cards, `P(>=3) = 0.20`.

This measures how often the drafter gets a *rich* on-plan selection, not just
one card. `P(>=3) = 0.00` means not a single pick in that phase offered 3+
on-plan cards.

### Output format

```
Convergence (shown-N, post-commitment):
  On-plan density (mid):  mean=1.8, P(>=3)=0.25
  On-plan density (late): mean=2.3, P(>=3)=0.40
```

### What "post-commitment" means

The convergence metric only counts picks *after* commitment. The commitment pick
itself and all earlier picks are excluded. Commitment is detected by the
concentration-based method: `max(w) / sum(w) >= commitment_threshold` sustained
for `stability_window` consecutive picks with the same argmax. See
`commitment.py` and the commitment detection section of `draft_simulation.md`.

### Diagnosing low convergence

- `mean < 1.0` on shown-N: the show-N strategy isn't surfacing on-plan cards.
  Switch to `top_scored` or `curated`, or increase `agents.show_n`.
- `mean < 1.0` on full-pack: the card pool or refill strategy doesn't maintain
  archetype density in late packs. Increase `refill.fidelity`, enable refill
  (change from `no_refill` to `constrained_refill`), or lower
  `metrics.on_plan_threshold`.
- `P(>=3) = 0.00`: on-plan cards are too sparse for any pick to offer 3+ at
  once. Pack size or on-plan threshold may need adjustment.

### Configurable parameters

| Parameter         | Config key                  | Default | Effect                                                      |
| ----------------- | --------------------------- | ------- | ----------------------------------------------------------- |
| on_plan_threshold | `metrics.on_plan_threshold` | 0.5     | Minimum fitness for committed archetype to count as on-plan |

## Forceability

**Question**: Can a drafter succeed by blindly forcing a single archetype every
draft, ignoring signals?

**Dataclass**: field on `DraftMetrics` (`metrics.py:176-178`)

Forceability requires cross-run data from sweep mode — it cannot be computed
from a single draft.

### Calculation

For each archetype `a`, a batch of drafts is run with the `force` policy
targeting `a`. Another batch uses the `adaptive` policy. The forceability index
for archetype `a` is:

```
FI(a) = mean(force_deck_value[a]) / mean(adaptive_deck_value)
```

The reported `forceability_max` is `max(FI(a))` across all archetypes, and
`forceability_archetype` identifies which archetype achieves it.

**Target**: no archetype above 0.95 under the hard preset. Values near or above
1.0 mean forcing that archetype produces decks as good as or better than
adapting — indicating a degenerate draft environment.

### Output format

```
Forceability: max=0.82 (archetype 3)
```

Or `N/A (requires sweep)` in single mode.

## Signal Benefit

**Question**: Does reading supply signals from pack contents meaningfully
improve draft outcomes?

**Dataclass**: field on `DraftMetrics` (`metrics.py:179`)

Signal benefit requires cross-run data from sweep mode.

### Calculation

Two batches of drafts are run: one with the `adaptive` policy (which uses
openness signals) and one with the `signal_ignorant` policy (which uses uniform
openness). The signal benefit is:

```
signal_benefit = (mean_adaptive_deck_value - mean_ignorant_deck_value)
                 / mean_ignorant_deck_value * 100%
```

A positive value means signal-reading helps; a negative value means it hurts
(possible if the signal estimate is noisy).

**Targets**:

- Easy preset (`ai_optimality=0.4`, `ai_signal_weight=0.0`): below 2%. Signals
  shouldn't matter when AIs ignore them and competition is low.
- Hard preset (`ai_optimality=0.9`, `ai_signal_weight=0.8`): 5–15%. Signals
  should provide a meaningful edge when AIs are also reading them.

### Output format

```
Signal Benefit: +8.3% (adaptive vs signal-ignorant)
```

Or `N/A (requires sweep)` in single mode.

## Splashability

**Question**: After committing to an archetype, does the drafter still see
viable off-plan cards worth picking for raw strength?

**Dataclass**: `SplashabilityMetrics` (`metrics.py:149`)

### Calculation

For each post-commitment pick (all phases, not just late):

1. Get the card set (shown-N or full-pack surface).
2. A card is **off-plan** if `fitness[committed_archetype] < 0.3`.
3. An off-plan card is **splashable** if `power >= splash_power_threshold`
   (default 0.5) or `flex >= splash_flex_threshold` (default 0.6).
4. The pick **has a splash option** if any card in the set is splashable.

The output is:

```
splash_fraction = picks_with_splash_option / total_post_commitment_picks
```

**Target**: at least 0.40 (40% of post-commitment picks should offer a viable
off-plan option). This ensures the drafter isn't rigidly locked into only
on-plan cards and can make interesting splash decisions.

### Output format

```
Splashability: 0.65
```

### Configurable parameters

| Parameter              | Config key                       | Default | Effect                                              |
| ---------------------- | -------------------------------- | ------- | --------------------------------------------------- |
| splash_power_threshold | `metrics.splash_power_threshold` | 0.5     | Minimum power for an off-plan card to be splashable |
| splash_flex_threshold  | `metrics.splash_flex_threshold`  | 0.6     | Minimum flex for an off-plan card to be splashable  |

## Early Openness

**Question**: Do the first few picks expose the drafter to many archetypes
before requiring commitment?

**Dataclass**: `EarlyOpennessMetrics` (`metrics.py:157`)

Two sub-metrics:

### Archetypes exposed

The number of distinct archetypes seen with `fitness >= exposure_threshold`
(default 0.5) across the first 5 picks (picks 0–4) of the human seat.

```
for each card seen in picks 0-4:
    for each archetype where card.fitness[arch] >= exposure_threshold:
        add arch to exposed_set
archetypes_exposed = len(exposed_set)
```

**Target**: mean >= 5 distinct archetypes across the first 5 picks. Higher is
better — the drafter should encounter a broad range of options before narrowing
down.

### Preference entropy

Mean Shannon entropy of the normalized preference vector `w` during picks 0–5.

```
for each pick 0-5:
    normalized_w = w / sum(w)
    entropy = -sum(p * log2(p) for p in normalized_w)
preference_entropy = mean(entropies)
```

Higher entropy means `w` is still spread across archetypes (drafter hasn't
prematurely narrowed). Lower entropy means `w` has concentrated early, which
could indicate the draft is pushing commitment too soon.

### Output format

```
Early Openness: 7.0 archetypes exposed, preference entropy=2.85
```

### Configurable parameters

| Parameter          | Config key                   | Default | Effect                                                             |
| ------------------ | ---------------------------- | ------- | ------------------------------------------------------------------ |
| exposure_threshold | `metrics.exposure_threshold` | 0.5     | Minimum fitness for a card to "expose" the drafter to an archetype |

## Evaluation Surfaces

Every metric except forceability and signal benefit is computed on both
surfaces:

| Surface       | What it measures              | When it matters                                                                         |
| ------------- | ----------------------------- | --------------------------------------------------------------------------------------- |
| **shown-N**   | Cards the human actually sees | Tuning show-N strategy, evaluating player experience                                    |
| **full-pack** | All cards in the pack         | Diagnosing whether low shown-N metrics are caused by poor filtering vs poor environment |

If a metric is healthy on full-pack but poor on shown-N, the show-N strategy is
the bottleneck. If both are poor, the underlying draft environment (card pool,
pack generation, refill) needs attention.

## Validation Checks

The validation suite (`validation.py`) runs after sweep mode and checks metric
health:

| Check                           | Target                                        | What it catches                                                       |
| ------------------------------- | --------------------------------------------- | --------------------------------------------------------------------- |
| Choice richness baseline        | mean near-optimal >= 1.5 (shown-N)            | Drafters on rails with no real choice                                 |
| Metric stability (CV)           | CV < 0.15 for means, < 0.30 for tails         | High variance suggesting insufficient runs or unstable parameters     |
| Commitment timing — mean        | pick in [4, 8]                                | Commitment too early (< 4) or too late (> 8)                          |
| Commitment timing — std         | [1.5, 4.0]                                    | All seats committing at the same pick (too uniform) or wildly varying |
| Commitment timing — uncommitted | < 10%                                         | Too many seats never committing                                       |
| Archetype density               | each archetype in [0.08, 0.25] of total seats | One archetype dominating or being absent                              |
| Difficulty knobs — easy         | signal_benefit < 2%                           | Signals mattering too much at easy difficulty                         |
| Difficulty knobs — hard         | signal_benefit > 5%                           | Signals not mattering enough at hard difficulty                       |
| Cross-config: rounds → openness | more rounds increase early openness           | Round structure not improving exploration                             |

## CSV Column Reference

Run-level CSV columns for metrics (from `output.py:build_run_record`):

| Column                       | Metric                                 | Surface   |
| ---------------------------- | -------------------------------------- | --------- |
| `cr_shown_near_opt_{phase}`  | Choice richness: near-optimal count    | shown-N   |
| `cr_shown_score_gap_{phase}` | Choice richness: score gap mean        | shown-N   |
| `cr_shown_entropy_{phase}`   | Choice richness: choice entropy        | shown-N   |
| `cr_full_near_opt_{phase}`   | Choice richness: near-optimal count    | full-pack |
| `cr_full_score_gap_{phase}`  | Choice richness: score gap mean        | full-pack |
| `cr_full_entropy_{phase}`    | Choice richness: choice entropy        | full-pack |
| `conv_shown_mid_mean`        | Convergence: on-plan density mid mean  | shown-N   |
| `conv_shown_mid_p3`          | Convergence: P(>=3) mid                | shown-N   |
| `conv_shown_late_mean`       | Convergence: on-plan density late mean | shown-N   |
| `conv_shown_late_p3`         | Convergence: P(>=3) late               | shown-N   |
| `conv_full_mid_mean`         | Convergence: on-plan density mid mean  | full-pack |
| `conv_full_mid_p3`           | Convergence: P(>=3) mid                | full-pack |
| `conv_full_late_mean`        | Convergence: on-plan density late mean | full-pack |
| `conv_full_late_p3`          | Convergence: P(>=3) late               | full-pack |
| `splash_shown`               | Splashability: splash fraction         | shown-N   |
| `splash_full`                | Splashability: splash fraction         | full-pack |
| `openness_shown_archetypes`  | Early openness: archetypes exposed     | shown-N   |
| `openness_shown_entropy`     | Early openness: preference entropy     | shown-N   |
| `openness_full_archetypes`   | Early openness: archetypes exposed     | full-pack |
| `openness_full_entropy`      | Early openness: preference entropy     | full-pack |
| `signal_benefit`             | Signal benefit %                       | cross-run |
| `forceability_max`           | Forceability index (worst archetype)   | cross-run |

Phase suffixes are `_early`, `_mid`, `_late`, `_overall`.
