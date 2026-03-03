# Resonance Draft System V10: Final Synthesis Report

## 1. Unified Comparison Table

All results: Graduated Realistic fitness, committed player strategy, 1000 drafts
x 30 picks. V9 Hybrid B baseline included for reference.

| Algorithm | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | Pass |
|-----------|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:---:|:---:|:----:|
| **V9 Hybrid B** | PASS | PASS | **2.70** | PASS | 9.6 | 86% | PASS | PASS | 1.08 | 3.8 | **3.25** | **8/10** |
| D1 Open Table | 4.50 | 1.01 | 0.60 | 3.40 | 26.7 | 48% | 2.4% | PASS | 0.60 | 5.3 | 0.00 | 5/11 |
| Hybrid X (D1+D3) | 3.87 | 1.87 | 0.84 | 3.16 | 6.0 | 51% | 17% | PASS | 0.92 | 12.15 | 0.69 | 7/11 |
| D4 Escalating | 3.83 | 1.55 | 0.20 | 3.04 | 28.1 | 22% | 23% | PASS | 0.42 | 22.43 | 0.07 | 5/11 |
| Hybrid Y (D1+D4) | 3.11 | 0.86 | 0.48 | 3.52 | 14.3 | 36% | 5.5% | PASS | 0.68 | 18.50 | 0.19 | 4/10 |
| D2 Sentinel | 2.90 | 0.58 | 0.39 | 3.13 | 29.1 | 31% | 3.3% | PASS | 0.43 | 5.16 | 0.00 | 4/11 |
| D3 Competitive | 3.04 | 0.60 | 0.51 | 3.49 | 24.9 | 43% | 7.1% | PASS | 0.65 | 14.8 | 0.48 | 4/10 |

**All six V10 algorithms fail M3, M10, and M11.** The best M3 is 0.84 (Hybrid
X), which is 69% below V9's 2.70. The best M11 is 0.69 (Hybrid X), which is 79%
below V9's 3.25. These are not marginal failures -- they are order-of-magnitude
shortfalls. No parameter tuning can bridge gaps of this size.

---

## 2. The Key Question: Does the AI Drafter Framing Produce Equivalent Metrics?

**No.** The AI drafter paradigm, as implemented across six distinct algorithms
spanning the design space (static to reactive, 5-7 AIs, flat to escalating pick
rates, with and without saturation and market culling), cannot produce metrics
comparable to V9's abstract contraction. The gap is not tunable -- it is
structural.

### Three Root Causes

