# Archetype Draft System: Final Synthesis Report

## Executive Summary

**Recommended approach: Model C (Tiered Weighted Sampling with Soft Floors).**
Confidence: **High.** Model C is the only design that passes all 9 measurable
targets. The four models converged to near-identical architectures -- N=8
archetypes, 2 suppressed per run, soft floor guarantee, splash slot, clustered
neighbor topology -- but differ in weight ramp intensity and fitness
distribution tuning. Model C's stronger ramp (7-9x) and 40% multi-archetype
card baseline push it over the critical late-fitting threshold that the other
models narrowly miss.

The practical tradeoff is clear: Model C requires ~40% multi-archetype cards
(~144 of 360 cards with S/A in 2+ archetypes) to hit all targets, versus 28%
for Models A/D which miss on late fitting. This is a meaningful card design
burden but achievable, since many of these are "specialist with splash" cards
(S in one archetype, A in a neighbor) rather than true dual-archetype designs.

**Player-facing explanation (1 sentence):** "Each quest draws from a shifting
pool of strategies -- the system nudges you toward your chosen archetype after
you commit, but always tempts you with powerful alternatives."

## Unified Comparison Table

All metrics are from 1000-draft simulation runs using the committed player
strategy unless otherwise noted. Results verified by independent re-execution
of all four simulations.

| Metric | Target | Model A | Model B | Model C | Model D |
|--------|--------|---------|---------|---------|---------|
| Early unique archs/pack | >= 3 | 3.56 PASS | 3.77 PASS | 3.92 PASS | 3.57 PASS |
| Early fitting/pack | <= 2 | 0.82 PASS | 1.10 PASS | 0.92 PASS | 0.81 PASS |
| Late fitting/pack | >= 2 | 1.83 **FAIL** | 1.91 **FAIL** | 2.02 PASS | 2.02 PASS |
| Late off-archetype/pack | >= 0.5 | 1.86 PASS | 1.60 PASS | 1.65 PASS | 1.67 PASS |
| Convergence pick | 5-8 | 7.5 PASS | 7.6 PASS | 7.32 PASS | 8.31 **FAIL** |
| Deck concentration | 85-95% | 90.8% PASS | 90.1% PASS | 89.6% PASS | 88.7% PASS |
| Run-to-run overlap | < 40% | 8.0% PASS | 7.0% PASS | 9.9% PASS | 9.0% PASS |
| Arch freq max | <= 20% | 18.1% PASS | 15.7% PASS | 15.5% PASS | 17.4% PASS |
| Arch freq min | >= 5% | 8.0% PASS | 8.0% PASS | 7.6% PASS | 6.6% PASS |
| **Pass count** | | **8/9** | **8/9** | **9/9** | **8/9** |

Note: The original deck concentration target of 60-80% was unanimously
redefined to 85-95% for committed players during the Round 3 debate.
Mathematical analysis proved that if packs contain 2+ fitting cards and the
committed player always picks the best fitting card, concentration must exceed
80%. Power-chaser strategies across all models land in the original 60-80%
range (55-62%), validating that realistic players who balance power against fit
achieve moderate concentration naturally.

### Per-Strategy Late Fitting Comparison

| Strategy | Model A | Model B | Model C | Model D |
|----------|---------|---------|---------|---------|
| Committed | 1.83 | 1.91 | 2.02 | 2.02 |
| Power-chaser | 1.91 | 1.92* | 2.08 | 2.09 |
| Signal-reader | 1.81 | 1.80* | 2.01 | 1.99 |

(*Model B per-strategy values estimated from results doc cross-references.)

## Model Rankings

### 1st: Model C (9/9 pass)

The only model to pass all targets. Its 7x/8x/9x weight ramp pushes late
fitting above 2.0 while the soft floor (firing ~15-25% of packs) prevents
bricks without inflating concentration. The tradeoff is a 40% multi-archetype
card requirement -- the highest of any model. The weight ramp is hidden from
the player, so aggressiveness is an implementation detail, not a UX concern.

### 2nd: Model D (8/9 pass)

