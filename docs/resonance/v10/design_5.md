# Design 5: Deckbuilding-Aware AI Drafters

## Key Findings

- **Deckbuilding awareness is the single most-cited gap in draft bot realism.** Research Agent A found that the top player complaint about Arena bots is that they do not track what they have already drafted. Ryan Saxe's ML bot model showed that tracking draft state raised archetype-appropriate picks from ~80% to 91% by Pack 3, producing emergent "I have enough removal" behavior without explicit rules.

- **Diminishing returns set in quickly.** The jump from no-state (pure preference function) to lightweight state (count cards by role/type, track archetype density) is large. The jump from lightweight state to full synergy modeling (pairwise card interactions, curve analysis, mana cost distribution) is negligible for the metrics that matter. The player cannot distinguish "the AI stopped taking creatures because it has 12" from "the AI evaluated the marginal synergy of creature #13 against spell #7."

- **State-tracking AIs naturally solve the saturation artifact.** Stateless AIs take their archetype's best card every pick, depleting the top of the archetype early and leaving only weak cards late. State-tracking AIs that recognize "I have enough of this role" shift to secondary needs mid-draft, leaving strong primary-role cards in the pool longer. This produces smoother pack quality curves and directly helps M10.

- **Deckbuilding logic creates natural aggression curves.** A state-tracking AI is aggressive early (needs everything) and becomes selective late (has enough creatures, now wants tricks). This produces a per-AI aggression curve that declines over the draft without explicit tuning -- and mirrors human drafting behavior.

- **The ~40% human-pick prediction ceiling warns against over-sophistication.** Ward 2020 found even the best ML bot predicts only ~40% of human picks. AIs that are too optimal feel mechanical. State-tracking should make AIs feel more human, not more optimal -- the goal is believable imperfection, not perfect play.

- **Pair-affinity discrimination remains mandatory.** V9 proved that M11 >= 3.0 requires 8-bit pair-affinity data. Deckbuilding awareness cannot substitute for this. The AI's preference function still needs pair-affinity scores to distinguish Warriors from Sacrifice within Tide cards. Deckbuilding logic sits on top of that preference, modulating pick urgency rather than replacing card evaluation.

- **The sweet spot is "role-aware" state tracking.** Each AI tracks counts of cards in 3-4 broad roles (e.g., creatures, spells, tricks, high-cost). When a role is saturated, the AI deprioritizes additional cards of that role. This is simple enough to be deterministic and seed-reproducible, complex enough to produce realistic draft curves.

---

## Three Algorithm Proposals

### Proposal A: Stateless Baseline (Control)

**Player description:** "Seven AI opponents draft alongside you, each preferring a different archetype."

**Technical description:** 7 AIs, each assigned an archetype. Each AI picks the highest pair-affinity card for its archetype from the available pool each round, taking 4-5 cards per round total (distributed across AIs). No state tracking. Level 0 reactivity. Supplemental culling removes 6% of remaining pool per round (lowest-relevance cards relative to the most-contested archetype) to reach V9-equivalent contraction.

**AI drafter behavior:** Greedy top-card selection every pick. AIs strip the best cards from their archetype early, leaving weaker cards late. No saturation, no role balancing.

**Predicted metrics:** M3 ~ 2.50, M10 ~ 4.5, M11 ~ 3.05, M6 ~ 82%. Weak M10 because stateless AIs create uneven depletion (all top cards taken early, producing quality cliffs mid-draft).

---

### Proposal B: Role-Saturating AIs

**Player description:** "Seven AI opponents are building decks alongside you -- they draft what they need, not just the best card available."

**Technical description:** 7 AIs, each assigned an archetype and initialized with role quotas (e.g., Warriors AI wants ~10 creatures, ~5 combat tricks, ~4 removal, ~3 utility). Each AI evaluates cards using pair-affinity score multiplied by a role-need multiplier: need_multiplier = max(0.15, 1.0 - (current_count / quota)) for the card's role. AIs take 4-5 cards per round, picking in archetype order (randomized per game). Supplemental culling at 5% per round. Level 0 reactivity -- all AI picks are seed-determined before the draft begins.

