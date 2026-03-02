# Weighted Lottery with Wildcard Slot — Corrected Results

## One-Sentence Algorithm

"Each resonance starts at weight 1; each drafted symbol adds to weights
(primary +2, others +1); 3 of 4 pack slots pick a resonance proportionally to
weights; the 4th slot is always a random card."

## Target Scorecard (Archetype-Committed, Baseline: sw=1, pm=2, wildcard=on)

All metrics measured at **archetype level** (S/A/B/C/F fitness), not resonance
level. The previous simulation inflated convergence and splash by conflating
resonance matching with archetype fitness.

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Picks 1-5: unique archetypes w/ S/A per pack | >= 3 | 6.56 | **PASS** |
| Picks 1-5: S/A for emerging archetype per pack | <= 2 | 2.02 | **FAIL** (marginal) |
| Picks 6+: S/A for committed archetype per pack | >= 2 | 2.32 | **PASS** |
| Picks 6+: C/F-tier cards per pack | >= 0.5 | 0.46 | **FAIL** (marginal) |
| Convergence pick (regular 2+ streak) | 5-8 | 8.5 | **FAIL** (marginal) |
| Deck archetype concentration (S/A) | 60-80% | 92.3% | **FAIL** |
| Card overlap between runs | < 40% | 8.1% | **PASS** |
| Archetype frequency range | 5-20% each | 6.4%-20.3% | **FAIL** (marginal) |

**Score: 3/8 PASS** (down from 5/8 at resonance level). The critical failure
is deck concentration at 92%: once committed, the weighted lottery floods packs
with the primary resonance, and 92% of primary-resonance cards are S/A for
the committed archetype. Old splash (1.69 "off-resonance") became 0.46 C/F.
Old convergence (6.4) became 8.5. The circle model produces zero C-tier cards
(pairs either share a resonance or share none), so the C/F metric measures only
F-tier (opposite-side) cards.

## Parameter Sensitivity

| Parameter | S/A early | S/A late | C/F late | Conv. | Deck Conc. |
|-----------|-----------|----------|----------|-------|------------|
| sw=1 (baseline) | 2.02 | 2.32 | 0.46 | 8.5 | 92.3% |
| sw=3 | 1.73 | 2.14 | 0.56 | 9.8 | 90.3% |
| sw=5 | 1.65 | 2.04 | 0.62 | 10.7 | 89.1% |
| pm=1 | 1.87 | 2.19 | 0.52 | 9.3 | 91.1% |
| pm=3 | 2.12 | 2.40 | 0.43 | 8.2 | 93.1% |
| wildcard=off | 2.25 | 2.73 | 0.26 | 7.5 | 94.6% |

Higher starting weight improves splash but delays convergence past target.
Primary multiplier has minimal effect on deck concentration (91-93%),
confirming this is structural. Wildcard off drops C/F to 0.26. **No parameter
combination fixes 92%+ deck concentration** -- this is the algorithm's
fundamental limitation at archetype level.

## Draft Traces

**Early Committer** (Blink at pick 6): Weights reach Ember 45% by pick 6.
After commitment, packs deliver 1-3 S/A Blink cards with variance (pick 8
shows 0 S/A). Final: 86.7% S/A, convergence pick 7, regular convergence
pick 10.

**Flexible Player** (power_chaser): Takes highest power regardless. Weights
stay diffuse (<40% any resonance). Final: 53.3% S/A -- only strategy
approaching the 60-80% band. Regular convergence delayed to pick 14. Algorithm
correctly does not railroad unfocused drafters.

**Signal Reader** (Blink at pick 8 via Ember signal): Ember hits 65% weight by
pick 4. By picks 9-11, sees 3-4 S/A Blink cards per pack. Final: 93.3% S/A.
Archetype frequency badly skewed (Blink 35.6%, Self-Mill 3.0%) -- amplifies
pool composition artifacts rather than rewarding skill.

## Self-Assessment

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | **9** | One sentence is the complete algorithm. Player can predict "more Tide = more Tide cards." |
| 2. Not on rails | **7** | Power-chaser at 53% S/A proves flexible play is viable, but committed players face 92% convergence. |
| 3. Can't force same deck | **9** | 8.1% card overlap ensures high variety. |
| 4. Flexible archetypes | **6** | B-tier cards (44% of pool) appear but are invisible to metric. Weight system pushes primary resonance. |
| 5. Convergent | **5** | Passes >= 2 S/A late (2.32) but convergence pick 8.5 is late and 92% deck concentration overshoots. |
| 6. Splashable | **5** | 0.46 C/F per pack, just under 0.5. Wildcard helps but cannot overcome zero C-tier in circle model. |
| 7. Open early | **9** | 6.56 unique archetypes per pack in picks 1-5, far above >= 3 target. |
| 8. Signal reading | **4** | Signal reader converges fastest but produces 3-36% archetype frequency skew. |
