# Agent 7: Open Exploration -- Algorithm Design

## Key Takeaways from V3/V4/V5

1. **The 50% dilution ceiling is the core problem.** Each resonance is shared by
   4 archetypes, so a resonance-matched card is S/A for the target archetype
   only ~50% of the time. Probabilistic approaches that weight by resonance are
   structurally capped at ~1.7 S/A. This is mathematical, not tunable.

2. **Crossing 2.0 requires injection or deterministic placement.** V3 Lane
   Locking (deterministic slot locks) hit 2.72 S/A. V4 Pack Widening (bonus
   cards added to pack) hit 3.35 S/A. Every probabilistic-only approach in V4
   capped at 1.26--1.74. The threshold is clear: you must ADD or PLACE cards to
   break through.

3. **Lane Locking's weakness is rigidity, not convergence.** 99% deck
   concentration, permanent locks, no pivoting, mechanical feel. The algorithm
   works too well -- it removes all tension from the draft after commitment.

4. **Pack Widening's weakness is the spending decision.** V6 bans player
   decisions beyond card selection. Auto-spending on the highest resonance is
   the obvious fix, but "always auto-spend on highest" is equivalent to Lane
   Locking with extra steps -- it mechanically always enhances the dominant
   resonance.

5. **Dual-resonance cards are precision signals but too rare to rely on.** At
   15% (54 cards), they provide archetype-specific identification but cannot be
   a primary convergence mechanism. They are a supplementary tool.

6. **The variance-convergence tradeoff is the design frontier.** Lane Locking
   maximizes convergence at the cost of variance. Probabilistic approaches
   maximize variance but cannot converge. The winning V6 algorithm must find a
   third path: injecting archetype-relevant cards while preserving stochastic
   pack composition.

7. **History and trajectory matter more than totals.** V3/V4 algorithms mostly
   track cumulative symbol counts. But a player who drafts Tide, Tide, Tide,
   Zephyr, Zephyr is on a different trajectory than one who drafts Zephyr,
   Zephyr, Tide, Tide, Tide -- the former is likely pivoting, the latter is
   deepening. Algorithms that read trajectory can respond more intelligently
   than those that read only totals.

______________________________________________________________________

## Five Algorithm Proposals

### 1. Momentum Injection

**One sentence:** After each pick, if the picked card shares a primary resonance
with the previous pick, replace one random pack slot in the next pack with a
card matching that resonance; otherwise all 4 slots are random.

**Technical description:** Track the primary resonance of the most recently
picked card. When the player picks a card whose primary resonance matches the
previous pick's primary resonance (a "streak"), one slot in the next pack is
filled with a random card whose primary resonance matches the streak resonance.
If the player picks a different primary resonance or a generic card, the streak
breaks and all 4 slots are fully random. Only consecutive matches count -- the
streak is binary (on/off), not cumulative.

**Design goal assessment:** Passes simplicity (one clear trigger condition), no
player decisions, creates natural variance (streaks break and reform). Weakness:
binary on/off means only 1 guaranteed slot at best -- insufficient for 2.0+ S/A.
A committed player who always picks matching cards gets 1 guaranteed resonance
slot, which is ~0.5 S/A (50% archetype dilution) plus ~0.75 S/A from the 3
random slots, totaling ~1.25 S/A. Does not cross 2.0.

**Symbol distribution:** Standard -- 15% dual-type (54 cards), 55% 2-symbol, 25%
1-symbol, 20% 3-symbol.

### 2. Echo Accumulator

**One sentence:** Each drafted symbol adds 1 point to that resonance's
accumulator (primary adds 2); whenever any accumulator reaches 5, it resets to 0
and the next pack gets one slot filled with a random card of that resonance.