**AI drafter behavior:** Early picks favor the AI's strongest archetype cards (high affinity, all roles needed). Mid-draft, the AI becomes selective: if it already has 8 creatures, creature affinity is multiplied by 0.15, so it shifts to tricks or removal. Late-draft, the AI takes whatever fills remaining gaps or passes to generic/off-archetype power cards. This creates a natural "the AI stopped hoarding creatures" signal around picks 10-14.

**Predicted metrics:** M3 ~ 2.65, M10 ~ 2.8, M11 ~ 3.20, M6 ~ 84%. Better M10 than Proposal A because role saturation prevents AIs from stripping all top cards early -- some strong archetype cards survive into mid-draft as AIs shift to secondary roles.

---

### Proposal C: Synergy-Chasing AIs

**Player description:** "Seven AI opponents draft synergistic decks -- they value cards that work with what they have already picked."

**Technical description:** Same 7-AI structure as Proposal B, but instead of role quotas, AIs track a synergy graph. Each card in the AI's drafted pool defines synergy bonuses for future cards (e.g., having drafted a "sacrifice trigger" card increases the value of future "sacrifice payoff" cards by +0.3). The evaluation function is: pair_affinity * (1.0 + synergy_bonus) * diminishing_count_factor. Synergy data is encoded as card-level tags (2-3 tags per card: "sacrifice-trigger", "creature", "combat-trick", etc.). Level 0 reactivity.

**AI drafter behavior:** Early picks are driven by pair-affinity (no synergy yet). From pick 3-4, the AI begins chasing synergies: a Warriors AI that drafted two "when attacks" creatures starts valuing "combat boost" cards more highly. This produces emergent archetype-specific behavior -- the Warriors AI naturally builds a coherent deck rather than just taking the top Warriors card. However, synergy-chasing makes AI behavior harder to predict and creates more variance in which cards survive.

**Predicted metrics:** M3 ~ 2.60, M10 ~ 3.2, M11 ~ 3.15, M6 ~ 85%. Slightly worse M10 than Proposal B because synergy-chasing can produce erratic depletion patterns (the AI suddenly wants a specific card type, creating micro-shortages). The synergy model adds complexity for marginal metric improvement over role saturation.

---

## Champion Selection: Proposal B (Role-Saturating AIs)

**Justification:** Proposal B hits the sweet spot of AI sophistication. It is complex enough to produce realistic draft patterns (AIs that stop taking what they have enough of, natural aggression curves, smoother pool depletion) but simple enough to be fully deterministic and seed-reproducible. Role quotas are explainable ("the AI already has enough creatures") while synergy graphs (Proposal C) are opaque and add implementation complexity for negligible metric gain. Proposal A (stateless) fails M10 because greedy depletion creates quality cliffs that role saturation avoids.

The role-saturation model also maps cleanly to V9's proven foundations: pair-affinity scores handle card evaluation, role quotas handle pick modulation, and the combination produces V9-equivalent M3/M11 with better M10.

---

## Champion Deep-Dive: Role-Saturating AIs

### How It Works

The draft table has 7 AI drafters and 1 human player. Each game, 7 of the 8 archetypes are randomly assigned to AIs (the 8th is left uncontested -- the "open lane"). Each AI is initialized with:

- **Archetype preference:** Determines which pair-affinity column the AI reads.
- **Role quotas:** 4 broad roles derived from the archetype's mechanical identity. Warriors AI: {creatures: 12, combat_tricks: 6, removal: 4, utility: 3}. Storm AI: {spells: 10, creatures: 6, card_draw: 5, removal: 4}. Quotas sum to ~25 (a reasonable AI "deck size" over 30 picks).
- **Pick evaluation:** For each available card, score = pair_affinity_for_my_archetype * role_need_multiplier. The role_need_multiplier = max(0.15, 1.0 - (cards_in_role / quota)). The AI picks the highest-scoring card.

Each round, before the player sees their pack, all 7 AIs make picks from the shared pool. Each AI takes 1 card per round (7 cards total removed). Supplemental culling removes an additional 5% of the pool (lowest average affinity across all active AI archetypes), bringing total per-round removal to roughly 7 + 5% of pool. Over 30 picks, this produces V9-equivalent contraction.

