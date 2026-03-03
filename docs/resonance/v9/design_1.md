# V9 Algorithm Design 1: Visible-Only Baseline + Minimal Enhancement

**Agent 1 — Exploring the design space between pure visible and minimally hidden**

---

## Key Findings

- **SF+Bias R1's ceiling is structural, not tunable.** On a 10% visible
  dual-res pool, R1 filtering alone achieves P(S/A) = 0.58 avg across
  archetypes, requiring 4+ targeted slots to reach M3 = 2.0 — leaving zero
  room for splash (M4 fail) or variance (M9 fail). Flash and Ramp at F=25%
  need exactly 4 full slots to hit 2.0; any splash kills them below threshold.
  M10 failures (streak = 6-8) are the direct consequence: R1 filtering cannot
  distinguish a bad streak from a good one mid-pack.

- **A 3-bit archetype tag multiplies targeting precision by ~4x relative to
  R1 filtering.** R1 filtering narrows from 360 to ~80 cards (one resonance),
  with ~58% S/A precision. A hidden archetype tag narrows to exactly 40 home
  cards with P(S/A) ~1.0. The subpool grows from ~4-5 pair-matchable visible
  cards to 40 home-tagged cards — enabling 2 reliable home-targeted slots per
  pack without exhaustion risk.

- **M10 (streak failures) are mechanically separable from M3.** M10 fails in
  SF+Bias R1 because R1 cannot reliably fill even 2 slots with S/A cards in
  transition-zone packs (picks 6-10), when the player has just committed but
  the pool is still large. A hidden tag lets the algorithm guarantee 2 home
  slots regardless of pool state — this is a precision fix, not a M3 boost.

- **M11 >= 3.0 is impossible without pool contraction.** At picks 15+, a
  slot-filling algorithm with archetype tags achieves M3_late ≈ 2 * 1.0 + 2 *
  0.125 = 2.25 — same as earlier picks, no improvement. Pool contraction using
  hidden tags is the only mechanism that makes random slots improve over time,
  converting contraction pressure into late-draft density.

- **V1 >= 50% is achievable with a 3-bit tag at M3 ~2.4-2.5.** Visible symbols
  contribute a fixed ~2.05 M3 baseline; hidden tags lift to ~2.5. V1 estimate:
  2.05/2.50 ≈ 82% of baseline, with delta attribution approximately 50-60%
  visible (the visible signal drives archetype inference picks 1-5, which
  determines which tag set is used). This is the sweet spot the V9 plan targets.

- **The 3-bit tag is more honest than any alternative at equal bit-count.** A
  single archetype assignment per card is a natural, endorsable simplification:
  "this Tide card is a Warriors card, not a Sacrifice card, because its mechanic
  cares about creatures attacking." Any player who looked up the tags would
  likely agree. Design integrity score ~8/10.

- **Combining the tag with SF+Bias (not Narrative Gravity) minimizes hidden
  contribution.** If we apply the tag only as a tiebreaker within an R1-filtered
  pool, the visible symbols do the primary targeting and the tag refines within
  the visible-filtered candidates. This keeps V1 high (~60-65%) while
  meaningfully fixing M10 and lifting weak archetypes.

---

## Three Algorithm Proposals

### Proposal A: SF+Bias R1 (Pure Visible Baseline)

**Visible description:** "Each pack biases toward cards sharing your primary
resonance symbol."

**Technical description:** R1 filtering on all 4 slots (bias weight 3:1 toward
primary resonance). No hidden metadata. Visible dual-res cards scored as pair
matches. Direct port of V8's SF+Bias R1 to 10% visible dual-res pool.

**Hidden metadata:** None (0 bits per card).

**Predicted metrics (Graduated Realistic, 10% visible dual-res):**
- M3: ~2.0-2.1 avg; Flash/Ramp ~1.95-2.05 (at threshold or below)
- M10: 6-8 (fails — structural streak problem from R1 precision ceiling)
- M11: ~2.1 (fails target >= 3.0 — no late-draft improvement mechanism)
- M6: ~75-80% (passes)
- V1: 100% (but fails hard metrics)

---

### Proposal B: Tagged R1 Bias (3-bit Tag as Tiebreaker)

**Visible description:** "Each pack biases toward cards sharing your primary
resonance symbol."

