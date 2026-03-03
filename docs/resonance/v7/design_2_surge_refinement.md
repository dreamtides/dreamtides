# Agent 2: Surge Packs Refinement

## Key Takeaways

- **The realistic fitness problem is a per-slot precision problem.** Under V6's
  optimistic model, each resonance-matched surge slot delivers ~100% S/A. Under
  moderate fitness (50% A, 30% B, 20% C for cross-archetype cards), precision
  drops to ~75%. Three surge slots at 75% precision yield ~2.25 S/A instead of
  ~3.0, dragging overall late-game S/A from 2.05 toward ~1.7.
- **Normal packs are the weakest link.** In V6, normal packs contribute ~1.0
  S/A. Since committed players see normal packs ~40% of the time after
  convergence, improving normal-pack quality is the highest-leverage change.
  Even a small floor (1 resonance-matched slot in non-surge packs) adds ~0.3-0.5
  S/A to the overall average.
- **Surge frequency matters more than surge intensity under degraded fitness.**
  When each surge slot is worth less, having more surges (lower threshold)
  compensates more effectively than having bigger surges (more slots per surge).
- **Dual-resonance targeting in surges improves archetype precision but has
  limited payoff.** Splitting surge slots between primary and secondary
  resonance narrows archetype ambiguity from 4 to 1-2, but the secondary pool
  delivers lower raw S/A because secondary-resonance cards are B-tier in the
  home archetype.
- **Simplification is possible but costly.** Removing token counters in favor of
  streak-based triggers halves the state tracking but reduces surge frequency
  and removes the rhythmic predictability that makes Surge Packs work.
- **The minimum viable S/A under moderate fitness is ~1.8-1.9.** Pure Surge
  Packs (V6 spec) likely falls to ~1.65-1.75 under moderate fitness. Refinements
  must recover 0.25-0.35 S/A without breaking variance or convergence timing.

## Five Algorithm Proposals

### 1. Turbo Surge (Lower Threshold)

**One-sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when
any counter reaches 3, spend 3 and fill 3 of the next pack's 4 slots with cards
of that resonance, fourth slot random."

**Technical description:** Identical to V6 Surge Packs except the trigger
threshold drops from 4 to 3. A committed player drafting a double-symbol card
(+2 primary) triggers a surge in ~1.0 picks instead of ~1.3. This increases
surge frequency from ~60% to ~75% of late-game packs, compensating for reduced
per-slot precision under realistic fitness. The tradeoff is faster convergence
(possibly below pick 5) and reduced normal-pack frequency, which may hurt
variance and early openness.

**Predicted behavior:** Under optimistic fitness, likely pushes S/A above 2.2
and may converge too fast (pick 4). Under moderate fitness, the higher surge
rate should keep S/A above 1.9. Risk of failing M5 (convergence timing) and M9
(stddev) if surges become too frequent.

### 2. Surge + Floor (Guaranteed Minimum)

**One-sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when
any counter reaches 4, spend 4 and fill 3 of 4 slots with cards of that
resonance; on non-surge packs, 1 slot always shows a card of the player's top
resonance."

**Technical description:** Extends V6 Surge Packs with a guaranteed floor: when
no surge triggers, one of the four pack slots is filled with a random card whose
primary resonance matches the player's current highest token counter, and the
other three are random. This eliminates the "drought" problem where normal packs
deliver ~1.0 S/A. The floor slot tracks the same non-permanent counter state as
surges, so it follows pivots naturally. Token earning and surge mechanics are
unchanged from V6.

**Predicted behavior:** Under optimistic fitness, floor packs deliver ~1.75 S/A
(up from ~1.0), pushing overall S/A toward 2.3. Under moderate fitness, floor
packs deliver ~1.5 S/A, keeping overall above 1.9. The floor also accelerates
convergence slightly (possibly to pick 4-5) because even non-surge packs provide
some archetype support. Risk: M2 (early S/A) may increase above 2 if the floor
activates too early, and M9 (stddev) may drop because the floor smooths out the
surge/normal contrast.

### 3. Dual-Resonance Surge