**Root cause 1: Physical pool depletion.** All V10 algorithms physically remove
cards from the pool. Flat per-round removal (20-35 cards/round) exhausts the
360-card pool by pick 12-15. V9's percentage-based contraction naturally
decelerates -- it removes 43 cards at pick 1 but only 14 at pick 20. This
self-regulating behavior sustains the draft for 30 rounds. V10's flat removal
does not decelerate. D1's pool was exhausted by round 12. D4's Phase 4 (the
design's signature feature) never executed because the pool was already empty.

**Root cause 2: S/A preferential depletion.** AI drafters take the BEST cards
for their archetype first, because high pair-affinity correlates with high S/A
fitness. This depletes S/A density in the remaining pool. V9's contraction
removes LOW-relevance cards from the player's perspective, enriching S/A
density. These mechanisms work in opposite directions. V9 makes the pool better
for the player with each contraction step. V10's AI drafters make the pool worse
with each pick -- the highest-quality cards leave first, leaving C/F remnants.

**Root cause 3: Level 0 targeting dilution.** Even with percentage-based
contraction, Level 0 AIs can only concentrate the pool toward open archetypes
generally (the player's archetype is 1 of 3 in a 5-AI model). V9's contraction
targets the player's SPECIFIC archetype, concentrating the pool 5-7x toward
their committed archetype by late draft. V10's best result (open-lane M3 = 0.78
in D3) reflects the natural concentration achievable when 7 AIs each deplete
their own lanes: the player's archetype ratio improves from 11% to 18% -- a 1.7x
concentration, far below V9's 5-7x.

### The Structural Impossibility

V9's contraction is virtual: it biases sampling without physically removing
cards. The full 360-card pool remains available for weighted pack construction.
Cards with low relevance are down-weighted but not deleted. This preserves the
pool for the entire 30-pick draft while concentrating pack quality.

V10's AI drafters physically remove cards. This creates an irreconcilable
tension: removing enough cards to create lane signals (20+ per round) exhausts
the pool before pick 15. Removing fewer cards (5-10 per round) preserves the
pool but creates no meaningful concentration. There is no removal rate that
simultaneously sustains a 30-pick draft and achieves V9-level concentration
through physical depletion.

---

## 3. The Reactivity Question

**Level 0 (static) is unanimously confirmed as optimal for the AI drafter
narrative.** All six design agents converged on this conclusion. D3 -- assigned
to defend Level 3 lane avoidance -- explicitly rejected its own mandate and
pivoted to a Level 0 saturation model.

D2's Sentinel Draft tested Level 1 pool-pressure reactivity against a Level 0
control. The result: **zero measurable difference** across all metrics. The
experiment was conclusive but for the wrong reason -- the pool was exhausted
before Phase 2 could activate meaningfully. Reactivity cannot be evaluated when
physical depletion kills the draft at pick 12.

The research findings are clear:
- Level 0 (static) produces authentic signal reading because AI behavior is
  independent of the player
- Levels 3-4 (lane avoidant, fully dynamic) corrupt signal reading, violate the
  transparency principle, and transform skill into automatic rewards
- Level 1 (delayed reaction) is theoretically defensible but produced no
  measurable benefit in simulation

However, Level 0 alone cannot achieve M3 >= 2.0 because it cannot target the
player's specific archetype. The concentration it provides (1.7x) falls far
short of V9's targeted concentration (5-7x). Adding reactivity would help
concentration but at the cost of the narrative integrity that is V10's raison
d'etre. This is the fundamental dilemma V10 cannot resolve within its own
paradigm.

---

## 4. The Structural Finding: Physical vs Virtual Contraction

V10's most important contribution is proving that AI drafters and V9 contraction
are fundamentally different mechanisms, not alternative framings of the same
math.

| Dimension | V9 Contraction | V10 AI Drafters |
|-----------|---------------|-----------------|
| Pool modification | Virtual (bias sampling) | Physical (remove cards) |
| Direction | Removes LOW-relevance cards | Removes HIGH-relevance cards |
| Effect on S/A density | Enriches (pool gets better) | Depletes (pool gets worse) |
| Targeting | Player-specific archetype | General open archetypes |
| Concentration achievable | 5-7x by late draft | 1.7x by late draft |
| Self-regulation | Yes (% of shrinking pool) | No (flat removal) |
| Draft sustainability | 30 picks reliably | 12-15 picks before exhaustion |

The critic review identified this midway through V10: "Supplemental culling is
V9 contraction in disguise." Every V10 algorithm that attempted competitive
metrics included market culling or supplemental removal -- which is V9's
contraction re-labeled. The AI drafter layer added narrative justification and
lane signals, but the mathematical engine underneath was still pool contraction.
When the AI picks were isolated from the culling, they produced M3 values of
0.2-0.8 -- inadequate by any standard.

---

## 5. Per-Archetype Convergence: Least-Bad Algorithms

The two least-bad algorithms are Hybrid X (best M3, 7/11 pass) and D3
Competitive Pressure (most human-like AI behavior).

### Hybrid X Per-Archetype

| Archetype | M3 | M11 | Open Lane M3 |
|-----------|:--:|:---:|:------------:|
| Warriors | 0.88 | -- | ~1.10 |
| Sacrifice | 0.90 | -- | ~1.15 |
| Self-Discard | 0.86 | -- | ~1.08 |
| Self-Mill | 0.85 | -- | ~1.05 |
| Flash | 0.80 | -- | ~0.98 |
| Blink | 0.79 | -- | ~0.96 |
| Storm | 0.73 | -- | ~0.90 |
| Ramp | 0.88 | -- | ~1.10 |

Spread: 0.17. Tide-primary archetypes (Warriors/Sacrifice at 50% sibling rate)
slightly outperform Ember-primary (Storm at 30%). Even open-lane values (~1.0
M3) are half the V9 target. The archetype gradient is correct -- higher sibling
rates produce higher M3 -- but the absolute level is catastrophically low.

### D3 Per-Archetype

| Archetype | M3 | M11 |
|-----------|:--:|:---:|
| Warriors | 0.57 | 0.56 |
| Sacrifice | 0.56 | 0.50 |
| Self-Discard | 0.52 | 0.49 |
| Self-Mill | 0.51 | 0.47 |
| Flash | 0.46 | 0.41 |
| Blink | 0.46 | 0.41 |
| Storm | 0.46 | 0.40 |
| Ramp | 0.46 | 0.43 |

Spread: 0.118. Remarkably uniform failure. The saturation mechanic produces the
best per-archetype equity among V10 algorithms, but all values are 75% below
target.

---

## 6. V10 vs V9: What We Gained and What We Cannot Replicate

### What V10 Gained (Genuine Contributions)

**1. A player-facing narrative.** "You are drafting against AI opponents" is
immediately intuitive. V9's "the game removes cards that don't match your style"
is vague and potentially feels like hidden manipulation. The AI drafter framing
provides a universally understood justification for pool concentration.

**2. Lane signals as a skill axis.** V10 proved that Level 0 static AIs create
readable, honest signals. The open-lane advantage (3x M3 in D3, 1.5x in Hybrid
Y) confirms that signal reading rewards correct identification of which
archetypes are uncontested. This is a genuine skill axis that V9 cannot provide
-- V9's contraction is invisible.

**3. Signal reading is compatible with fairness.** The research phase
established that Level 0 AIs with per-game variety pass the transparency test.
Players accept that "other drafters at the table have their own strategies" as a
fair competitive dynamic. This framing is robust against discovery -- unlike
V9's hidden contraction, which could feel like manipulation if players
understood the mechanism.

**4. The 5-AI / 3-open-lane structure.** This produces the best signal reading
quality, player choice breadth, and game-to-game variety. C(8,5) = 56 possible
AI compositions create natural replayability. This structural finding carries
forward regardless of the contraction engine.

**5. Deckbuilding-aware saturation.** D3's saturation mechanic (AIs ease off
after 12 archetype cards) produces the most human-like AI behavior in the field.
This is a pick-logic refinement worth preserving in any design that includes AI
drafters.

