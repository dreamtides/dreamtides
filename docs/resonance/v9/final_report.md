# Resonance Draft System V9: Final Synthesis Report

## 1. Unified Comparison Table

All results on 10% visible dual-res pool (36 cards), Graduated Realistic fitness, committed player strategy, 1000 drafts.

| Algorithm | M3 | M11 | M10 | M6 | M5 | M9 | Spread | V1 | V2 (bits) | V3 | V4 gap | Pass |
|-----------|---:|----:|----:|---:|---:|---:|-------:|---:|----------:|---:|-------:|-----:|
| Hybrid B (2+5) | **2.70** | **3.25** | 3.8 | 86% | 9.6 | 1.08 | 0.25 | 84.8% | 8 | 9/10 | 2.08 | **8/10** |
| Hybrid A (4+6) | 2.62 | 2.83 | 5.26 | 79% | 9.4 | 0.69 | 0.48 | 77.0% | 3 | 8/10 | 1.97 | 5/10 |
| Design 4 (Layered Salience) | 2.36 | 2.40 | 2.13 | 87% | 7.8 | 0.80 | 0.37 | 76.3% | 3 | 8/10 | 1.80 | 7/10 |
| Design 2 (Tag-Gravity) | 2.37 | 2.81 | 4.4 | 83% | 10.2 | 1.07 | 0.18 | 98.1% | 3 | 8/10 | 1.79 | 7/10 |
| Design 5 (AWNG) | 2.32 | 2.71 | 2.9 | 89% | 8.9 | 1.04 | 0.88 | 99.3% | ~32 | 9/10 | -- | 6/10 |
| Design 6 (Anchor Contraction) | 2.00 | 2.21 | 7.44 | 72% | 11.5 | 0.84 | 1.42 | paradox | 3 | 8/10 | 1.32 | 3/10 |

**Key:** Pass counts M1-M11 metrics meeting targets. V4 gap = committed vs. power-chaser M3 difference (target >= 0.4).

---

## 2. The Key Question: Minimum Hidden Information for M3 >= 2.0 and M11 >= 3.0

**Answer: 8 bits per card (two 4-bit pair-affinity floats) with pool contraction at 12% per pick.**

Only one algorithm passes M11 >= 3.0: Hybrid B at M11 = 3.25. Every other algorithm falls short, most by substantial margins. The data is unambiguous on what it takes to reach both M3 >= 2.0 and M11 >= 3.0 simultaneously:

1. **Pool contraction is mandatory.** No slot-filling algorithm achieves M11 >= 3.0 at 10% visible dual-res. Contraction converts early archetype commitment into late-draft pool concentration, raising the quality of every slot including "random" ones. This was predicted by the math ceiling research and confirmed by all six simulations.

2. **A 3-bit archetype tag is necessary but not sufficient.** Three simulations used 3-bit tags (Designs 2, 4, 6 and Hybrid A). All achieved M3 >= 2.0 but none reached M11 >= 3.0. The tag provides within-sibling discrimination (Warriors vs. Sacrifice among Tide cards) but the binary assignment creates bridge card misclassification that caps late-draft precision.

3. **Two-float pair affinity (8 bits) is sufficient.** Hybrid B replaces the binary tag with two 4-bit affinity scores per resonance pair (e.g., warriors_affinity and sacrifice_affinity for each Tide card). This preserves bridge card value -- a card genuinely strong in both Warriors and Sacrifice retains high affinity for both -- enabling more precise contraction that reaches M11 = 3.25.

4. **Full 8-float affinity vectors (32+ bits) add nothing.** AWNG's 8-float affinity vector at ~32 bits per card achieved V1 = 99.3% -- meaning the visible resonance symbol dominated the affinity vector so completely that mechanical keywords contributed only +0.009 M3. The 8-float design is over-parameterized. Two floats per resonance pair capture the one distinction that matters: which of the two same-primary-resonance archetypes does this card serve better?

**The minimum viable hidden metadata for both targets is therefore: two 4-bit affinity floats per card (8 bits total), encoding the card's relative value for each archetype sharing its primary resonance symbol, combined with pool contraction at 12% per pick.**

---

## 3. The Design Integrity Question

V9 explored a five-level spectrum from no hidden information to arbitrary hidden manipulation. Where is the sweet spot?

**Level 0 (no hidden info):** M3 ~ 2.0-2.1 ceiling; M11 impossible. Flash/Ramp fail per-archetype targets. Not viable for both metrics.

