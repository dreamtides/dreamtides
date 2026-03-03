# Design 2: Delayed-Reaction AI Drafters (Level 1 Reactivity)

## 1. Key Findings

- **Two-phase AI behavior is mechanically coherent as "pool pressure response."** AIs that lock in their lanes for picks 1-5 and then adjust pick urgency based on remaining pool composition (not player-specific tracking) are explainable as "the other drafters noticed good cards thinning out and started grabbing them faster." This framing passes the fairness test from the reactivity research.

- **Late-game reactivity improves M10 without corrupting early signals.** The core insight: if AIs are static in picks 1-5, those early signals are guaranteed honest. If AIs then increase their pick aggression in response to pool scarcity from pick 6 onward, the early signals remain valid -- the player's read of which lane is open was based on real AI behavior, and the AI's later urgency is a response to the pool thinning, not to the player's picks specifically.

- **Pool-pressure reactivity solves a different problem than lane avoidance.** Lane avoidance (Level 3) guarantees the player's lane opens up. Pool-pressure reactivity does the opposite: it causes AIs to draft their own lanes more aggressively as competition for remaining cards increases. This naturally concentrates the remaining pool toward archetypes that no AI is aggressively pursuing -- which is the player's open lane. The effect is convergence-positive without being player-tracking.

- **The transition to reactive behavior can address M5 convergence delay.** V9's M5 = 9.6 occurs because archetype inference is unreliable through picks 5-9. If the player can observe lane signals from static AI behavior in picks 1-5 and commit earlier based on those signals, M5 improves because the system begins targeting the correct archetype sooner.

- **Supplemental culling is still required.** Even with 7 AIs picking 4 cards per round (28 removed), the contraction rate at pick 4 is approximately 8% -- below V9's 12%. The two-phase design must include a "discard pile" mechanic (cards that no AI wanted are removed) to reach V9-equivalent pool concentration.

- **The reactive phase must be bounded.** Research strongly warns that reactivity which always helps the player removes agency. Pool-pressure reactivity must have a ceiling: AIs become more aggressive in their own lane, which helps non-competing archetypes but also means the AI's lane becomes genuinely depleted. A player who chose to fight an AI's lane faces stiffer competition in the late game, preserving the risk/reward of lane reading.

- **Per-game AI composition variety is the primary replayability driver, not reactivity.** The two-phase structure adds texture to individual games but game-to-game variety comes from which 5-7 AIs are active and which archetypes they target. Reactivity is a secondary parameter.

---

## 2. Three Algorithm Proposals

### Proposal A: Pressure Ramp

**Player description:** "AI opponents start with a plan, then fight harder for their cards as the draft goes on."

**Technical description:** 7 AIs, each assigned one non-player archetype (randomly selected from 8, leaving 1 uncontested). Picks 1-5: each AI takes its top 4 archetype-affinity cards per round from the pool (predetermined at draft start using pair-affinity scores). Picks 6+: each AI re-evaluates the pool each round, taking the top 4 remaining cards by affinity, with a scarcity multiplier that increases pick urgency for cards whose archetype pool has thinned. Supplemental culling: after all AI picks, remove the bottom 4% of remaining pool by generic "desirability" (lowest power among cards no AI wanted).

**AI drafter behavior:** Static early (locked pick lists), dynamic late (re-rank by current pool state). AIs never track the player. Scarcity multiplier = (initial_archetype_pool / current_archetype_pool), capped at 2.0x.

**Predicted metrics:** M3 = 2.55, M10 = 3.0, M11 = 3.10, M6 = 84%.

### Proposal B: Tidal Urgency

**Player description:** "AI opponents draft at their own pace -- some are relaxed early and aggressive late, some are aggressive throughout."

**Technical description:** 5 AIs with variable urgency profiles. Each AI has a "temperature" curve over 30 picks: low-temperature AIs pick only top-affinity cards; high-temperature AIs pick from a wider affinity range. All AIs start at low temperature (picks 1-5, taking only strong home cards). From pick 6, each AI's temperature rises at its own rate based on how depleted its archetype pool is. Supplemental culling: 6% per round of lowest-desirability cards removed after AI picks (framed as "cards discarded from the shared pool because no one wanted them").

