# Resonance V5 -- Final Synthesis Report

## Executive Summary

V5 investigated five domains of passive draft convergence algorithms, all using
ordered resonance pairs as their matching unit. The central finding: **pair
matching breaks the archetype dilution ceiling** that capped V4's probabilistic
approaches at ~1.7 S/A. Pair-matched cards achieve ~80% S-tier precision for
the target archetype, compared to ~50% for single-resonance matching. This
enabled two V5 algorithms to cross 2.0 S/A with zero player decisions.

**Recommended algorithm: Pair-Escalation Slots** (D2 at cap=0.50, K=6), which
achieves 2.61 S/A, convergence pick 6.3, stddev 0.98, and 0.70 off-archetype
cards per pack -- matching or exceeding both Lane Locking and Pack Widening on
convergence while requiring zero player decisions and producing natural
pack-to-pack variance.

---

## 1. Unified Comparison Table (Archetype-Level)

All metrics from committed player strategy, 1000 drafts, 30 picks each,
15/60/25 symbol distribution.

| Metric | Target | D1 PairThr | D2 (.65) | D2 (.50) | D3 Seed | D4 (3,7) | D4 (2,5) | D5 Hybrid | Lane Lock | Pack Wide | Random |
|--------|--------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Early unique archs | >=3 | 6.48 P | 6.30 P | 6.32 P | 6.47 P | 6.47 P | 6.49 P | 6.62 P | 6.54 P | 6.78 P | 6.47 P |
| Early S/A emerging | <=2 | 1.54 P | 2.00 P | 1.98 P | 1.52 P | 1.54 P | 1.64 P | 1.94 P | 1.91 P | 1.87 P | 1.53 P |
| Late S/A committed | >=2 | **1.54 F** | **2.97 P** | **2.61 P** | **1.50 F** | **2.32 P** | **2.52 P** | **1.96 F** | **2.56 P** | **2.02 P** | **1.35 F** |
| Late off-arch (C/F) | >=0.5 | 1.36 P | 0.52 P | 0.70 P | 1.27 P | 0.85 P | 0.74 P | 1.37 P | 0.89 P | 1.53 P | 1.36 P |
| Convergence pick | 5-8 | 14.7 F | 5.9 P | 6.3 P | 16.3 F | 9.6 F | 6.9 P | 9.6 F | 5.6 P | 8.6 F | 18.9 F |
| Deck concentration | 60-90% | 83.3% P | 96.9% F | 96.2% F | 83.3% P | 93.4% F | 95.9% F | 82.9% P | 97.1% F | 91.7% F | 80.2% P |
| Run-to-run overlap | <40% | 10.7% P | 28.0% P | 25.1% P | 8.0% P | 20.8% P | 26.6% P | 14.3% P | 18.1% P | 13.4% P | 9.5% P |
| S/A StdDev (late) | >=0.8 | 1.02 P | 0.92 P | 0.98 P | 0.97 P | 0.90 P | 0.78 F | 1.25 P | 0.87 P | 1.04 P | 0.94 P |
| **Targets passed** | | **6/8** | **7/8** | **7/8** | **6/8** | **6/8** | **6/8** | **6/8** | **7/8** | **6/8** | **6/8** |

**Key findings:**
- D2 at both cap values passes 7/8 targets, tying Lane Locking. All other V5
  algorithms pass 6/8.
- All algorithms that cross 2.0 S/A fail deck concentration (60-90%). This is a
  fitness model artifact: when pair-matched slots deliver S/A cards ~80% of the
  time, committed players rarely draft off-archetype.
- D4 (2,5) narrowly fails variance (0.78 vs 0.80 target). D4 (3,7) passes
  variance but fails convergence speed.
- Pack Widening auto-spend barely crosses 2.0 (2.02) and fails convergence
  speed (8.6). Single-resonance matching limits its effectiveness.

---

## 2. V5 Algorithm Ranking