**Technical description:** Maintain 4 resonance accumulators starting at 0.
After each pick, add points per the standard weighting (primary=2,
secondary/tertiary=1). Whenever an accumulator hits or exceeds 5, subtract 5 and
inject one resonance-matched card into the next pack (replacing a random slot).
Multiple accumulators can fire on the same pick if a multi-symbol card pushes
several over the threshold. Generic cards add nothing. The rhythm is automatic
-- no player choice about when or where to spend.

**Design goal assessment:** Simple (accumulate and auto-fire), zero decisions,
transparent (player can count symbols to predict when the next injection fires).
A committed player earning ~3 weighted symbols per pick in their primary
resonance fires the injection roughly every 2 picks. Over picks 6+, that
averages 0.5 injected slots per pack plus ~0.75 S/A from the 3 random slots =
~1.25 S/A. Still below 2.0 with a single injection per firing. Would need
threshold 3 and multi-slot injection to approach 2.0, which complicates the
mechanism.

**Symbol distribution:** Standard 15% dual-type.

### 3. Resonance Gravity

**One sentence:** Each pack slot independently draws from a weighted pool where
cards matching your highest-count resonance get 3x weight and cards matching
your second-highest get 2x weight, but whenever a slot would show a third
consecutive resonance-matched card, it is rerolled to a random card instead.

**Technical description:** Track cumulative weighted resonance counts. Identify
the player's top-2 resonances by count. When generating each of the 4 pack
slots, draw from a weighted pool: cards whose primary resonance matches the top
resonance get 3x draw weight, second-highest get 2x, all others get 1x. However,
maintain a per-slot memory: if the last 2 packs both had a resonance-matched
card in the same slot position, the third consecutive pack rerolls that slot to
a uniform random draw. This creates a "gravity" pull toward the player's
resonances with an anti-monotony circuit breaker.

**Design goal assessment:** This is fundamentally probabilistic with weight
boosting. Even at 3x weight for the top resonance, the expected S/A per slot is
roughly (3x * 50% archetype match * resonance fraction of pool) -- still likely
below 2.0 due to the 50% dilution. The anti-monotony reroll further reduces
convergence. Elegant but mathematically capped. Estimated ~1.6-1.8 S/A at best.

**Symbol distribution:** Standard 15% dual-type.

### 4. Surge Packs (CHAMPION)

**One sentence:** Each drafted symbol adds 1 token to that resonance (primary
adds 2); when any resonance reaches 6 tokens, all 6 are automatically spent to
make the next pack a "surge pack" where 2 of 4 slots are filled with cards of
that resonance.

**Technical description:** Maintain 4 resonance token counters. After each pick,
add tokens per standard weighting. When any counter reaches or exceeds 6,
automatically subtract 6 and flag the next pack as a "surge pack" for that
resonance. In a surge pack, 2 of the 4 slots are filled with random cards whose
primary resonance matches the surge resonance; the other 2 slots are filled
randomly from the full pool. If multiple resonances cross 6 simultaneously, only
the highest fires (ties broken randomly); the other retains its tokens.
Non-surge packs have all 4 slots random. The cycle repeats indefinitely.

**Design goal assessment:** Hits simplicity (one sentence, concrete operations),
zero decisions, natural variance (alternating surge/non-surge packs),
convergence potential (2 resonance-matched slots in surge packs). A committed
player earning ~3 primary tokens per pick fires a surge every 2 picks. In surge
packs: 2 resonance slots give ~1.0 S/A (50% archetype match each) + 2 random
slots give ~0.5 S/A = ~1.5 S/A. In non-surge packs: ~1.0 S/A from 4 random
slots. Averaging surge and non-surge: ~1.25 S/A. Still below 2.0 with 50%
dilution.

