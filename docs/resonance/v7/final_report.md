# Resonance Draft System V7: Final Synthesis Report

## Executive Summary

V7 investigated the central question left open by V6: **how do zero-decision draft algorithms perform under realistic card design constraints?** V6's Surge Packs (T=4/S=3) achieved 2.05 S/A and 9/9 metrics, but assumed that every card sharing a primary resonance with the player's archetype would be A-tier -- an assumption that requires extraordinary card design discipline. V7 introduced three fitness models (Optimistic at 100% cross-archetype A-tier, Moderate at 50%, Pessimistic at 25%) and tested seven algorithm families across all three.

The investigation produced a clear and sobering answer: **no zero-decision algorithm achieves M3 >= 2.0 under Moderate fitness.** The best measured result is 1.88 S/A (Surge Packs T=3/S=3), with the refined Surge+Floor variant reaching 1.85 while also fixing convergence timing. All seven agents unanimously converged on **Surge Packs + Floor (T=3, S=3, floor_start=3)** as the recommended algorithm, with a revised M3 target of 1.8 under Moderate fitness. The investigation also conclusively eliminated several mechanism classes -- Aspiration Packs, Compass Packs, cost-based filtering, and dual-resonance pair targeting -- producing structural insights that constrain future design efforts.

The gap between the achievable 1.85 S/A and the aspirational 2.0 target is not an algorithm problem but a card design problem. Closing it requires increasing the cross-archetype A-tier rate from 50% to approximately 65%, which translates to designing 6-7 out of every 10 archetype cards to be playable in the sibling archetype sharing the same primary resonance.

## Unified Comparison Table

All values from Round 3 simulations. M3 = average S/A cards per pack (picks 6+). Pass counts use original M3 >= 2.0 target.

| Algorithm | M3(A) | M3(B) | M3(C) | Pass(A) | Pass(B) | Pass(C) | M5(B) | M9(B) |
|-----------|:-----:|:-----:|:-----:|:-------:|:-------:|:-------:|:-----:|:-----:|
| **Surge+Floor T=3** | **2.70** | **1.85** | **1.42** | **9/9** | **8/9** | **8/9** | **5.0** | **1.15** |
| Surge V6 T=3/S=3 | 2.66 | 1.88 | 1.49 | 6/9 | 8/9 | 7/9 | 5.8 | 1.21 |
| DualCounter T=3 | ~2.02 | 1.88 | ~1.48 | 9/9 | 8/9 | 7/9 | 5.7 | 1.21 |
| Surge V6 T=4/S=3 | 2.03 | 1.43 | 1.09 | 9/9 | 7/9 | 6/9 | 10.8 | 1.23 |
| Compass 2+1+1 | 2.21 | 1.66 | 1.32 | 6/9 | 7/9 | 6/9 | 7.2 | 0.73 |
| Asp+Bias 3.0x | 1.87 | 1.39 | 1.13 | 8/9 | 7/9 | 7/9 | 16.0 | 0.95 |
| Floor+Pair T=4 | 1.94 | 1.30 | 0.98 | 8/9 | 8/9 | 8/9 | 5.0 | 0.98 |
| Asp+PairPref | 1.02 | 0.84 | 0.78 | 6/8 | 5/8 | 5/8 | 26.0 | 0.75 |
| Pure Aspiration (best) | 0.92-1.02 | 0.72-0.84 | 0.63-0.75 | 7/9 | 5-6/9 | 5/9 | 22-28 | 0.75-0.83 |

**With revised M3 >= 1.8 under Moderate fitness:** Surge+Floor T=3 passes 9/9. Surge V6 T=3 and DualCounter T=3 also pass 9/9 (M3=1.88). All other algorithms fail.

## Robustness Ranking

Algorithms ranked by performance under Moderate fitness (the realistic scenario), not peak Optimistic performance.