**Technical description:** R1 filtering as the primary targeting mechanism
(same as Proposal A). Within the R1-filtered candidate pool, add a secondary
bias: cards whose hidden archetype tag matches the player's inferred committed
archetype receive an additional weight multiplier (2x). Archetype inference uses
the resonance signature vector from V8 (pick 5+ with dominant resonance = committed
archetype). The player experiences only visible resonance biasing; the tag
operates as an invisible quality filter within the already-resonance-filtered pool.
No pool contraction.

**Hidden metadata:** 3-bit archetype tag per card (one of 8 archetypes). Derived
from mechanical fit. 360 * 3 = 1080 bits total.

**Predicted metrics (Graduated Realistic, 10% visible dual-res):**
- M3: ~2.3-2.4 avg; worst arch (Flash/Ramp) ~2.2-2.3
- M10: ~3-4 (improved from baseline but not target — slot-filling still suffers
  transition zone; tag helps precision but doesn't prevent dry spells from pool
  variance)
- M11: ~2.3 (fails >= 3.0 — no pool contraction)
- M6: ~75-80% (passes)
- V1: ~60-65% (visible does primary filtering; tag refines within filtered set)

---

### Proposal C: Tagged Narrative Gravity (3-bit Tag + Pool Contraction)

**Visible description:** "As you draft, the game removes cards that don't match
your style, so future packs get better."

**Technical description:** V8's Narrative Gravity algorithm, substituting hidden
archetype tags for visible dual-res pair-matching in the relevance score. Maintain
a 4-element resonance signature vector (visible symbols, +2 primary +1 secondary).
From pick 4, compute relevance per card using: (a) standard dot-product of visible
symbols against signature (V8 formula), plus (b) bonus relevance for cards whose
hidden archetype tag matches the player's inferred committed archetype (+0.5 to
relevance score). Remove bottom 12% by combined score per pick. Generics protected
at 0.5 baseline. 1 top-quartile floor slot from pick 3. The visible signal drives
the signature vector and primary relevance; the hidden tag differentiates within-
resonance cards, enabling Sacrifice to be deprioritized for a Warriors-committed
player even though both have Tide symbols.

**Hidden metadata:** 3-bit archetype tag per card. Same 1080 bits total.

**Predicted metrics (Graduated Realistic, 10% visible dual-res):**
- M3: ~2.4-2.6 avg; worst arch (Flash/Ramp) ~2.3-2.5
- M10: ~2-3 (floor slot helps; contraction concentrates pool by transition zone)
- M11: ~2.9-3.2 (pool contraction with tags enables late-draft concentration)
- M6: ~80-85% (passes; contraction raises concentration vs. baseline)
- V1: ~45-55% (visible drives signature vector; hidden tag does within-resonance
  differentiation — roughly equal contributors to M3 improvement over baseline)

---

## Champion Selection: Proposal C (Tagged Narrative Gravity)

**Justification:**

Proposal A fails M10, M11, and per-archetype equity for Flash/Ramp. It is the
correct baseline for V1 measurement but not a viable algorithm.

Proposal B fixes M3 and partially fixes M10 but leaves M11 unreachable. Its
advantage — V1 at ~65% — does not compensate for a fundamental failure on the
late-draft quality target. A system that fails M11 will feel like it plateaus
after pick 10; the player commits but the packs stop improving.

Proposal C is the minimum addition that achieves all primary targets. The
3-bit archetype tag serves two functions simultaneously: (1) within-resonance
differentiation for contraction quality scoring (Warriors vs. Sacrifice among
Tide cards), and (2) a precision boost for the floor slot (draw from top-quartile
home-tagged cards). The hidden metadata requirement — 1080 bits total, one
3-bit label per card — is the minimum discrete unit. The design integrity is
high: the tag is a simplification of mechanical reality, not an arbitrary label.

The key design insight: the visible resonance signature (V8-style, +2/+1) remains
the primary driver of contraction. The tag provides the refinement that visible
symbols cannot: distinguishing which of two Tide archetypes a single-symbol Tide
card belongs to. Without the tag, Warriors-committed players cannot distinguish
Warriors Tide cards from Sacrifice Tide cards during contraction, and the pool
remains mixed with same-resonance noise. The tag solves exactly this problem and
nothing more.

---

## Champion Deep-Dive: Tagged Narrative Gravity

### How It Works

