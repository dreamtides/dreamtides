# Agent 3: Soft Slot Targeting

## Key Takeaways from V3/V4/V5

- **The 50% dilution ceiling is real and structural.** V4 proved that any
  mechanism which merely increases the probability of seeing a resonance-matched
  card caps at ~1.7 S/A because each resonance is shared by 4 archetypes.
  Roughly half of resonance-matched cards belong to the wrong archetype. This is
  a mathematical limit, not a tuning problem.

- **Crossing 2.0 requires ADDING or PLACING cards, not just biasing
  probabilities.** Lane Locking deterministically places cards into locked
  slots; Pack Widening adds bonus cards. Pure probabilistic weighting (Sqrt
  Affinity, Weighted Lottery, Exile Pressure) never crossed 2.0. A soft slot
  system that merely adjusts draw probabilities will hit the same ceiling.

- **Multiple targeted slots can compound past the ceiling.** If a single slot
  has a 70% chance of being resonance-matched, and ~50% of those are S/A, that
  slot contributes ~0.35 S/A. But if ALL FOUR slots have independent 70%
  targeting, you get ~1.4 S/A from targeting alone plus ~0.5 from baseline
  random hits on the remaining 30%. The path to 2.0 is aggressive multi-slot
  targeting, not a single soft slot.

- **Lane Locking's weakness is mechanical rigidity, not its core idea.** The
  binary lock is too deterministic -- once slots lock, every pack looks the
  same. Soft targeting preserves the "slots become more likely to show your
  resonance" idea while introducing natural variance through probability rather
  than certainty.

- **V5's pair-matching is off-limits as a primary strategy** under the 15%
  dual-resonance constraint, but dual-resonance cards can serve as high-value
  signals within a broader system.

- **Variance is a genuine design goal.** V4's Pack Widening scored well partly
  because non-spend packs were fully random. Any soft targeting system should
  produce meaningful pack-to-pack variation -- sometimes great packs, sometimes
  mediocre ones.

- **Zero decisions is non-negotiable for V6.** Pack Widening's spending
  decision, despite being its biggest strategic asset, disqualifies it. All soft
  slot logic must be fully automatic.

______________________________________________________________________

## Five Algorithm Proposals

### Proposal 1: Sigmoid Saturation

**One sentence:** Each of 4 pack slots independently draws a resonance-matched
card with probability sigmoid(player_symbols - 4) for the player's top
resonance, clamped to [5%, 85%].

**Technical description:** Track weighted symbol counts per resonance. For each
slot, compute p = 1 / (1 + exp(-(top_count - 4))) clamped to [0.05, 0.85]. With
probability p, draw a card whose primary resonance matches the player's
strongest resonance; otherwise draw randomly. All 4 slots use the same
probability targeting the same resonance.

**Goal assessment:** Hits simplicity (one clear formula), no decisions, good
variance from probabilistic slots. Misses convergence target -- at 85% cap
across 4 slots, expected resonance-matched cards = 3.4, but only ~1.7 are S/A
due to 50% dilution. Fails the 2.0 S/A threshold. The sigmoid curve means you
never get 100% or 0%, preserving openness and variance, but structurally capped.

**Preferred distribution:** 15% dual-type (54 cards), heavy on 2-symbol cards
for fast accumulation.

______________________________________________________________________

### Proposal 2: Dual-Resonance Cascade

**One sentence:** Each slot targets the player's top resonance at probability
min(top_count * 8%, 80%), and if a targeted card happens to also carry the
player's second resonance, the NEXT slot's probability gets a +20% bonus for
this pack.

**Technical description:** Compute base probability for each slot as p =
min(weighted_symbol_count_of_top_resonance * 0.08, 0.80). Process slots
sequentially: if slot N drew a dual-resonance card matching both the player's
top and second resonance, slot N+1 gets p + 0.20. This cascade rewards the
natural emergence of archetype identity (since archetypes are defined by a
primary+secondary pair). The cascade is within a single pack generation, not
across picks.

**Goal assessment:** Creative attempt to leverage dual-resonance cards as
archetype signals. But with only 15% dual-type cards, the cascade triggers
rarely (~12% of targeted draws). The bonus is too inconsistent to meaningfully
change expected S/A. Still capped near 1.7-1.8 S/A. Adds complexity without
sufficient payoff.