**AI drafter behavior:** Each AI has a fixed archetype and a pick threshold that tightens as its pool shrinks. Early picks are from the top of affinity rankings; late picks dip into bridge and secondary-resonance cards. AIs with depleted pools start taking off-archetype power cards (simulating a human drafter running out of options).

**Predicted metrics:** M3 = 2.45, M10 = 2.8, M11 = 3.05, M6 = 82%.

### Proposal C: Sentinel Draft (Champion)

**Player description:** "AI opponents establish their strategies early. As the card pool shrinks, they draft more urgently -- and the leftovers pile up in your favor if you found the open lane."

**Technical description:** 6 AIs, each assigned a distinct archetype from the 8 (2 archetypes uncontested per game, varying by seed). Two phases:

- **Phase 1 (picks 1-7):** Each AI has a predetermined pick list generated at draft start: top 4 cards per round by pair-affinity for its archetype, from the full 360-card pool. AIs execute these picks in order. If a card was already taken (by another AI or the player), the AI skips it and takes the next on its list. No pool-awareness, no reactivity. This creates clean, readable lane signals.

- **Phase 2 (picks 8-30):** Each AI switches to live evaluation. Each round, the AI scans the current pool and takes the top 4 cards by pair-affinity for its archetype. The key difference from Phase 1: the AI now sees what actually remains, so its picks naturally concentrate on its remaining best options. AIs whose archetype pools are thin start taking bridge cards and secondary-resonance cards, creating a realistic "running out of good picks" pattern.

**Supplemental culling:** After all AI picks each round, remove the lowest 5% of remaining pool by a "nobody wanted this" score (average affinity across all active AIs, inverted -- cards that no AI values highly are culled). This is framed as "cards that fell out of circulation because no drafter was interested."

**AI drafter behavior:** Phase 1 is fully static. Phase 2 is pool-responsive but not player-tracking. AIs use pair-affinity scores (8-bit equivalent) for card evaluation. Each AI takes 4 cards per round across both phases.

**Predicted metrics:** M3 = 2.65, M10 = 2.5, M11 = 3.20, M6 = 85%.

---

## 3. Champion Selection: Sentinel Draft

Sentinel Draft (Proposal C) is the champion for three reasons:

1. **Clean phase separation preserves signal integrity.** Phase 1's fully predetermined picks guarantee that early lane signals are honest and readable. The player's picks 1-7 experience is identical to a Level 0 system, preserving the signal reading skill axis. Phase 2's pool-responsive behavior is invisible in its mechanism -- the player cannot distinguish "the AI re-evaluated the pool" from "the AI was always going to pick that."

2. **Six AIs leaving two lanes open is the right competitive density.** With 6 of 8 archetypes covered, the player has exactly 2 uncontested lanes and 6 contested ones per game. This creates genuine lane reading (find the open pair) without being overwhelming (3 viable options: 2 uncontested + 1 least-aggressive contested).

3. **Supplemental culling is narratively clean.** "Cards no one wanted fell out of circulation" is a natural explanation for pool shrinkage beyond AI picks. Combined with 6 AIs taking 4 cards each (24 per round) plus 5% culling, the system achieves roughly 30-35 cards removed per round -- approaching V9's 38-card target.

---

## 4. Champion Deep-Dive: Sentinel Draft

### How It Works

At draft start, the system seeds 6 AI drafters with archetype assignments. Two archetypes are left uncontested (e.g., no Warriors AI and no Storm AI this game). Each AI generates its Phase 1 pick list: the top 28 cards (7 rounds x 4 picks) by pair-affinity for its archetype from the full 360-card pool.

Each round, AIs pick before the player's pack is constructed. In Phase 1, AIs execute their predetermined lists. In Phase 2, AIs re-scan the remaining pool and take their top 4 available cards by affinity. After AI picks, 5% supplemental culling removes the least-desired remaining cards. Then the player's 4-card pack is drawn from what survives.