**Algorithm state:** Resonance signature vector `S[4]` (Ember, Stone, Tide,
Zephyr), initialized to zero. Pool of 360 cards with visible symbols and hidden
3-bit archetype tags.

**Each pick (from pick 4):**
1. Compute relevance `R(card)` for every pool card:
   - Base: `dot(card.visible_symbols_weighted, S)` using (+2 primary, +1
     secondary) encoding — identical to V8 Narrative Gravity
   - Tag bonus: if `card.hidden_tag == inferred_committed_archetype`, add +0.5
   - Generic protection: `R(generic) = max(R(generic), 0.5)`
2. Sort pool by `R`. Remove bottom 12% of pool.
3. Generate pack: 1 slot draws from top-quartile of surviving pool (floor slot);
   3 slots draw randomly from full surviving pool.
4. Player picks 1 card. Update `S` by card's visible symbols.
5. After pick 5, infer committed archetype from `S`: archetype whose pair
   (primary + secondary resonance) has highest dot-product with `S`.

**Tag inference (picks 1-4):** Before archetype commitment is inferred, the tag
bonus is inactive. Contraction uses only visible dot-product in the pre-commitment
phase. This preserves early exploration (V9 Goal 10: open-ended early).

### What the Player Sees vs. What the Algorithm Does

| Draft Phase | Player sees | Algorithm does |
|---|---|---|
| Picks 1-5 | Mixed resonance packs. About 4-6 visible dual-res cards in these picks. The resonance they pursue produces more on-resonance cards. | Builds resonance signature from visible picks. Contraction begins (bottom 12% removed per pick). Tag bonus inactive. |
| Picks 6-10 (transition) | Packs start improving for committed resonance. Occasionally a pack feels weak (transition zone). The floor slot ensures at least one strong card. | Tag bonus activates. Combined relevance score starts deprioritizing sibling-resonance cards. Pool has contracted ~25-30%. |
| Picks 11-20 | Consistently 2-3 on-archetype S/A cards per pack. Occasional strong off-archetype card for splash. | Pool has contracted ~50-60%. Tag differentiation has removed most same-resonance sibling cards. All 4 slots draw from heavily concentrated pool. |
| Picks 21-30 (late) | Nearly every pack has 3 on-archetype S/A cards (M11). Dual-res signpost cards may appear as pool concentrates. | Pool contracted to ~40-60 cards. Nearly all surviving cards are home-archetype tagged. Random slots yield ~0.5 S/A probability from contraction alone. |

### Example Draft (Warriors / Tide-Zephyr Committed)

**Picks 1-5:** Player sees packs with Tide cards (~30-35% of pack), Zephyr cards
(~25-30%), some dual-res, some generics. Player drafts 2 Tide single-symbol cards,
1 Tide-Zephyr dual-res signpost, 1 Zephyr single-symbol, 1 generic removal.
Resonance signature: S[Tide] = 5, S[Zephyr] = 3. Contraction has removed ~55 cards
(weakest by visible resonance). Inferred committed archetype after pick 5: Warriors.

**Pick 6 transition:** Tag bonus activates. The 40 Sacrifice-tagged Tide cards
are now deprioritized relative to the 40 Warriors-tagged Tide cards. The pool
contraction accelerates differentiation between them. Pack contains: 1 floor-slot
Warriors-tagged Tide card (S/A tier), 1 Warriors-tagged Tide/Zephyr dual-res, 1
Zephyr single-symbol (may or may not be Warriors-tagged), 1 generic. Player gets
2 S/A for Warriors from this pack.

**Picks 15-20:** Pool has contracted to ~80 cards. Approximately 35 are Warriors-
tagged, 10 are Sacrifice-tagged (still scoring high from Tide primary), 15 generics/
splash. Essentially every pack contains 3 Warriors-relevant S/A cards + 1 generics
or splash. M11 ≈ 3.0-3.2.

### Failure Modes

1. **Early pivot after pick 5:** If a player switches from Tide to Ember after
   pick 5, the resonance signature is already Tide-heavy, and the pool has
   contracted toward Tide. Pivot produces several weak packs (transition) before
   contraction re-orients. This is a moderate failure — late pivots are always
   costly, but the contraction mechanism exacerbates it. Mitigation: contraction
   rate 12% per pick means ~50% of pool survives to pick 10; a genuine pivot at
   pick 6-7 can recover.

