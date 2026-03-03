# Design 3: Lane-Avoidant AI Drafters (Level 3 Reactivity)

## 1. Key Findings

- **Lane avoidance fundamentally corrupts signal reading.** When AIs retreat from the player's archetype, late-appearing cards no longer mean "nobody is in that lane" -- they mean "the AI moved away because of me." The player cannot distinguish earned openness from gifted openness, destroying the inferential skill axis that makes draft reading meaningful.

- **Full lane avoidance guarantees the open lane, removing the puzzle.** If AIs always avoid the player's committed archetype, commitment alone is sufficient to produce convergent drafts. The player never needs to read signals; they simply pick a lane and the system opens it. This is dynamic difficulty adjustment disguised as opponent behavior.

- **Partial lane avoidance (50% reduction) threads no needle.** A 50% pick-rate reduction for the player's archetype creates an intermediate state where signals are partially corrupted: late cards might mean "open lane" or might mean "AI is half-avoiding." This ambiguity is worse than either extreme because the player cannot calibrate their reads.

- **Lane avoidance violates the transparency principle.** The V10 framing promises "AI opponents drafting their own decks." An AI that watches the player's picks and adjusts its strategy has information no real draft opponent would have (seeing your picks from across the table). Discovery of this mechanic produces the deepest betrayal: "the game was secretly helping me."

- **Combining static early + avoidant late preserves signal reading only in theory.** If picks 1-5 are static and picks 6+ are avoidant, the player reads real signals early but then enters a phase where the system gifts convergence. The transition is detectable: the player notices that commitment always produces better packs, regardless of which lane was "open" early. The early signal reading becomes decorative.

- **Lane avoidance is the most natural floor-slot equivalent.** The V9 translation research correctly identifies that an AI backing off the player's lane is mechanically analogous to a floor slot -- it guarantees archetype card supply. This is lane avoidance's genuine strength, and the one context where partial avoidance might be defensible: not as a general mechanism, but as a targeted M10 fix.

- **The research warnings are substantially correct, but overstated for partial-reactive hybrids.** Pure Level 3 (full lane avoidance from pick 1) is indefensible. However, a system where a single AI reduces its pick rate in the player's lane by a modest amount after pick 8-10 is functionally similar to V9's floor slot and may be narratively defensible as "one drafter shifted strategy after seeing the table thin out."

## 2. Three Algorithm Proposals

### Proposal A: Full Lane Clearance

**Player description:** "When you commit to an archetype, the AI opponents gradually shift to other strategies, rewarding your commitment with better cards."

**Technical description:** 7 AIs, each assigned an archetype. Each AI drafts 4-5 cards per round from its archetype using pair-affinity scores. Starting at pick 6, AIs whose archetype matches the player's inferred commitment reduce their pick rate by 80%, effectively clearing the lane. Other AIs increase their aggression by 15% to maintain total pool depletion.

**AI drafter behavior:** Before pick 6, all AIs draft independently using pair-affinity preferences. After pick 6, the player's committed archetype is inferred. The matching AI drops from 4-5 cards/round to 1 card/round. Adjacent AIs (sharing a resonance) reduce by 30%. Distant AIs increase by 15%.

**Predicted metrics:** M3 = 2.9-3.1, M10 = 1.5-2.0, M11 = 3.5-3.7, M6 = 88-92%. M5 = 6-7 (fast convergence from lane clearing). Signal reading: near zero -- commitment alone produces convergence regardless of which lane was open.

### Proposal B: Subtle Drift

**Player description:** "AI opponents have preferred archetypes, but some occasionally shift focus mid-draft as the card pool evolves."

**Technical description:** 7 AIs, each assigned an archetype. All AIs draft independently through the entire draft. However, the AI assigned to the player's committed archetype (detected at pick 8) reduces its archetype-specific pick rate by 35% for remaining picks, replacing those picks with generic or off-archetype cards. This is framed as the AI "pivoting" in response to pool thinning, not player tracking.

**AI drafter behavior:** Picks 1-8: fully static, pair-affinity-driven, 4 cards/round per AI. Picks 9+: the player's lane AI reduces archetype picks from 4 to ~2.5 per round, replacing with generics. All other AIs unchanged. Total pool depletion rate stays constant.

**Predicted metrics:** M3 = 2.65-2.75, M10 = 2.5-3.0, M11 = 3.15-3.30, M6 = 84-88%. M5 = 8-9 (static early phase delays convergence). Signal reading: moderate -- early signals are genuine (picks 1-8 static), late drift is subtle enough that the player attributes improved packs to "reading correctly" rather than "system adjustment."

### Proposal C: Competitive Pressure with Safety Valve

**Player description:** "AI opponents draft aggressively in their lanes. Sometimes an AI that's been hoarding one type of card will start branching out, leaving opportunities."