| Rank | Algorithm | M3(B) | M5(B) | Pass(B) at M3>=1.8 | Simplicity | Rationale |
|:----:|-----------|:-----:|:-----:|:-------------------:|:----------:|-----------|
| 1 | **Surge+Floor T=3** | 1.85 | 5.0 | **9/9** | 7/10 | Best M5; floor eliminates non-surge dead zone |
| 2 | Surge V6 T=3 | 1.88 | 5.8 | 9/9 | 8/10 | Highest raw M3; slightly worse M5 |
| 3 | DualCounter T=3 | 1.88 | 5.7 | 9/9 | 4/10 | Identical to Surge T=3; cost filter adds nothing |
| 4 | Compass 2+1+1 | 1.66 | 7.2 | 7/9 | 6/10 | Neighbor slot wasted; M9 fails everywhere |
| 5 | Surge V6 T=4 | 1.43 | 10.8 | 7/9 | 8/10 | V6 champion; T=4 fires too infrequently |
| 6 | Asp+Bias 3.0x | 1.39 | 16.0 | 7/9 | 7/10 | Bias validated; base mechanism too weak |
| 7 | Floor+Pair T=4 | 1.30 | 5.0 | 8/9 | 6/10 | R2 surge slot costs more S/A than it gains |
| 8 | Asp+PairPref | 0.84 | 26.0 | 5/8 | 10/10 | Gate paradox; R2 delivers 3% S/A |
| 9 | Pure Aspiration | 0.72-0.84 | 22-28 | 5-6/9 | 10/10 | Structural floor: 2 slots cannot compete |

Surge+Floor T=3 ranks above Surge V6 T=3 despite marginally lower M3 (1.85 vs 1.88) because it achieves superior M5 convergence (5.0 vs 5.8, both within target but Floor is centered while V6 T=3 is near the edge) and provides a structural safety net on non-surge packs. DualCounter T=3 ties Surge V6 T=3 on metrics but ranks below due to significantly greater complexity for no measurable benefit.

## Per-Archetype Convergence Table (Top 3 Algorithms, Moderate Fitness)

### Surge+Floor T=3 (Recommended)

| Archetype | Convergence Pick | Within Target (5-8)? |
|-----------|:----------------:|:--------------------:|
| Flash | 7.4 | Yes |
| Blink | 7.7 | Yes |
| Storm | 7.6 | Yes |
| Self-Discard | 7.9 | Yes |
| Self-Mill | 7.7 | Yes |
| Sacrifice | 7.9 | Yes |
| Warriors | 7.8 | Yes |
| Ramp | 8.0 | Borderline |

All 8 archetypes converge within the 5-8 target window. Range is tight (7.4-8.0), indicating uniform convergence across the archetype circle.

### Surge V6 T=3

| Archetype | Convergence Pick | Within Target? |
|-----------|:----------------:|:--------------:|
| Flash | 6.5 | Yes |
| Blink | 9.0 | No |
| Storm | 10.4 | No |
| Self-Discard | 9.9 | No |
| Self-Mill | 8.9 | No |
| Sacrifice | 9.0 | No |
| Warriors | 9.3 | No |
| Ramp | 10.2 | No |

Only Flash converges within target. Average (M5=5.8) is inside the window, but most individual archetypes are outside it. The floor mechanism in Surge+Floor specifically addresses this dispersion.

### Asp+Bias 3.0x

| Archetype | Convergence Pick | Within Target? |
|-----------|:----------------:|:--------------:|
| Flash | 10.8 | No |
| Blink | 11.0 | No |
| Storm | 12.4 | No |
| Self-Discard | 14.8 | No |
| Self-Mill | 10.2 | No |
| Sacrifice | 13.9 | No |
| Warriors | 13.8 | No |
| Ramp | 15.7 | No |

No archetype converges within target. The algorithm essentially never converges under Moderate fitness.

## The Key Question Answered

**What is the best zero-decision draft algorithm when we honestly account for realistic card design constraints?**

**Surge Packs + Floor (T=3, S=3, floor_start=3).** It achieves the highest realistic S/A (1.85 under Moderate fitness) while maintaining all 8 non-M3 metrics in target ranges. It is the only algorithm where every archetype individually converges within the 5-8 pick window under realistic fitness.

**What S/A level should we target?**

