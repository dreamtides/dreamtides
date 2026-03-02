# Model A Results: 4 Big Archetypes with Internal Variety

## Target Scorecard (Committed Strategy, 1000 drafts)

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Picks 1-5: unique archetypes per pack | >= 3 of 4 | 2.65 | **FAIL** |
| Picks 1-5: fitting cards per pack | <= 2 of 4 | 1.67 | PASS |
| Picks 6+: fitting cards per pack | >= 2 of 4 | 2.47 | PASS |
| Picks 6+: off-archetype strong per pack | >= 0.5 of 4 | 1.34 | PASS |
| Convergence pick | Pick 5-8 | Pick 7 (median) | PASS |
| Deck archetype concentration | 60-80% S/A | 94.3% | **FAIL** |
| Run-to-run overlap | < 40% | 5.5% | PASS |
| Archetype frequency (max) | <= 35%* | 27.3% | PASS |
| Archetype frequency (min) | >= 5% | 22.2% | PASS |

*Note: The original target of "no archetype > 20%" assumes 7-10 archetypes
where 20% represents significant over-representation. With 4 archetypes the
expected baseline is 25% each, so we use an adjusted target of 35% (1.4x
expected) to test for proportional balance.

**Summary: 7 of 9 targets pass. Two failures are structural consequences of
having only 4 archetypes.**

## Analysis of Failures

### Failure 1: Early Unique Archetypes (2.65 vs target 3.0)

This is the predicted and mathematically unavoidable weakness of N=4. With
only 4 archetypes and each card primarily belonging to one, a random 4-card
pack has only a ~66% chance of showing 3+ archetypes (vs ~90% at N=8). On
average, packs show 2.65 unique S-tier archetypes, meaning roughly a third of
early packs show only 2 archetypes.

This is not fixable within the N=4 framework without artificial pack
construction (e.g., guaranteeing archetype diversity in every pack), which
would conflict with the simplicity goal. It is the primary structural cost of
choosing few archetypes.

### Failure 2: Deck Concentration Too High (94.3% vs target 60-80%)

The committed player ends up with 94% S/A-tier cards -- far above the
60-80% target. This is because with 4 archetypes and ~40-45% of the pool
fitting any given archetype, the player almost always has fitting cards
available. The system provides too much archetype support, not too little.

This is actually a mixed signal. It means convergence is trivially easy (good
for goal 5) but suggests players lack meaningful tension between "take the
fitting card" and "take the powerful off-archetype card" (bad for goal 2). The
94% concentration means almost every pick is "on-archetype," reducing the
interesting splash/pivot decisions.

## Cross-Strategy Comparison

| Metric | Committed | Power-Chaser | Signal-Reader |
|--------|-----------|--------------|---------------|
| Late fitting/pack | 2.47 | 2.52 | 2.46 |
| Deck S/A % | 94.3% | 68.2% | 95.5% |
| Run overlap | 5.5% | 6.5% | 5.6% |
| Conv. pick | 7 | 7 | 6 |

The power-chaser naturally lands at 68% S/A (within the target range),
suggesting the 60-80% target is achievable but only when the player is NOT
trying to stay on-archetype. The signal-reader converges fastest (pick 6) and
achieves the highest concentration (95.5%), indicating that signals are
readable and rewarding -- but perhaps too rewarding.

## Multi-Archetype Card Sensitivity

The sensitivity sweep varies the percentage of cards that are S/A-tier in 2+
archetypes from 20% to 80%, measuring key metrics for the committed strategy.

| Multi-Arch % | Late Fit/Pack | Deck S/A | Conv. Pick | Off-Arch Strong | Overlap |
|--------------|---------------|----------|------------|-----------------|---------|
| 20% | 1.97 | 87.8% | 7 | 1.96 | 5.6% |
| 30% | 2.10 | 89.6% | 7 | 1.80 | 4.9% |
| 40% | 2.23 | 91.9% | 7 | 1.63 | 5.6% |
| 50% | 2.34 | 92.7% | 7 | 1.49 | 5.6% |
| 60% | 2.44 | 94.4% | 7 | 1.36 | 5.7% |
| 70% | 2.58 | 95.7% | 7 | 1.20 | 6.0% |
| 80% | 2.72 | 96.6% | 6 | 1.03 | 6.1% |

