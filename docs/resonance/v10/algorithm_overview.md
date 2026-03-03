# V10 Algorithm Overview: Complete Catalog

V10's defining question was whether AI drafters -- concrete opponents who physically remove cards from a shared pool -- could replace V9's abstract pool contraction while adding a narrative skill axis (lane signal reading). The answer is no: physical card removal is structurally incompatible with the concentration levels V9 achieves through virtual contraction. V10's contribution is the AI drafter narrative layer, which provides player-facing justification and signal reading on top of V9's proven engine.

---

## Recommended: V9 Hybrid B + AI Drafter Narrative Layer

**One sentence (player-facing):** "You are drafting against AI opponents who compete for cards from the same pool -- find the open lane."

**One sentence (technical):** V9 Hybrid B contraction engine (12% per pick, 40/60 visible/affinity blend) provides all pool concentration; 5 AI drafters provide narrative framing, lane signals, and signal reading skill axis with no mechanical effect on pack construction.

### How It Works

V9's contraction engine runs unchanged: blended relevance scoring, 12% contraction per pick from pick 4, floor slot from pick 3, archetype inference from pick 5. Pack construction draws from the surviving weighted pool exactly as V9 specifies.

The AI narrative layer is presentation-only. Five AI drafters are randomly assigned archetypes each game (3 archetypes uncontested). When V9's contraction removes cards, the removals are attributed to AI drafters as "picks." The player observes which archetypes are abundant (open lanes) and which are thin (contested lanes) -- signals that emerge naturally from V9's contraction but are now explained as "other drafters took those cards."

### Metrics

Identical to V9 Hybrid B:

| Metric | Value | Target | Status |
|--------|------:|--------|:------:|
| M3 | 2.70 | >= 2.0 | PASS |
| M11 | 3.25 | >= 3.0 | PASS |
| M10 | 3.8 | <= 2 | FAIL |
| M6 | 86% | 60-90% | PASS |
| M5 | 9.6 | 5-8 | FAIL |
| V1 | 84.8% | -- | Visible symbols primary |
| V3 | 9/10 | -- | Honest metadata |

Plus narrative benefits: lane signals, signal reading skill, fairness framing.

### Key Parameters

- Contraction: 12% per pick from pick 4. Blend: 40% visible + 60% pair-affinity.
- Hidden metadata: 8 bits/card (two 4-bit pair-affinity floats).
- AI count: 5 per game. Open archetypes: 3. Compositions: C(8,5) = 56.
- Mechanical effect of AIs: None. Presentation layer only.
- Signal reading: Player infers open lanes from pack composition.

---

## Viable Alternative: V9 Hybrid B + Lightweight Physical AI Removal

A variant where AIs have a small mechanical effect: each AI physically removes 1 card per round from its archetype (5 cards total per round) before V9's contraction processes the remaining pool. This strengthens the narrative ("the AI literally took that card") at the cost of slightly faster pool depletion.

### Tradeoffs

- **Pro:** Stronger narrative -- AI picks are real, not attributed.
- **Pro:** Creates visible card scarcity in AI lanes without relying on V9 contraction alone.
- **Con:** 5 physical removals per round accelerate pool depletion. Pool reaches minimum floor ~5 picks earlier than V9 alone.
- **Con:** Physical removal depletes S/A cards (AIs take the best), partially counteracting V9's S/A enrichment.

### Estimated Metrics

M3: ~2.55-2.65 (slight degradation from S/A depletion). M11: ~3.0-3.15 (earlier pool exhaustion reduces late-draft quality). M10: ~3.5-4.0 (comparable to V9). The degradation is modest because 5 cards/round is small relative to V9's virtual contraction volume.

This variant is worth playtesting if the pure-narrative approach (recommended) feels too disconnected from the AI framing. The 5 physical removals per round are a small enough perturbation that V9's engine remains dominant.

---

## Eliminated: All Six V10 Pure Designs

### Failure Mode 1: Pool Exhaustion (D1, D2, D4)

**D1 Open Table.** 5 AIs x 4 cards + 10 market cull = 31 cards/round. Pool exhausted at round 12. M3 = 0.60, M11 = 0.00. The design's predicted 30-round draft was physically impossible at the specified removal rate.

**D2 Sentinel Draft.** 6 AIs x 4 cards + 5% cull = ~30 cards/round. Pool exhausted at pick 12. M3 = 0.39, M11 = 0.00. The Level 1 reactivity experiment was moot -- the pool died before Phase 2 activated. Sentinel vs Level-0 control: zero measurable difference across all metrics.