### What V10 Cannot Replicate

**1. Targeted pool concentration.** V9 concentrates the pool toward the player's
SPECIFIC archetype. V10's Level 0 AIs can only create general open-archetype
enrichment. This 3x targeting dilution is the primary cause of M3 failure.

**2. S/A density enrichment.** V9 removes low-quality cards, making the pool
better. V10's AI drafters remove high-quality cards, making the pool worse. This
directional mismatch cannot be resolved without fundamentally changing what AI
drafters do.

**3. Virtual contraction sustainability.** V9 never removes cards -- it biases
sampling. The full pool sustains 30 picks. V10's physical removal exhausts the
pool by pick 12-15.

---

## 7. Recommendation Tiers

### Simple: Best Pure AI Drafter Design

**Algorithm:** Hybrid X (D1 Open Table + D3 Saturation). 5 random AIs, 3 open
lanes, saturation at 16 archetype cards, 12% market culling.

**Metrics:** M3 = 0.84, M11 = 0.69. Passes 7/11 metrics. Fails all core
concentration metrics.

**Honest assessment:** This is the best pure AI drafter design in the field, but
it fails the primary metric targets by wide margins. It should not be
implemented as a standalone system. Its value is as a design reference for the
narrative layer in the recommended hybrid.

### Standard (Recommended): V9 Hybrid B Contraction + AI Drafter Narrative Layer

**The core insight:** V10's contribution is not replacing V9's math -- it is
providing a player-facing narrative that makes V9's invisible contraction feel
justified and skill-expressive. The recommended design combines V9's proven
contraction engine with V10's AI drafter presentation layer.

