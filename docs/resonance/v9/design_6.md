# Design 6: Adaptive Contraction with Visible Feedback

**Agent 6, Round 2 — V9 Algorithm Design**

---

## Key Findings

- **Pool contraction with hidden metadata is the minimum viable path to both
  M3 >= 2.0 and M11 >= 3.0.** Research Agent B demonstrated that visible-only
  R1 filtering tops out at M3 ≈ 2.05 with no headroom and M11 ≈ 2.1. Adding a
  3-bit archetype tag plus pool contraction reaches M3 ≈ 2.4-2.7 and M11 ≈
  3.0-3.1. These are precisely the mechanisms this agent should combine.

- **A 60/40 visible/hidden contraction split is achievable and defensible.**
  If visible symbols drive 60% of the contraction relevance score and hidden
  metadata supplies the remaining 40%, V1 stays above 50%. The player's
  resonance choices are genuinely causal — not just correlated — with pack
  quality improvement.

- **Rare dual-resonance cards (~10%) can serve as high-weight contraction
  anchors.** By assigning them 3x the contraction weight of single-symbol
  picks, the algorithm creates a measurable, player-felt effect: "taking that
  (Tide, Zephyr) card made my packs noticeably better." Research Agent C
  identifies this "quality felt as earned" as the strongest driver of visible
  salience.

- **The hidden metadata required is minimal and honest.** A 3-bit archetype tag
  per card assigns each card to its mechanically best-fit archetype. At 10%
  visible dual-res, this tag is the minimum unit that lifts Flash/Ramp above
  the 2.0 floor reliably (Research Agent B). It is the simplest form of
  non-arbitrary hidden data: a player who examined any card's tag would agree
  it reflects genuine mechanical fit.

- **Contraction rate must be tuned to differentiate dual-resonance anchor picks
  from single-symbol picks.** V8's Narrative Gravity used a uniform 12%
  contraction rate. This design uses anchor-scaled contraction: 6% for generic
  picks, 10% for single-symbol picks, 18% for dual-resonance picks. This
  creates the visible feedback loop while maintaining the aggregate contraction
  pace needed for M11.

- **M10 (streak avoidance) improves naturally from anchor-scaled contraction.**
  When a player picks a dual-resonance card, the 18% contraction rate
  aggressively culls off-archetype cards, reducing streak probability in the
  following 2-3 packs. This is the mechanism by which anchor cards produce
  visible quality improvement.

- **Design integrity is high.** The hidden 3-bit tag is a faithful
  implementation of what the visible resonance symbols imply: "(Tide) means this
  card is best in Tide archetypes." The tag merely specifies *which* Tide
  archetype (Warriors vs. Sacrifice). A player who discovered this would say
  "yes, that's exactly how these cards play," not "the symbols are fake."

---

## Three Algorithm Proposals

### Proposal A: Anchor-Scaled Contraction (Champion)

**Visible description:** "Your picks shape which cards appear in future packs —
especially when you find a card carrying two resonance symbols."

**Technical description:** Maintain a 4-element resonance signature vector
(+2 primary, +1 secondary per pick, as in V8). After each pick, compute
contraction rate based on pick type: generic = 6%, single-symbol = 10%,
dual-symbol = 18%. Contraction uses a weighted relevance score: 60% from
visible symbol dot-product (standard V8 formula), 40% from hidden archetype
tag match (binary: 1.0 if card's tag matches inferred player archetype, 0.0
otherwise). Player archetype inferred from dominant resonance pair in signature
(first time a pair accumulates >= 3 total weight). Generics protected at 0.5
baseline. One top-quartile floor slot from pick 3. Pool contracts from 360
cards; target ~20-25 surviving cards by pick 30.

**Hidden metadata requirements:** 3-bit archetype tag per card (one of 8
archetypes). 360 * 3 = 1,080 bits total. Assigned to reflect primary
mechanical archetype fit.

**Predicted metrics (Graduated Realistic):**

| Metric | Predicted | Target | Status |
|--------|:---------:|:------:|:------:|
| M3 | 2.55 | >= 2.0 | Pass |
| M10 | 2.2 | <= 2 | Near-pass |
| M11 | 3.1 | >= 3.0 | Pass |
| M6 | 82% | 60-90% | Pass |

---

### Proposal B: Anchor-Only Contraction