**M3 >= 1.8 under Moderate fitness (50% sibling A-tier).** This is the practical ceiling for any zero-decision algorithm operating through resonance-level targeting. The original 2.0 target should be retained as a card-design quality gate: if designers achieve 65%+ cross-archetype A-tier, the algorithm automatically delivers 2.0+. The 2.0 target is achievable but requires deliberate, testable card design discipline -- it is not an algorithm problem.

## Simplicity Test

Each algorithm's one-sentence description evaluated for implementability by a programmer with no context.

| Algorithm | One-Sentence | Simplicity |
|-----------|-------------|:----------:|
| Pure Aspiration | "Compute top resonance pair; if second reaches a threshold, show 1 card of each in the pack, 2 random; otherwise all random." | 10/10 |
| Compass | "Each pack has 1 card from top resonance, 1 from an adjacent resonance alternating each pick, 2 random." | 8/10 |
| Surge V6 | "Drafted symbols add tokens; when any counter reaches 4, spend 4, fill 3 of 4 slots with that resonance." | 8/10 |
| **Surge+Floor** | "Drafted symbols add tokens; when counter reaches 3, spend 3, fill 3 slots with that resonance; on non-surge packs from pick 3+, 1 slot shows top resonance." | **7/10** |
| Asp+Bias | "Compute top resonance pair; if gate opens, 1 R1 slot + 1 R2 slot + 2 slots weighted 2x toward R1; otherwise all weighted-random." | 6/10 |
| DualCounter | "Surge Packs plus cost-band filtering: surge slots filtered to player's average cost +/-1, widening if insufficient." | 4/10 |

Surge+Floor scores 7/10 -- above the minimum threshold. It has two modes (surge and floor) but both are concrete operations describable in one sentence. The floor rule ("on non-surge packs from pick 3 onward, 1 slot shows your top resonance") is a simple conditional.

## Card Designer's Brief

For the recommended algorithm (Surge+Floor T=3, S=3, floor_start=3):

**The binding constraint is sibling A-tier rate.** Every resonance is primary for exactly 2 archetypes. The algorithm delivers cards from the player's primary resonance pool. Half those cards come from the player's home archetype (always S-tier) and half from the sibling archetype. The sibling cards' fitness determines the algorithm's S/A performance.

**Minimum requirements:**

- **50% sibling A-tier** (the Moderate baseline): M3=1.85. Passes 9/9 at revised M3 target of 1.8. This means 5 out of every 10 cards in archetype X must be at least A-tier for sibling archetype Y.
- **65% sibling A-tier** (the stretch target): M3 reaches ~2.0. This means 6-7 out of every 10 cards must be cross-archetype playable.
- **Below 50%**: No algorithm passes M3=1.8. The draft system underperforms regardless of algorithmic choice.

**Concrete guidance:**

1. For each primary resonance pair (e.g., Warriors + Sacrifice sharing Tide), design cards with broadly applicable effects that gain archetype-specific upside. Example: "When a creature leaves play, draw a card" works for both Warriors (combat) and Sacrifice (abandon), whereas "When you sacrifice a creature" is narrow to Sacrifice.

2. Target 2-3 "signal cards" per archetype that are S-tier in their home but B/C-tier in the sibling. These serve as draft signals helping players identify which specific archetype is available.

3. Secondary resonance fitness is irrelevant. No algorithm successfully exploits it. Do not invest design effort in cross-secondary playability.

4. Maintain 15+ cards per primary resonance in the pool to avoid depletion during heavy surge/floor drafting.

5. **Test every card:** "Is this at least A-tier in the sibling archetype?" Track the per-resonance-pair percentage. If any pair drops below 50%, the algorithm will underperform for those archetypes.

**Is 65% achievable?** Yes, with intentional design. The key is ensuring sibling archetype pairs have complementary rather than orthogonal mechanics. When Warriors (tribal combat) and Sacrifice (death triggers) overlap on "creatures matter" effects, most cards can serve both. When archetypes are designed as isolated islands, sibling A-tier drops below 50% and the algorithm cannot compensate.

## Recommended Algorithm: Complete Specification

### One-Sentence Description (Player-Facing)