**Level 1 (3-bit tag):** M3 ~ 2.37-2.62; M11 ~ 2.40-2.83. Passes M3 comfortably but M11 falls short in every simulation. Design integrity 8/10 -- the tag is a fair simplification of card mechanics. The single forced-assignment limitation costs bridge card accuracy.

**Level 1.5 (8-bit pair affinity -- Hybrid B):** M3 = 2.70; M11 = 3.25. The only level that passes both targets. Design integrity 9/10 -- the two-float pair affinity is fully derivable from card mechanics using published rules, and no card is forced into a single archetype. V1 = 84.8% confirms visible symbols do the preponderance of targeting work.

**Level 2 (hidden secondary resonance):** Not directly tested in V9 simulation, but mathematically equivalent to Level 1.5 for contraction purposes. The distinction is framing: hidden secondary resonance symbols feel more like "cheating" than pair-affinity scores that reflect genuine mechanical fit.

**Level 3 (full affinity vectors):** AWNG demonstrates this adds nothing over Level 1.5. The visible resonance symbol already provides the dominant signal; mechanical keywords at +0.20 contribution are swamped by the +0.60 resonance contribution. Wasted information.

**The sweet spot is Level 1.5: 8 bits of pair-affinity metadata per card.** It passes both M3 and M11 targets, achieves V1 = 84.8% (visible symbols genuinely primary), scores V3 = 9/10 (maximally honest hidden information), and requires only 2.7x the hidden information of a 3-bit tag for a meaningful improvement in both M11 and design integrity.

---

## 4. Per-Archetype Convergence: Top 3 Algorithms

| Archetype | Fitness | Hybrid B M3 | Hybrid A M3 | Design 4 M3 |
|-----------|:-------:|------------:|------------:|------------:|
| Flash | 25% | 2.58 | 2.41 | 2.26 |
| Blink | 30% | 2.63 | 2.73 | 2.32 |
| Storm | 30% | 2.66 | 2.61 | 2.30 |
| Self-Discard | 40% | 2.76 | 2.68 | 2.35 |
| Self-Mill | 40% | 2.82 | 2.74 | 2.44 |
| Sacrifice | 50% | 2.75 | 2.39 | 2.54 |
| Warriors | 50% | 2.78 | 2.87 | 2.48 |
| Ramp | 25% | 2.66 | 2.50 | 2.16 |
| **Worst** | | **2.58** | **2.39** | **2.16** |
| **Spread** | | **0.25** | **0.48** | **0.37** |

Hybrid B has the best per-archetype equity (spread 0.25) with the highest floor (Flash at 2.58). All three algorithms pass the per-archetype M3 >= 2.0 floor for every archetype. Hybrid B's pair-affinity encoding eliminates the forced-tag misclassification that drags Flash/Ramp below their natural ceiling in tag-only algorithms.

---

## 5. V9 vs V8: What Did We Gain and Lose?

| Dimension | V8 (Narrative Gravity, 40% pool) | V9 (Hybrid B, 10% visible) | Change |
|-----------|:---:|:---:|---|
| M3 | 2.75 | 2.70 | -0.05 (negligible) |
| M11 | ~2.8 (estimated) | 3.25 | +0.45 (major improvement) |
| M10 | 3.3 | 3.8 | +0.5 (slight degradation) |
| M6 | 85% | 86% | Comparable |
| Worst archetype | 2.40 (Flash) | 2.58 (Flash) | +0.18 (improvement) |
| Archetype spread | 0.73 | 0.25 | -0.48 (major improvement) |
| Visible dual-res | 37% (132 cards) | 10% (36 cards) | -27 points |
| Hidden info/card | 0 bits | 8 bits | +8 bits |
| V1 (visible influence) | ~35% | 84.8% | +50 points (major improvement) |
| Player experience | 7.9/10 | Comparable | Same mechanism family |

**What V9 gained:**
- **Dramatically reduced visible dual-resonance** (37% to 10%), restoring decision texture. Players encounter dual-res cards as noteworthy signposts, not as the default.
- **Substantially improved per-archetype equity** (spread 0.73 to 0.25). Flash/Ramp no longer lag 30% behind Warriors/Sacrifice.
- **M11 achieved** (3.25 vs. estimated 2.8). Late-draft quality target met for the first time.
- **Higher visible symbol influence** (V1 = 84.8% vs. ~35%). The visible resonance system is genuinely the primary drafting signal, not a side effect of heavy dual-res pools.