**Key observations:**

1. **Late fitting cards scale linearly with multi-archetype %.** Even at 20%
   multi-archetype cards, the system still hits ~2.0 fitting cards per pack --
   barely meeting the target. This confirms that N=4 provides enough baseline
   density that multi-archetype cards are helpful but not critical for
   convergence.

2. **Off-archetype strong cards decrease as multi-archetype % rises.** This
   is counterintuitive: more multi-archetype cards means *fewer* strong
   off-archetype options, because those cards become fitting (S/A) rather than
   off-archetype. At 80%, only 1.03 strong off-archetype cards per pack,
   heading toward the 0.5 minimum.

3. **Deck concentration rises monotonically and is always above 80%.** The
   N=4 system inherently over-concentrates. Even at the minimum multi-archetype
   level (20%), decks are 88% S/A. This confirms the failure is structural.

4. **Convergence pick is rock-stable.** It barely moves across the sweep
   (pick 6-7), confirming that at N=4 the convergence timing is driven by
   the commitment detection threshold (3 picks) rather than pool composition.

5. **Run-to-run overlap is excellent everywhere** (~5-6%), far below the 40%
   target. The large per-archetype card pool (90 S-tier cards) ensures high
   variety even within the same archetype.

**Minimum viable multi-archetype %:** For this model, even 20% works
adequately because N=4 provides sufficient baseline density. The system does
not depend heavily on multi-archetype cards. This is a significant practical
advantage: fewer cross-archetype designs needed.

## What Works

- **Convergence is effortless.** The system reliably delivers 2+ fitting cards
  per pack after commitment with minimal algorithmic intervention. The gentle
  1.5-3.0x weight ramp is sufficient.
- **Run-to-run variety is excellent.** 5-6% overlap means almost no two runs
  share the same deck, thanks to the 90-card S-tier pool per archetype.
- **Archetype balance is good.** All 4 archetypes see 22-27% representation,
  close to the expected 25%.
- **Off-archetype splashing is abundant.** 1.3+ strong off-archetype cards
  per pack is well above the 0.5 target.
- **Signal reading works.** The signal-reader converges one pick earlier
  than other strategies, confirming that per-run archetype weighting creates
  detectable signals.

## What Doesn't Work

- **Early-draft diversity is weak.** The 3-archetype-per-pack target is
  structurally impossible to meet consistently at N=4. Players will see many
  packs with only 2 archetypes represented, making early exploration feel
  limited.
- **Deck concentration is too high.** 94% S/A means there is almost no tension
  in card selection -- nearly every offered card is acceptable for the committed
  archetype. This undermines the "interesting decisions" goal.
- **Archetypes may lack identity.** With 90 S-tier cards per archetype, each
  archetype is so broad that it may not feel like a coherent strategy. This
  cannot be measured in simulation but is a real design concern.
- **Hybrid decks are trivially easy** but lack strategic depth with only 6
  possible pairs and high cross-archetype overlap.

## Conclusion

Model A demonstrates that N=4 trivially solves convergence and variety but
creates two structural problems: insufficient early-draft diversity and
excessive late-draft concentration. The system is too generous -- not because
the algorithm is wrong, but because 4 archetypes spread across 360 cards
means each archetype covers 25% of the pool, and with multi-archetype overlap,
40-45% of any pack naturally fits the committed player's needs.

The model validates Q1's prediction that low-N systems "trivially solve
convergence but kill early-draft exploration." It also confirms Q2's insight
that the fitness distribution matters less at low N because the math is
forgiving regardless. The primary value of this model is as a lower bound:
it shows what works (convergence, variety, balance) and what breaks (diversity,
interesting decisions) when archetypes are maximally broad.