**The key insight for breaking through 2.0:** Surge packs should use the
player's *resonance pair* (top two resonances) to select the 2 injected slots,
not just the top resonance. A card that matches BOTH the player's top-2
resonances (either as primary+secondary or through the dual-resonance type) is
far more likely to be S/A for their archetype. With the surge slots drawing from
the *intersection* of the top-2 resonance pools (i.e., cards whose primary
resonance matches #1 AND that also carry a symbol of resonance #2), the
archetype precision jumps from ~50% to ~80-90% for dual-type cards and remains
~50% for mono-type, averaging well above 50%.

**Revised with pair filtering:** In a surge pack, the 2 surge slots first
attempt to draw cards that match the player's top resonance as primary AND carry
any symbol of their second resonance. If insufficient cards exist in this
intersection, fall back to top-resonance-only. With 54 dual-type cards (15%) and
~270 mono-type, the intersection pool is small but non-zero (~6-7 cards per
archetype pair). This is a supplementary boost, not a primary mechanism.

**Revised estimate:** Surge packs with pair filtering: ~1.3-1.5 S/A for the 2
surge slots + ~0.5 S/A from 2 random slots = ~1.8-2.0 S/A. Non-surge: ~1.0 S/A.
Average across all late packs: ~1.4-1.5 S/A. Still short. The 15% dual-resonance
cap limits pair filtering's power.

**Final revision -- lower threshold to 4, stronger surge:** At threshold 4, a
committed player surges roughly every 1.3 picks (nearly every pack
post-commitment). With 2 surge slots per pack plus 2 random: ~1.0 + 0.5 = 1.5
S/A per surge pack. With nearly every pack being a surge pack post-commitment,
this sustains ~1.5. Still below 2.0.

**Symbol distribution:** 15% dual-type (54 cards), heavy 2-symbol (60%), light
3-symbol (15%), moderate 1-symbol (25%).

### 5. Draft Imprint

**One sentence:** After pick 5, the system identifies your top resonance and
permanently replaces 1 pack slot with a card of that resonance; after pick 10,
it identifies your top resonance pair and replaces a second slot with a card
matching both resonances (falling back to primary-only if no dual-type card is
available).

**Technical description:** For the first 5 picks, all packs are fully random (4
random slots). After pick 5, compute the player's highest weighted resonance
count; permanently assign 1 of the 4 slots to draw from that resonance's card
pool. After pick 10, compute the player's top-2 resonances as an ordered pair;
assign a second slot to preferentially draw from the intersection (dual-type
cards matching both resonances), falling back to the primary resonance pool if
the intersection is empty or depleted. The remaining 2 slots stay random. Slot
assignments update if the player's top resonance or pair changes (they are not
permanent locks -- they track the current leader).

**Design goal assessment:** Crosses 2.0 if the pair-filtered slot has high
archetype precision. With 1 primary-resonance slot (~50% S/A) and 1
pair-filtered slot (~80% S/A for dual-type, ~50% for fallback) plus 2 random
(~25% S/A each): estimated 0.5 + 0.65 + 0.5 = 1.65 S/A. Still below 2.0 because
the pair-filtered slot falls back to primary-only most of the time (small
intersection pool). Supports pivoting (slots track current leader, not
permanently locked). Simple and transparent. The weakness is the same 50%
dilution on the primary slot.

**Symbol distribution:** Maximize dual-type at 15% (54 cards) to make pair
filtering more effective.

______________________________________________________________________

## Champion Selection: Surge Packs