| Rank | Algorithm | Late S/A | Conv Pick | StdDev | Key Advantage | Key Weakness |
|------|-----------|:-:|:-:|:-:|---|---|
| 1 | D2 PairEsc (cap=0.50) | 2.61 | 6.3 | 0.98 | Best convergence+variance balance | Deck concentration 96.2% |
| 2 | D4 DualThr (2,5) | 2.52 | 6.9 | 0.78 | Simplest algorithm crossing 2.0 | Variance 0.78, borderline fail |
| 3 | D2 PairEsc (cap=0.65) | 2.97 | 5.9 | 0.92 | Highest raw convergence | Over-convergence, 0.52 off-arch |
| 4 | D4 DualThr (3,7) | 2.32 | 9.6 | 0.90 | Best splash (0.85 off-arch) | Convergence pick too slow |
| 5 | D5 Hybrid Trigger | 1.96 | 9.6 | 1.25 | Best organic variance | Fails 2.0 S/A threshold |
| 6 | D1 Pair Threshold | 1.54 | 14.7 | 1.02 | Clean threshold-reset design | Structurally capped at ~1.5 |
| 7 | D3 Pool Seeding | 1.50 | 16.3 | 0.97 | Most natural feel, signal reading | Pool bloat caps at ~1.5 |

---

## 3. Per-Archetype Convergence Table

| Archetype | D2(.50) | D4(2,5) | D4(3,7) | LaneLock | PackWide |
|-----------|:-:|:-:|:-:|:-:|:-:|
| Flash/Tempo/Prison | 7.3 | 7.8 | 9.9 | 6.1 | 8.3 |
| Blink/Flicker | 8.0 | 7.8 | 9.9 | 5.9 | 8.6 |
| Storm/Spellslinger | 8.0 | 8.1 | 10.7 | 5.8 | 8.7 |
| Self-Discard | 7.9 | 7.9 | 10.5 | 5.8 | 8.8 |
| Self-Mill/Reanimator | 7.2 | 7.8 | 10.2 | 5.5 | 8.0 |
| Sacrifice/Abandon | 7.7 | 7.6 | 10.4 | 5.4 | 8.4 |
| Warriors/Midrange | 6.8 | 7.3 | 10.0 | 5.8 | 8.4 |
| Ramp/Spirit Animals | 7.4 | 8.2 | 10.2 | 6.4 | 8.5 |
| **Average** | **7.5** | **7.8** | **10.2** | **5.8** | **8.5** |

D2(.50) and D4(2,5) both converge within the 5-8 target window for most
archetypes. Lane Locking is fastest (5.8 avg) but uses single-resonance
matching. Pack Widening is slowest of the viable algorithms (8.5 avg). All
algorithms show excellent archetype balance (spread <1.5 picks).

---

## 4. Simplicity Test

| Algorithm | One-Sentence | Can Write Code? | Verdict |
|-----------|---|---|---|
| D2 PairEsc | "Track the ordered pair (first, second symbol) of each 2+ symbol card you draft; each pack slot independently shows a card matching your top pair with probability min(count/6, 50%), otherwise random." | Yes -- count pairs, compute probability, roll per slot. | **PASS** |
| D4 DualThr | "Track each 2+ symbol card's ordered pair; at 2 matching picks one slot is pair-matched, at 5 a second slot is pair-matched, rest random." | Yes -- count pairs, check thresholds, fill slots. | **PASS** (clearest) |
| D1 PairThr | "Each 2+ symbol card adds 1 to its pair count; at 3 your next pack gets a bonus card of that pair and the count resets." | Yes -- threshold, bonus, reset. | **PASS** |
| D5 Hybrid | "Draw 4 random; if any card's primary resonance matches your top resonance, add 1 bonus card matching your top pair." | Yes -- check trigger, add bonus. | **PASS** |
| D3 PoolSeed | "After each pick, if you have 2+ cards of the same ordered pair, 4 cards matching your top pair are added to the pool from a reserve." | Yes -- but invisible to player. | **MARGINAL** |
| Lane Lock | "At 3 symbols in a resonance, one slot locks; at 8, a second locks." | Yes -- deterministic, transparent. | **PASS** |
| Pack Wide | "Spend 3 tokens to add a bonus card of that resonance." | Requires player decision -- violates V5. | **FAIL (V5)** |