**Preferred distribution:** Maximum 15% dual-type (54 cards), concentrated in
high-symbol-count cards to maximize cascade triggers.

______________________________________________________________________

### Proposal 3: Aggressive Multi-Slot Replacement

**One sentence:** For each of the 4 pack slots, if the player has 3+ weighted
symbols in any resonance, replace that slot's random card with a card matching
the player's top resonance with probability equal to min(top_count / 12, 0.90),
applied independently to each slot.

**Technical description:** After drawing 4 random cards for a pack, inspect each
slot independently. For each slot, with probability p = min(top_resonance_count
/ 12, 0.90), REPLACE the random card with a card drawn from the pool filtered to
the player's top resonance. This is a replacement operation, not an addition --
pack size stays at 4. At count 3, p = 25%. At count 6, p = 50%. At count 10, p =
83%. At count 12+, p = 90%. With 4 slots at 90%, expected resonance-matched =
3.6, yielding ~1.8 S/A. Still below 2.0 but approaches it.

**Goal assessment:** Simple, no decisions, strong variance from probabilistic
replacement. The replacement framing avoids pack size changes. But even at
maximum aggressiveness (90%), the 50% dilution keeps S/A around 1.8. Converges
faster than pure probabilistic approaches because the linear ramp is steep, but
ultimately hits the structural ceiling.

**Preferred distribution:** 10% dual-type (36 cards). Mono-resonance heavy to
make the "top resonance" signal clearer.

______________________________________________________________________

### Proposal 4: Split-Resonance Slot Pairs

**One sentence:** Slots 1-2 target the player's top resonance and slots 3-4
target their second resonance, each at probability min(respective_count / 10,
0.85), with slots targeting different resonances to triangulate the archetype.

**Technical description:** Track weighted symbol counts. The player's top
resonance (highest count) targets slots 1-2; second resonance targets slots 3-4.
Each slot independently replaces its random card with a resonance-matched card
at probability p = min(count / 10, 0.85). Because an archetype is defined by its
primary + secondary resonance pair, targeting BOTH resonances simultaneously
narrows the effective archetype from 4 candidates to 1-2. A Warriors player
(Tide primary, Zephyr secondary) gets Tide cards in slots 1-2 and Zephyr cards
in slots 3-4. Tide cards that are S/A for Warriors + Sacrifice, Zephyr cards
that are S/A for Flash + Ramp -- but the INTERSECTION (cards useful in a
Tide+Zephyr deck) skews toward Warriors and its neighbors.

**Goal assessment:** This is the key insight -- targeting two resonances
simultaneously narrows archetype ambiguity. Even though each resonance has 50%
dilution individually, a player seeing both Tide AND Zephyr cards is more likely
building Warriors than any other single archetype. The fitness model confirms:
Warriors cards are S-tier for Warriors, A-tier for the adjacent archetype
sharing Tide (Sacrifice) or Zephyr (Flash). Seeing both resonances concentrates
S/A hits on Warriors and its immediate neighbors. Expected S/A could reach
~2.0-2.2 because the two-resonance intersection reduces effective dilution from
50% to roughly 35-40%. Hits simplicity, no decisions, natural variance. The main
risk: early in the draft, second resonance count is low, so slots 3-4 barely
target anything -- convergence may be slow.

**Preferred distribution:** 15% dual-type (54 cards). Dual-type cards are
jackpots here -- a [Tide, Zephyr] card is S/A in both targeted slots.

______________________________________________________________________

### Proposal 5: Threshold-Triggered Soft Locks

**One sentence:** Each pack slot starts fully random; when the player's top
resonance count crosses 3, slot 1 targets that resonance at 75% probability; at
count 6, slot 2 targets at 75%; at count 9, slot 3 targets at 75%; slot 4 always
remains fully random.

**Technical description:** Combine Lane Locking's threshold-triggered
progression with soft probabilities instead of binary locks. Track weighted
symbol counts. When top resonance crosses 3, slot 1 becomes "soft-locked" -- it
draws a card matching that resonance with 75% probability, and a random card
with 25% probability. At count 6, slot 2 soft-locks similarly. At count 9, slot
3 soft-locks. Slot 4 is always random (guaranteed splash). Each soft-locked slot
targets the resonance that triggered it (could be different resonances for
different slots if the player diversifies). With 3 soft-locked slots at 75%,
expected resonance-matched cards from those slots = 2.25. At ~50% dilution, that
is ~1.13 S/A from targeting. Add ~0.5 from the random slot and random hits in
the 25% miss cases: total ~1.8-2.0 S/A.

