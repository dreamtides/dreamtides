# V7 Agent 4: Layered Mechanisms

## Key Takeaways

- **Sub-threshold mechanisms fail individually because they address different bottlenecks.** Pool sculpting improves the *base rate* of good cards but cannot guarantee them in any given pack. Bonus-card injection guarantees *additional* good cards but fires too infrequently. Combining a steady base-rate improver with an occasional injector attacks both bottlenecks simultaneously.

- **Synergy requires the layers to operate on different timescales.** A continuous background layer (always active, small effect) paired with a periodic foreground layer (occasional, strong effect) produces better S/A than two continuous layers or two periodic layers, because the background layer raises the floor on "bad" packs while the foreground layer raises the ceiling on "good" packs.

- **The simplicity constraint eliminates most combinations.** Pool sculpting + soft locks + bonus cards is three mechanisms and fails the one-sentence test. The viable path is finding two mechanisms that *feel like one* -- where the second layer is a natural consequence of the first rather than a separate system.

- **Realistic fitness models hit layered systems harder than single mechanisms.** Each layer that relies on resonance-level targeting degrades by the same ~25% (Moderate) or ~37% (Pessimistic) per matched slot. Two layers both degrading produces compounding disappointment. The strongest layered designs include at least one component that is fitness-model-independent.

- **Token accumulation is the natural unifying abstraction.** Most V6 mechanisms already use resonance tokens. A layered system that uses a single token pool but triggers two different effects at different thresholds feels like one mechanism to the player while delivering two distinct benefits.

- **The realistic target is 1.7-1.8 S/A under Moderate fitness.** Agent 1's baselines show all pure-resonance algorithms landing at 1.6-1.7 under Moderate. A well-designed layered system recovering 0.1-0.2 above the best baseline would represent a meaningful improvement, even if it does not cross 2.0.

- **Background pool bias is the cheapest layer to add.** A mild weight boost (2x for top resonance) on random slot draws costs zero additional complexity if the system already tracks resonance counters, and it contributes 0.15-0.25 S/A without any threshold or trigger.

## Five Algorithm Proposals

### 1. Biased Surge ("Surge Packs with Weighted Random Slots")

**One sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 4, spend 4 and fill 3 slots with that resonance's cards; the remaining slot and all non-surge pack slots draw from a pool weighted 2x toward your top resonance."

**Technical:** This layers Surge Packs (periodic PLACE mechanism) with continuous pool weighting (probabilistic background). During surge packs, 3 slots are filled deterministically and 1 slot is biased. During normal packs, all 4 slots are biased. The weighting is mild (2x, not exclusive), so normal packs remain exploratory while slightly favoring the emerging archetype. Under optimistic fitness, the bias adds ~0.3 S/A to normal packs (from ~1.0 to ~1.3) and ~0.1 to surge packs, for a blended improvement of ~0.15-0.20 above base Surge Packs. Under Moderate fitness, the improvement shrinks to ~0.10-0.15 because weighted draws still suffer from cross-archetype fitness degradation.

**Predicted:** Optimistic ~2.20 S/A. Moderate ~1.75 S/A. The bias helps but does not fundamentally change the math.

### 2. Sculpted Surge ("Pool Sculpting Between Surges")

**One sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 4, spend 4 and fill 3 slots with that resonance; between surges, replace 6 random pool cards with cards of your top resonance."

**Technical:** Combines Surge Packs (PLACE, periodic) with mild pool sculpting (probabilistic, continuous). After each pick, regardless of surge status, 6 off-resonance cards in the pool are swapped for top-resonance cards from a reserve. This is slower than V6's pool sculpting (6/pick vs 18/pick) to avoid oversaturation, but over 10 picks it shifts the pool composition by ~15-20% toward the top resonance. The sculpting improves normal-pack quality without requiring a separate description -- it is a natural extension of "the draft system tracks your resonance." Under optimistic fitness, normal packs improve from ~1.0 to ~1.35 S/A, blended improvement ~0.20. Under Moderate, improvement ~0.12.

**Predicted:** Optimistic ~2.25 S/A. Moderate ~1.72 S/A. Pool sculpting adds modest but consistent value.

### 3. Dual-Threshold Surge ("Mini-Surge at 2, Full Surge at 5")

**One sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 2, spend 2 and fill 1 slot with that resonance; when any reaches 5, spend 5 and fill 3 slots with that resonance."