D4 has the clearest description. D2's probability formula adds mild complexity
but remains concrete. All V5 algorithms satisfy the no-player-decisions
constraint.

---

## 5. Recommended Algorithm: Pair-Escalation Slots

### One-Sentence Description

"Track the ordered symbol pair (first, second) of each 2+ symbol card you
draft; each pack slot independently shows a card matching your most common pair
with probability equal to half that pair's count divided by 6 (capped at 50%),
otherwise a random card."

### One-Paragraph Description

When you draft a card with two or more resonance symbols, the system records its
ordered pair -- the first and second symbols in order. Your pair profile tracks
how many times you have drafted each specific pair. When generating your next
pack, each of the 4 slots independently rolls against a probability: the count
of your most common pair divided by 6, capped at 50%. If the roll succeeds,
that slot shows a random card from the pool whose ordered pair matches yours.
If it fails, the slot shows a fully random card. This creates natural variance:
sometimes 3 slots are targeted, sometimes 0. A committed player averaging 2-3
targeted slots per pack sees roughly 2.6 S/A-tier cards for their archetype,
with genuine pack-to-pack fluctuation.

### Complete Specification

1. Initialize a pair counter dictionary: {ordered_pair -> count}, all zero.
2. For each pack:
   a. Identify the player's top pair (highest count) and its count.
   b. Compute probability P = min(top_count / 6.0, 0.50).
   c. Pre-filter pool to find all cards whose ordered pair matches the top pair.
   d. For each of 4 pack slots:
      - Roll random [0, 1). If < P and pair-matched cards exist: draw a random
        card from the pair-matched subset (with replacement within the subset).
      - Otherwise: draw a random card from the full pool (without replacement
        within the pack to avoid duplicates).
   e. Player picks 1 card from the pack.
3. After picking: if the chosen card has 2+ symbols, extract its ordered pair
   (symbols[0], symbols[1]) and increment that pair's counter by 1.
4. Repeat from step 2.

### Implementation Notes

- **Pair-matched draws use with-replacement** from the pair subset to avoid
  exhausting the ~25-30 cards per pair. Random draws use without-replacement
  within the pack.
- **1-symbol and generic cards** contribute nothing to pair counts. This creates
  natural "slow picks" that preserve variance.
- **Tie-breaking:** If two pairs are tied for highest, use the one drafted most
  recently. Ties are rare after pick 5.
- **Pivoting:** No permanent state. If a player shifts from (Tide, Zephyr) to
  (Tide, Stone), the old pair's count remains but the new pair's count grows.
  The system tracks the new leader automatically once it overtakes.
- **Cap at 50%:** Ensures at least 2 expected random slots per pack. Higher
  caps (65%, 80%) increase convergence but crush splash and variety.

### Recommended Symbol Distribution

| Symbol Count | % of Non-Generic | Cards |
|---|---|---|
| 0 (generic) | -- | 36 |
| 1 symbol | 15% | 49 |
| 2 symbols | 60% | 194 |
| 3 symbols | 25% | 81 |

### Per-Archetype Convergence (D2 cap=0.50)

| Archetype | Convergence Pick |
|-----------|:-:|
| Flash/Tempo/Prison | 7.3 |
| Blink/Flicker | 8.0 |
| Storm/Spellslinger | 8.0 |
| Self-Discard | 7.9 |
| Self-Mill/Reanimator | 7.2 |
| Sacrifice/Abandon | 7.7 |
| Warriors/Midrange | 6.8 |
| Ramp/Spirit Animals | 7.4 |
| **Average** | **7.5** |