2. **Archetype inference error (picks 5-7):** If the player drafted equally
   across Tide and Stone by pick 5, the committed archetype inference may be
   ambiguous. The tag bonus will split between two archetypes, underperforming
   a clean commitment. Resolution: delay tag bonus until signature confidence is
   clear (e.g., dominant resonance > 40% of total signature weight), or use soft
   weighting proportional to inferred confidence.

3. **Power-chaser penalty:** A player who ignores visible resonance and drafts by
   power alone will not build a coherent signature, so contraction will be slow
   and unfocused. Packs will not improve in the Narrative Gravity manner. This is
   the intended behavior — visible resonance choices are genuinely rewarded — but
   new players who power-chase will feel the algorithm is punishing them. Per
   research findings, this requires tutorial framing.

4. **M10 residual:** The floor slot reduces M10 to ~2-3, but transition zone packs
   (picks 6-10) still show some variance before the tag bonus fully differentiates.
   M10 likely lands at 3, marginally failing the target of <= 2. Tuning the floor
   slot to activate more aggressively at pick 4-5 (not pick 3) may reduce this.

### V1-V4 Metrics

| Criterion | Score | Justification |
|---|---|---|
| V1 (Visible symbol influence) | ~50-55% | Visible signature drives contraction base; tag provides +0.5 within-resonance differentiation. Run without tags: M3_visible ≈ 2.1-2.2 (contraction on visible only, like V8 Tier 1). Full: M3 ≈ 2.5. Delta ~0.3-0.4. Visible accounts for ~55% of improvement over random baseline. |
| V2 (Hidden info quantity) | 3 bits/card = 1080 bits total | Minimum possible unit of hidden archetype information. No floats, no vectors, no synergy graphs. |
| V3 (Reverse-engineering defensibility) | 8/10 | A player who discovered the tag system would read: "each card is labeled with its best archetype; Warriors Tide cards are tagged Warriors, not Sacrifice." This is a fair simplification of mechanical reality. Defensible statement: "The game knows each card's primary strategic home and uses that to improve packs as you commit." No player would feel deceived. |
| V4 (Visible resonance salience) | High | In picks 1-5, all decisions are driven by visible symbols (tag inactive). In picks 6-15, the "best visible pick" (highest resonance match) and "best hidden pick" (highest combined relevance) diverge only when a card has the right visible symbol but wrong tag (e.g., a Sacrifice Tide card offered to a Warriors player). Divergence estimated ~15-20% of picks — within the target < 20%. |

---

## Pool Specification

### Visible Symbol Distribution

| Symbol Count | Cards | % | Notes |
|:---:|:---:|:---:|---|
| 0 (generic) | 40 | 11% | No visible resonance. Protected at 0.5 relevance baseline. |
| 1 visible symbol | 284 | 79% | Primary resonance only. The core visible targeting signal. |
| 2 visible symbols | 36 | 10% | Rare dual-resonance signpost cards. ~4-5 per archetype. |

Visible dual-resonance at exactly 10% (36 cards). This is below V8's 15% floor
for visible-only algorithms — acceptable because hidden tags substitute for the
precision that visible dual-res provided.

### Per-Archetype Visible Distribution

| Archetype | Single-Symbol (Primary) | Dual-Res (Visible) | Total |
|---|:---:|:---:|:---:|
| Each of 8 archetypes | ~31-32 | ~4-5 | 40 |
| Generic | 0 | 0 | 40 |
| **Total** | **~252-256** | **~36** | **360** |

Note: visible dual-res cards per archetype should be uniform (4-5 per archetype,
no compensation). The hidden tag provides the compensation that visible pool
compensation provided in V8. Designers do NOT need to create more visible dual-res
cards for Flash/Ramp — the tag handles the precision shortfall.

### Hidden Metadata Schema

**One 3-bit field per card (log₂(8) = 3 bits):**

```
card.hidden_tag ∈ {Flash, Blink, Storm, Self-Discard, Self-Mill, Sacrifice,
                   Warriors, Ramp}
```

**Assignment rules:**
1. Every non-generic card receives exactly one tag.
2. Tag = the archetype for which this card is highest-tier (S/A) relative to
   other archetypes. If a card is equally strong in two archetypes (bridge card),
   assign the tag of the archetype whose R1 pool is less saturated — effectively
   the lower-fitness archetype (Flash, Ramp, Blink, Storm prefer over Warriors,
   Sacrifice, Self-Mill, Self-Discard).