**D4 Escalating Aggression.** 7 AIs escalating from 2 to 5 cards/round + 8 cull. Pool exhausted at pick 12. Phase 4 (the design's signature feature -- 5 cards/AI/round at 95% focus) never executed. M3 = 0.20, M11 = 0.07. The worst-performing algorithm in the field.

**Common cause:** Flat per-round card removal against a fixed 360-card pool. V9's percentage-based contraction naturally decelerates as the pool shrinks. Physical removal at 20-35 cards/round does not decelerate -- it exhausts the pool in 10-13 rounds regardless of phase structure.

### Failure Mode 2: S/A Preferential Depletion (All Six)

AI drafters select cards by pair-affinity score. High pair-affinity correlates with high S/A fitness (by design -- the affinity scores encode archetype fit). AIs therefore take S/A cards first, depleting the pool of exactly the cards the player needs. The remaining pool enriches in C/F cards.

V9's contraction works in the opposite direction: it removes cards with LOW relevance to the player, enriching the pool in S/A cards. This directional mismatch is the deepest structural reason V10 cannot match V9. It is not a parameter issue -- it is inherent to the AI drafter paradigm where AIs take good cards and leave bad ones.

D4's simulation made this explicit: "AIs' 15% power picks and market culling erode open-lane card pools alongside contested ones. The player's archetype concentration DECREASES from 12% to near 0% over the draft -- the opposite of V9's behavior."

### Failure Mode 3: Targeting Dilution (All Six)

V9's contraction targets the player's SPECIFIC archetype. V10's Level 0 AIs target their OWN archetypes. For a player in an open lane, the net effect is that other archetypes are depleted while theirs is preserved. But preserved is not concentrated. The player's archetype ratio improves from ~11% (40/360) to ~18% (40/216) in the best case -- a 1.7x improvement. V9 achieves 5-7x concentration by actively removing non-archetype cards.

The 3x dilution (player's archetype is 1 of 3 open lanes) means even perfect AI depletion of contested lanes cannot match V9's targeted enrichment.

---

## Eliminated Algorithms: Individual Assessments

### Hybrid X (D1 + D3 Saturation) -- Least Bad

M3 = 0.84, M11 = 0.69, 7/11 pass. The saturation mechanic added realism (AIs easing off after 16 archetype cards) but triggered too early (pick 5.6 average) to create a readable mid-draft signal. Market culling at 12% was V9 contraction by another name. The best V10 result but still 69% below V9 on M3.

### D3 Competitive Pressure -- Most Human-Like

M3 = 0.51, M11 = 0.48, 4/10 pass. The saturation mechanic produced the most realistic AI behavior (critic score 9/10). Open-lane M3 = 0.78 was the highest per-archetype result across V10. But the 7-AI / 1-open-lane structure reduced signal reading to "find the gap," and the saturation mechanic's contribution to concentration was negligible (+40% late-draft improvement from a very low base).

### Hybrid Y (D1 + D4 Escalation) -- Best Signal Incentive

M3 = 0.48, M11 = 0.19, 4/10 pass. Open-lane M3 (0.71) was 3x contested-lane M3 (0.23), confirming the signal-reading incentive exists. But both outcomes are catastrophically below target. The escalation mechanism could not fire because the pool was exhausted before Phase 4.

### D1 Open Table -- Baseline

M3 = 0.60, M11 = 0.00, 5/11 pass. The simplest design, and the one that most clearly revealed the pool exhaustion problem. AIs distributed picks uniformly across archetypes (11.3-11.5% each) rather than concentrating on their home archetype, undermining the lane-signal premise.

### D2 Sentinel Draft -- Reactivity Test

M3 = 0.39, M11 = 0.00, 4/11 pass. Conclusively proved Level 1 reactivity adds nothing over Level 0 (delta = zero on all metrics). The reactivity finding is valid but the reason is wrong: the pool died before reactivity could matter.

### D4 Escalating Aggression -- Worst Performer

M3 = 0.20, M11 = 0.07, 5/11 pass. The design's thesis ("early openness, late concentration") was contradicted by its own math. Phase 4's intended concentration arrived after the pool was empty. 55% of drafts hit 25 consecutive bad packs (the maximum).

---

## Structural Findings

### 1. Physical card removal and virtual pool contraction are not interchangeable

This is V10's most important finding. Every design agent assumed that "N AIs taking cards from the pool" would produce mathematically equivalent results to "the game removes cards from the pool." The simulation proved this assumption false. Physical removal depletes S/A cards (AIs take the best), exhausts the pool (flat removal vs percentage-based), and dilutes targeting (general archetype depletion vs player-specific concentration). Virtual contraction enriches S/A cards, sustains the pool, and targets precisely.

### 2. Level 0 reactivity is optimal for the AI drafter narrative

The universal convergence of all six design agents toward Level 0 -- including D3's explicit rejection of its own Level 3 mandate -- is the strongest possible endorsement. Level 0 preserves signal reading integrity, passes the transparency test, and produces authentic player agency. Levels 3-4 corrupt signal reading and violate the fairness narrative.

### 3. The 5-AI / 3-open-lane structure is optimal for signal reading

5 AIs with 3 open lanes produces the best balance of signal density, player choice breadth, and game-to-game variety (56 compositions). The 7-AI / 1-open-lane structure (D3, D4, D5) reduces signal reading to "find the gap" and makes the open lane strictly dominant.

### 4. Supplemental culling is V9 contraction re-labeled

Every V10 algorithm that attempted V9-competitive metrics included "market culling" or "supplemental removal" -- which is V9's pool contraction under a different name. The AI drafter layer adds narrative value and lane signals but cannot independently achieve the concentration V9's contraction provides. The honest framing: V10's AI drafters are a narrative and skill layer, not a contraction mechanism.

### 5. Deckbuilding-aware saturation is the best AI pick logic

D3's saturation mechanic (AIs ease off after 12 archetype cards) and D5's role quotas both produce more human-like AI behavior than stateless preference functions. D3's single threshold is simpler and produces equivalent results to D5's 32-parameter role quotas. The saturation mechanic should be included in any design that presents AI drafters to the player, as a narrative refinement.

### 6. The AI drafter paradigm's genuine contribution is narrative, not mathematical

V10 did not find a better contraction algorithm. It found a better explanation for contraction. "Other players took those cards" is more intuitive, more fair-feeling, and more skill-expressive than "the game removed cards that don't match your style." This narrative contribution is real and valuable -- it justifies V9's hidden contraction to the player and adds a signal reading skill axis that V9 lacked. The recommended design leverages this contribution by layering the AI narrative on top of V9's proven engine.