**One-sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when
any counter reaches 4, spend 4 and fill the next pack with 2 cards of the top
resonance, 1 card of the second-highest resonance, and 1 random card."

**Technical description:** Modifies the surge pack composition to target both
the player's primary and secondary resonance. Instead of 3 slots of the
triggering resonance + 1 random, surge packs use 2 slots for the triggering
(primary) resonance, 1 slot for the player's second-highest resonance counter,
and 1 random slot. The secondary slot targets the other half of the player's
archetype identity, narrowing from 4 candidate archetypes to 1-2. Under
optimistic fitness this slightly reduces S/A because secondary-resonance cards
are B-tier more often. Under realistic fitness it may improve precision because
the secondary signal helps the algorithm "find" the specific archetype rather
than the broad resonance group.

**Predicted behavior:** Under optimistic fitness, slightly lower S/A than V6
(~1.95) because the secondary slot delivers B-tier instead of A-tier cards.
Under moderate fitness, possibly comparable or slightly better than V6 because
archetype precision improves. The secondary slot is a hedging mechanism: it
trades peak performance for robustness.

### 4. Mega Surge (Full Pack)

**One-sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when
any counter reaches 5, spend 5 and fill all 4 of the next pack's slots with
cards of that resonance."

**Technical description:** Increases both the threshold (to 5) and surge slot
count (to 4). Every surge pack is a complete resonance pack with no random wild
card. The higher threshold reduces surge frequency (committed players surge
~every 1.7 picks instead of 1.3), but each surge is more powerful. The absence
of a random slot eliminates the guaranteed splash opportunity in surge packs.

**Predicted behavior:** Under optimistic fitness, surge packs deliver ~4.0 S/A
but occur less often, producing overall S/A comparable to V6 (~2.0-2.1). Under
moderate fitness, the full-resonance surge is ~3.0 S/A, which is strong, but
lower frequency means normal packs dilute the average more heavily. Risk of
failing M4 (off-archetype cards) because surge packs have zero splash slots.

### 5. Streak Surge (Simplified)

**One-sentence:** "After pick 3, if the card just drafted shares its primary
resonance with the player's most-drafted primary resonance, the next pack fills
3 of 4 slots with cards of that resonance."

**Technical description:** Eliminates token counters entirely. The surge trigger
is purely reactive: if the most recent pick's primary resonance matches the
player's historically most-common primary resonance (by count, not weighted), a
surge fires for the next pack. No thresholds, no spending, no accumulation.
State is just "which resonance have I drafted most?" and "what was my last
pick's resonance?" This makes the algorithm dramatically simpler but less
predictable -- drafting a high-power off-resonance card breaks the streak and
cancels the upcoming surge.

**Predicted behavior:** Under optimistic fitness, surge rate for committed
players is ~55-65% (similar to V6), so S/A should be in the 1.9-2.1 range. Under
moderate fitness, the lower surge reliability compared to token accumulation
could push S/A to ~1.5-1.7. The main advantage is radical simplicity; the main
risk is that power-chaser and signal-reader strategies may see chaotic surge
patterns.

## Champion Selection: Surge + Floor

**Justification:** The core V7 problem is fitness degradation -- Surge Packs
drops below 2.0 under moderate fitness because normal packs contribute too
little. Surge + Floor is the most targeted fix: it directly addresses the
normal-pack weakness by guaranteeing 1 resonance-matched slot in every non-surge
pack. This is the smallest possible change to V6's design that has the largest
impact on robustness.

Why not the other proposals:
- **Turbo Surge** risks converging too fast and losing variance -- it trades one
  passing metric (M3) for potential failures on M5 and M9.
- **Dual-Resonance Surge** slightly reduces optimistic-case S/A while providing
  uncertain improvement under realistic fitness. The secondary slot's B-tier
  precision may not offset the loss.
- **Mega Surge** has the right intuition (bigger surges) but infrequent surges
  mean more normal-pack exposure, which is the core problem.
- **Streak Surge** sacrifices the rhythmic predictability that is Surge Packs'
  signature strength.