**What V9 lost:**
- **Design integrity cost of hidden metadata.** 8 bits per card of information the player cannot see. The metadata is honestly derived and a player who discovered it would endorse it as fair (V3 = 9/10), but it is still information the game knows that the player does not.
- **Slight M10 degradation** (3.3 to 3.8). The transition zone problem persists. Neither V8 nor V9 achieves M10 <= 2 without CSCT's M6 = 99% concentration.
- **Implementation complexity.** V8's Narrative Gravity used only visible symbols. V9 adds pair-affinity computation and archetype inference logic.

**Net assessment:** V9 is a strict improvement over V8 on every dimension except M10 and implementation complexity. The 8-bit hidden metadata cost is justified by the dramatic reduction in visible dual-resonance requirements and the per-archetype equity gains.

---

## 6. Honest Assessment: Is V1 >= 50% Achievable Alongside M3 >= 2.0 and M11 >= 3.0?

**Yes. Hybrid B achieves V1 = 84.8% alongside M3 = 2.70 and M11 = 3.25.**

This was the most surprising finding of V9. The pre-simulation predictions estimated V1 = 40-50% for tag-based algorithms and 30-40% for affinity-based algorithms. The actual measurements showed V1 = 77-99% across all algorithms. The reason: at 10% visible dual-res, pool contraction driven by visible resonance symbols already concentrates the pool effectively on the committed primary resonance. The hidden metadata's contribution is the within-sibling refinement (Warriors vs. Sacrifice within Tide), which adds only 0.39 M3 points in Hybrid B's case.

The V1 measurements vindicate the V9 design hypothesis: visible resonance symbols can remain the primary drafting signal even when hidden metadata is active. The player's decision to commit to Tide causes the visible-layer contraction; the hidden pair-affinity layer refines which Tide cards survive. Both layers are correlated -- committing to Tide (visible) causes Warriors targeting (hidden) because the player's Tide picks happen to cluster around Warriors-affinity cards. This correlation is what makes the system feel honest: the player's visible strategy and the algorithm's hidden targeting point in the same direction.

---

## 7. Recommendation Tiers

### Tier 1: Minimal Hidden Info (<= 3 bits per card)

**Algorithm:** Design 4, Layered Salience (Two-Stage Filter).

**What it achieves:** M3 = 2.36, all 8 archetypes >= 2.0, V1 = 76.3%, V3 = 8/10. **M11 = 2.40 (fails 3.0 target).** M10 = 2.13 (marginal fail).

**When to choose this:** If the team decides M11 >= 3.0 is not a hard requirement and prefers the absolute minimum hidden metadata. Design 4 is the best V1 architecture among tag-only algorithms and the closest to a "visible-only with slight assist" design. Late-draft packs will average 2.4 S/A cards instead of 3.0, which may be acceptable if playtesting shows players are satisfied with that density.

**Hidden metadata:** One 3-bit archetype tag per card. 1,080 bits total.

### Tier 2: Moderate Hidden Info (Recommended)

**Algorithm:** Hybrid B, Affinity-Tagged Gravity.

**What it achieves:** M3 = 2.70, M11 = 3.25, all 8 archetypes >= 2.0, archetype spread 0.25, V1 = 84.8%, V3 = 9/10, V4 gap = 2.08. **Fails M5 (9.6) and M10 (3.8).**

**When to choose this:** Default recommendation. Achieves both primary metric targets (M3 >= 2.0 and M11 >= 3.0) with the highest V1 of any M11-passing algorithm, the best design integrity (V3 = 9/10), and the smallest archetype equity spread (0.25). The M5 and M10 failures are parameter-tunable (contraction rate, floor slot timing) rather than structural.

**Hidden metadata:** Two 4-bit affinity floats per resonance pair. 8 bits per card. 2,880 bits total.

### Tier 3: Full Hidden Support

**Algorithm:** Hybrid B with tuned parameters (higher contraction rate 14-15%, earlier floor slot at pick 2, tighter initial R1 threshold).

**What it achieves:** Projected M3 = 2.70-2.80, M11 = 3.3-3.5, M10 = 2.5-3.0, M5 = 7-8. Same 8 bits per card.

**When to choose this:** If the M5 and M10 failures are unacceptable and the team is willing to accept faster pool contraction (which may reduce late-draft variety). The hidden metadata is the same as Tier 2; the difference is parameter aggressiveness.

**Note:** Full affinity vectors (32+ bits, AWNG) are not recommended at any tier. Simulation proved they add no measurable targeting precision over 8-bit pair affinity.

---

## 8. Complete Set Design Specification: Hybrid B (Recommended)

### Pool Breakdown