The player's 4-card pack is drawn from the remaining pool after AI picks and culling. One floor slot guarantees a top-quartile card for the player's inferred archetype from pick 3.

### What the Player Sees vs. What the AIs Do

**The player sees:** 4-card packs that gradually concentrate toward their chosen archetype. Early packs (picks 1-5) show broad variety across archetypes. Mid-draft packs (picks 6-14) show increasing density of the player's archetype. Late packs (picks 15+) are heavily concentrated.

**What the player does not see:** The 7 AIs making picks, role saturation logic, supplemental culling. The player experiences only the resulting pool composition.

**The signal:** The "open lane" (the archetype with no AI) has noticeably more high-quality cards surviving into mid and late draft. A signal-reading player notices: "I keep seeing strong Blink cards late -- maybe Blink is open this game." This signal is consistent because the absent AI never takes Blink cards.

**The saturation effect on what the player sees:** Because AIs saturate on roles, they leave some strong archetype cards in the pool even for contested archetypes. A Warriors player competing against the Warriors AI still sees some good Warriors creatures after pick 10 -- because the Warriors AI shifted to tricks and removal. This produces the "fighting for a lane is harder but not impossible" experience.

### Example Draft (Player Commits Warriors, Warriors AI Present)

| Pick | Warriors AI Action | Player Pack | Player Pick |
|:----:|-------------------|-------------|-------------|
| 1 | Takes best Warriors creature (affinity 0.92) | Mixed: Flash, Storm, Warriors, generic | Generic bomb (exploring) |
| 3 | Takes Warriors creature #3 (affinity 0.87) | Tide-heavy: 2 Warriors, 1 Sacrifice, 1 generic | Warriors creature (signaling) |
| 6 | Takes Warriors removal (creatures saturating) | 2 Warriors cards, 1 Ramp, 1 generic | Warriors trick (committing) |
| 10 | Takes Warriors utility (creatures full, switching) | 2 Warriors, 1 bridge, 1 off-arch | Warriors creature (strong -- AI left it because role full) |
| 15 | Takes generic power card (most roles filled) | 3 Warriors, 1 generic | Warriors spell (late-draft density) |
| 25 | Takes off-archetype filler (Warriors pool thin) | 3 Warriors, 1 bridge | Warriors creature (pool concentrated) |

Key observation: at pick 10, the player gets a strong Warriors creature that the Warriors AI passed because it already had enough creatures. This is the deckbuilding-awareness payoff -- the card survives because the AI behaves like a real drafter with a plan, not a vacuum that takes every top card.

### Failure Modes

1. **Role quota miscalibration.** If quotas are too high, AIs never saturate and behave like stateless bots. If too low, AIs saturate too early and become passive, weakening lane signals. Mitigation: quotas are tuning parameters per archetype, testable in simulation.

2. **Supplemental culling breaks the AI narrative.** The 5% per-round culling is not directly attributable to any AI's action. If the player discovers this mechanism, it undermines the "AI opponents" framing. Mitigation: keep culling percentage low enough that its effect is indistinguishable from AI picks; frame as "cards no one wanted are removed from the table."

3. **Open lane too obvious.** With one archetype always uncontested, experienced players will learn to identify it every game. Mitigation: the open archetype varies per game (7 of 8 are assigned, the missing one rotates). Also, bridge cards and shared-resonance effects mean "open" is a spectrum, not binary.

4. **Determinism enables memorization.** Since AIs are Level 0 (seed-determined), a player who replays the same seed sees identical AI behavior. In a roguelike with different seeds each run, this is not a practical concern -- but if seeds are visible or limited, it could become exploitable.

---

## AI Drafter Specification