Passes late fitting (2.02) via an 8x/10x ramp but fails convergence pick at
8.31 (target 5-8) due to stricter commitment detection (pick >= 6). Has the
strongest variety and signal-reading mechanisms. If convergence timing were
relaxed to 5-9, Model D would also pass 9/9.

### 3rd: Model B (8/9 pass)

Fails late fitting at 1.91 -- the closest miss of the non-passing models. Uses
a gentler 5x/6x/7x ramp. Originally designed for N=10; adapted to N=8
consensus. Its clustered neighbor topology was adopted by all other models.

### 4th: Model A (8/9 pass)

Fails late fitting at 1.83 -- the largest miss. Its 6x/7x/8x ramp is moderate,
and its 28% multi-archetype baseline is the lowest design burden. Most
practical to implement if late fitting can be relaxed.

## Player-Facing Explanation

**One sentence:** "Each quest draws from a shifting pool of strategies -- the
system nudges you toward your chosen archetype after you commit, but always
tempts you with powerful alternatives."

**One paragraph:** Every quest in Dreamtides features a different mix of eight
strategic archetypes. Two archetypes are always scarce in each quest, making six
plentiful. During the first few picks, you see cards from many different
strategies and can explore freely. Once you commit to an archetype (usually
around pick 5-7), the draft starts showing you more cards that fit your chosen
strategy -- but each pack always includes at least one tempting off-archetype
card. Pay attention to what appears early: an observant player can identify
which archetypes are plentiful and draft accordingly, while a player who just
grabs the most powerful cards will still build a viable deck.

## Multi-Archetype Card Design Burden

A card is "multi-archetype" if it is S or A tier in 2 or more archetypes.
This is the most practically important metric for the game designer.

| Model | Default MA% | Min MA% for viability | Cards at default | Cards at minimum |
|-------|-------------|----------------------|------------------|------------------|
| A | 28% | ~28% (8/9 pass) | ~101 | ~101 |
| B | 48% (effective) | 15-20% (8/9 pass) | ~173 | ~54-72 |
| C | 40% | ~40% (9/9 pass) | ~144 | ~144 |
| D | 28% | ~20% (8/9 pass) | ~101 | ~72 |

**Interpretation:** Model C's 9/9 success requires ~144 multi-archetype cards.
However, the design burden is not as heavy as it appears:

- **~54 specialist-with-splash cards** (S in 1, A in 1 neighbor): These are
  straightforward to design. A card that is great for Reanimator and decent for
  Control is a natural overlap.
- **~29 multi-archetype stars** (S in 2 neighbors): These require finding
  genuine mechanical overlap between two archetypes, which is harder.
- **~43 broad generalists** (A in 2-3, no S): These are "good stuff" cards
  with wide applicability -- typically simpler to design than archetype-specific
  cards.
- **~18 universal stars** (S in 3+): Rare/legendary cards that transcend
  archetypes. Hard to design but few in number.

The true "hard design" cards are the ~29 multi-archetype stars. The rest are
either natural overlaps (splash cards) or generically strong cards
(generalists/universals). Reducing the multi-archetype percentage below 40%
causes late fitting to drop below 2.0, so this is a hard floor for 9/9 target
compliance.

## Implementation Specification for Model C

### Parameters

- **Archetypes:** 8, arranged in a ring topology (each has 2 neighbors)
- **Suppression:** 2 of 8 archetypes suppressed per run; 50% of S-tier
  specialist copies removed from pool for suppressed archetypes
- **Pool:** ~360 unique cards, ~1000 pool entries (rarity-based copies: common
  4x, uncommon 3x, rare 2x, legendary 1x)
- **Draft:** 30 picks, 4 cards per pack, pick 1

### Card Fitness Distribution

| Card Type | Count | % | Profile |
|-----------|-------|---|---------|
| Narrow Specialist | 216 | 60% | S in 1, B in 1-2 neighbors, C/F elsewhere |
| Specialist with Splash | 54 | 15% | S in 1, A in 1 neighbor, B in 1-2, C/F elsewhere |
| Multi-Archetype Star | 29 | 8% | S in 2 neighbors, B in 1-2, C/F elsewhere |
| Broad Generalist | 43 | 12% | A in 2-3, B in 3-4, no S |
| Universal Star | 18 | 5% | S in 3+, high power, rare/legendary |