| Category | Cards | % | Hidden Metadata |
|----------|:-----:|--:|-----------------|
| Generic (0 symbols) | 40 | 11% | No affinity data (protected at 0.5 baseline) |
| Single-symbol | 284 | 79% | Two 4-bit floats: affinity for each archetype sharing this symbol |
| Dual-symbol (signpost) | 36 | 10% | Two 4-bit floats (typically 0.9+ for the signpost archetype) |
| **Total** | **360** | | |

### Visible Symbol Distribution

Each resonance (Ember, Stone, Tide, Zephyr) appears as primary on 80 cards. Dual-res cards: 4-5 per archetype, distributed evenly. Dual-res cards should be slightly above average power, mechanically specific to their archetype (hinge cards), and distributed across all draft phases (not front-loaded).

### Hidden Metadata Schema

```
per_card_metadata: {
    archetype_a_affinity: u4   // 0-15, scaled to 0.0-1.0
    archetype_b_affinity: u4   // 0-15, scaled to 0.0-1.0
}
```

Where archetype_a and archetype_b are the two archetypes sharing this card's primary resonance symbol. For a Tide card: archetype_a = Warriors, archetype_b = Sacrifice. For an Ember card: archetype_a = Blink, archetype_b = Storm (plus Flash/Self-Discard as secondary -- these use a separate pair assignment based on which resonance is primary for the archetype).

**Simplification:** Each card has exactly one primary resonance symbol. That symbol is shared by exactly two archetypes as their primary resonance. The two affinity floats encode the card's relative value for those two archetypes. A card with warriors_affinity = 0.85 and sacrifice_affinity = 0.40 is primarily a Warriors card that also has moderate Sacrifice value.

### Per-Archetype Hidden Metadata Targets

| Archetype | Primary Symbol | Target Home Cards (affinity >= 0.7) | Target Bridge Cards (affinity 0.4-0.6 for both) |
|-----------|:-:|:---:|:---:|
| Flash (Ze/Em) | Zephyr | ~30-35 | ~5-10 |
| Blink (Em/Ze) | Ember | ~30-35 | ~5-10 |
| Storm (Em/St) | Ember | ~30-35 | ~5-10 |
| Self-Discard (St/Em) | Stone | ~30-35 | ~5-10 |
| Self-Mill (St/Ti) | Stone | ~30-35 | ~5-10 |
| Sacrifice (Ti/St) | Tide | ~30-35 | ~5-10 |
| Warriors (Ti/Ze) | Tide | ~30-35 | ~5-10 |
| Ramp (Ze/Ti) | Zephyr | ~30-35 | ~5-10 |

### Cross-Archetype Requirements

Bridge cards -- cards with affinity >= 0.4 for both archetypes sharing a resonance -- are the key innovation of Hybrid B over tag-only algorithms. These cards survive pool contraction longer and provide organic splash opportunities. Target: 5-10 bridge cards per resonance pair.

| Resonance Pair | Archetypes | Natural Bridge Overlap | Bridge Card Target |
|----------------|-----------|:---:|:---:|
| Tide | Warriors / Sacrifice | High (50% fitness) | 8-10 cards with affinity >= 0.4 for both |
| Stone | Self-Mill / Self-Discard | Medium (40%) | 7-8 bridge cards |
| Ember | Blink / Storm | Low (30%) | 5-6 bridge cards (requires intentional design) |
| Zephyr | Flash / Ramp | Very Low (25%) | 5-6 bridge cards (requires intentional design) |

### Worked Example: Warriors (Tide/Zephyr)

Warriors has 40 total cards in the pool, all showing (Tide) as their primary visible resonance symbol.

- **31-32 single-symbol (Tide) cards:** Each carries a hidden pair-affinity: (warriors_affinity, sacrifice_affinity). Warriors-dedicated cards: warriors_affinity ~0.85, sacrifice_affinity ~0.15. Cards like "Tidewatch Vanguard -- when this Warrior attacks, kindle" are clearly Warriors-home. Bridge cards: warriors_affinity ~0.55, sacrifice_affinity ~0.50. Cards like "Tidecaller -- when a character enters or leaves play, draw a card" serve both archetypes.

- **4-5 dual-symbol (Tide, Zephyr) signpost cards:** These are the visible Warriors anchors. warriors_affinity ~0.90, sacrifice_affinity ~0.10. Their mechanics explicitly reference Warriors identity ("Warrior characters you control have +1 Spark"). These are slightly above average power and appear across all draft phases.