| Parameter | Value |
|-----------|-------|
| Number of AIs | 7 |
| Archetype assignment | Random 7 of 8 per game (seed-determined) |
| Cards per AI per round | 1 |
| Supplemental culling | 5% of remaining pool per round (lowest average affinity) |
| Pick logic | pair_affinity * role_need_multiplier; pick highest score |
| Role categories | 4 per archetype (archetype-specific, e.g., creatures/tricks/removal/utility) |
| Role quotas | Sum to ~25 per AI; archetype-specific distribution |
| Saturation floor | 0.15 (saturated roles still picked at 15% weight, not zero) |
| Reactivity level | Level 0 (fully predetermined, seed-based) |
| Aggression | High early (all roles needed), declining mid-draft (roles filling), low late (most roles full) |
| Off-archetype picks | Emerge naturally when all archetype roles saturate; AI takes best generic or bridge card |
| Hidden metadata per card | 8 bits (pair-affinity, same as V9 Hybrid B) + 2-3 bits role tag |
| Floor slot | 1 top-quartile card from pick 3 (same as V9) |
| Archetype inference | From pick 5: player's committed archetype = archetype with highest mean pair-affinity among drafted cards sharing dominant resonance |

---

## Post-Critique Revision

### 1. The Contraction Math Problem

The critic is correct. This was not a close call.

At pick 4, pool size is approximately 340 cards. Seven AIs taking 1 card each removes 7 cards (2.1%). Supplemental culling at 5% removes roughly 17 cards. Total: ~24 cards, or 7% contraction. V9 Hybrid B removes ~12%. The gap is structural, not a tuning issue.

There is no honest fix that preserves the current architecture. The options are: (a) increase to 3-4 AI picks per round, which pushes total removal to 12-15% but requires more AI picks than the design specifies; (b) increase culling to 10-12%, which overwhelms the AI pick contribution and makes the "AI opponent" narrative an elaborate framing for pool contraction; or (c) reduce to 5 AIs but adopt multi-card picks (4 per AI), which is structurally D1 with role quotas added.

None of these preserve the distinctive 7-AI / 1-card-per-round identity. The contraction math forces either fewer AIs with more picks or more culling. The design as submitted does not work at its stated pick rate.

### 2. Role Quotas vs. D3's Saturation Threshold

Reluctantly: no, role quotas do not justify the complexity over D3's single saturation threshold.

D3 tracks one number per AI (archetype card count) and pivots behavior at one threshold (12 cards). This produces the same emergent "AI stopped taking creatures" signal because the saturation event coincides with general archetype card accumulation. Role quotas produce a finer-grained version of the same phenomenon -- the AI stops taking creatures specifically rather than archetype cards generally -- but the player cannot distinguish these behaviors. The additional signal is real but invisible.

The tuning cost is concrete: 32 parameters (8 archetypes x 4 role quotas) versus 1 parameter (saturation threshold). The marginal player-experience benefit is speculative. D3's simpler threshold is the correct design choice.

### 3. Would 5 AIs Have Been Better?

Yes. The decision to use 7 AIs was driven by wanting 1 open lane (8 archetypes minus 7 AIs). But 1 open lane is the weakest possible signal structure -- experienced players learn to find the gap mechanically, and the open lane becomes strictly dominant over any contested archetype.

Five AIs produce 3 open lanes, which is the configuration D1 and both critic-recommended hybrids use. It also makes the contraction math tractable: 5 AIs taking 4 cards each removes 20 cards per round (~6%), and 8-10 cards of supplemental culling reaches V9-equivalent levels without the culling overwhelming the AI picks. If this design were rebuilt from scratch, it would use 5 AIs with 3-4 picks each and role quotas layered on top of that structure.

### 4. What Role Quotas Actually Contribute

Separated from the broken contraction mechanism, the role-quota idea has one genuine contribution: it prevents early-draft archetype stripping by distributing AI demand across card types rather than concentrating it on highest-affinity cards regardless of type. This directly addresses M10 by leaving strong primary-role cards available into mid-draft (as shown in the pick-10 example in the deep-dive: the Warriors AI passed a strong creature because it already had enough).

This effect is real and the critic's 8/10 player experience score reflects it. The question is whether the implementation complexity is proportionate. As an enhancement bolted onto a passing algorithm (D1 or Hybrid X) rather than a standalone system, role-quota modulation is a reasonable tuning knob. As the defining feature of a top-level design competing for a simulation slot, it was overweighted relative to its actual contribution.

The critic's recommendation to test role quotas as an enhancement to a passing algorithm rather than as a primary simulation candidate is the correct disposition.