Surge + Floor retains all of V6 Surge Packs' strengths (non-permanent state,
rhythmic surges, natural variance, zero decisions) while adding a safety net
that specifically targets the realistic-fitness weakness.

## Champion Deep-Dive: Surge + Floor

### One-Sentence Description

"Each drafted symbol adds tokens (+2 primary, +1 others); when any counter
reaches 4, spend 4 and fill 3 of 4 slots with cards of that resonance; on
non-surge packs, 1 slot always shows a card of the player's top resonance."

### Example Draft Sequences

**Committed Warriors player (Tide primary, Zephyr secondary):**
- Picks 1-3: Normal packs (no floor yet, tokens accumulating). Tide tokens: 0,
  2, 4.
- Pick 4: Surge fires (Tide). Pack: 3 Tide + 1 random. Picks a Warriors card.
- Pick 5: Non-surge, floor active. Pack: 1 Tide + 3 random. Tokens: Tide 2
  (earned from pick 4).
- Pick 6: Non-surge, floor. Pack: 1 Tide + 3 random. Tokens: Tide 4. Surge
  queued.
- Pick 7: Surge pack. 3 Tide + 1 random.
- Pattern continues: surge every ~2 picks, floor pack between surges.

**Signal reader exploring, then committing to Storm (Ember primary, Stone
secondary):**
- Picks 1-3: Floor shows top-resonance card, but top resonance shifts as the
  player explores. Ember overtakes by pick 3.
- Pick 4-5: Floor consistently shows Ember. Tokens accumulating.
- Pick 6: First Ember surge fires. Player sees 3 Ember cards, confirms Storm
  commitment.
- Pick 8+: Alternating surge/floor packs, both Ember-focused.

### Failure Modes

1. **Early overcommitment (M2 risk):** The floor provides resonance support from
   pick 1, potentially pushing M2 above 2.0 if the floor activates before the
   player has accumulated enough signal. Mitigation: delay floor activation
   until pick 3 or until any token counter reaches a minimum (e.g., 2).
2. **Reduced variance (M9 risk):** The floor smooths out the surge/normal
   contrast. V6's 1.42 stddev comes from surge packs (~2.5 S/A) alternating with
   normal packs (~1.0 S/A). If floor packs rise to ~1.5-1.75, the contrast
   shrinks and stddev may drop below 0.8.
3. **Concentration creep (M6 risk):** The floor adds resonance support to every
   pack, potentially pushing deck concentration above 90%.

### Parameter Variants

**Variant A (Delayed Floor):** Floor activates only after pick 4 (or after any
counter reaches 3). This preserves early openness and avoids M2 failure.

**Variant B (Probabilistic Floor):** The floor slot shows a resonance-matched
card 75% of the time instead of 100%. This preserves more variance while still
providing uplift. Estimated S/A improvement over V6: +0.2-0.3 instead of
+0.4-0.5.

**Variant C (Turbo Floor):** Combine Turbo Surge (T=3) with the floor. More
frequent surges + floor support = maximum S/A resilience. Risk: may converge too
fast and lose variance.

### Proposed Fitness Models for Testing

1. **Optimistic (V6 baseline):** Adjacent-resonance cards are always A-tier.
   Cross-archetype S/A precision = 100%.
2. **Moderate:** 50% A, 30% B, 20% C for adjacent-resonance cards.
   Cross-archetype S/A precision ~75%. This is the primary realistic target.
3. **Pessimistic:** 25% A, 40% B, 35% C. Precision ~62%. Stress test for highly
   specialized archetypes.

Under the moderate model, I predict:
- V6 Surge Packs: ~1.65-1.75 S/A (fails M3)
- Surge + Floor (Variant A): ~1.85-1.95 S/A (borderline M3)
- Surge + Floor (Variant C / Turbo): ~1.95-2.10 S/A (may pass M3 but risk M5/M9)

The card designer's budget this implies: if Surge + Floor achieves 1.9 S/A under
moderate fitness, and the card designer can achieve 60% cross-archetype A-tier
(halfway between moderate and optimistic), then S/A lands at ~2.0 -- passing M3
with the algorithm doing its job and the card design doing a reasonable share.