3. Visible dual-res signpost cards are tagged to match their visible symbols:
   a (Tide, Zephyr) card is tagged Warriors. No exceptions.
4. Generic cards receive no tag (or a special "generic" value). The +0.5
   baseline protection applies regardless.
5. Cards that are B-tier in one archetype and C-tier in all others get tagged
   to their B-tier archetype. The tag is "best fit," not "only fit."

**Tag distribution target (per archetype):**

| Archetype | Tagged cards target |
|---|:---:|
| Each of 8 archetypes | ~40 cards |
| Generic (untagged) | ~40 cards |
| **Total** | **~360** |

The 40 cards tagged per archetype = the 40 home cards naturally. This means
roughly: 31-32 single-symbol home cards + 4-5 dual-res cards tagged to the
archetype + 3-4 "bridge" cards from adjacent archetypes that are strong in
this archetype.

**Why this is design-honest:** The card designer is assigning tags that reflect
each card's actual mechanical home. A Tide card that cares about creature attack
power is Warriors. A Tide card that cares about sacrifice effects is Sacrifice.
The tag encodes the same judgment a well-informed player would make. It is not
an arbitrary optimization label.

**Implementation note:** The tag should be assigned during card design, not
derived algorithmically post-hoc. The card designer who creates "Tidewatch
Warrior — when this character attacks, draw a card" naturally knows it is
Warriors-tagged. This keeps the system honest at the design layer.

---

## Post-Critique Revision

### Accepting the Displacement

The critic's decision to displace Design 1 (Tagged Narrative Gravity) in favor
of Hybrid A (Visible-First Anchor Gravity) is correct and I accept it. The
specific criticism is accurate: my champion does not differentiate meaningfully
from Design 2 (Tag-Gravity). Both use a 60/40-ish visible/hidden blend for
contraction relevance, both predict M3 ≈ 2.4-2.6, both use pool contraction for
M11. The 0.5 additive tag bonus I use is functionally similar to Design 2's 40%
tag weight in the contraction relevance score. There is no structural difference
that justifies running both in simulation.

### Defending the Mechanism Family

The criticism that my design "adds a 0.5 tag bonus on top of V8's Narrative
Gravity but does not fundamentally differ from Tag-Gravity" is fair as a
relative comparison, but I want to note what my design gets right that should
carry forward to Hybrid A:

**Tag bonus activation at pick 5, not pick 12.** My proposal activates tag
weighting after archetype commitment inference (pick 5+). Design 4's late
contraction (pick 12+) is the identified weakness: M10 improvement relies
entirely on R1 filtering and 4x weighting without contraction support until
pick 12. Hybrid A should adopt early contraction start (pick 5) from my
design, not Design 4's delayed start — the critic's Hybrid A specification
confirms this with "pick 5, not pick 12."

**Tag-inactive early phase.** My explicit design choice to keep the tag bonus
inactive for picks 1-4 is preserved in Hybrid A and is the right call for V9
Goal 10 (open-ended early). This is a concrete implementation constraint that
Hybrid A must carry forward.

### One Concern About Design 4's Dominance

The critic ranks Design 4 first on V1/V4 salience due to the layered
architecture "structurally guaranteeing V1 >= 60%." I accept the architectural
argument. My residual concern: Design 4's R1 filtering creates an 80-card
subpool (Zephyr: 40 Flash + 40 Ramp), and Flash/Ramp at F=25% Graduated
Realistic fitness may produce M3 per-archetype floors below 2.0 even at 4x
weighting. The critic flags this as Open Question 1. If simulation confirms
Flash/Ramp fall below 2.0 under Design 4's structure, the R1-first approach
has a fundamental Flash/Ramp equity problem that my 0.5 additive bonus
(operating across the full pool, not a pre-filtered subpool) handles more
gracefully. I flag this for simulation monitoring, not as a reason to resurrect
my champion.

### Summary

I support advancing Hybrid A over Design 1. The anchor mechanic (Design 6's
differentiated contraction rates) is a genuine qualitative improvement over my
uniform contraction approach. Hybrid A incorporating early contraction start
(pick 5) and tag-inactive early phase should be the test. My design's primary
contribution to the V9 synthesis is confirming the mechanism family — 3-bit
tag + contraction + floor slot — is sound. That contribution is preserved in
Hybrid A and Design 2 advancing to simulation.