**Justification:** Among the five proposals, Surge Packs offers the best
combination of simplicity, zero-decision operation, natural variance, and
convergence potential. The alternating rhythm of surge and non-surge packs
creates genuine pack-to-pack variance (unlike Lane Locking's steady state),
while the automatic token-and-fire mechanic requires zero player decisions
(unlike Pack Widening's spend choice).

The honest assessment: Surge Packs in its basic form (top-resonance-only surge
slots) likely caps at ~1.5 S/A due to 50% archetype dilution. This is above pure
probabilistic approaches (1.26--1.74) but below the 2.0 target. However, Surge
Packs has a critical tuning lever that other proposals lack: **surge
intensity.** By adjusting the threshold (tokens needed to fire) and the number
of surge slots (2 vs 3), the algorithm can push harder on convergence at the
cost of variance. A 3-slot surge at threshold 4 would give ~2.0-2.25 S/A in
surge packs, and with surges firing nearly every pack post-commitment, the
average approaches 2.0.

The secondary advantage is the mechanism's transparency. Players can see their
token counts rising and anticipate when the next surge will fire. "I'm 2 tokens
away from a surge" creates excitement and anticipation -- a qualitatively
different feeling from Lane Locking's permanent state change or pure randomness.

______________________________________________________________________

## Champion Deep-Dive: Surge Packs

### Example Draft Sequences

**Scenario: Committed Warriors player (Tide primary, Zephyr secondary)**

| Pick | Card Drafted             | Tokens After (T/E/S/Z)         | Surge? | Next Pack                 |
| ---- | ------------------------ | ------------------------------ | ------ | ------------------------- |
| 1    | [Tide, Tide] generic-ish | 3/0/0/0                        | No     | Random                    |
| 2    | [Tide] Warriors card     | 5/0/0/0                        | No     | Random                    |
| 3    | [Zephyr] random card     | 5/0/0/2                        | No     | Random                    |
| 4    | [Tide, Zephyr] Warriors  | 7/0/0/3 → surge Tide (7-6=1)   | Yes    | 2 Tide slots + 2 random   |
| 5    | [Tide, Tide] from surge  | 4/0/0/3                        | No     | Random                    |
| 6    | [Tide, Zephyr] Warriors  | 6/0/0/4 → surge Tide (6-6=0)   | Yes    | 2 Tide slots + 2 random   |
| 7    | [Tide] from surge pack   | 2/0/0/4                        | No     | Random                    |
| 8    | [Zephyr, Tide] Ramp card | 3/0/0/6 → surge Zephyr (6-6=0) | Yes    | 2 Zephyr slots + 2 random |
| 9    | [Tide, Tide]             | 6/0/0/0 → surge Tide           | Yes    | 2 Tide slots + 2 random   |
| 10   | [Tide]                   | 2/0/0/0                        | No     | Random                    |

The player surges on Tide most often (~every 2 picks) but occasionally surges
Zephyr when secondary tokens accumulate. This creates a natural rhythm where
most enhanced packs favor the primary resonance but occasionally surprise with
the secondary -- potentially offering excellent off-archetype splash.

**Scenario: Flexible/exploring player (picks 1-5 spread across resonances)**

| Pick | Card Drafted     | Tokens After (T/E/S/Z) | Surge? |
| ---- | ---------------- | ---------------------- | ------ |
| 1    | [Ember, Stone]   | 0/2/1/0                | No     |
| 2    | [Tide]           | 2/2/1/0                | No     |
| 3    | [Zephyr, Zephyr] | 2/2/1/3                | No     |
| 4    | [Stone, Ember]   | 2/3/3/3                | No     |
| 5    | [Tide, Zephyr]   | 4/3/3/4                | No     |

No surges through 5 picks because tokens are spread. The player sees fully
random packs throughout exploration, preserving early openness. If they commit
to Tide from pick 6 onward, they reach the first surge at pick 7 or 8 --
convergence at pick 7-8, within the target range.

### Failure Modes

1. **Token spread prevents surging.** A player who drafts evenly across all 4
   resonances may never reach threshold 6 in any single resonance, getting zero
   surge packs. Mitigation: this player is not committing to an archetype, so
   random packs are appropriate. Once they commit, tokens accumulate quickly
   (2-3 per pick in primary).

2. **Wrong-resonance surges.** A Warriors player whose Zephyr tokens (secondary)
   fire before Tide (primary) gets 2 Zephyr-primary slots -- these are
   Flash/Ramp cards, not Warriors. Mitigation: over many picks, the primary
   resonance surges far more often. Occasional secondary surges provide useful
   splash. At threshold 6, a committed Warriors player earning 2 Tide tokens per
   pick versus 1 Zephyr fires Tide surges 2:1 over Zephyr.

3. **50% dilution in surge slots.** Even in a Tide surge pack, half the
   Tide-primary cards belong to Sacrifice (the other Tide-primary archetype).
   The player sees 2 Tide cards but only ~1 is Warriors-relevant. Mitigation:
   this is the fundamental challenge of mono-resonance signals. The 2 random
   slots provide additional lottery tickets. The net effect is still better than
   no surging.

4. **Threshold too low causes near-constant surging.** If threshold is set too
   low (e.g., 3), committed players surge almost every pack, and the mechanism
   degenerates into quasi-Lane Locking with extra steps. Mitigation: threshold 6
   ensures surges fire roughly every 2 picks, creating genuine alternation.

### Parameter Variants to Test

**Variant A: Standard (threshold 6, 2 surge slots)**

- Surge fires every ~2 picks for committed player
- 2 resonance slots + 2 random in surge packs
- Expected late S/A: ~1.5 (surge) / ~1.0 (non-surge), average ~1.25
- Best for: variance, natural feel

**Variant B: Aggressive (threshold 4, 3 surge slots)**

- Surge fires nearly every pick for committed player
- 3 resonance slots + 1 random in surge packs
- Expected late S/A: ~2.0 (surge) / ~1.0 (non-surge), average ~1.75
- Best for: convergence, approaching the 2.0 target
- Risk: feels too mechanical once committed

**Variant C: Burst (threshold 8, 4 surge slots -- full surge)**

- Surge fires every ~2.5-3 picks for committed player
- All 4 slots are resonance-matched in surge packs
- Expected late S/A: ~2.0 (surge) / ~1.0 (non-surge), average ~1.6
- Best for: dramatic pack-to-pack swings, high variance
- Risk: non-surge packs feel empty; feast-or-famine

### Proposed Symbol Distribution

| Category                  | Count   | % of 360 |
| ------------------------- | ------- | -------- |
| Generic (0 symbols)       | 36      | 10%      |
| 1 symbol, mono-resonance  | 81      | 22.5%    |
| 2 symbols, mono-resonance | 130     | 36.1%    |
| 2 symbols, dual-resonance | 54      | 15.0%    |
| 3 symbols, mono-resonance | 59      | 16.4%    |
| **Total**                 | **360** | **100%** |

**Justification:** Maximizing dual-resonance at the 15% cap (54 cards) gives the
pair-filtering fallback the largest possible pool. Concentrating dual-type cards
at 2 symbols (rather than 3) keeps the average token yield moderate (~2.5
weighted tokens per non-generic pick), which with threshold 6 produces surges
roughly every 2.4 picks -- a good rhythm. The 81 one-symbol cards provide
low-commitment early picks that do not rapidly accumulate tokens, preserving
early openness. The 59 three-symbol cards are high-commitment picks that can
push a player over the surge threshold in a single pick, creating exciting
moments.

**Per-archetype dual-type allocation:** 54 dual-type cards / 8 archetypes = 6-7
dual-type cards per archetype. Each archetype gets 6 cards with its signature
pair (e.g., Warriors gets 6 [Tide, Zephyr] cards), plus 2 remaining cards
distributed to archetypes that benefit most from pair identification.

### Honest Assessment

Surge Packs is the most promising mechanism in this exploration set, but honesty
demands acknowledging: under the 15% dual-resonance constraint, it likely falls
short of 2.0 S/A in its standard variant. The aggressive variant (threshold 4, 3
surge slots) may approach 1.75-2.0 but risks feeling mechanical. The fundamental
barrier is that surge slots draw from a resonance pool where ~50% of cards
belong to the wrong archetype.

The path to genuinely crossing 2.0 may require combining Surge Packs with a
complementary mechanism -- for example, using dual-resonance cards as "priority
draws" in surge slots (checking the dual-type pool first before falling back to
mono-type). This hybrid approach maintains the one-sentence core while adding a
card-pool ordering rule. Simulation will reveal whether this supplementary
filtering provides enough lift to clear the threshold.
