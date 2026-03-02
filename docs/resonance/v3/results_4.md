# Results 4: Echo Window (Corrected Archetype-Level Metrics)

## One-Sentence Algorithm

"Count the resonance symbols across your last 3 picks (primary symbols count
as 2, others as 1); your top resonance fills 2 pack slots, your second
resonance fills 1, and the last slot is random."

## Critical Correction

The previous simulation measured "fitting" at the resonance level (does this
card have a Tide symbol?). The corrected version measures at the archetype
level (is this card S or A tier for Warriors specifically?). A Tide card
could be S-tier for Warriors or Sacrifice, A-tier for SelfMill or Ramp --
the old metric counted all as "fitting." Old late arch-fit: 2.83 (PASS).
Corrected: 1.54 (FAIL). Each resonance serves 4 archetypes, so roughly half
the resonance-matched cards are S/A for the wrong archetype.

## Target Scorecard (Archetype-Committed, 1000 runs)

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Picks 1-5: unique archetypes w/ S/A per pack | >= 3 | 5.18 | PASS |
| Picks 1-5: S/A cards for emerging arch/pack | <= 2 | 1.61 | PASS |
| Picks 6+: S/A cards for committed arch/pack | >= 2 | 1.54 | FAIL |
| Picks 6+: C/F-tier cards per pack | >= 0.5 | 0.38 | FAIL |
| Convergence pick | 5-8 | 7.01 | PASS |
| S/A concentration (final deck) | 60-80% | 84% | FAIL (high) |
| Run-to-run overlap | < 40% | 7% | PASS |
| Arch freq max | <= 20% | 16% | PASS |
| Arch freq min | >= 5% | 10% | PASS |

**6/9 pass.** The three failures are interconnected: resonance-based slot
filling cannot reliably deliver archetype-specific cards. High S/A
concentration (84%) despite low per-pack S/A (1.54) occurs because committed
players always pick the best card available.

## Symbol Distribution

20/60/20 (1-sym/2-sym/3-sym) among non-generic; 36 generic.

## Parameter Sensitivity

### Window Size (3/4/5)

| Window | Late S/A | Late C/F | Convergence | S/A Conc |
|--------|----------|----------|-------------|----------|
| 3 | 1.54 | 0.37 | 7.0 | 84% |
| 4 | 1.55 | 0.36 | 7.0 | 85% |
| 5 | 1.55 | 0.34 | 6.9 | 85% |

No effect. Strengthening the resonance signal does not strengthen the
archetype signal.

### Slot Allocation (2/1/1 vs 3/1/0/0 vs 2/1/0+1)

| Allocation | Late S/A | Late C/F | Convergence | S/A Conc |
|------------|----------|----------|-------------|----------|
| 2/1/1 | 1.53 | 0.39 | 7.0 | 84% |
| 3/1/0/0 | 1.87 | 0.13 | 6.6 | 89% |
| 2/1/0+1 | 1.54 | 0.46 | 7.0 | 84% |

3/1/0/0 approaches the 2.0 target (1.87) but destroys C/F (0.13). No
allocation passes both late S/A and late C/F simultaneously.

### Primary Weight (2 vs 3)

| Weight | Late S/A | Late C/F | Convergence | S/A Conc |
|--------|----------|----------|-------------|----------|
| 2 | 1.54 | 0.37 | 6.9 | 85% |
| 3 | 1.55 | 0.37 | 7.1 | 85% |

No meaningful difference.

## Draft Traces

**Trace 1 -- Early Committer (seed 100):** Commits to Blink by pick 3.
Packs 6-30 average 1.5 S/A cards. Some packs deliver 2-3 S/A (picks 12, 14,
15); others deliver 0 (picks 9, 17, 18) when Ember slots draw Storm/Flash
instead of Blink. Final: 25/30 S/A (83%).

**Trace 2 -- Power-Chaser (seed 200):** Wanders through Warriors, Ramp,
SelfMill, Flash, Sacrifice. Final archetype Sacrifice: 10/30 S/A (33%).
Algorithm cannot help an uncommitted player.

**Trace 3 -- Signal Reader (seed 300):** Locks onto Ember early, converges
on Storm then Blink. Consistent resonance focus delivers Blink/Storm cards
(S/A for each other). Final: 27/30 S/A for Blink (90%).

## Self-Assessment

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | 9/10 | One sentence fully specifies the algorithm. |
| 2. Not on rails | 8/10 | 3-pick window allows pivots at any time. |
| 3. No forced decks | 9/10 | 7% overlap proves high variety. |
| 4. Flexible archetypes | 7/10 | Adjacent archetypes blend via shared resonance. |
| 5. Convergent | 4/10 | 1.54 S/A per pack; structural ceiling from resonance ambiguity. |
| 6. Splashable | 4/10 | 0.38 C/F per pack; tradeoff with convergence is unresolvable. |
| 7. Open early | 9/10 | 5.18 unique archetypes with S/A cards in early packs. |
| 8. Signal reading | 3/10 | Algorithm responds to picks, not offers. |

**Structural finding:** Resonance-to-archetype ambiguity is the central
limitation. Each resonance serves 4 archetypes, giving roughly 50% archetype
accuracy per resonance-matched slot. Mitigations: (a) archetype-specific
symbol patterns on cards, (b) lower the convergence target to match resonance
granularity, or (c) archetype-aware mechanism (violating transparency).