However, this estimate assumes all 3 soft-locked slots target the SAME
resonance. If we adopt the split-resonance insight from Proposal 4 -- allowing
soft-locked slots to target the player's top TWO resonances -- the archetype
precision improves. Specifically: the first two thresholds lock to the top
resonance, the third threshold locks to the second resonance.

**Goal assessment:** Hits simplicity (threshold triggers are easy to explain and
predict), no decisions, preserves the 25% miss rate as natural variance,
guaranteed splash from slot 4. The threshold progression gives players a visible
sense of the draft narrowing. Convergence speed depends on symbol density --
with 2-symbol cards averaging ~3 weighted symbols, threshold 3 hits by pick 2,
threshold 6 by pick 3-4, threshold 9 by pick 4-5. This is aggressive but the 75%
probability keeps it from feeling deterministic. Expected S/A of ~1.8-2.1
depending on whether the split-resonance targeting reduces dilution as
theorized.

**Preferred distribution:** 12% dual-type (43 cards). Moderate dual-type to
support second-resonance identification without relying on it.

______________________________________________________________________

## Champion Selection: Proposal 5 — Threshold-Triggered Soft Locks

**Justification:** Proposal 5 is the strongest design for three reasons.

First, it inherits Lane Locking's greatest strength -- visible, predictable
threshold progression -- while replacing its greatest weakness (deterministic
100% locks) with 75% probability targeting. The 25% miss rate creates natural
pack-to-pack variance without undermining convergence.

Second, the split-resonance variant (first two soft-locks target top resonance,
third targets second resonance) borrows Proposal 4's insight about using two
resonances to triangulate archetype identity. This is the most promising path
past the 50% dilution ceiling without adding cards or requiring player
decisions.

Third, slot 4's permanent randomness guarantees splash and prevents the "on
rails" feeling that plagued Lane Locking. Even at maximum commitment, 25% of the
time a soft-locked slot shows an off-resonance card, and slot 4 is always wild.

Proposals 1-3 all hit the ~1.7-1.8 structural ceiling without a mechanism to
break through. Proposal 4's split-resonance idea is strong but lacks the
threshold progression that makes the system feel understandable and progressive.
Proposal 5 combines the best of both.

______________________________________________________________________

## Champion Deep-Dive: Threshold-Triggered Soft Locks

### One-Sentence Description

"Each pack slot starts random; when your top resonance count crosses 3, 6, and
9, one more slot begins showing a resonance-matched card 75% of the time (the
first two slots target your top resonance, the third targets your second), while
slot 4 stays fully random."

### Example Draft Sequence

**Player drafting Warriors (Tide primary, Zephyr secondary):**

- **Pick 1:** All counts at 0. Pack is 4 fully random cards. Player takes a
  [Tide, Tide] card. Tide count: 3 (2 primary + 1 secondary). Threshold 3
  crossed -- slot 1 soft-locks to Tide at 75%.
- **Pick 2:** Slot 1 targets Tide (75% hit). Slots 2-4 random. Pack might show
  1-2 Tide cards. Player takes another Tide card. Tide count: 6. Threshold 6
  crossed -- slot 2 soft-locks to Tide at 75%.
- **Pick 3:** Slots 1-2 target Tide (75% each). Slots 3-4 random. Expected ~1.5
  Tide cards from targeting + ~0.5 from random = ~2 Tide cards total. ~1 of
  those is S/A for Warriors. Player takes a [Tide, Zephyr] card. Tide count: 8,
  Zephyr count: 2.
- **Pick 4:** Slots 1-2 target Tide at 75%. Slots 3-4 random. Player takes a
  [Zephyr] card. Zephyr count: 4.
- **Pick 5:** Player takes another Zephyr card. Zephyr count: 6. But wait -- the
  thresholds track the TOP resonance. Tide is still top at 8 vs Zephyr at 6. No
  new soft-lock yet.
- **Pick 6:** Player takes a [Tide, Zephyr] card. Tide: 11, Zephyr: 7. Threshold
  9 was already crossed for Tide (at count 9, somewhere during accumulation).
  Slot 3 soft-locks -- to the player's SECOND resonance (Zephyr) at 75%. Now:
  slots 1-2 target Tide, slot 3 targets Zephyr, slot 4 random.
