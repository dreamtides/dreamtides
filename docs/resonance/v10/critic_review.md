# V10 Critic Review: Cross-Proposal Analysis

## 1. Proposal Rankings

### Axis 1: M3/M11 Potential

| Rank | Proposal | Predicted M3 | Predicted M11 | Assessment |
|:----:|----------|:------------:|:-------------:|------------|
| 1 | D1 Open Table | 2.65 | 3.2 | Credible. 5 AIs x 4 cards + 12-card cull = ~33 cards/round. Contraction math checks out. |
| 2 | D6 Syndicate | 2.65 | 3.2 | Credible. Escalating bulk picks match V9 mid/late. Early contraction (6%) may underdeliver M3 in picks 6-10. |
| 3 | D2 Sentinel | 2.65 | 3.2 | Plausible but optimistic. 6 AIs x 4 cards + 5% cull = ~29-35/round. Phase transition at pick 8 introduces a risk of micro-dip that could drag M10. |
| 4 | D4 Escalating | 2.65 | 3.15 | Slightly optimistic. Early phases remove only 14 cards/round (4.4% contraction), which delays pool concentration. M11 depends heavily on late-phase 35 cards/round actually achieving concentration before pool floor hits. |
| 5 | D3 Competitive Pressure | 2.55-2.65 | 3.0-3.2 | Range is honest. The saturation threshold (12 cards) is a single tuning knob that M11 hinges on. Could easily land at 2.8 or 3.2 depending on calibration. |
| 6 | D5 Role-Saturating | 2.65 | 3.2 | Mathematically suspect. Only 7 AI picks per round + 5% cull. At pick 4 (pool ~340), total removal is roughly 7 + 17 = 24 cards, which is 7% contraction -- well below V9's 12%. The predicted M3/M11 assumes this insufficient contraction somehow matches V9. The floor slot partially compensates, but this design likely lands at M3 ~2.4, M11 ~2.8 without increasing AI pick rates. |

**Key challenge:** D5 claims 7 AI cards per round (1 per AI) achieves V9-equivalent results. The V9 translation research explicitly shows this produces only 2.2% contraction. The 5% supplemental culling adds another ~17 cards, totaling ~7%. This is roughly 60% of V9's contraction intensity. The predicted metrics are the most likely to disappoint in simulation.

### Axis 2: Player Experience (Feels Like Drafting Against Real Opponents)

| Rank | Proposal | Score | Rationale |
|:----:|----------|:-----:|-----------|
| 1 | D3 Competitive Pressure | 9/10 | Saturation mechanic is the most human-like AI behavior. "I have enough creatures" is exactly how real drafters think. Level 0 throughout. |
| 2 | D5 Role-Saturating | 8/10 | Same deckbuilding-awareness benefit as D3 but with explicit role quotas that feel more engineered than D3's simpler threshold. |
| 3 | D4 Escalating | 7/10 | Escalating aggression is plausible human behavior ("I'm getting more focused"), but the synchronized escalation across all 7 AIs is detectable as artificial. Real drafters don't all escalate on the same schedule. |
| 4 | D1 Open Table | 7/10 | Clean and simple. The "market culling" mechanic slightly undermines the pure AI-opponent narrative -- 12 cards vanishing per round "because nobody wanted them" is a lot of vanishing. |
| 5 | D2 Sentinel | 6/10 | Phase transition is invisible to the player in theory, but 17Lands-style analysis would detect it. The staggered transition (picks 7/8/9) is a thoughtful mitigation. |
| 6 | D6 Syndicate | 5/10 | "Syndicates buying cards in bulk" is a narrative stretch. The proposal acknowledges this. Five entities each removing 4-6 cards per round feels like a market mechanism, not a draft table. |

### Axis 3: Simplicity

| Rank | Proposal | One-Sentence | Implementation Complexity |
|:----:|----------|:------------:|:-------------------------:|
| 1 | D1 Open Table | "5 AI opponents draft from the same pool; find the 3 archetypes they aren't taking." | Low. Predetermined picks, simple culling. |
| 2 | D4 Escalating | "7 AI opponents start casual and get more focused as the draft progresses." | Low-medium. Phase-based pick rates, otherwise straightforward. |
| 3 | D3 Competitive Pressure | "7 AIs draft their archetypes until they have enough, then branch out." | Medium. Requires tracking per-AI accumulation counts. |
| 4 | D2 Sentinel | "6 AIs lock in their strategy early, then adapt to what's left mid-draft." | Medium. Two-phase logic, supplemental culling, staggered transition. |
| 5 | D5 Role-Saturating | "7 AIs draft what their deck needs, not just the best card." | Medium-high. Per-AI role quotas, need multipliers, archetype-specific quota tuning. |
| 6 | D6 Syndicate | "5 organized groups buy cards in bulk from the shared pool." | Medium. Escalating pick rates, floor-card logic, but the framing adds cognitive load. |