**Technical description:** 7 AIs, fully static (Level 0) for all picks. No lane avoidance whatsoever. Instead, each AI has a "saturation threshold": after accumulating 12+ cards from its primary archetype, it begins taking 40% of its picks from adjacent archetypes or generics. This is deckbuilding-aware behavior, not player-reactive behavior. The effect on the player's lane is indirect: if the player's lane AI reaches saturation early (because it was aggressive), it naturally thins out its own lane pressure late in the draft.

**AI drafter behavior:** Each AI tracks its own drafted cards. Picks 1-12: strong archetype preference (85% archetype, 15% generic/adjacent). Picks 13+: if 12+ archetype cards accumulated, shifts to 50% archetype, 30% adjacent, 20% generic. This produces a natural convergence ramp: AI pressure on any single archetype peaks mid-draft and eases late.

**Predicted metrics:** M3 = 2.55-2.65, M10 = 2.8-3.5, M11 = 3.0-3.20, M6 = 82-87%. M5 = 8-9. Signal reading: genuine -- all AI behavior is independent of the player. The saturation mechanic is deckbuilding logic, not lane avoidance.

## 3. Champion Selection

**Champion: Proposal C (Competitive Pressure with Safety Valve).**

The research warnings against Level 3 reactivity are well-founded. Proposals A and B both involve the system reacting to the player's picks, which corrupts signal reading and violates the transparency principle. Even Proposal B's "subtle drift" is detectable over multiple runs: committed players will notice that their lane always eases up, regardless of which lane they chose.

Proposal C achieves the convergence benefit of lane avoidance without any player reactivity. The AI saturation mechanic is genuinely deckbuilding-aware behavior -- a real drafter who already has 12 Warriors cards would naturally start looking at other options. This produces a late-draft easing of archetype pressure that helps M11 without corrupting signals. The player's experience is: "I committed to the open lane, the AI in my lane ran out of things to take, and my late packs were great." This narrative is honest.

Proposal C also honestly confronts the finding that Level 3 reactivity is incompatible with the V10 goals. Rather than trying to "thread the needle," it sidesteps the problem entirely by using a Level 0 mechanism (static, deckbuilding-aware AIs) that produces a Level 3-like outcome (late-draft lane easing) through a completely different causal pathway.

## 4. Champion Deep-Dive: Competitive Pressure with Safety Valve

### How It Works

Seven AI drafters are assigned archetypes at game start (one per non-player archetype, with one archetype left uncontested as the "free lane"). Each AI maintains an internal deck tracker and uses pair-affinity scores to evaluate cards. The pick logic has two phases, determined not by the player's behavior but by the AI's own accumulation:

**Accumulation phase (AI has < 12 archetype cards):** The AI picks the highest pair-affinity card for its archetype from the available pool. 85% of picks are archetype-targeted; 15% are the highest-power generic or adjacent-archetype card available. This 85/15 split creates slight imperfection that makes AIs feel like real drafters who occasionally take a generically powerful card.

**Saturation phase (AI has >= 12 archetype cards):** The AI has "enough" core archetype cards and begins diversifying. 50% archetype, 30% adjacent archetype (sharing one resonance), 20% generic. This mirrors real human deckbuilding behavior: after accumulating a critical mass, drafters look for support cards and splashes.

### What the Player Sees vs. What the AIs Do

**Player sees:** Packs of 4 cards drawn from the remaining pool after AIs draft. Early packs (1-5) show broad archetype representation with observable gaps where AIs are active. Mid-draft packs (6-12) show increasing concentration as AIs deplete their lanes. Late packs (15+) show high concentration in the open lane because all AIs in other lanes have thinned out other archetypes, and the player's lane AI (if present) has hit saturation and eased off.

**AIs do:** Draft independently, tracking their own decks. No AI knows or cares what the player is doing. The saturation mechanic is purely self-referential. The convergence ramp is an emergent property: as AIs saturate, total archetype-specific pressure across the pool decreases, leaving more archetype cards for the player.

### Example Draft: Player Drafts Warriors (Open Lane)

Setup: 7 AIs assigned to Flash, Blink, Storm, Self-Discard, Self-Mill, Sacrifice, Ramp. Warriors is the uncontested lane.

