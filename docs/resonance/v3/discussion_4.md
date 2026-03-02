# Discussion 4: Reactive/Immediate Domain — Echo Window

## Simplicity Ranking (Most to Least Simple)

1. **Lane Locking** — One threshold check (hit 3 symbols?), one permanent
   result (slot locks). A 12-year-old can predict pack structure after seeing
   it once. Fewest moving parts of any champion.
2. **Echo Window** — Two operations: count symbols in last 3 cards, apply
   2/1/1/random formula. Requires remembering 3 cards (physically present)
   and one ranking step.
3. **Weighted Lottery** — Simple concept (weighted random) but requires
   probability math. "Tide is 11 out of 19 total, so 58%" is not something
   a 12-year-old computes. The player cannot predict exact pack composition,
   only tendencies.
4. **Resonance Swap** — The rule is simple ("replace 2 non-matching with 2
   matching") but the effect is invisible. Player cannot see the 360-card
   pool, cannot count its composition, cannot predict the result. Simple
   mechanism, opaque outcome.
5. **Rotating Wheel** — Four interacting mental operations: track wheel
   position (mod 4 counter), know the fixed order, compute majority
   resonance, apply conditional duplication rule. Most complex champion
   despite sounding elegant.

## Scorecard Table

| Design Goal | Weighted Lottery | Rotating Wheel | Lane Locking | Echo Window | Resonance Swap |
|---|---|---|---|---|---|
| 1. Simple | 7 | 4 | 9 | 8 | 6 |
| 2. Not on rails | 5 | 6 | 3 | 9 | 7 |
| 3. No forced decks | 4 | 6 | 5 | 8 | 7 |
| 4. Flexible archetypes | 6 | 5 | 3 | 8 | 7 |
| 5. Convergent | 9 | 5 | 6 | 7 | 5 |
| 6. Splashable | 5 | 7 | 7 | 8 | 6 |
| 7. Open early | 5 | 8 | 9 | 9 | 8 |
| 8. Signal reading | 3 | 8 | 5 | 2 | 6 |
| **Weighted Total** | **5.9** | **5.8** | **5.8** | **7.4** | **6.5** |

Weighted total uses priority order: Goal 1 weight 8, Goal 2 weight 7,
Goal 3 weight 6, Goal 4 weight 5, Goal 5 weight 4, Goal 6 weight 3,
Goal 7 weight 2, Goal 8 weight 1.

## Final Championed Algorithm: Echo Window

**One-sentence description:** "Count the resonance symbols across your last 3
picks (primary symbols count as 2, others as 1); your top resonance fills 2
pack slots, your second resonance fills 1, and the last slot is random."

I am staying with Echo Window. The discussion reinforced my conviction for
three reasons:

**1. Three-domain convergence validates the 2/1/1 formula.** Agent 1's
Running Tally Slots, Agent 2's Majority Rules, and my Echo Window all
independently arrived at "top resonance gets 2 slots, second gets 1, last
is random." This formula is a strong equilibrium — it provides exactly 2
archetype-fitting cards per pack (matching the convergence target) while
guaranteeing splash through the random slot and secondary resonance slot.

**2. The priority ordering favors flexibility.** The orchestration plan ranks
Not-on-rails (Goal 2) and No-forced-decks (Goal 3) above Convergent
(Goal 5). Echo Window is the strongest champion on Goals 2-4 because its
3-pick memory means the player is never permanently committed. A pivot costs
exactly 3 picks — genuine but not punishing. Weighted Lottery and Lane
Locking both create permanent commitments that conflict with the top-priority
goals.

**3. The honest weakness is tolerable.** Echo Window's signal reading
(Goal 8) is genuinely poor — it responds to picks, not offers. But
Goal 8 is the lowest-priority goal. Trading the lowest-priority goal for
dominance on Goals 2-4 is the right tradeoff.

## Modifications for Round 3 Simulation

1. **Window size sweep: 3 vs 4.** Window of 3 allows fastest pivots but may
   be too volatile. Window of 4 smooths noise while still allowing pivots
   within 4 picks. Both will be simulated.

2. **Graceful early handling.** With 0 picks, pack is fully random (4 random
   slots). With 1 pick, use that pick's data but only fill 1 top-resonance
   slot (not 2), leaving 3 random. With 2 picks, fill 2 top-resonance and
   1 second-resonance only if the data supports it; otherwise more random
   slots. Full 2/1/1/random kicks in at pick 3+.

3. **Slot allocation variants.** Test 2/1/1 (base), 2/1/0+1random (weaker
   convergence, more splash), and 3/1/0/0 (stronger convergence, less
   splash).

4. **Primary weight sweep: 2 vs 3.** Higher primary weight makes the top
   resonance separate faster from the pack, potentially improving
   convergence stability at the cost of secondary resonance representation.

## Proposed Symbol Distribution

| Symbol Count | Percentage | Card Count |
|---|---|---|
| 0 symbols (generic) | 10% | 36 |
| 1 symbol | 20% | 65 |
| 2 symbols | 60% | 194 |
| 3 symbols | 20% | 65 |

**Rationale:** The 20/60/20 split (among non-generic cards) centers on
2-symbol cards. Each 2-symbol pick contributes ~3 weighted points (2 primary
+ 1 secondary). A 3-card window accumulates ~9 points, distributed across
resonances. A committed player picking three [Tide, Zephyr] cards has
Tide = 6, Zephyr = 3, others = 0 — a clear separation that reliably places
Tide in the 2-slot position and Zephyr in the 1-slot position. The 20%
1-symbol cards provide concentrated resonance signals, and the 20% 3-symbol
cards reward deep archetype commitment with higher total window contribution.