All archetypes converge within 1.2 picks of each other -- excellent balance.

---

## 6. V5 vs V3 vs V4 Deep Comparison

### Quantitative Comparison

| Metric | Target | V5: D2(.50) | V3: Lane Lock | V4: Pack Wide | Winner |
|--------|--------|:-:|:-:|:-:|---|
| Late S/A | >=2 | **2.61** | 2.56 | 2.02 | V5 (+2%) |
| Conv pick | 5-8 | 6.3 | **5.6** | 8.6 F | V3 |
| S/A StdDev | >=0.8 | **0.98** | 0.87 | 1.04 | V4 |
| Off-arch | >=0.5 | 0.70 | **0.89** | 1.53 | V3 |
| Deck conc | 60-90% | 96.2% F | 97.1% F | 91.7% F | V4 (least bad) |
| Overlap | <40% | 25.1% | **18.1%** | 13.4% | V4 |
| Early unique | >=3 | 6.32 | 6.54 | **6.78** | V4 |
| Early S/A | <=2 | 1.98 | 1.91 | **1.87** | V4 |
| Player decisions | None | **None** | **None** | Spending | V5/V3 |
| Per-arch range | -- | 1.2 picks | **0.9** picks | 0.8 picks | V4 |

V5 wins on the primary convergence metric (2.61 vs 2.56 vs 2.02) and on
variance (0.98 vs 0.87 vs 1.04 -- V4 has higher variance but requires player
decisions). Lane Locking converges slightly faster (5.6 vs 6.3) and has better
splash. All three fail deck concentration, a fitness model artifact.

### Qualitative Comparison

**Player experience.** D2 Pair-Escalation produces probabilistic pack
composition -- sometimes 3 targeted slots fire, sometimes 0. The player
experiences genuine variation in pack quality, with occasional great packs (3-4
S/A cards) and occasional weak packs (0-1 S/A). This feels like natural
variation in the card pool rather than mechanical delivery. Lane Locking
delivers mechanically consistent packs after slots lock -- the player knows
what to expect. Pack Widening requires active spending decisions that add
cognitive load. **Winner: V5** -- natural variance with zero decisions.

**Cognitive load.** D2 requires the player to understand nothing beyond "draft
cards; the system does the rest." Lane Locking requires monitoring lock state
(which slots are locked, when the next threshold triggers). Pack Widening
requires tracking token balances and making spend/save decisions before each
pack. **Winner: V5** -- zero cognitive overhead.

**Transparency.** Lane Locking is most transparent -- binary lock state is
immediately visible. D2's probability is harder to observe directly but the
player can see that their packs are improving. Pack Widening's tokens are
visible but spending is a non-trivial decision. **Winner: V3** -- but
transparency matters less when the algorithm requires no decisions.

**Flexibility.** D2 supports pivots gracefully: pair probabilities shift as new
pairs are drafted. No permanent state transitions. Lane Locking's permanent
locks make pivoting impossible after pick 6-8. Pack Widening supports pivots
through redirecting spending. **Winner: V5** -- no permanent commitment, smooth
probability shifts.

**Skill expression.** D2 rewards archetype commitment (faster pair accumulation
= higher probability) and reading which pair is becoming dominant. Lane Locking
rewards reading threshold timing. Pack Widening rewards spending optimization.
**Winner: V4** -- spending decisions add genuine skill expression that D2 lacks.

**Simplicity.** D4 (2,5) is simplest: binary thresholds with deterministic
slots. D2's probability formula is concrete but requires understanding
"probability per slot." Lane Locking is equally simple. Pack Widening adds
resource management. **Winner: V3/D4 tie** -- both use threshold-based rules.