**Technical:** This replaces the single-threshold Surge Packs with a dual-threshold system: frequent mini-surges (1 guaranteed slot, fires every ~1 pick for committed players) and occasional full surges (3 guaranteed slots, fires every ~2.5 picks). The mini-surge provides a steady floor of ~1.5 S/A on "normal" packs, while full surges deliver the 2.5+ S/A spikes. The two thresholds operate from the same token counter, making this feel like one system with two intensities rather than two mechanisms. Under optimistic fitness, the mini-surge adds ~0.5 S/A to frequent packs, but the higher full-surge threshold (5 vs 4) means full surges fire ~20% less often. Net effect: slightly higher average S/A but lower variance. Under Moderate fitness, each guaranteed slot is worth 0.75 instead of 1.0, so mini-surges add ~0.375 instead of ~0.5.

**Predicted:** Optimistic ~2.15 S/A. Moderate ~1.70 S/A. The mini-surge floor compensates for the less frequent full surge.

### 4. Surge + Guaranteed Floor ("Always One, Sometimes Three")

**One sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); one pack slot always shows a card of your top resonance (once any counter reaches 3); when any counter reaches 5, spend 5 and fill all 4 slots with that resonance."

**Technical:** Combines a permanent guaranteed slot (activated once, never deactivated, tracks current top resonance non-permanently) with an infrequent full-pack surge. The guaranteed slot acts as a continuous floor: once active, every pack contains at least one on-resonance card (~0.75-1.0 S/A contribution). The full-pack surge (all 4 slots) is a high-impact spike that fires less frequently than Surge Packs' 3-of-4 surge. The guaranteed floor is non-permanent -- it tracks whichever resonance currently leads, allowing pivoting. This feels like one mechanism: "the draft learns your preference and increasingly caters to it." Under optimistic fitness, the floor contributes ~1.0 S/A per pack and the surge contributes ~4.0 S/A when it fires (~every 3 picks for committed players). Blended: ~2.0 S/A. Under Moderate, floor drops to ~0.75 and surge to ~3.0, blended ~1.60.

**Predicted:** Optimistic ~2.00 S/A. Moderate ~1.60 S/A. The guaranteed floor helps consistency but the infrequent surge hurts the average.

### 5. Token Cascade ("Earn Fast, Spend Twice")

**One sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 3, spend 3 and add 1 bonus card of that resonance to the next pack plus weight the next pack's random slots 2x toward that resonance."

**Technical:** This combines bonus-card injection (ADD, conditional) with pool weighting (probabilistic, conditional), both triggered by the same token spend. The threshold of 3 means committed players trigger roughly every pick after the first few. Each trigger delivers a 5-card pack (4 random-but-weighted + 1 guaranteed) instead of a 4-card pack. The weighting on the random slots adds ~0.15 S/A and the bonus card adds ~0.75-1.0 S/A. Because both effects fire together and use the same token pool, this reads as one mechanism. Under optimistic fitness, the bonus card is worth ~1.0 S/A (100% resonance-matched), and weighted randoms add ~0.3 across 4 slots, yielding a triggered pack of ~2.3 S/A. With ~70% trigger rate late, blended ~2.0. Under Moderate, bonus card drops to ~0.75 and weighting to ~0.22, triggered pack ~1.85, blended ~1.65.

**Predicted:** Optimistic ~2.00 S/A. Moderate ~1.65 S/A. Clean mechanism but marginal improvement.

## Champion Selection: Biased Surge

**Justification:** Biased Surge is the champion for three reasons.

First, **simplicity.** It is the most natural layering -- Surge Packs already has "random" slots, and replacing "random" with "weighted random" is a single-word change in the description. A programmer reading the one-sentence description immediately understands both layers. The other proposals either introduce new thresholds (Dual-Threshold, Surge + Floor), variable pack sizes (Token Cascade), or hidden pool manipulation (Sculpted Surge).

Second, **synergy not just addition.** The bias layer specifically compensates for Surge Packs' known weakness: normal packs averaging only ~1.0 S/A create deep valleys between surges. The 2x bias raises normal-pack floors without touching the surge-pack ceiling, directly addressing M9 variance without sacrificing M3 convergence. The two layers are genuinely complementary.

Third, **fitness robustness.** Under Moderate fitness, Biased Surge is predicted at ~1.75 S/A -- the highest of the five proposals. More importantly, its failure mode is graceful: if bias proves weak, it degrades to standard Surge Packs (a known 9/9 algorithm). None of the other proposals have this fallback property.

The 2x weight bias is also fitness-model-partially-independent: even under pessimistic assumptions, weighting toward your top resonance increases the chance of drawing home-archetype cards (which are always S-tier regardless of fitness model). This gives Biased Surge a structural advantage over mechanisms that add resonance-matched cards generically.

## Champion Deep-Dive: Biased Surge

### Mechanism Specification

Maintain 4 resonance counters (Ember, Stone, Tide, Zephyr), starting at 0. After each pick, add +2 for primary symbol, +1 for secondary/tertiary. Before generating each pack:

1. **Surge check:** If the highest counter >= T (threshold), subtract T. Generate a surge pack: S slots filled with random cards of that resonance, remaining (4-S) slots use biased draw.
2. **Normal pack:** All 4 slots use biased draw.
3. **Biased draw:** Draw from full pool with 2x weight multiplier on cards whose primary resonance matches the player's current top resonance. All other cards at 1x weight.

### Example Draft Sequences

**Sequence A (Early Committer, Warriors/Tide):**
- Picks 1-3: Drafts Tide cards. Counter reaches ~6 Tide. Surge fires at pick 3.
- Pick 3 surge pack: 3 Tide-primary cards (mix of Warriors/Sacrifice home) + 1 biased draw (Tide card ~35% likely vs ~22% base).
- Picks 4-5: Normal packs, all biased. Each has ~1.3 S/A instead of ~1.0.
- Pick 5: Counter reaches 4 again. Another surge. Pattern: surge, 1-2 biased normals, surge.
- By pick 10: ~6 surges delivered. Deck has ~65% Tide cards, ~15% splash.

**Sequence B (Slow Reader, Storm/Ember):**
- Picks 1-4: Mixed Ember/Stone/Zephyr picks. Counters diffuse. No surge fires.
- All packs biased toward whatever leads -- initially mild because lead changes.
- Pick 5-6: Commits to Ember. Bias stabilizes. Counter reaches 4.
- Picks 7+: Regular surge/normal cycle with Ember bias. Slightly delayed convergence vs Sequence A.

**Sequence C (Power Chaser):**
- Picks cards purely by power. Resonance varies.
- Bias oscillates between resonances, providing minimal benefit (~0.05 S/A).
- Surges fire irregularly. Deck is ~55% scattered, 45% incidental archetype.
- Outcome: lower concentration (~65%), acceptable for non-committed play.

### Failure Modes

1. **Bias too weak to matter.** At 2x weight, top-resonance cards go from ~22% to ~35% of draws. Under Moderate fitness, only 75% of those are S/A, so the bias adds ~0.13 * 75% = ~0.10 S/A per slot. Across 4 normal-pack slots, that is only ~0.40 additional S/A per normal pack. If surge frequency stays the same, the net improvement may be only ~0.15 blended -- barely distinguishable from noise in a 1000-draft simulation.

2. **Bias too strong homogenizes.** If tuned higher (3x or 4x), normal packs start resembling surge packs, eliminating the surge/normal rhythm that gives Surge Packs its distinctive variance. The sweet spot must maintain the "valley" between surges.

3. **Dual-resonance interference.** The 15% dual-type cards with primary resonance matching the bias target get double-benefit (biased in AND contribute tokens to a non-primary counter). This slightly accelerates counter accumulation for secondary resonances, potentially causing unexpected secondary surges.

### Parameter Variants

**Variant A (Conservative): T=4, S=3, Bias=1.5x.**
Minimal bias, closest to pure Surge Packs. Expected improvement: +0.08 S/A optimistic, +0.05 Moderate. Safe baseline -- if this passes, any stronger bias also passes (modulo concentration).

**Variant B (Standard): T=4, S=3, Bias=2.0x.**
The primary proposal. Expected improvement: +0.15-0.20 optimistic, +0.10-0.15 Moderate. The 2x weight means top-resonance cards are twice as likely to appear in any draw. This is the "one-sentence natural" variant.

**Variant C (Aggressive): T=4, S=3, Bias=3.0x.**
Stronger bias, normal packs have ~40% top-resonance cards. Expected improvement: +0.30 optimistic, +0.20 Moderate. Risk: may push deck concentration above 90% and reduce M4 off-archetype splash below 0.5. Also reduces surge/normal contrast, potentially failing M9 variance.

### Proposed Fitness Models for Testing

**Model A (Optimistic):** Cross-archetype cards 100% A-tier. Resonance-matched slot S/A = 100%. Expected Biased Surge S/A: ~2.20.

**Model B (Moderate):** Cross-archetype cards 50% A, 30% B, 20% C. Resonance-matched slot S/A = 75%. Expected Biased Surge S/A: ~1.75. This is the critical test -- does the bias layer push Biased Surge above Surge Packs' ~1.60 Moderate baseline?

**Model C (Pessimistic):** Cross-archetype cards 25% A, 40% B, 35% C. Resonance-matched slot S/A = 62.5%. Expected Biased Surge S/A: ~1.50. At this level, the question is whether Biased Surge degrades more gracefully than plain Surge Packs (~1.38).

The key simulation question: **Under Moderate fitness, does the 2x bias recover enough S/A to close the gap between Surge Packs (~1.60) and the 2.0 target?** The analytical prediction is ~1.75 -- a meaningful 0.15 improvement but still short of 2.0. If this holds, the conclusion is that layering alone cannot compensate for the fitness model problem, and the card designer must be given a concrete cross-archetype fitness target.