"As you draft, the game tracks which resonance types you're collecting; every few picks you'll get a focused pack of 3 cards matching your top resonance, and in between, one card in each pack always matches your strongest resonance."

### One-Paragraph Technical Description

Maintain 4 resonance token counters (Ember, Stone, Tide, Zephyr), all starting at 0. After each pick, add tokens from the drafted card's symbols: +2 for the primary (leftmost) symbol, +1 for each secondary/tertiary symbol. Before generating each pack, check if the highest counter has reached 3 or more. If so, subtract 3 from that counter and generate a surge pack: 3 of the 4 slots are filled with random cards whose primary resonance matches the surge resonance, the 4th slot is random from the full pool. If no counter has reached 3, generate a floor pack (from pick 3 onward): 1 slot is filled with a random card whose primary resonance matches the player's highest counter, the other 3 slots are random from the full pool. Picks 1-2 are always fully random (4 random slots). Ties between counters are broken randomly.

### Parameter Values

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Token earning | +2 primary, +1 secondary/tertiary | Standard weighted counting from V6 |
| Surge threshold (T) | 3 | More frequent surges compensate for reduced per-slot fitness |
| Surge slots (S) | 3 of 4 | 75% targeting; 4/4 would eliminate splash |
| Floor start | Pick 3 | Preserves early openness (picks 1-2 fully random) |
| Floor slots | 1 of 4 | Minimum viable signal on non-surge packs |

### Key Properties

- **Non-permanent state:** Surges follow the current highest counter. If the player pivots, surges adapt.
- **Rhythmic pacing:** Committed players surge approximately every 1.5-2 picks, creating a surge/floor alternation.
- **Pack-type variance:** Surge packs deliver ~2.5 S/A (Moderate), floor packs ~1.2 S/A, early packs ~0.7 S/A. The alternation produces M9 stddev of 1.15.
- **Early openness:** Picks 1-2 are fully random. Floor activates at pick 3 with only 1 targeted slot, preserving exploration.
- **Convergence:** All 8 archetypes converge within picks 7.4-8.0 under Moderate fitness.

### Implementation Notes

- Generic cards (0 symbols) earn no tokens. Players drafting generics delay surges -- this is by design.
- Dual-resonance cards contribute +2 primary and +1 secondary, slightly accelerating secondary surges for splash support.
- Show token counters to the player. "2 tokens from a surge" creates anticipation.
- When generating surge/floor slots, draw from the resonance-filtered pool without replacement within a single pack.

## V7 vs V6 Comparison

**Did V7 find anything better than Surge Packs?**

Yes and no. V7 found a **better configuration** of Surge Packs -- T=3 with floor instead of T=4 without -- but did not find a fundamentally different mechanism that outperforms the Surge framework. The refinement is meaningful:

| Dimension | V6 (Surge T=4/S=3) | V7 (Surge+Floor T=3/S=3) |
|-----------|:-------------------:|:------------------------:|
| M3 Optimistic | 2.03 | 2.70 |
| M3 Moderate | 1.43 | 1.85 |
| M3 Pessimistic | 1.09 | 1.42 |
| M5 Moderate | 10.8 (FAIL) | 5.0 (PASS) |
| Pass(B) at M3=2.0 | 7/9 | 8/9 |
| Pass(B) at M3=1.8 | 7/9 | 9/9 |

The T=3 threshold fires surges more often, compensating for reduced per-slot fitness under realistic conditions. The floor slot eliminates the "dead zone" on non-surge packs where players previously saw 4 fully random cards. Together, these changes add +0.42 M3 under Moderate fitness and fix M5 convergence from 10.8 (FAIL) to 5.0 (PASS).

**What did V7 learn about the limits of zero-decision algorithms?**

1. The degradation from Optimistic to Moderate fitness is structural and universal. Every algorithm loses ~30-50% of its Optimistic M3.
2. Secondary resonance targeting (R2 slots, neighbor rotation, pair matching) provides near-zero S/A value. The R2 slot delivers 3-17% S/A, not the 37-50% predicted in design.
3. Cost-based disambiguation provides +0.05 S/A -- indistinguishable from noise.
4. The Surge mechanism (concentrated resonance bursts) remains structurally superior to every alternative tested: continuous soft targeting (Aspiration), always-on pack modification (Compass), probabilistic weighting (Bias), and secondary signals (cost filtering).
5. The only levers that meaningfully improve S/A under realistic fitness are (a) more primary-resonance slots per pack and (b) more frequent delivery of those slots.