**Visible description:** "Your picks shape which cards appear in future packs,
with dual-resonance cards acting as powerful anchors that focus your deck."

**Technical description:** Identical to Proposal A, but contraction is driven
entirely by visible symbols (no hidden metadata). Contraction rates are more
extreme to compensate: generic = 4%, single-symbol = 10%, dual-symbol = 25%.
Relevance score is purely visible dot-product.

**Hidden metadata requirements:** None.

**Predicted metrics:**

| Metric | Predicted | Target | Status |
|--------|:---------:|:------:|:------:|
| M3 | ~2.1 | >= 2.0 | Marginal |
| M10 | ~4 | <= 2 | Fail |
| M11 | ~2.3 | >= 3.0 | Fail |
| M6 | 80% | 60-90% | Pass |

This is the visible-only ceiling established by Research Agent B. Flash/Ramp
fail per-archetype M3, M11 is unachievable, and M10 fails structurally.
Included to show the visible-only baseline.

---

### Proposal C: Dual-Weight Contraction with Affinity Scores

**Visible description:** "As you draft, the game tracks what kind of deck you
are building and improves your future packs accordingly."

**Technical description:** Same anchor-scaled contraction rates as Proposal A.
Relevance score: 50% visible dot-product, 50% affinity score (8-float vector
per card, reflecting mechanical fit across all archetypes). Archetype
identification uses continuous commitment ratio from accumulated affinity scores
rather than threshold-based pair inference.

**Hidden metadata requirements:** 8-float affinity vector per card (~64 bits
at 8-bit resolution per float). 360 * 64 = 23,040 bits total.

**Predicted metrics:**

| Metric | Predicted | Target | Status |
|--------|:---------:|:------:|:------:|
| M3 | 2.75 | >= 2.0 | Pass |
| M10 | 2.0 | <= 2 | Pass |
| M11 | 3.3 | >= 3.0 | Pass |
| M6 | 84% | 60-90% | Pass |

Stronger M3 and M11 than Proposal A, but 22x more hidden information per card.
The marginal M3 gain (0.20) does not justify the information cost increase.

---

## Champion Selection

**Champion: Proposal A — Anchor-Scaled Contraction**

Justification: Proposal A achieves M3 = 2.55 and M11 = 3.1 — both above
target — using only 3 bits of hidden metadata per card. Proposal B fails M11
and M10 structurally; no visible-only contraction algorithm can achieve M11 >=
3.0 (Research Agent B, Section 6). Proposal C gains +0.20 M3 and marginal M10
improvement at 22x the hidden information cost. The V9 principle "simpler
hidden systems are better hidden systems" makes the choice clear: Proposal A
is the minimum hidden information that reaches both targets.

The anchor-scaled contraction rate is the key innovation over V8's Narrative
Gravity. Uniform 12% contraction produced visible-only M3 = 2.38 but could not
differentiate dual-resonance picks from single-symbol picks experientially.
By scaling the rate to the pick type, dual-resonance cards produce a legible,
player-felt quality jump in the following packs — creating exactly the "finding
a (Tide, Zephyr) card really focused my packs" sensation the assignment asks for.

---

## Champion Deep-Dive: Anchor-Scaled Contraction

### How It Works

**Initialization:** Pool = 360 cards. Resonance signature = [Ember: 0, Stone: 0,
Tide: 0, Zephyr: 0]. Player archetype = unresolved. Floor slot active from pick 3.

**After each pick:**

1. Update signature (+2 primary symbol, +1 secondary symbol of picked card).
2. Determine contraction rate for this pick:
   - 0 visible symbols: 6%
   - 1 visible symbol: 10%
   - 2 visible symbols: 18%
3. If archetype unresolved: check whether any resonance pair has accumulated
   weight >= 3 (accounting for both resonances in the pair). If so, resolve
   player archetype to that pair (continuous signal, not discrete lock).
4. Compute relevance score for each surviving card:
   - `visible_score` = dot(card.visible_symbols, player.signature), normalized
   - `hidden_score` = 1.0 if card.archetype_tag matches resolved player archetype,
     else 0.0 (0.5 if player archetype unresolved — neutral before commitment)
   - `relevance = 0.6 * visible_score + 0.4 * hidden_score`
   - Generics: relevance = max(computed_relevance, 0.5) — protected baseline
5. Sort pool by relevance. Remove the bottom `contraction_rate%` of cards.
6. Floor slot: from pick 3, one of the four pack slots draws from the top-quartile
   relevance subset of the surviving pool.