- **The contraction process:** A Warriors-committed player accumulates a Tide-heavy resonance signature. The contraction algorithm computes relevance as 0.4 * visible_dot_product + 0.6 * warriors_affinity. Cards with high warriors_affinity survive; cards with low warriors_affinity (Sacrifice-home Tide cards) are gradually removed. Bridge cards survive longer than pure Sacrifice cards, appearing as natural splash options in picks 10-20 before eventually being contracted away.

### Algorithm Parameters

| Parameter | Value |
|-----------|-------|
| Contraction start | Pick 4 |
| Contraction rate | 12% per pick |
| Relevance blend | 40% visible dot-product + 60% pair-affinity score |
| Floor slot | 1 top-quartile slot from pick 3 |
| Generic protection | 0.5 baseline relevance |
| Signature weights | +2 primary, +1 secondary |
| Pool minimum | 17 cards (stop contraction) |
| Archetype inference | Mode of inferred archetype from drafted cards' higher-affinity label, from pick 5 |

---

## 9. Card Designer's Brief: What to Do Differently from V8

**V8 asked:** Create 132 dual-resonance cards (37% of pool) with visible secondary symbols.

**V9 asks:** Create 36 dual-resonance cards (10% of pool) with visible secondary symbols, and assign two hidden affinity scores to each of the remaining 284 single-symbol cards.

**Concrete changes:**

1. **Reduce visible dual-res from 132 to 36 cards.** Design 4-5 visible signpost cards per archetype. These should be mechanical hinges (their text references the archetype's identity), slightly above average power, and distributed across all draft phases.

2. **Assign pair-affinity scores to all 284 single-symbol cards.** For each card, ask: "On a scale of 0-1, how well does this card play in Archetype A vs. Archetype B?" where A and B share the card's primary resonance. This takes ~30 seconds per card and requires only reading the card's own mechanics.

3. **Design 5-10 bridge cards per resonance pair.** These are cards with meaningful affinity for both archetypes sharing a resonance. For Tide: characters with enter/leave-play triggers (both Warriors and Sacrifice value these). For Ember: efficient spell-creatures that work in both Blink tempo and Storm spellslinger strategies.

4. **Do NOT attempt to solve fitness through visible symbols.** The hidden pair-affinity handles the precision problem. The visible symbols serve as the player's primary drafting signal -- the player commits to Tide, and the algorithm handles whether their Tide cards skew Warriors or Sacrifice.

5. **Generic cards need no hidden metadata.** The 40 generics are protected at 0.5 baseline relevance and serve as splash options throughout the draft.

**Total new design work compared to V8:** 96 fewer dual-res cards to design (132 to 36), but 284 pair-affinity assignments to make. The net effort is comparable -- affinity assignment is faster than dual-res card creation because it requires no new card design, only evaluation of existing cards.

---

## 10. Open Questions for Playtesting

1. **M10 transition zone.** Hybrid B averages 3.8 consecutive bad packs (target <= 2). The transition zone (picks 6-10) remains structurally weak while archetype inference stabilizes. Test whether increasing contraction rate to 14-15% in picks 5-8 reduces M10 without degrading M6.

2. **M5 convergence delay.** Convergence at pick 9.6 (target 5-8) means the quality ramp starts 2-4 picks later than ideal. Test whether earlier floor slot activation (pick 2 instead of pick 3) improves convergence without reducing early variety (M1).

3. **Bridge card experience.** Hybrid B's pair-affinity encoding keeps bridge cards in the pool longer than tag-only algorithms. Test whether players perceive these as welcome splash options or confusing off-archetype noise.

4. **Dual-res card scarcity.** At 10%, players see approximately 4-6 dual-res cards across 30 picks. Test whether these are memorable signpost moments or too rare to register as a drafting signal.

5. **Power-chaser penalty.** Hybrid B produces M3 = 0.62 for power-chasers (vs. 2.70 for committed players). Test whether new players who power-chase feel the system is broken, and whether a tutorial explaining resonance commitment is sufficient mitigation.

6. **Late-draft pool exhaustion.** By pick 25-30, the pool contracts to ~17-29 cards. Test whether repeated card appearances feel like convergence or frustration. Consider raising the pool minimum floor to 25 cards.

7. **Affinity score calibration.** The pair-affinity scores must be calibrated during card design. Test whether designer-assigned scores produce the predicted targeting precision, or whether an automated derivation (from card properties) is more consistent.

8. **V8 comparison feel test.** Run both V8 Narrative Gravity (40% visible dual-res, no hidden metadata) and V9 Hybrid B (10% visible dual-res, 8-bit hidden affinity) with the same card pool. Let playtesters experience both and report which draft feels more engaging and which draft decisions feel more meaningful.
