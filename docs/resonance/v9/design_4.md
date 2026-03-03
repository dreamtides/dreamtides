# V9 Algorithm Design: Layered Salience

**Agent 4 — Round 2**

---

## 1. Key Findings

- **The visible layer is structurally limited but not useless.** R1 filtering alone at 10% visible dual-res achieves M3 ≈ 2.05 with Flash/Ramp at the exact threshold, M10 = 6-8, and M11 well below 3.0. It cannot be the sole targeting mechanism — but it can do roughly half the work, keeping V1 at 50-60%.

- **The hidden layer needs only 3 bits to close the gap.** A hidden archetype tag (3 bits per card, assigning each card to its best-fit archetype) multiplies the pair-matchable subpool from ~4-5 visible dual-res cards per archetype to the full 40 home-archetype cards. This is the minimum unit of hidden metadata that meaningfully changes outcomes.

- **The layering structure directly solves V1.** When the visible layer runs first (R1 filtering) and the hidden layer runs second (within-pool bias), visible symbols are the primary gate. A player who completely ignores resonance symbols will see the same degradation as visible-only algorithms: ~M3 = 2.05. A player who commits on visible signals will get M3 ≈ 2.4-2.5. The hidden layer refines but cannot substitute for visible commitment.

- **V8's M10 failure (streaks 6-8) is structural under R1-only.** The transition zone (picks 6-10) produces bad streaks because R1 filtering has ~58-62% precision and the committed archetype has no separate high-confidence subpool to draw from. The hidden archetype tag provides exactly this: a high-confidence subpool (P = 1.0 for home cards) that can guarantee 1 slot even on bad-sibling-draw packs.

- **Layered Salience naturally avoids pair alignment catastrophe.** The visible layer never locks to a specific pair. It only sees "primary resonance = Tide." The hidden layer, using archetype tags, detects Warriors vs. Sacrifice from which cards the player has been picking — but because tags are used for soft bias (not hard locking), misidentification degrades gracefully rather than catastrophically.

- **Design integrity is high.** A hidden archetype tag derived from card mechanics ("this Tide card cares about creatures entering play — it is tagged Warriors") passes the honesty test. A player who discovered the system would agree the tag is accurate. V3 score = 8/10.

- **M11 requires pool contraction; visible symbols alone cannot deliver it.** At picks 15+, the algorithm needs to boost random slots, which requires either pool contraction or 3+ targeted slots (violating M6). The lightest path to M11 >= 3.0 is to use the hidden archetype tag as input to a mild pool contraction applied only in the late draft (picks 12+).

---

## 2. Three Algorithm Proposals

### Proposal A: R1-First Visible Bias

**One-sentence visible description:** "As you commit to a resonance, packs weight toward cards matching that resonance."

**Technical description:** Maintain a resonance signature vector. All 4 pack slots draw from an R1-filtered pool (cards with matching primary resonance). Within each R1-filtered slot, bias toward home-archetype cards using hidden archetype tags: weight home-tagged cards 3x relative to sibling-tagged cards. No slots draw from the full pool after pick 5. The player's inferred archetype is the mode of tagged cards among their drafted cards, resolved continuously.

**Hidden metadata required:** 3-bit archetype tag per card (one of 8 archetypes). ~3 bits × 360 = 1,080 bits total.

**Predicted metrics (Graduated Realistic):**
- M3 ≈ 2.3 | M10 ≈ 4-5 | M11 ≈ 2.2 | M6 ≈ 75%
- Fails M10 and M11. The 3x bias within R1 doesn't fully override sibling contamination. M11 stalls because no late-draft pool enrichment occurs.

---

### Proposal B: Layered Salience (Two-Stage Filter)

**One-sentence visible description:** "As you draft, packs shift toward your resonance and then toward your specific style within it."

**Technical description:**

Stage 1 (visible): R1 filtering on primary resonance. Three pack slots draw from the R1-committed resonance pool; one slot draws from the full pool (splash window). This is pure visible targeting — same as V7 SF+Bias. Stage 1 alone reaches M3 ≈ 2.05.

Stage 2 (hidden): Within each R1-filtered slot, apply archetype tag bias. After the player has drafted 5+ cards, infer the probable archetype from the mode of hidden tags on drafted cards. Promote home-archetype-tagged cards to appear with 4x probability relative to sibling-tagged cards within the R1 pool. This converts the R1 pool's ~58% precision to approximately ~80% precision.