### Axis 4: Signal Reading Quality

| Rank | Proposal | Score | Rationale |
|:----:|----------|:-----:|-----------|
| 1 | D1 Open Table | 9/10 | 3 open lanes from 5 AIs is the ideal signal density. Not too obvious (1 lane), not too noisy (7 AIs). Player must identify *which* 3 archetypes are open -- a genuine multi-game puzzle. |
| 2 | D6 Syndicate | 8/10 | Same 3-open-lane structure. Inter-syndicate rivalry on shared resonances creates secondary signals (resonance scarcity). Slightly less intuitive framing reduces the score. |
| 3 | D4 Escalating | 7/10 | Low early AI activity means early signals are weak. The signal only becomes readable around picks 6-8 when AIs escalate, which is when the player should be committing -- tight timing that may feel like the signal arrives too late. |
| 4 | D3 Competitive Pressure | 7/10 | Strong early signals (7 AIs all active), but only 1 uncontested lane is a thin margin. The saturation mechanic creates late-draft secondary signals, but these require subtle reading. |
| 5 | D2 Sentinel | 6/10 | 2 open lanes is reasonable but the Phase 2 transition muddies late-game signals. The player cannot tell if late card availability reflects the original AI composition or Phase 2 re-evaluation. |
| 6 | D5 Role-Saturating | 6/10 | 1 uncontested lane (7 of 8 AIs). Role saturation creates secondary "the AI stopped taking creatures" signals, but these are subtle and require understanding the AI's deckbuilding logic -- not the "reading the table" skill axis V10 targets. |

### Axis 5: "Not on Rails" Score

| Rank | Proposal | Score | Rationale |
|:----:|----------|:-----:|-----------|
| 1 | D1 Open Table | 9/10 | 3 open lanes = genuine 3-way choice. Contested lanes are weaker but viable. |
| 2 | D6 Syndicate | 9/10 | Same 3-open structure. |
| 3 | D4 Escalating | 7/10 | 1 fully open lane + 6 contested at varying levels. Player can fight a lane, but the uncontested archetype is strictly dominant. |
| 4 | D3 Competitive Pressure | 7/10 | 1 open lane. The saturation mechanic makes contested lanes viable late, which expands real choice, but the open lane is always best. |
| 5 | D2 Sentinel | 7/10 | 2 open lanes is solid choice space. Phase 2 creates additional nuance. |
| 6 | D5 Role-Saturating | 6/10 | 1 open lane. Role saturation helps contested lanes, but experienced players will learn to find the gap every time. |

---

## 2. Best Reactivity Level

**Verdict: Level 0 (static) is sufficient and preferable.**

The research findings are overwhelming and all six design agents converged on this conclusion. Design 3 explicitly rejected its own lane-avoidance mandate and pivoted to a Level 0 saturation model -- the strongest possible endorsement of the research. Design 2 proposed Level 1 pool-pressure reactivity but acknowledged it may be indistinguishable from Level 0 in practice.

Design 3's pivot was the right call. The agent honestly confronted the research showing that lane avoidance corrupts signal reading, violates the transparency principle, and transforms a skill-based puzzle into an automatic reward for commitment. Rather than proposing a compromised Level 3 design, it found a Level 0 mechanism (deckbuilding-aware saturation) that produces the convergence benefit of lane avoidance through a completely different causal pathway. This is the most intellectually honest move any design agent made.

The one context where mild reactivity (Level 1) might help is M10: the transition zone (picks 6-10) is structurally resistant to improvement under pure Level 0. Design 2's pool-pressure response could smooth this zone. However, simulation must determine whether this improvement is real or whether static AIs with escalating aggression (D4) or saturation (D3) achieve the same effect without the complexity and narrative cost of reactivity.

---

## 3. Best AI Pick Logic

**All proposals converge on pair-affinity scores as the card evaluation function.** This is the correct consensus. V9 proved that visible-only evaluation caps M11 at ~2.1. The AI drafter framing does not escape this mathematical constraint -- it merely re-labels "hidden metadata used by the contraction algorithm" as "hidden knowledge used by the AI's preference function." This is framing, not architecture.

The more interesting question is what sits *on top* of pair-affinity evaluation:

- **Pure affinity (D1, D6):** Simple, effective, proven by V9 analogy.
- **Affinity + imperfection noise (D1, D4, D6):** 10-30% off-archetype power picks. Adds verisimilitude. Likely has minimal metric impact but improves player experience.
- **Affinity + saturation threshold (D3):** A single state variable (archetype card count) modulates aggression. Elegant. The threshold is one tunable parameter.
- **Affinity + role quotas (D5):** Multiple state variables (per-role counts) modulate evaluation. More realistic but harder to tune. Per-archetype quota design is substantial additional work.