**Degeneracy resistance.** D2's probabilistic slots mean identical strategies
produce different results. Lane Locking's deterministic locks create identical
pack structures. Pack Widening has random bonus cards. **Winner: V5** --
probabilistic mechanisms are inherently harder to exploit.

**Archetype balance.** All three algorithms show near-perfect archetype balance.
D2: 6.8-8.0 pick range. Lane Locking: 5.4-6.4. Pack Widening: 8.0-8.8. All
spreads under 1.5 picks. **Verdict: Tie.**

### Clear Verdict

**V5 Pair-Escalation Slots is the recommended algorithm across V3-V5.** It
achieves the highest convergence (2.61 S/A) with zero player decisions, natural
pack-to-pack variance (stddev 0.98), and no permanent commitment. The only
tradeoffs compared to Lane Locking are slightly slower convergence (6.3 vs 5.6)
and slightly less splash (0.70 vs 0.89), both well within target ranges. The
zero-decision interface is a meaningful advantage: players focus entirely on
card evaluation rather than system management.

If simplicity is the overriding priority, **D4 Dual-Threshold (2,5)** is the
alternative recommendation. It passes 6/8 targets with the clearest one-sentence
description, though it narrowly fails the variance target (0.78 vs 0.80).

Pack Widening (V4) should be retired as a candidate. Its spending decisions are
unnecessary complexity, and single-resonance matching limits convergence to 2.02
-- barely above the threshold and 0.59 below D2.

Lane Locking (V3) remains a strong fallback for implementations where
deterministic transparency is valued over variance. Its permanent locks are the
main drawback.

---

## 7. The Pair-Matching Breakthrough

V5's most important finding is that ordered resonance pairs (primary, secondary)
achieve ~80% S-tier precision for the target archetype, compared to ~50% for
single-resonance matching. This eliminates the structural ceiling that capped
V4's probabilistic approaches at ~1.7 S/A. The 80% figure (rather than the
theoretically predicted ~100%) comes from 3-symbol cards where the pair
(primary, primary) maps to the same archetype but with non-standard symbol
ordering.

Pair matching is transformative for slot-targeting mechanisms (D2, D4) where each
targeted slot leverages the precision gain directly. For bonus-injection
mechanisms (D1, D5), the bottleneck is fire rate rather than per-bonus precision,
so pair matching provides less benefit.

---

## 8. Open Questions for Playtesting

1. **Is 96% deck concentration acceptable?** The fitness model may be too
   generous -- widening the gap between S/A and C/F tiers could bring
   concentration into the 60-90% target without algorithm changes.

2. **Does the probability formula feel natural?** Players cannot directly
   observe per-slot probabilities. Playtesting should verify that the improving
   pack quality feels organic rather than arbitrary.

3. **Is cap=0.50 the right ceiling?** Cap 0.50 guarantees 2 expected random
   slots per pack. If splash feels insufficient, cap 0.40 would improve
   off-archetype visibility at the cost of ~0.3 S/A.

4. **Should the pair concept be surfaced in the UI?** The algorithm uses ordered
   pairs internally but the player sees packs improving as they commit. Testing
   whether players intuit the pair mechanism naturally or need UI hints (e.g.,
   showing pair counts) is critical for the transparency goal.

5. **1-symbol card experience.** Cards with 1 symbol contribute no pair data.
   Drafting several 1-symbol cards in sequence creates a "stall" in convergence.
   Playtest whether this feels frustrating or creates interesting tension.

6. **Signal reading.** D2 scores poorly on signal reading (pool-independent).
   If signal reading matters, layering D3's pool seeding (+4 pair-matched cards
   per pick from reserve) would add run-to-run variety. This is a two-mechanism
   system requiring a longer description.

7. **Adjacent archetype drafting.** Warriors (Tide, Zephyr) and Ramp (Zephyr,
   Tide) are distinct pairs. A player straddling both builds counts in two
   pairs, splitting probability. Playtest whether this creates frustration or
   appropriate commitment pressure.