- **Picks 7-10:** 3 soft-locked slots producing ~2.25 resonance-matched cards
  per pack (1.5 Tide + 0.75 Zephyr). With archetype triangulation from dual
  targeting, ~55-65% of resonance-matched cards are S/A for Warriors
  specifically. Expected S/A: ~1.3-1.5 from targeting + ~0.5-0.6 from random and
  miss cases = ~1.8-2.1 total.

### Failure Modes

1. **Slow second-resonance accumulation.** If the player drafts mostly mono-Tide
   cards, their Zephyr count stays low and the third soft-lock triggers late (or
   targets Tide again, losing the triangulation benefit). Mitigation: the 12%
   dual-type cards naturally seed the second resonance. The third threshold (9)
   is high enough that by the time it triggers, most players have some secondary
   signal.

2. **Dilution still limits peak S/A.** Even with dual-resonance targeting, the
   50% dilution problem is only partially solved. The two-resonance intersection
   narrows from 4 candidate archetypes to ~2, reducing dilution from 50% to
   ~35-40%. This means S/A may plateau at 1.8-2.1 rather than the 2.5+ that Lane
   Locking achieves with deterministic placement. This is the honest structural
   challenge.

3. **Early threshold trigger on 3-symbol cards.** A [Tide, Tide, Tide] card
   gives 4 weighted Tide symbols on pick 1, instantly crossing threshold 3. This
   is fine for committed players but may feel premature. The 75% probability
   prevents it from being oppressive -- 25% of the time slot 1 still shows a
   random card.

4. **Split-resonance misidentification.** If the player's second-highest
   resonance does not match their intended archetype's secondary (e.g., they
   drafted opportunistically), slot 3 targets the wrong resonance. This is
   self-correcting: if the player commits to an archetype, their
   primary+secondary counts will reflect that archetype's resonance pair over
   time.

### Parameter Variants for Simulation

**Variant A (Conservative):** Thresholds at 4/8/12, probability 70%. Slower
progression, lower ceiling. Tests whether delayed convergence maintains early
openness better.

**Variant B (Baseline):** Thresholds at 3/6/9, probability 75%. The main
proposal. Tests the core hypothesis that 3 soft-locked slots with
split-resonance targeting can approach 2.0 S/A.

**Variant C (Aggressive):** Thresholds at 3/6/9, probability 85%. Higher
targeting probability to push past the dilution ceiling by brute force. Risk:
may feel too much like Lane Locking without the transparency of binary locks.

### Proposed Symbol Distribution

| Category            | Count | % of 360 |
| ------------------- | ----- | -------- |
| Generic (0 symbols) | 36    | 10%      |
| 1-symbol mono       | 65    | 18%      |
| 2-symbol mono       | 130   | 36%      |
| 2-symbol dual-type  | 43    | 12%      |
| 3-symbol mono       | 54    | 15%      |
| 3-symbol dual-type  | 32    | 9%       |

**Total dual-type: 75... that exceeds the 54-card cap.** Corrected:

| Category            | Count | % of 360 |
| ------------------- | ----- | -------- |
| Generic (0 symbols) | 36    | 10%      |
| 1-symbol mono       | 65    | 18%      |
| 2-symbol mono       | 140   | 39%      |
| 2-symbol dual-type  | 30    | 8%       |
| 3-symbol mono       | 65    | 18%      |
| 3-symbol dual-type  | 24    | 7%       |

**Total dual-type: 54 (exactly at the 15% cap).** Distributed as ~6-7 dual-type
cards per archetype. The 2-symbol dual-types (e.g., [Tide, Zephyr]) are the
clearest archetype signals. The 3-symbol dual-types (e.g., [Tide, Tide, Zephyr])
provide heavy primary accumulation plus the secondary signal.

**Justification:** Heavy 2-symbol mono cards (39%) provide the backbone for fast
threshold triggers. The full 54 dual-type cards maximize the archetype
triangulation effect that makes the third soft-lock valuable. A committed player
drafting 2-symbol cards averages ~3 weighted symbols per pick, reaching
threshold 3 by pick 1-2, threshold 6 by pick 3, and threshold 9 by pick 4-5 --
fast enough for convergence by pick 6.