| Pick | AI behavior | Player's pack | Player picks |
|------|-------------|---------------|-------------|
| 1-3 | Each AI takes 4 cards/round from its archetype. Sacrifice AI takes Tide cards (some Warriors-adjacent). | 3-4 diverse options, 0-1 Warriors S/A | Best available, exploring |
| 4-5 | AIs continue accumulating. Sacrifice AI has 8 Tide cards, some with Warriors affinity. | 1-2 Warriors S/A per pack (Sacrifice AI competes for Tide pool but prefers Sacrifice-affinity cards) | Commits to Warriors after seeing Tide flowing |
| 6-10 | All AIs in accumulation phase. Sacrifice AI takes Sacrifice-affinity Tide cards, leaving Warriors-affinity Tide cards. Pool thins across all archetypes. | 2-2.5 Warriors S/A per pack. Sacrifice AI's pair-affinity preference naturally separates Warriors from Sacrifice cards. | Drafts Warriors cards confidently |
| 11-15 | Sacrifice AI hits 12 archetype cards around pick 12, enters saturation. Reduces Tide picks from 85% to 50%. Other AIs also approaching saturation. | 2.5-3 Warriors S/A per pack. Reduced Sacrifice AI pressure on Tide pool means more Warriors cards survive. | Strong Warriors picks, occasional splash |
| 16-20 | Most AIs saturated. Total archetype-specific picks across all AIs drops from ~28/round to ~20/round. Pool depletion slows. | 3-3.5 Warriors S/A per pack. Late-draft concentration peaks. | Best packs of the draft |

### Example Draft: Player Drafts Sacrifice (Contested Lane)

Setup: Same 7 AIs. Sacrifice AI is present and active.

| Pick | Player's pack | Player picks |
|------|---------------|-------------|
| 1-5 | Sacrifice AI is taking Sacrifice-affinity Tide cards. Player sees fewer Sacrifice S/A (1-1.5 per pack). Signal: Sacrifice looks thin. | Player commits to Sacrifice anyway (forcing) |
| 6-10 | Competition with Sacrifice AI. Both want the same cards. Player gets 1.5-2 S/A per pack -- viable but thin. | Workable but not exciting picks |
| 11-15 | Sacrifice AI hits saturation at pick 13. Pressure eases. Player gets 2-2.5 S/A per pack. | Improving quality as AI diversifies |
| 16-20 | Sacrifice AI fully saturated, taking only 50% Sacrifice cards. Player gets 2.5-3 S/A per pack. | Strong late packs recover the draft |

The contested lane produces a weaker but playable deck (M6 ~75-80% vs ~85-88% for open lane). The player who reads the signal and avoids the contested lane gets a better outcome -- genuine signal reading skill.

### Failure Modes

1. **Saturation threshold too low:** If AIs saturate at 8 cards, they ease off too early, reducing mid-draft competition and weakening lane signals. The pool stays too open and M6 drops.

2. **Saturation threshold too high:** If AIs saturate at 16 cards, they never ease off meaningfully. Late-draft concentration doesn't improve and M11 suffers.

3. **Adjacent-archetype picks during saturation create unexpected lane pressure.** A saturated Sacrifice AI taking adjacent (Stone, Ember) cards could thin Self-Mill or Storm unexpectedly. This needs tuning to ensure saturation picks don't create cascade effects.

4. **The "free lane" (uncontested archetype) is too obvious.** If one archetype has no AI, it is always the best lane. Players may learn to identify the free lane immediately, reducing signal reading to "find the gap." Mitigation: vary which archetype is uncontested per game, and ensure the free lane advantage over contested-but-open lanes is moderate, not overwhelming.

## 5. AI Drafter Specification

| Parameter | Value |
|-----------|-------|
| Number of AIs | 7 |
| Archetype assignment | Random 7 of 8 archetypes per game; 1 archetype uncontested |
| Card evaluation | Pair-affinity scores (8-bit, same as V9 Hybrid B) |
| Reactivity level | Level 0 (fully static, no player awareness) |
| Pick rate | 4 cards per AI per round (28 AI picks + 1 player pick = 29 cards removed per round) |
| Accumulation phase | < 12 archetype cards: 85% archetype, 15% generic/adjacent |
| Saturation phase | >= 12 archetype cards: 50% archetype, 30% adjacent, 20% generic |
| Saturation threshold | 12 archetype cards (tunable, test range 10-14) |
| Aggression | Moderate-high: AIs take highest pair-affinity card available, with 15% imperfection |
| Pool depletion | ~29 cards/round = 8% of pool at pick 1, rising to ~20% by pick 12 as pool shrinks |
| Effective contraction by pick 12 | Pool reduced from 360 to ~360 - (29 * 12) = ~12 cards remaining -- this is too aggressive; reduce to 2 cards/AI/round for 14 AI picks + 1 player = 15/round, pool at pick 12 ~180. Tuning required. |
| Pair-affinity encoding | Identical to V9 Hybrid B: two 4-bit floats per card |
| Floor slot | Not explicitly modeled; saturation mechanic serves as organic floor equivalent |

**Revised pick rate note:** At 4 cards per AI per round with 7 AIs, the pool depletes in ~12 picks (360 / 29 = 12.4 rounds). This is far too fast. The correct calibration is 2 cards per AI per round (14 AI picks + 1 player pick = 15 cards/round), which depletes the pool over 24 rounds -- appropriate for a 30-pick draft. At this rate, the pool at pick 12 is ~180 cards, providing adequate variety while the saturation mechanic produces late-draft concentration.