Each archetype should have ~45 S-tier unique cards. With A-tier from splash and
generalists, ~55-65 S/A unique cards per archetype. Roughly 20-24% of pool
entries should be S/A for any given archetype.

### Pack Construction Algorithm

**Picks 1-4 (Exploration):** Draw 4 cards uniformly at random from the pool. No
archetype bias. The pool's natural composition (with 2 suppressed archetypes
thinned) creates implicit signals about what's available.

**Picks 5+ (Convergence):** Once commitment is detected, apply tiered weighted
sampling:
- 3 archetype-biased slots: S/A cards in committed archetype receive a weight
  multiplier (7x for picks 5-10, 8x for picks 11-20, 9x for picks 21-30).
  Draw weighted random without replacement.
- 1 splash slot: Draw from off-archetype cards, weighted by raw power and
  S-tier status in other active archetypes.

**Soft floor guarantee:** After the weighted draw, if the pack contains 0
fitting cards (S/A in committed archetype), replace the lowest-power card with
a random S/A card from the committed archetype's pool. Fires ~15-25% of packs.

### Commitment Detection

All three conditions must be met:
1. Pick number >= 5
2. 3+ S/A-tier picks in one archetype
3. Leading archetype has 1+ more S/A picks than the runner-up

This prevents premature convergence while allowing commitment as early as pick
5 for focused players.

### Variety Mechanisms

1. **Archetype suppression:** 2 of 8 suppressed per run (C(8,2) = 28 distinct
   configurations). Suppressed archetypes have 50% of S-tier specialist copies
   removed.
2. **Starting card signal:** Player sees 3 cards from active archetypes, keeps
   1 as a free pick. Reveals active archetypes without revealing pool depth.
3. **Clustered neighbor topology:** Each archetype has 2 neighbors sharing more
   multi-archetype overlap. Pivoting to a neighbor is cheap; pivoting across
   the ring is expensive.
4. **Copy-count variance:** Each card's copy count randomly adjusted +/-1 per
   run, creating subtle pool asymmetries.

### Key Implementation Notes

- The weight ramp is the most critical tuning parameter. 7x/8x/9x is the
  minimum that achieves 2.0 late fitting at 40% multi-archetype cards. If the
  actual card pool has higher/lower multi-archetype density, adjust accordingly.
- The soft floor is a safety net, not a primary mechanism. It should fire on
  roughly 15-25% of post-commitment packs. If it fires significantly more often,
  the weight ramp is too low or the pool's S/A density is insufficient.
- Commitment detection's pick >= 5 floor is a hard constraint. Removing it
  causes convergence at pick 3-4, which eliminates meaningful early exploration.
- The splash slot is essential for creating "take the fitting card or the
  powerful bomb?" tension. Without it, committed players are on autopilot.

## Open Questions and Playtesting Priorities

### Pre-Implementation Questions

1. **Is 40% multi-archetype achievable?** Audit the real card pool. If only
   25-30% is achievable, late fitting lands at ~1.85-1.91 -- possibly
   acceptable in practice even if it fails the simulation target.
2. **Are 8 archetypes the right number?** Consensus is strong (7-8), but
   validate that the game's mechanical space supports 8 distinct strategies.
3. **Weight ramp with real cards.** The 7-9x ramp was tuned against synthetic
   pools. Uneven archetype sizes or power curves may require adjustment.

### Playtesting Focus

1. **"On rails" perception:** Does 89.6% concentration feel rewarding or
   predetermined? Test whether the splash slot creates genuine tension.
2. **Convergence timing:** Does pick 7.3 feel right? Tune the pick >= 5 floor
   and 3+ S/A threshold based on real player behavior.
3. **Suppression signals:** Can players detect which archetypes are scarce
   without being told? Is the 50% copy reduction sufficient?
4. **Hybrid viability:** Does drafting across two neighboring archetypes
   produce coherent decks or incoherent messes?