### What the Player Sees vs. What the AIs Do

**Player sees:** 4-card packs. In early packs (picks 1-5), all 8 archetypes appear in roughly equal proportion because AIs have only removed a fraction of the pool. By picks 3-5, the player notices certain archetypes appearing more frequently -- these are the uncontested lanes. By picks 8-10, the uncontested archetype's cards dominate available options if the player committed to it.

**AIs do:** Phase 1 -- execute fixed lists, creating consistent depletion patterns the player can read. Phase 2 -- re-evaluate each round, naturally concentrating picks on their remaining best cards. An AI whose archetype is nearly depleted starts taking bridge cards and secondary-resonance cards, which appears to the player as that archetype "drying up" in the pool.

### Example Draft (Player drafts Warriors, uncontested)

**Setup:** 6 AIs on Flash, Blink, Self-Discard, Self-Mill, Sacrifice, Ramp. Warriors and Storm uncontested.

- **Picks 1-3 (exploration):** Player sees mixed packs. Tide cards appear slightly more often than expected because no AI is taking Warriors cards. Ember cards appear at normal rates because Storm is also uncontested but Blink and Self-Discard AIs overlap on Ember. Player notices the Tide abundance.

- **Picks 4-7 (Phase 1 continues, signal strengthens):** The Sacrifice AI is removing some Tide cards (sacrifice-affinity ones), but warriors-affinity Tide cards survive. Player sees increasing warriors-affinity Tide cards. Player commits to Warriors around pick 5-6 based on the Tide signal.

- **Picks 8-15 (Phase 2 begins):** AIs switch to live evaluation. The Sacrifice AI, having depleted its top sacrifice-affinity picks, starts taking bridge Tide cards. But warriors-home Tide cards remain abundant because no AI prioritizes them. Player consistently sees 2-3 S/A Warriors cards per pack.

- **Picks 16-30 (late draft):** Pool is concentrated. Warriors cards dominate the player's packs (3+ S/A per pack). The Sacrifice AI has mostly moved to secondary-resonance Stone cards. Supplemental culling has removed most generic/irrelevant cards.

### Failure Modes

1. **Player fights an AI's lane.** If the player commits to Flash despite the Flash AI, they compete for the same pair-affinity cards. Phase 2 makes this worse: the Flash AI urgently grabs remaining Flash cards. The player gets a weaker deck (M3 drops to ~1.8 for the contested archetype) but bridge cards from the Zephyr pair provide enough playables.

2. **Both uncontested lanes are unappealing.** If the two open archetypes are Flash and Ramp (both 25% fitness), the player might prefer a 40-50% fitness contested lane. This is a real tradeoff -- the system is not on rails.

3. **Phase 2 transition creates a micro-dip.** When AIs switch from predetermined to live evaluation at pick 8, their picks may cluster differently for 1-2 rounds. This could create 1-2 packs where the player's lane quality dips. Mitigation: stagger the transition (AIs switch at picks 7, 8, 9 individually) to smooth the effect.

---

## 5. AI Drafter Specification

| Parameter | Value |
|-----------|-------|
| Number of AIs | 6 per game |
| Archetype assignment | 6 of 8 archetypes, randomly selected per game seed |
| Uncontested lanes | 2 per game (the archetypes with no AI) |
| Cards per AI per round | 4 |
| Total AI picks per round | 24 |
| Supplemental culling | 5% of remaining pool after AI picks |
| Total removal per round | ~29-35 cards (24 AI + 5% cull) |
| Phase 1 duration | Picks 1-7 (predetermined pick lists) |
| Phase 2 duration | Picks 8-30 (live pool evaluation) |
| Card evaluation | Pair-affinity scores (8-bit, two 4-bit floats per card) |
| Reactivity level | Level 0 in Phase 1, Level 1 (pool-pressure) in Phase 2 |
| AI aggression | Top-4-by-affinity in Phase 1; top-4-by-affinity from remaining pool in Phase 2 |
| Bridge card behavior | Phase 2 AIs take bridge cards when home cards depleted |
| Player tracking | None. AIs respond to pool state, never to player picks |
| Generic card handling | AIs ignore generics; culling removes low-power generics gradually |
| Phase transition | Staggered: AIs switch at picks 7/8/9 (2 each) to smooth the transition |