**How it works:**

1. **Under the hood:** V9 Hybrid B runs exactly as specified. Pool contraction
   at 12% per pick using blended relevance (40% visible dot-product + 60%
   pair-affinity). Floor slot from pick 3. Archetype inference from pick 5. The
   contraction engine is the mathematical foundation.

2. **Player-facing layer:** 5 AI drafters are presented as opponents at the
   table. Each AI is assigned one of 8 archetypes (3 archetypes uncontested per
   game). The AIs have no mechanical effect on pack construction -- V9's
   contraction engine handles all pool concentration. Instead, the AIs provide:
   - **Lane signals:** The system communicates which archetypes are "contested"
     by AIs. Cards from contested archetypes are described as "taken by other
     drafters." Cards from open archetypes appear more frequently in packs --
     because V9's contraction naturally concentrates toward the player's
     archetype, which correlates with open archetypes.
   - **Narrative justification:** When V9's contraction removes low-relevance
     cards, the player sees "other drafters took those cards" rather than "the
     game removed cards." The explanation is cosmetic but psychologically
     powerful.
   - **Signal reading skill:** The player observes which archetypes have
     abundant high-quality cards (the open lanes) and which are thin (the
     contested lanes). Committing to an open lane aligns the player's archetype
     with V9's contraction direction, producing the best results. This is a
     genuine skill axis.

3. **AI archetype assignment:** Each game, 5 of 8 archetypes are randomly
   assigned to AI drafters. The 3 uncontested archetypes are the lanes where
   V9's contraction will be most effective (because the player's archetype cards
   are not "taken" by any AI). This creates the illusion that AI competition
   drives scarcity, when V9's contraction actually drives concentration.

4. **Saturation as flavor:** AIs can display "saturation" behavior (slowing
   their apparent drafting after accumulating cards) as a narrative signal. This
   has no mechanical effect but provides the player with a readable mid-draft
   signal that an AI's lane may be easing up.

**Metrics:** Identical to V9 Hybrid B (M3 = 2.70, M11 = 3.25, M10 = 3.8) plus
the narrative and signal-reading benefits of the AI drafter framing.

**Hidden metadata:** 8 bits per card (same as V9 Hybrid B). The AI drafter
presentation requires no additional hidden data.

### Advanced: Future Exploration

Two directions are worth exploring in V11:

**A. Weighted virtual AI drafting.** Instead of physical card removal, AIs
"claim" cards virtually. Claimed cards are down-weighted in pack construction
(like V9's contraction) rather than removed. This preserves the AI narrative
while using virtual contraction. The challenge: defining what "an AI claimed
this card" means mechanically, and ensuring the weighting produces V9-equivalent
concentration.

**B. Hybrid physical/virtual.** AIs physically remove a small number of cards
per round (3-5 total, not per AI) to create visible lane signals, while V9's
virtual contraction handles the bulk of pool concentration. The physical removal
creates the narrative ("the AI took that card") while the virtual contraction
does the mathematical work.

---

## 8. Complete Specification: Recommended Hybrid (V9 Engine + AI Narrative)

### Contraction Engine (V9 Hybrid B, unchanged)

| Parameter | Value |
|-----------|-------|
| Contraction start | Pick 4 |
| Contraction rate | 12% per pick |
| Relevance blend | 40% visible dot-product + 60% pair-affinity |
| Floor slot | 1 top-quartile from pick 3 |
| Generic protection | 0.5 baseline relevance |
| Signature weights | +2 primary, +1 secondary |
| Pool minimum | 17 cards |
| Archetype inference | Mode of inferred archetype from pick 5 |
| Hidden metadata | 8 bits/card (two 4-bit pair-affinity floats) |

### AI Narrative Layer (new)

| Parameter | Value |
|-----------|-------|
| Number of AIs | 5 per game |
| Archetype assignment | Random 5 of 8, no duplicates |
| Open archetypes | 3 per game |
| Mechanical effect | None -- AIs are presentation only |
| Signal display | Show which archetypes are "contested" via UI |
| Narrative framing | Cards removed by contraction are attributed to AI picks |
| Saturation display | AIs visually slow down after ~12 apparent picks |
| Per-game variety | C(8,5) = 56 possible compositions |
| AI personality | Optional: aggression/focus labels for flavor |

### Signal Reading Mechanic

The player observes pack composition and infers which archetypes are open.
Mechanically, this works because:

1. V9's contraction concentrates toward the player's committed archetype
2. If the player commits to an archetype that no AI is "assigned" to, the
   contraction aligns with the narrative (those cards are not being "taken")
3. If the player commits to a contested archetype, V9's contraction still works
   but the narrative signals suggest competition, priming the player to expect
   slightly lower quality

The signal reading is cosmetically driven but produces real skill expression:
players who identify open lanes and commit early get the full benefit of V9's
contraction from pick 5-6. Players who commit late or to contested archetypes
get weaker results -- exactly as V9 already produces, but with a comprehensible
explanation.

---

## 9. Implementation Guide

### Phase 1: V9 Contraction Engine

Implement V9 Hybrid B exactly as specified in the V9 algorithm overview. This is
the mathematical foundation. Test against V9's metrics (M3 = 2.70, M11 = 3.25)
before proceeding.

### Phase 2: AI Presentation Layer

1. At draft start, randomly select 5 of 8 archetypes for AI assignment
2. Create UI elements showing 5 AI "opponents" with archetype indicators
3. As V9's contraction removes cards each round, attribute removals to the AI
   whose archetype is closest to the removed card's pair-affinity
4. Display AI "draft picks" as a log or visual indicator
5. Optionally show AI "saturation" (slowing apparent picks) after ~12 attributed
   picks per AI

### Phase 3: Signal Reading UI

1. Display pack composition statistics (how many cards of each resonance are
   appearing)
2. Optionally show "cards remaining by archetype" as an advanced stat
3. Highlight when an archetype appears unusually frequently (open lane signal)
   or rarely (contested lane signal)

### Phase 4: Playtesting

1. Test whether the AI narrative improves player comprehension of the draft
   system
2. Test whether signal reading feels like a genuine skill or an obligation
3. Compare player satisfaction with V9-only (abstract contraction) vs V9+AI
   narrative
4. Test whether attribution of V9 removals to AI picks feels natural or forced

---

## 10. Open Questions for V11

1. **Can virtual AI drafting achieve V9-equivalent concentration?** Instead of
   physical removal, AIs "claim" cards with weighted probability, biasing pack
   construction away from claimed cards. This would unify the AI narrative with
   the contraction engine rather than layering them.

2. **Does the recommended hybrid's attribution feel natural?** V9's contraction
   removes cards based on relevance to the player's archetype. Attributing these
   removals to AI drafters whose archetypes differ from the player's is
   narratively consistent but requires careful UI design to avoid "the Warriors
   AI took this Sacrifice card" contradictions.

3. **Should AIs have any mechanical effect at all?** The recommended hybrid
   treats AIs as pure presentation. A small mechanical contribution (AIs
   physically remove 2-3 cards per round from their archetype, with V9 handling
   the rest) could strengthen the narrative without significantly impacting V9's
   math.

4. **Is the 5-AI / 3-open-lane structure optimal for signal reading?** V10
   tested 5-7 AI counts but never with V9's contraction engine underneath. The
   optimal AI count for signal reading may differ when V9 handles concentration.

5. **Can the M3-M10-M6 triangle be broken?** V9 fails M5 (9.6) and M10 (3.8).
   The AI drafter narrative provides earlier commitment signals (Hybrid X
   achieved M5 = 6.0). Can this earlier commitment, combined with V9's
   contraction, improve M5 and M10 without degrading other metrics?

6. **What level of AI visibility is optimal?** Should players see AI pick logs?
   AI archetype labels? Only inferred signals from pack composition? The answer
   affects implementation complexity and the signal reading skill axis.

7. **Does the saturation narrative add value?** D3's saturation mechanic
   produced the most human-like AI behavior but negligible metric improvement.
   As a narrative element (AIs visually slowing down), it may still improve
   player experience without needing mechanical effect.