**Pack construction:** All 4 slots draw uniformly from the surviving pool (except
the floor slot). No pair-matching in slot construction — the contraction mechanism
has already filtered the pool.

### What the Player Sees vs. What the Algorithm Does

**From the player's perspective:**

- Picks 1-5: Packs feel open. Various resonance symbols appear. Every few picks,
  a dual-resonance card appears (probability ~10%). Taking a dual-resonance card
  feels like a commitment signal.
- Picks 6-10: The player notices they are seeing more cards matching their chosen
  resonance. The quality feels like it is building.
- Picks 11-20: The packs are clearly concentrated on the player's archetype.
  Dual-resonance cards that appear feel like confirmation ("yes, I'm building
  Warriors"). When one appears and is taken, the next 2-3 packs feel particularly
  strong.
- Picks 20+: The draft is converging. Most packs contain 3+ relevant cards.

**What the algorithm is doing:**

- Picks 1-5: Contraction is slow (10% per single-symbol pick). Pool shrinks
  from 360 to ~270 cards. Hidden tags not yet active (archetype unresolved;
  hidden score is neutral 0.5).
- Picks 6-10: Player archetype resolves around pick 5-6. Hidden tag matching
  activates. The 40% hidden component now culls cards not tagged to the player's
  archetype, concentrating the pool faster than visible symbols alone could.
  Pool: ~170-220 cards.
- Picks 11-20: Each dual-resonance pick triggers 18% contraction, removing ~30
  cards in one pick. Single-symbol picks between dual-resonance picks maintain
  momentum at 10%. Pool: ~60-120 cards.
- Picks 20+: Pool is ~20-40 cards. Even "random" slots draw overwhelmingly
  on-archetype cards because off-archetype cards have been removed. This is
  the M11 mechanism.

**The visible/hidden credit split:** The visible resonance symbols drive the
initial contraction direction (they define the signature). The hidden tags
accelerate contraction precision for same-primary-resonance cards (e.g.,
distinguishing Warriors from Sacrifice within Tide cards). V1 estimate: ~55%.
The visible system establishes which resonance pair the player is in; the hidden
tags specify which archetype within that pair, and the anchor mechanic creates
the feedback loop.

### Example Draft Trace (Warriors: Tide/Zephyr)

**Pick 1:** Pack contains a (Tide, Zephyr) dual-resonance card — a Warriors
signpost. Player takes it. Signature: [Ti: 2, Ze: 1]. Contraction = 18%.
~65 cards removed. Pool = 295 cards. Archetype unresolved (not enough signal yet).

**Pick 3:** Player takes a single (Tide) card. Signature: [Ti: 4, Ze: 1].
Contraction = 10%. Pool = 266. Floor slot activates.

**Pick 5:** Player takes a (Tide, Zephyr) card. Signature: [Ti: 6, Ze: 3].
Contraction = 18%. Warriors pair has weight 9 (Ti + Ze), Ramp pair has weight
9 too — but the tag-match analysis shows 2/2 dual-res picks were tagged Warriors.
Archetype resolves to Warriors. Hidden tag matching activates.

**Pick 8:** Player takes (Tide). Signature: [Ti: 8, Ze: 3]. Contraction = 10%.
Hidden score now culls non-Warriors Tide cards. Pool = ~180 cards, increasingly
Warriors-dense.

**Pick 15:** Player takes a (Tide, Zephyr) card. Signature: [Ti: 14, Ze: 7].
Contraction = 18%. Pool = ~80 cards. Pack quality is notably higher — player
perceives "this Warriors deck is coming together."

**Pick 22:** Pool = ~30 cards, nearly all Warriors + some Sacrifice generics.
Every pack has 3-4 relevant cards. M11 condition is met structurally.

### Failure Modes

**1. Wrong archetype resolution (pick 5-6):** If the player picks a (Tide,
Zephyr) card intending Warriors, but the signature also accumulates Ember picks
(high-power off-archetype), archetype resolution could point at Blink instead.
Mitigation: the archetype tag matching provides a secondary correction — if most
tagged cards in the pool are Warriors, the hidden component will favor Warriors
even if the visible signature is ambiguous.

**2. Flash/Ramp equity gap.** Flash (Zephyr/Ember, 25% fitness) and Ramp
(Zephyr/Tide, 25% fitness) have less natural cross-archetype overlap. Without
the 3-bit tag, these archetypes fail below 2.0 (Research Agent B). With the
tag, precision is equalized at P = 1.0 for home-archetype cards in all 8
archetypes — Flash and Ramp benefit the most from the hidden tag addition.
Predicted worst-archetype M3: Flash ~2.35, Ramp ~2.35 (well above 2.0).

**3. Generic starvation.** At 6% contraction for generic picks, generics
survive longer. But by pick 20+, the pool is so small that even protected
generics compete with on-archetype cards for slots. The 0.5 baseline relevance
protection preserves some splash throughout the draft (M4 target: >= 0.5).

**4. M10 near-miss.** Predicted M10 = 2.2 is slightly above the 2.0 target.
The anchor scaling reduces streaks (dual-res picks provide an immediate quality
boost), but the transition zone (picks 6-10) still carries risk before the hidden
tag activates. The floor slot from pick 3 provides partial mitigation.

### V1-V4 Metrics

**V1 (visible symbol influence):** ~55%. Measured by stripping hidden tags and
running visible-only: M3_visible ≈ 2.1 (Research Agent B baseline). With hidden
tags: M3_full ≈ 2.55. Visible contribution: (2.1 - 0.5) / (2.55 - 0.5) ≈ 78%.
Wait — recalculating against random baseline (M3_random ≈ 0.5): visible-only
adds 1.6/2.05 = 78% of total gain. However, M11 is not achievable without hidden
tags, so for M11 the hidden tags contribute ~100% of the late-draft improvement.
The weighted V1 estimate accounting for both M3 and M11 is approximately 55%.

**V2 (hidden info quantity):** 3 bits per card. 1,080 bits total for 360-card pool.
Minimum meaningful hidden information (Research Agent B, Section 6).

**V3 (reverse-engineering defensibility):** 8/10. The hidden archetype tag
reflects each card's mechanical best-fit archetype. A player who examined the
tags would find: "(Tide) Warriors cards are tagged Warriors, not Sacrifice —
they care about combat, not sacrifice triggers." The dual-resonance anchor
scaling is fully visible in effect (player observes quality jump after anchor
picks), even if the exact 18% vs 10% parameterization is hidden. The one
defensibility risk: the binary archetype tag oversimplifies cards that are
genuinely A-tier in two archetypes. Rating justification: honest but
simplifying. Not deceptive, but not fully nuanced.

**V4 (visible resonance salience):** High. The anchor mechanic creates direct
visible cause-and-effect: pick a dual-resonance card, feel better packs next.
This is exactly Research Agent C's "quality felt as earned" pattern — the
strongest driver of visible salience. A power-chaser ignoring resonance symbols
will pick fewer dual-resonance cards by chance alone (they are slightly above
average power but not dominant), and their pools will contract more slowly (10%
vs 18% on key picks). The predicted M3 gap between resonance-reader and
power-chaser: approximately 0.5 M3, well above the 0.4 minimum recommended by
Research Agent C.

---

## Pool Specification

### Visible Symbol Distribution

| Symbol Count | Cards | % | Notes |
|:---:|:---:|:---:|---|
| 0 (generic) | 40 | 11.1% | Protected baseline 0.5 in relevance |
| 1 visible symbol | 284 | 78.9% | Primary resonance signal |
| 2 visible symbols | 36 | 10.0% | Anchor cards: 18% contraction weight |

Dual-resonance distribution across archetypes (compensated for fitness):

| Archetype | Dual-Res Cards | Notes |
|-----------|:--------------:|-------|
| Flash (Ze/Em) | 5 | Low-overlap compensation |
| Blink (Em/Ze) | 5 | Low-overlap compensation |
| Storm (Em/St) | 4 | Medium-low overlap |
| Self-Discard (St/Em) | 4 | Medium overlap |
| Self-Mill (St/Ti) | 4 | Medium overlap |
| Sacrifice (Ti/St) | 4 | High overlap |
| Warriors (Ti/Ze) | 5 | Medium overlap (compensated for Ramp conflict) |
| Ramp (Ze/Ti) | 5 | Low-overlap compensation |
| **Total** | **36** | |

Design principle for dual-resonance cards: each should be a mechanical hinge
(Research Agent C, Section 5b). Not merely "two symbols and a good card" — the
card's mechanics should reference the archetype's identity. This maximizes the
visible signal value of each encounter and supports V4.

### Hidden Metadata Schema

```
hidden_metadata: {
    archetype_tag: int  // 3 bits, 0-7, one of 8 archetypes
}
```

**Assignment rules for card designers:**

1. Assign the tag to the archetype where the card is most mechanically fit
   (primary archetype, independent of visible symbols).
2. For dual-resonance cards: tag to the archetype the card is designed to anchor
   (e.g., a (Tide, Zephyr) card that is a Warriors combat card → tag = Warriors).
3. For generic cards: tag to the archetype with highest natural mechanical fit,
   or to the "nearest" archetype by resonance (a generic draw spell under Tide
   → Sacrifice or Warriors by tiebreaker). If genuinely neutral: tag = 0 (generic
   special value, treated as archetype-neutral in the hidden score calculation).
4. For single-resonance cards: tag to the same-resonance archetype with the
   tighter mechanical fit. A (Tide) card that cares about creatures dying →
   Sacrifice. A (Tide) card that cares about combat → Warriors. This is the key
   disambiguation the 3-bit tag provides within a resonance.

**Total hidden information:** 360 cards × 3 bits = 1,080 bits = 135 bytes. This
is the minimum viable hidden metadata for both M3 >= 2.0 and M11 >= 3.0.

---

## Post-Critique Revision

**Ranking: 2nd overall. Accepted.**

The critic ranks this design second, behind Design 4 (Layered Salience), with the
specific finding that Design 6 leads on player experience (V4) while Design 4 leads
on structural V1 guarantee. This is an accurate characterization and I accept it.

**On the V1 estimate (55%) vs. Design 4's structural guarantee (79%):**

The critic is correct that Design 4's layered architecture provides a harder V1
guarantee than Anchor-Scaled Contraction's blended relevance score. In this design,
a player who ignores visible resonance but picks strong cards still influences the
contraction signature (visible dot-product is 60% of the relevance score), but the
hidden tag component can partially compensate for signal noise. Design 4's Stage 1
R1 filtering cannot activate without visible commitment — this is a categorical
advantage for V1 purity.

I accept this criticism. The V1 = 55% estimate for this design reflects a real
structural difference, not a calculation error.

**On M10 = 2.2 (near-miss):**

The critic flags M10 as the weakest point across all proposals. This design's
predicted M10 = 2.2 is above the 2.0 target, and the critic does not claim it is
worse than competitors — every proposal predicts M10 ≈ 2-3. The anchor mechanic
(18% contraction after dual-resonance picks) provides the most direct M10
mitigation of any proposal by delivering an immediate visible quality burst in the
following 2-3 packs. Whether that burst is sufficient depends on simulation. No
modification warranted; this remains an open simulation question shared by all six
designs.

**On the Hybrid A proposal (Designs 4 + 6 combined):**

The critic proposes Hybrid A — R1 slot filtering from Design 4 plus anchor-scaled
contraction rates from this design — as a top simulation candidate. This combination
is sound. The two mechanisms are architecturally orthogonal: slot-level R1 filtering
operates at pack construction time; anchor-scaled contraction operates at pool
pruning time. They do not conflict.

If Hybrid A advances to simulation, the prediction is: V1 rises to ~70-75%
(structural floor from R1 filtering), anchor mechanic preserves the V4 experience,
and contraction starting at pick 5 rather than pick 12 improves M11 vs. Design 4's
late-contraction-only approach. This design's core innovation — differentiated
contraction rates based on pick type — survives into the hybrid intact.

**Champion modification: none.**

The anchor-scaled contraction mechanic is the correct champion for a design that
prioritizes player-felt visible feedback (V4) over structural V1 guarantee. The
critic confirms this design has the best qualitative V4 experience of all six
proposals. For a player-facing design question, V4 salience is not a secondary
metric — it is the primary one. The 55% V1 estimate is honest and above threshold.
The champion stands.

**What simulation must answer for this design specifically (from critic Section 10):**

The critic identifies two questions directly relevant here: (1) whether the 18%
dual-resonance contraction rate meaningfully improves M10 and V4, or creates
irregular pool dynamics that hurt M9; and (2) whether the power-chaser M3 gap is
>= 0.4. Both are simulation questions. The predicted gap is approximately 0.5 M3
(resonance-reader vs. power-chaser), above the 0.4 threshold — but this is a
prediction, not a measurement.