---

## Post-Critique Revision

### 1. Fixing the Contraction Math

The critic identified the core structural failure: neither pick rate works in isolation.

- **4 cards/AI/round:** 28 AI picks + 1 player = 29 cards/round. Pool depleted in 12.4 rounds. Fatal.
- **2 cards/AI/round:** 14 AI picks + 1 player = 15 cards/round. Contraction rate at pick 4 (pool ~360) is 4.2%. V9's target is ~12%. Gap: 7.8 percentage points unaddressed.

The honest conclusion: the saturation mechanic does not solve the contraction problem. It is a pick-logic refinement layered on top of a pool-depletion engine that must be specified separately.

**Revised specification for simulation (Slot 6):**

- **AI picks:** 2 cards/AI/round (7 AIs = 14 AI picks/round).
- **Supplemental culling:** Remove the 10 lowest pair-affinity cards from the pool each round after AI picks. No narrative justification needed for Slot 6 -- this is a simulation parameter.
- **Total removal:** 14 AI + 10 culled + 1 player = 25 cards/round. At pick 4 (pool ~360): 6.9% contraction. At pick 12 (pool ~180): 13.9% contraction. The culling rate rises as the pool shrinks, naturally producing the late-draft concentration ramp that M11 requires.
- **Saturation threshold:** unchanged at 12 archetype cards (tunable 10-14).

This resolves the structural gap the critic identified. The supplemental culling is openly V9 contraction by another label, and that is acceptable -- the saturation mechanic's value is pick-logic realism and the convergence ramp it creates, not pool depletion.

### 2. On AI Count: 7 AIs vs. 5-6 AIs

The critic's structural finding is persuasive: 7 AIs with 1 open lane reduces signal reading to "find the gap," and the open lane is strictly dominant. This narrows real player choice.

This design should not pivot to 5-6 AIs for Slot 6. The purpose of Slot 6 is to isolate the saturation mechanic from the lane-count variable. If Slot 6 used 5 AIs with 3 open lanes, any metric improvements would be attributable to lane structure rather than saturation. The critic's simulation plan is correct: keep 7 AIs / 1 open lane for Slot 6, and test lane count separately via D1 and Hybrid X.

The implication is that if the saturation mechanic clears its hurdle in Slot 6, the recommendation is to carry it into the 5-AI / 3-open-lane structure rather than deploy the 7-AI version independently.

### 3. Endorsement of Hybrid X

Yes, Hybrid X is the better vehicle for the saturation mechanic, and this design endorses it without reservation.

D1's 5-AI / 3-open-lane structure is strictly superior on signal reading, "not on rails" score, and player experience relative to this design's 7-AI / 1-open-lane structure. The saturation mechanic does not require 7 AIs -- it is a per-AI pick-logic rule that transplants cleanly into any AI count. Hybrid X captures D3's best innovation and pairs it with the lane structure the critic identified as the strongest in the field.

The saturation threshold in Hybrid X may require retuning. With 5 AIs each taking 4 cards/round, AIs reach 12 archetype cards around pick 3 (12 cards / 4 picks/round = 3 rounds). That is too fast -- saturation would trigger before the player has meaningful signal. Recommended Hybrid X threshold: 16-18 archetype cards, tuned so saturation onset lands at picks 8-12. Simulation should test threshold range 14-20.

### 4. Revised Metric Predictions (Slot 6, Corrected Contraction)

With 7 AIs at 2 cards/round + 10 supplemental cull:

| Metric | Original Prediction | Revised Prediction | Rationale |
|--------|--------------------|--------------------|-----------|
| M3 | 2.55-2.65 | 2.45-2.55 | 6.9% early contraction is still below V9. Open packs will be slightly leaner. |
| M10 | 2.8-3.5 | 2.6-3.0 | Saturation onset around pick 8 improves mid-draft quality, but thinner early pool limits the ceiling. |
| M11 | 3.0-3.20 | 2.9-3.1 | Late contraction (13.9% by pick 12) is V9-competitive. Saturation mechanic adds a second convergence driver alongside culling. Combined effect may recover the original upper estimate. |
| M6 | 82-87% | 80-85% | 1-open-lane structure makes open-lane advantage too strong; contested lane outcomes pull the average down. |

The single largest uncertainty is whether the supplemental cull's late-draft intensification (as pool shrinks, 10 fixed culls represent a larger percentage) is sufficient to drive M11 to 3.0+. If not, the cull count should increase to 12-14 per round. Simulation will answer this directly.

The revised predictions are honest rather than aspirational. The design's value in Slot 6 is not to match Hybrid X's expected metrics -- it is to isolate the saturation mechanic's independent contribution, which remains the most human-like pick logic in the field regardless of lane count.
