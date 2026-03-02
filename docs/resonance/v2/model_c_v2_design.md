# Model C v2: Tiered Weighted Sampling with Soft Floors

## One-Sentence Player Explanation

"Each quest has a different mix of strategies available -- the system nudges you toward your chosen archetype after you commit, but always tempts you with powerful alternatives."

## What Changed from v1

The Round 3 debate demolished the carousel. All four agents agreed its complexity did not justify its outputs: Model D's uniform random already achieved 4.24 unique archetypes per early pack (40% above the 3.0 target), while the carousel's 5.66 exposed 81% of the archetype space, removing mystery rather than adding it. The anchor slot's hard guarantee eliminated pack-to-pack tension. Model C v1's convergence at pick 3 was catastrophic -- caused by over-sensitive commitment detection, not the carousel itself, but the carousel amplified the problem by feeding fitting cards too aggressively.

The v2 design retains Model C's structural insights (sub-pool thinking, soft floor concept, commitment detection sensitivity analysis) but rebuilds pack construction from scratch around the debate consensus.

## Number of Archetypes: 8 (2 Suppressed per Run)

Adopted from Model D. N=8 gives 45+ S-tier cards per archetype, 28 suppression configurations (C(8,2)), and strong early diversity (90% of random 4-card packs show 3+ active archetypes). With 2 suppressed, 6 active archetypes per run get an effective density boost of ~33%.

## Card Fitness Distribution

360 unique cards across 8 archetypes with clustered neighbor topology (ring with 2 neighbors each):

| Card Type | Count | % | Profile |
|-----------|-------|---|---------|
| Narrow Specialist | 216 | 60% | S in 1, B in 1-2 neighbors, C/F elsewhere |
| Specialist with Splash | 54 | 15% | S in 1, A in 1 neighbor, B in 1-2, C/F elsewhere |
| Multi-Archetype Star | 29 | 8% | S in 2 neighbors, B in 1-2, C/F elsewhere |
| Broad Generalist | 43 | 12% | A in 2-3, B in 3-4, no S |
| Universal Star | 18 | 5% | S in 3+, high power, rare/legendary |

**Multi-archetype percentage: ~40% of cards are S/A in 2+ archetypes** when counting all cards with any S/A in 2+ (including generalists with A-tier breadth). The intentionally designed dual-archetype cards (splash + multi-star) account for ~23% of the pool. Generalists contribute A-tier flexibility across archetypes without requiring archetype-specific design work.

**Per-archetype totals:** Each archetype has ~45 S-tier unique cards (27 narrow + 7 splash + 3-4 multi-star + 2-3 universal). With A-tier from splash and generalists, each archetype has ~55-65 S/A unique cards. In the ~1000-entry pool, ~20-24% of entries fit any given archetype at S/A tier.

## Pack Construction: Tiered Weighted Sampling

The v2 mechanism is deliberately simple -- a single weighted sampling pass with two special rules.

**Picks 1-4 (Exploration):** Draw 4 cards uniformly from the pool. No archetype bias. The pool's natural composition (with 2 suppressed archetypes thinned to 50% copies) creates implicit signals.

**Picks 5+ (Convergence):** Once commitment is detected (earliest at pick 5), apply a weight multiplier to S/A-tier cards in the committed archetype. The multiplier uses a strong ramp:
- Picks 5-10: 7.0x
- Picks 11-20: 8.0x
- Picks 21-30: 9.0x

The ramp is stronger than v1's because with N=8 and ~40% multi-archetype cards, each archetype's S/A pool is only ~22% of the total pool. A 7x multiplier makes fitting cards roughly 60% of the weighted distribution for the 3 archetype-biased slots -- strong enough for reliable 2+ fitting without eliminating off-archetype options.

**Soft floor (the key structural innovation carried from v1):** After the weighted draw, if the pack contains 0 fitting cards (S/A in committed archetype), replace the lowest-power card with a random S/A card from the committed archetype's pool. This fires only when needed, preventing brick packs without inflating concentration. This is the minimum viable version of v1's anchor slot -- it guarantees a floor of 1 without guaranteeing a ceiling.

**Dedicated splash slot:** One of the 4 pack slots is always drawn from off-archetype cards, biased toward high power or S-tier in other active archetypes. This ensures the splashable target (>= 0.5 strong off-archetype per pack) and creates genuine "take the bomb or stay on-plan?" tension.

## Commitment Detection (Fixed)

The v1 failure at pick 3 taught the most important lesson of the debate. The v2 detection requires ALL of:
1. Pick number >= 5 (hard floor -- exploration guaranteed for picks 1-4)
2. 3+ S/A-tier picks in one archetype
3. 1+ lead over the runner-up archetype (prevents multi-archetype cards from triggering false commitment in tied archetypes)

This directly addresses v1's failure mode where multi-archetype cards triggered commitment in multiple archetypes simultaneously. The minimum pick floor ensures 4 picks of genuine exploration.

## Variety Mechanisms

**Primary: Archetype suppression** (from Model D). 2 of 8 archetypes suppressed per run, their S-tier specialists reduced to 50% copies. 28 structurally distinct configurations.

**Secondary: Starting card signal.** Player sees 3 cards from active archetypes, keeps 1 as a free pick. This provides a semi-explicit signal about which archetypes are active without revealing pool depth.

**Tertiary: Copy-count variance** (from Model A). Each card's copy count randomly adjusted +/-1 per run, creating subtle pool asymmetries that reward observation.

No depletion mechanism. The debate consensus was that depletion's signal-reading value was unvalidated and hard to explain. The suppression + starting card provide sufficient variety and signaling.

## Why This is Structurally Different from v1

v1 used sub-pools as first-class objects with slot roles and phase transitions. v2 uses a single pool with weight modifiers -- structurally similar to Model D's approach but with two innovations: (1) the soft floor guarantee as a safety net rather than a structural feature, and (2) commitment detection designed around the multi-archetype card sensitivity that v1's simulation discovered. The simplification is the point: the debate showed that pack construction complexity does not correlate with draft quality.