Late-draft contraction (picks 12+): Apply mild pool contraction (8% per pick) using archetype tag relevance score (home-tag = 1.0, sibling-tag = 0.5, off-resonance = 0.1) to concentrate the random splash slot and boost M11.

**Hidden metadata required:** 3-bit archetype tag per card. Same 1,080 bits total.

**Predicted metrics (Graduated Realistic):**
- M3 ≈ 2.45 | M10 ≈ 2-3 | M11 ≈ 3.0-3.1 | M6 ≈ 80-85%
- All 8 archetypes ≥ 2.0 (worst: Flash ≈ 2.25). M10 likely passes. M11 borderline. Full metrics in deep-dive below.

---

### Proposal C: Salience + Affinity Blend

**One-sentence visible description:** "As you commit to a resonance, future packs emphasize cards that synergize with your draft."

**Technical description:** Same two-stage structure as Proposal B, but Stage 2 uses per-archetype affinity scores (4 floats per card, covering the two home-archetype candidates for the player's committed resonance) rather than a binary tag. The affinity score replaces the 4x binary promotion: cards are drawn with probability proportional to their affinity score for the inferred archetype. This supports smoother targeting of cross-archetype multi-use cards.

**Hidden metadata required:** 4 floats per card (primary and secondary resonance's two archetypes: a Tide card gets scores for Warriors and Sacrifice). ~128 bits per card for 4×32-bit floats, or ~32 bits at 8-bit quantization. Total: ~11,520 bits at 32-bit.

**Predicted metrics (Graduated Realistic):**
- M3 ≈ 2.55 | M10 ≈ 2-3 | M11 ≈ 3.1-3.2 | M6 ≈ 82-87%
- Marginal M3 gain over Proposal B (~0.1). Substantially more complex hidden metadata. Information cost is ~10× higher for ~4% M3 improvement. The honesty gain (affinity reflects real card properties) is real but modest in practice.

---

## 3. Champion Selection

**Champion: Proposal B (Layered Salience, Two-Stage Filter)**

Proposal A fails M10 and M11 structurally — the 3x within-pool bias is insufficient without contraction. Proposal C has meaningfully higher information cost (10×) for a marginal M3 gain (~0.1) over a system that already passes all targets. Proposal B hits the minimum-hidden-information sweet spot:

- 3 bits per card (the absolute minimum meaningful unit of hidden metadata)
- V1 ≈ 55-60% (visible symbols drive the primary gate; hidden tags refine within it)
- M3 ≈ 2.45 (comfortably above 2.0, with headroom on worst archetypes)
- M11 ≈ 3.0-3.1 (late-draft density target met via mild contraction)
- V3 = 8/10 (archetype tags derived from card mechanics, not arbitrary labels)
- V4 = high (the best visible-resonance pick and best hidden-tag pick differ in ~15-20% of picks — mainly when a sibling-tagged card has higher power than a home-tagged card)

---

## 4. Champion Deep-Dive: Layered Salience

### How It Works

The algorithm operates in three phases:

**Phase 1: Open Draft (picks 1-5)**
All 4 slots draw from the full pool. The hidden archetype tag is used to track early signals (building an inference of the player's probable archetype) but is not yet used for slot construction. The player sees a variety of resonance types. Stage 1 begins once the player has drafted 2+ cards with the same primary resonance (typically pick 3-4); from that point, 3 slots shift to the committed resonance's R1 pool and 1 remains random.

**Phase 2: Committed Draft (picks 6-11)**
Stage 1 fully active: 3 slots draw from R1-filtered pool, 1 random (splash window). Stage 2 active: within each R1-filtered slot, home-archetype-tagged cards draw with 4x weight vs. sibling-tagged cards. The inferred archetype is the mode of hidden tags among the player's drafted cards, updated after each pick. If the inferred archetype is "Warriors," the algorithm upweights (Tide-primary, Warriors-tagged) cards in the R1 pool by 4x, depressing (Tide-primary, Sacrifice-tagged) cards to 1x weight. This lifts per-slot precision from ~58% (R1 only) to approximately ~78-82% S/A for the committed archetype.

**Phase 3: Late Draft (picks 12-30)**
Mild pool contraction activates at 8% per pick using archetype tag relevance: home-tagged cards = 1.0, sibling-tagged = 0.5, off-resonance = 0.1, generics = 0.4 (protected). The contraction concentrates the surviving pool heavily toward home-archetype cards by picks 20-25, lifting the random splash slot's implicit quality and driving M11 ≥ 3.0. No new slot structure changes — the player continues seeing 3 R1-filtered + 1 random slots, but by pick 20 the "random" slot is drawing from a pool that is 60-70% home-archetype tagged.

### What the Player Sees vs. What the Algorithm Does

**Player perspective:** "I started drafting Tide cards, and by pick 5 my packs are mostly Tide cards. As I pick more Tide cards, the packs keep improving — by pick 20 almost every card feels like it belongs in my deck. The rare (Tide, Zephyr) cards stand out as clear Warriors signals."

**Algorithm perspective:** R1 filtering gates three slots to the Tide primary pool starting around pick 3-4. From pick 6 onward, within those Tide slots, the algorithm silently depresses Sacrifice-tagged Tide cards relative to Warriors-tagged Tide cards because the player has been picking Warriors-tagged cards. From pick 12, pool contraction using hidden tags removes Sacrifice and Ember cards from even the random slot. By pick 20, the visible experience ("great packs") is produced by a combination of visible R1 filtering (~55% of the quality gain) and hidden tag-weighted contraction (~45%).

**V1 quantification:** Run the algorithm without hidden tags: Stage 1 (R1 filtering) alone yields M3 ≈ 2.05, M3_baseline ≈ 0.5. Full algorithm yields M3 ≈ 2.45. V1 = (2.05 - 0.5) / (2.45 - 0.5) = 1.55 / 1.95 ≈ **79%**. This is intentionally high — the visible layer is doing most of the work. The hidden layer adds a modest but meaningful refinement.

### Example Draft: Warriors (Committed Player Strategy)

- **Pick 1:** Full pool, 4 cards. Player drafts a (Tide) Warriors-tagged card.
- **Pick 3:** 3 slots shift to Tide R1 pool (R1 = 80 cards). Player sees 3 Tide cards + 1 random.
- **Pick 5:** Hidden tag inference: 3 of 4 drafted Tide cards are Warriors-tagged. Inferred archetype = Warriors. Stage 2 activates: within Tide slots, Warriors-tagged cards at 4x weight.
- **Pick 8:** Player sees a rare (Tide, Zephyr) dual-resonance card — a visible Warriors signpost. Taking it strongly reinforces Warriors tag inference. Pack feels focused.
- **Pick 12:** Pool contraction begins. Sacrifice-tagged Tide cards start being slowly removed. The random slot increasingly draws from the concentrated pool.
- **Pick 20:** Pool has contracted to ~100 cards (40 Warriors + some Sacrifice + some generics). Every pack feels like Warriors deck material. The player perceives this as "I built well."
- **Pick 28:** Pool is ~40-50 cards, almost entirely Warriors. M11 = 3.1 — the late draft delivers 3+ S/A cards per pack.

### Failure Modes

**Archetype misidentification (pick 5-8):** If the player's first 5 Tide cards happen to be 3 Sacrifice-tagged and 2 Warriors-tagged, the inferred archetype = Sacrifice. The Stage 2 weights will upweight Sacrifice cards. If the player later pivots toward Warriors (picks a Warriors dual-resonance signpost), the inference must update. Because inference uses the running mode of all drafted cards (not a lock), and because the 4x weight is a soft bias (not a hard filter), misidentification produces 1-2 suboptimal packs while correcting — not catastrophic failure.

**Sibling contamination in the transition zone (picks 6-10):** R1 filtering at 10% visible dual-res means 80 Tide-primary cards, of which ~40 are Warriors-tagged and ~40 are Sacrifice-tagged (in the pool's home-archetype structure). Stage 2 reduces but does not eliminate Sacrifice draws. At 4x weighting, effective pack precision = 4*P(Warriors) / (4*P(Warriors) + 1*P(Sacrifice)) ≈ 80%. Occasional packs with 2 Sacrifice cards and 2 Warriors cards remain possible (M10 ≈ 2-3 from these).

**Power-chaser invisibility:** A player who ignores visible resonance entirely will see M3 ≈ 1.8-2.0 — the hidden layer cannot target without a visible signal to bootstrap from, because Stage 1 never activates. This is the correct behavior: the system punishes ignoring visible resonance.

**Late-draft contraction + small pool repetition:** By picks 25-30, the surviving pool may contain only 20-30 cards. Repeated card appearances are possible. Mitigation: minimum pool floor of 25 cards (stop contraction once pool falls below this threshold).

### V1-V4 Metrics

| Metric | Value | Assessment |
|--------|:-----:|------------|
| V1: Visible symbol influence | ~79% | Stage 1 (R1) alone delivers 79% of the total M3 gain over random baseline. Exceeds the 60% target. |
| V2: Hidden info quantity | 3 bits/card (1,080 bits total) | Minimum viable hidden metadata. Each card needs only its archetype label (1 of 8). |
| V3: Reverse-engineering defensibility | 8/10 | Tags reflect real card mechanics. A Tide card tagged Warriors because it cares about combat is a fair assessment any player would endorse. Minor loss for simplification (a card that's B-tier in both Warriors and Sacrifice gets one tag). |
| V4: Visible resonance salience | High | In ~80-85% of picks, the "best visible pick" and "best hidden pick" agree. They diverge only when a sibling-tagged card has higher power than a home-tagged card within the same R1 pool — a genuine tension the player can navigate using mechanics. |

---

## 5. Pool Specification

### Visible Symbol Distribution

| Symbol Count | Cards | % | Notes |
|:---:|:---:|:---:|---|
| 0 (generic) | 40 | 11.1% | No visible resonance; protected in contraction |
| 1 visible symbol | 284 | 78.9% | Shows primary resonance only |
| 2 visible symbols | 36 | 10.0% | Rare dual-resonance signposts; ~4-5 per archetype |

This matches the V9 baseline exactly. No visible pool change from the V9 baseline is required.

### Visible Dual-Resonance Signpost Distribution

4-5 cards per archetype. These cards have visible (Primary, Secondary) pair symbols and are the player's clearest archetype commitment signals. They should be designed per Research Agent C's guidance: definitively good (slightly above average power), mechanically specific to the archetype, and distributed across all three draft phases (not front-loaded to picks 1-10 only).

### Hidden Metadata Schema

Each card carries a single hidden field:

```
archetype_tag: u8  // 0-7, one of 8 archetypes
```

3 bits per card. 360 cards × 3 bits = 1,080 bits (135 bytes) of hidden metadata for the entire pool.

**Assignment rules:**

1. Every card is tagged with its single best-fit archetype — the archetype in which it is most reliably S/A tier.
2. For cards that are S/A tier in multiple archetypes, tag with the archetype whose tag appears *least* in the current pool to maintain balance (compensation).
3. Generics receive a special tag value (tag = 255 or a sentinel) indicating "archetype-neutral." They are not used in Stage 2 inference and receive a 0.4 relevance floor in contraction.
4. The 36 visible dual-resonance signpost cards are tagged to their specific archetype (not just their primary resonance). A (Tide, Zephyr) card is tagged Warriors, not a generic Tide tag.

**Per-archetype tag counts (target):**

| Archetype | Primary Symbol | Home-Tagged Cards | Sibling-Tagged Cards |
|-----------|:---:|:---:|:---:|
| Flash (Ze/Em) | Zephyr | ~35 | ~0 (among non-Flash Ze cards) |
| Blink (Em/Ze) | Ember | ~35 | ~0 |
| Storm (Em/St) | Ember | ~35 | ~0 |
| Self-Discard (St/Em) | Stone | ~35 | ~0 |
| Self-Mill (St/Ti) | Stone | ~35 | ~0 |
| Sacrifice (Ti/St) | Tide | ~35 | ~0 |
| Warriors (Ti/Ze) | Tide | ~35 | ~0 |
| Ramp (Ze/Ti) | Zephyr | ~35 | ~0 |

Within each primary resonance pool (80 cards), approximately 35-40 cards carry each archetype's tag. For Tide: ~35-40 Warriors-tagged + ~35-40 Sacrifice-tagged + scattered tagged for off-primary archetypes + 5 generics. The 4x Stage 2 weighting operates within this 80-card R1 pool.

**What the card designer does:** Tag each non-generic card with the archetype it plays best in. The tag is a single label (Warriors, Sacrifice, etc.) — not a spectrum. For borderline cases (a card that's A-tier in both Warriors and Sacrifice), assign the tag to the archetype that shares fewer natural overlaps (to equalize tag distribution per archetype). No new cards need to be created. No visible symbol changes are required. Total design work: ~320 single-tag assignments.

---

## Post-Critique Revision

### Summary of Critic Feedback

The critic ranked Layered Salience #1 overall and identified it as the strongest V1 architecture among all six proposals. The critique raised two genuine weaknesses and proposed one significant modification (Hybrid A):

**Weakness 1:** M3 ≈ 2.45 is the lowest among proposals that achieve M11 >= 3.0, with meaningful headroom to Design 2 at 2.55-2.70. The critic specifically flagged Flash/Ramp worst-case under 4x weighting as unconfirmed.

**Weakness 2:** M10 = 2-3 is borderline, and the late-contraction-only mechanism (picks 12+) means M10 improvement relies entirely on R1 slot + 4x weighting in the transition zone.

**Hybrid A proposal:** The critic proposed combining Design 4's layered visible-first architecture with Design 6's anchor-scaled contraction (differentiated rates by pick type: 6%/10%/18%) and moving contraction start from pick 12 to pick 5.

### Accepted Criticisms

**Flash/Ramp M3 confirmation is a real gap.** The critic's calculation is valid: at 4x home-tag weighting with 80-card R1 pools split 40/40, effective per-slot precision ≈ 80%, yielding M3 = 3*0.80 + 1*0.125 = 2.525. Under Graduated Realistic fitness (F=25%), this should clear 2.0 comfortably — but the gap between 2.525 and 2.0 is not wide enough to dismiss without simulation confirmation. I accept this as a genuine open question that requires direct measurement.

**M10 borderline is real.** The R1 slot plus 4x weighting reduces sibling contamination from ~42% to ~20% per slot, but the transition zone (picks 6-10) remains structurally weak because inference quality is lowest when the player has drafted only 5-8 cards. This is an acknowledged structural limit of the design, not a flaw that can be papered over.

### Defended Points

**M3 ≈ 2.45 is the right tradeoff, not a weakness to fix.** The 0.1-0.25 M3 gap below competing proposals is the cost of V1 ≈ 79%. Designs that reach M3 ≈ 2.55-2.70 do so by front-loading hidden tag influence earlier or more aggressively — which is why their V1 scores are lower. Layered Salience is explicitly optimizing for V1 integrity at the cost of some M3 headroom. The target is M3 >= 2.0; reaching 2.45 provides a 22.5% margin above target. This is not a failure to fix.

**Contraction starting at pick 12 rather than pick 5 is intentional.** Early contraction (pick 5) compresses the splash window before the player has had time to observe variety and make a meaningful resonance commitment. The open draft phase (picks 1-11) exists precisely to let players build their resonance picture from visible signals before the pool narrows. Moving contraction to pick 5 sacrifices draft openness for marginal M11 gains. The current design reaches M11 ≈ 3.0-3.1 — at target — without early contraction.

### Modified Champion

**I adopt Hybrid A as a simulation test but do not modify the core champion.**

The Hybrid A proposal (Design 4 layered architecture + Design 6 anchor-scaled contraction at 6%/10%/18% rates, starting pick 5) is a legitimate candidate worth simulating. If anchor-scaled contraction starting at pick 5 improves M11 to 3.2+ and M10 to <= 2 without degrading V1 below 70%, the hybrid is an upgrade. I cannot determine this without simulation.

However, the champion as specified remains Proposal B (Layered Salience) because:

1. Hybrid A is an untested proposal, not a validated improvement. Adopting it as champion before simulation would be premature.
2. The V1 guarantee of the layered architecture is the most valuable single property of this design. Any modification that moves contraction earlier must be tested against V1 degradation before it can be endorsed.
3. The critic explicitly recommends advancing Design 4 unchanged as "the V9 integrity benchmark" — modifying it pre-simulation would destroy its value as a clean measurement baseline.

**Recommended simulation target added:** Simulate Hybrid A (Designs 4 + 6, contraction from pick 5 at anchor-scaled rates) alongside the unmodified Design 4 champion. The delta between them will directly measure the value of anchor-scaled early contraction within the layered visible-first architecture.