---

## Post-Critique Revision

### 1. Is Phase 2 Reactivity Worth the Complexity?

Probably not as a general principle, but the simulation will settle it. The critic's most pointed observation is that Level 0 was unanimously confirmed as optimal, and D3 -- the one agent assigned to defend reactivity -- explicitly abandoned it. That defection is the strongest evidence against Phase 2. D3 found that deckbuilding-aware saturation achieves convergence through a Level 0 mechanism; Sentinel Draft's Phase 2 adds implementation complexity to accomplish something that may already happen for free under static AIs with sufficiently aggressive pick rates.

The honest framing: Phase 2 reactivity was motivated by M10 improvement, and that motivation is legitimate. The critic acknowledged that picks 6-10 are structurally resistant to Level 0. But "resistant" does not mean "broken." D4's escalating aggression and D1 with market culling both produce plausible M10 curves without pool-pressure tracking. If Slot 1 (D1) hits M10 >= 2.8, Sentinel Draft's Phase 2 never needed to exist.

**Decision:** Preserve Phase 2 for the simulation as designed. Do not remove it preemptively. If the simulation shows Sentinel Draft's M10 matches D1's M10 within 0.1, the verdict is clear: reactivity adds no value, and D1's cleaner structure wins.

### 2. How to Make the Simulation Useful

The simulation question is narrow and should stay narrow: **Does Level 1 pool-pressure reactivity in Phase 2 produce measurably better M10 than D1 at the same AI count and culling rate?**

To isolate the variable cleanly, run a paired comparison: Sentinel Draft (Phase 1 static, Phase 2 reactive, 6 AIs) against a Level 0 control variant (same 6 AIs, same culling rate, Phase 2 replaced with continued predetermined picks). This is more informative than comparing Sentinel to D1, which differs on AI count (6 vs. 5), culling rate, and lane structure simultaneously. If the simulation team is willing to add a sub-variant, the Level 0 control is the right ask.

The metric to watch most closely is M10, not M3 or M11. If Sentinel's Phase 2 does not move M10 relative to the Level 0 control, the experiment is conclusive. Secondary watch: M6 (player choice score), which could degrade if Phase 2 re-evaluation causes AIs to encroach on secondary-resonance cards that would otherwise flow to the player.

### 3. Parameter Adjustments

One change to strengthen the design before simulation: **reduce the Phase 2 transition from pick 8 to pick 9, staggered to picks 8/9/10.** The critic's concern about a micro-dip at the transition is real. Moving the boundary one pick later gives Phase 1's clean signals more room to register before the player's commitment window closes at picks 9-11. The cost is two extra rounds of slightly lower live-evaluation accuracy, which is negligible.

If the simulation shows M10 underperforms: increase culling from 5% to 7% before adjusting reactivity parameters. Culling is the more predictable lever.

### 4. Honest Assessment: Differentiation from D1

Sentinel Draft is differentiated from D1 in one meaningful way: 6 AIs versus 5, leaving 2 uncontested lanes instead of 3. This is a real design difference. Two open lanes creates a narrower but more interesting search problem -- find the pair, not find one of three. Whether this is better or worse than D1's three-lane structure is a legitimate empirical question.

The Phase 2 reactivity is not a meaningful differentiator. It is invisible to the player by design and may be undetectable even in simulation output. If Sentinel Draft earns a place in the final algorithm, it will be because the 6-AI / 2-open-lane structure performs well -- not because of pool-pressure tracking. The design should be willing to shed Phase 2 entirely if the simulation supports it, and pivot to a Level 0 6-AI variant as the actual candidate. The lane structure is the idea worth testing. The reactivity is incidental.
