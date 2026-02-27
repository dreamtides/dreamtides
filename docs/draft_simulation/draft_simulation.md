# Draft Resonance Simulation

A Python simulation at `scripts/draft_sim/` that validates the resonance
weighting algorithm described in the
[resonance design doc](../plans/quests/resonance_and_tags.md) before Rust
implementation. It generates synthetic card pools, simulates full quest drafts,
and measures whether the algorithm produces the intended drafting dynamics:
decks converging to two dominant resonances, mono and tri being possible but
requiring deliberate effort, and splashing being viable but not trivial.

## What It Answers

- **Convergence speed.** At what pick number does a deck's identity crystallize?
- **Archetype distribution.** What fraction of quests produce mono, dual, or tri
  decks?
- **Splash viability.** How often does a third resonance appear with 1-3 cards?
- **Parameter sensitivity.** How do exponent, floor_weight, neutral_base, and
  staleness_factor affect all of the above?
- **Robustness.** Does the algorithm work even when the player ignores resonance
  and just picks the strongest card?

## Running

All stdlib Python, no external dependencies. Run from the project root:

```
python3 scripts/draft_sim/draft_sim.py [--mode {trace,aggregate,sweep,evolution}]
```

Key flags: `--exponent`, `--floor-weight`, `--neutral-base`,
`--staleness-factor` for algorithm tuning.
`--strategy {synergy,power_chaser,rigid}` for player behavior. `--runs N` for
sample size (default 1000). `--seed` for reproducibility.

## Output Modes

**aggregate** (default): Runs N quests, prints convergence stats (mean/median
pick where top-2 share exceeds 75%), final deck composition breakdown (top-2
share, HHI), archetype distribution (mono/dual/tri percentages), splash
analysis, and resonance pair frequency table.

**trace**: Single quest, pick-by-pick detail. Shows all offered cards with their
resonance weights, which card was picked and why, and profile evolution after
each pick. Use `--seed` to reproduce specific runs.

**sweep**: Varies one parameter across a range, printing a comparison table. Use
`--sweep-param` to select the parameter and `--sweep-values` for custom values.
Add `--sweep-strategies` to compare all three player strategies side-by-side per
value.

**evolution**: Shows how metrics evolve pick-by-pick (averaged over N runs),
with rows at picks 5, 10, 15, 20, 25, 30 showing top-2 share, effective color
count, HHI, and off-color rate at each stage.

## Player Strategies

Three strategies test different aspects of the algorithm:

- **synergy** (default): Picks based on a weighted sum of card power and
  resonance fit. Models a skilled player who values both power and staying
  on-color. The `--fit-weight` parameter controls how much they prioritize
  resonance alignment.
- **power_chaser**: Always picks the highest-power card, ignoring resonance.
  Stress-tests whether the algorithm robustly suppresses off-color offerings
  even when the player doesn't cooperate.
- **rigid**: Only picks cards where all resonances match the player's top-2.
  Prefers neutrals over off-color as fallback. Tests whether mono-resonance is
  viable when the player commits fully.

## Key Metrics

- **convergence_pick**: First pick where top-2 resonance share exceeds 75% of
  drafted cards (requires at least 5 resonance-bearing cards).
- **top2_share**: Fraction of resonance symbols in the dominant two resonances.
- **HHI**: Herfindahl-Hirschman Index. Approximately 1.0 for mono, 0.5 for dual,
  0.2 for scattered.
- **pct_mono/dual/tri**: Classification based on share thresholds. Mono requires
  one color above 85%. Dual requires top-2 above 75% with neither above 85%. Tri
  requires three colors each above 15%.
- **splash%**: Runs where the third-highest resonance has 1-3 cards.
- **off_color_offered%**: How often the algorithm offers cards outside the
  player's top-2 resonances.

## Module Structure

- `draft_sim.py`: CLI entry point, argparse, mode dispatch.
- `models.py`: Resonance/Rarity/Strategy enums, parameter dataclasses, SimCard,
  ResonanceProfile with HHI/top2_share/effective_colors helpers.
- `algorithm.py`: Pool generation (360 synthetic cards with rarity distribution
  and per-resonance variance), the weight calculation formula from the design
  doc, staleness penalty, and weighted sampling with resonance diversity and
  rarity guarantee checks.
- `simulation.py`: Full quest loop (7 dreamscapes with draft sites and battle
  rewards), player strategy dispatch, profile tracking.
- `output.py`: All four output modes, metrics computation, ASCII table
  formatting.

## Validated Results

With default parameters (exponent 1.4, floor 0.5, neutral base 3.0):

- **Synergy strategy**: 99.6% dual decks, convergence at pick 5, all 10
  resonance pairs at 9-11% (no systematic bias).
- **Power chaser**: 85% dual, 15% tri. Algorithm robustly converges even when
  the player ignores resonance entirely.
- **Rigid + mono dreamcaller**: 95.6% mono, HHI 0.963. Mono is viable when the
  player commits fully.
- **Higher exponents** produce faster convergence, more mono decks, and fewer
  off-color offerings. Lower exponents allow more tri decks and splashing.
- Weights match the design doc's worked example exactly (verified against all 7
  card types in the table).