## Honest Assessment

**What S/A target is realistic?**

1.8 under Moderate fitness (50% sibling A-tier). This is achievable with the recommended algorithm. The original 2.0 target requires 65% sibling A-tier, which is ambitious but possible with deliberate card design.

**Should the target be lowered?**

Yes, for algorithm evaluation purposes. M3 >= 1.8 under Moderate fitness is the appropriate benchmark. The 2.0 figure should be reframed as a card-design quality indicator rather than an algorithm metric.

**What does 1.85 S/A feel like to the player?**

At 1.85 S/A per pack (picks 6+), the committed player sees roughly 2 playable cards per pack in most packs, occasionally 3 (surge packs) and occasionally 1 (floor packs where the random slots miss). This is a meaningful draft experience: the player consistently has at least one good choice and occasionally faces a pleasant dilemma between two strong options. The floor mechanism ensures no pack is completely dead -- there is always at least 1 card matching the player's resonance.

**Player-experience mitigations if S/A remains below 2.0:**

1. **B-tier cards have value.** A card rated B-tier in the player's archetype is not unplayable -- it is merely not optimal. In a roguelike deckbuilder, B-tier cards serve as filler that gets replaced over multiple runs. The draft experience is "I see good cards most of the time, and occasionally take a weaker card that I know I'll improve later."

2. **Signal reading.** At 1.85 S/A, the player who reads signals (noticing which resonances have more S-tier cards available) achieves slightly better outcomes than a blind drafter. This preserves meaningful choice without requiring complex decisions.

3. **Surge packs feel rewarding.** Even at Moderate fitness, surge packs deliver ~2.5 S/A -- a noticeable spike in quality. The rhythm of "normal pack, normal pack, surge!" creates moments of excitement that the raw average does not capture.

4. **The draft is one part of the game.** In a roguelike deckbuilder, the draft feeds into combat encounters where card synergies matter more than individual card quality. A deck with 1.85 S/A plus good synergy outperforms a deck with 2.5 S/A and poor synergy. The algorithm's job is to provide a foundation; gameplay provides the rest.

## Open Questions for Playtesting

1. **Surge visibility:** Should players see their token counters and predict surges? Recommendation: yes. The "one token away from a surge" moment is the algorithm's signature experience.

2. **Floor perception:** Does 1 guaranteed resonance card on non-surge packs feel meaningful, or does it blend into the random noise? If players cannot distinguish floor packs from random packs, the floor's UX value is limited (though its statistical value remains).

3. **T=3 surge frequency:** At T=3, committed players surge every 1.5-2 picks. Does this feel too frequent? If surges become routine, they may lose their special quality. T=4 surges are rarer but hit harder. Playtesting should test both.

4. **The 65% sibling A-tier target:** Is it achievable in practice? Card designers should prototype one resonance pair (e.g., Warriors/Sacrifice) and measure what sibling A-tier rate emerges naturally, then iterate.

5. **Power-chaser support:** All algorithms perform poorly for power-chasers (players ignoring resonance entirely). Is this acceptable? In a roguelike, new players may power-chase. If the draft feels bad for them, consider a tutorial nudge.

6. **Splash card availability:** At M4=0.88 (Moderate), players see ~0.9 off-archetype cards per pack. Is this sufficient for splash strategies? If splash is important to gameplay, consider reducing floor_start to pick 2 (marginally more random early packs).

7. **Run-to-run variety:** M7=5.7% card overlap is extremely low (well under 40%). Verify in playtesting that drafts feel sufficiently different across runs.

8. **The hybrid question:** Surge+Floor+Bias (projected ~1.97 M3 under Moderate) was proposed but never simulated. If playtesting shows 1.85 is insufficient, this hybrid should be the first thing tested -- it adds ~0.12 M3 for one additional parameter (2x bias on non-surge random slots).