**Recommendation:** Pair-affinity + single saturation threshold (D3's approach) is the best complexity/benefit tradeoff. Role quotas (D5) add realism but the marginal benefit over a simpler saturation count is unproven and adds significant tuning surface.

---

## 4. Proposed Hybrid Designs

### Hybrid X: "Open Table with Saturation" (D1 + D3)

Take D1's core structure (5 random AIs, 3 open lanes, market culling) and add D3's saturation mechanic to each AI:

- **5 AIs**, randomly selected from 8 archetypes per game.
- **Each AI picks 4 cards per round** using pair-affinity scores.
- **Saturation at 12 archetype cards:** AI shifts from 85% archetype / 15% generic to 50% archetype / 30% adjacent / 20% generic.
- **Market culling:** 10 lowest-power cards removed per round after AI picks.
- **Total removal:** ~20 AI cards + 10 culled = 30 per round (~9.5% at pick 4).
- **Level 0 reactivity throughout.**

**Why this combination is better than either parent:**
- D1's 3-open-lane structure provides the best signal reading and "not on rails" score.
- D3's saturation adds the most human-like AI behavior and creates a natural late-draft convergence ramp without reactivity.
- The saturation mechanic means AIs in the player's contested lane eventually ease off, providing a natural floor-slot equivalent.
- Market culling bridges the remaining contraction gap to V9 levels.

**Predicted metrics:** M3 ~2.60, M10 ~2.8, M11 ~3.15, M6 ~84%.

**Key simulation question:** Does the combination of 5 AIs with saturation + market culling produce smoother pack quality curves than either mechanism alone?

### Hybrid Y: "Escalating Open Table" (D1 + D4)

Take D1's 5-AI / 3-open-lane structure and add D4's escalating pick rates:

- **5 AIs**, randomly selected.
- **Escalating picks:** 2 cards/AI in picks 1-5, 3 in picks 6-10, 4 in picks 11-15, 5 in picks 16+.
- **Market culling:** 8 lowest-power cards per round.
- **Total removal:** 10+8=18 early, 15+8=23 mid, 20+8=28 mid-late, 25+8=33 late.
- **Level 0 reactivity throughout.**

**Why this combination is better:**
- D4's escalation naturally produces the wide-open early / concentrated late draft arc, but with only 5 AIs instead of 7, the "free lane" puzzle is cleaner.
- 3 open lanes at all stages prevent the "only one right answer" failure mode.
- Escalation solves the early signal weakness that D4 suffers with 7 AIs (where all lanes are contested early, producing noisy signals).

**Predicted metrics:** M3 ~2.55, M10 ~2.5, M11 ~3.10, M6 ~83%.

**Key simulation question:** Does escalating 5-AI contraction produce better M5 (earlier convergence) than flat 5-AI contraction?

---

## 5. Flags

### Too Generous
- **D1 (if untuned):** 3 fully open lanes with no competing AI is generous. The player who identifies any open lane gets a strong deck with minimal effort. Mitigation: resonance-level competition (AIs on adjacent archetypes share resonances) ensures no lane is completely uncontested at the resonance level. Simulation must verify the open-lane advantage is moderate (M3 ~2.6) rather than excessive (M3 ~3.0+).

### Too Complex
- **D5 Role-Saturating:** Per-archetype role quotas (Warriors wants 12 creatures, 6 tricks, 4 removal, 3 utility; Storm wants 10 spells, 6 creatures...) create substantial design and tuning work. Eight sets of four-role quotas is 32 tuning parameters on top of the pair-affinity data. The marginal benefit over D3's single saturation threshold is unproven.
- **D6 Syndicate:** The "syndicate" framing requires more explanation than "AI opponents." The escalating pick rates (4/5/6 per syndicate), floor-card logic, and personality parameters (aggression 0.7-0.95, focus 0.75-0.95) combine into a system with many moving parts.

### Mathematically Suspect
- **D5 Role-Saturating:** As noted in the rankings, 7 AI picks per round + 5% culling produces ~7% contraction, far below V9's 12%. The predicted M3 ~2.65 and M11 ~3.2 are likely overestimates by 0.2-0.4 points. Simulation will almost certainly show this.
- **D3 Competitive Pressure (pick rate error):** The proposal itself flagged this: 7 AIs x 4 cards/round = 28 cards removed, depleting the 360-card pool in ~12 rounds. The revised rate of 2 cards/AI/round is more appropriate but drops contraction to ~4.2%, far below V9. Neither rate works without supplemental culling, which the proposal does not include. This is a structural gap that must be resolved before simulation.

### Convergence of Designs

Five of six proposals converge on a remarkably similar structure:

1. **N AIs with fixed archetype assignments** (N ranges from 5-7)
2. **Pair-affinity pick logic** (all use V9's 8-bit encoding)
3. **Level 0 reactivity** (even D2 and D3, assigned higher reactivity levels, pivoted toward Level 0)
4. **Supplemental culling** (all except D3 include explicit "market cleanup" removal)
5. **1-3 uncontested archetypes** as the player's primary signal

This convergence tells us several things:
- The design space is more constrained than the orchestration plan anticipated. The contraction gap forces all designs toward high AI counts and/or supplemental culling.
- Supplemental culling is V9's pool contraction re-labeled. When D1 removes "the 12 lowest-power cards nobody wanted," this is functionally identical to V9 removing the bottom 12% by relevance. The AI drafter layer adds narrative justification and lane signals, but the mathematical engine underneath is still pool contraction.
- The genuine novel contribution of V10 is not the contraction mechanism but the **signal reading skill axis**. V9's contraction was invisible to the player. V10's AI drafters create observable patterns the player can read. This is the real value proposition, and it does not require replacing V9's contraction -- it requires layering lane signals on top of it.

---

## 6. Recommended Algorithms for Simulation

### Slot 1: D1 Open Table (baseline static)
**Question:** Does the simplest viable AI drafter design match V9 Hybrid B metrics? This is the control. If it passes, complexity is unnecessary.

### Slot 2: Hybrid X (D1 + D3 saturation)
**Question:** Does adding deckbuilding-aware saturation to the Open Table improve M10 and produce more human-like AI behavior without sacrificing M3/M11?

### Slot 3: D4 Escalating Aggression
**Question:** Does escalating AI intensity produce the convergence ramp naturally, and does it solve V9's M5 failure (convergence delay)?

### Slot 4: Hybrid Y (D1 + D4 escalation)
**Question:** Is 5-AI escalation with 3 open lanes better than 7-AI escalation with 1 open lane? This directly tests whether signal clarity (more open lanes) or contraction intensity (more AIs) matters more.

### Slot 5: D2 Sentinel Draft
**Question:** Does Level 1 pool-pressure reactivity in Phase 2 produce measurably better M10 than pure Level 0? This is the reactivity experiment -- if Sentinel matches D1 on M10, reactivity adds no value and should be dropped.

### Slot 6: D3 Competitive Pressure (with revised pick rates and supplemental culling)
**Question:** Does the saturation mechanic alone, without the Open Table's 3-lane structure, produce competitive metrics? This isolates the saturation mechanism from the lane-count variable.

**Dropped from simulation:**
- **D5 Role-Saturating:** Insufficient contraction rate makes metric predictions unreliable. The role-quota mechanism can be tested as an enhancement to a passing algorithm later, but should not consume a simulation slot with its current pick-rate math.
- **D6 Syndicate:** The "syndicate" framing is a narrative liability. Its mathematical structure (5 entities, escalating bulk picks, 3 open lanes) is captured by Hybrid Y with a cleaner player-facing explanation.

---

## Summary of Critical Findings

**The contraction gap is real and unresolved by most proposals.** V9 removes ~38 cards per pick. Natural AI drafting (N AIs taking 1 card each) removes N cards. The gap is bridged by either (a) multi-card AI picks, (b) supplemental culling, or (c) both. Every proposal that achieves V9-equivalent contraction uses one of these bridges. The AI drafter paradigm does not inherently solve the contraction problem -- it solves the *narrative* problem while relying on the same mathematical mechanisms.

**Supplemental culling is V9 contraction in disguise.** This is the finding most design agents did not acknowledge. When D1 removes "12 lowest-power cards nobody wanted," this is pool contraction by another name. The honest framing: V10 is V9's contraction engine with an AI drafter narrative layer on top that creates lane signals. This is a good outcome -- the lane signals are genuinely new -- but we should not pretend the AIs alone drive pool concentration.

**The 5-AI / 3-open-lane structure is the winner.** It appears in D1, D6, and both hybrids. It produces the best signal reading, the widest real choice space, and natural game-to-game variety (C(8,5) = 56 compositions). The 7-AI / 1-open-lane structure (D3, D4, D5) reduces signal reading to "find the one gap" and makes the open lane strictly dominant, narrowing real player choice.

**Level 0 reactivity is confirmed optimal.** The universal convergence toward Level 0 -- including the explicit pivot by D3 -- validates the research finding that static AIs with per-game variety produce the best balance of signal reading, fairness perception, and genuine player agency.
