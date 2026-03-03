# Algorithm Design 5: Honest Affinity System

**Agent 5, Round 2**
**Focus:** Maximally honest hidden metadata — archetype affinity scores derived
directly from visible card properties, used transparently for pack targeting.

---

## Key Findings

- **Affinity scores are the most defensible hidden metadata.** Because affinities
  are derived from visible properties (cost, type, keywords, resonance symbols),
  a player who inspected the database would endorse every score as a fair
  mechanical assessment. No card is mislabeled relative to its visible identity.

- **The M3 ceiling with affinities matches hidden archetype tags, not significantly
  more.** Research Agent B shows Level 3 (8-float affinity) achieves ~2.7-2.9 M3,
  virtually identical to Level 1 (3-bit tag, ~2.5-2.7). The extra information
  primarily benefits M11 and design integrity, not raw M3.

- **Affinity scores solve the forced-tag dilemma.** A 3-bit tag must assign each
  card to exactly one archetype, discarding its secondary value. A card that is
  A-tier in both Warriors and Sacrifice gets tagged as Warriors and stops
  contributing to Sacrifice targeting. Affinity scores retain this cross-archetype
  value, improving multi-archetype discovery and M7 (run variety).

- **Pool contraction + affinity scoring is the power combination.** Contraction
  using affinity-weighted relevance scores produces smoother pool degradation than
  binary home/not-home removal. High-affinity multi-archetype cards survive longer,
  creating organic splash opportunities (M4) before being eventually removed.

- **V1 (visible symbol influence) stays in the 35-45% range.** The visible symbols
  provide the initial archetype signal in picks 1-5 and contribute to the relevance
  score. But from pick 6 onward, affinity-weighted contraction does most of the
  targeting work. This is honest (V3 = 9/10) but visible resonance is not the
  dominant mechanism (V1 < 50%).

- **The key V9 tradeoff for this design:** Maximum design integrity (V3) at moderate
  visible salience (V1 ~40%). If V1 >= 60% is the primary goal, this approach
  over-engineers the hidden layer. If design integrity is paramount and M11 >= 3.0
  is required, this is the right tool.

- **Affinity derivation from visible properties is both the system's strength and
  its complexity budget.** The derivation rules (symbol weight, keyword presence,
  card type) must be published or inspectable. This adds ~16-32 bits per card
  (4-8 floats at reasonable precision) but keeps the system fully auditable.

---

## Three Algorithm Proposals

### Proposal A: Affinity-Weighted Narrative Gravity (AWNG)

**One-sentence visible description:** "As you draft, the game removes cards
that do not fit your emerging strategy, so your packs improve over time."

**Technical description:** Pool contraction using per-card affinity scores as
the relevance signal. After each pick, update the player's archetype profile
(a weighted average of drafts' affinity vectors). From pick 4, compute each
pool card's relevance as the dot product of its affinity vector with the
player's profile. Remove the bottom 12% per pick (same rate as V8 Narrative
Gravity). One slot from pick 3 draws from the top-quartile by affinity score
(floor). The contraction is smoother than V8's binary symbol-matching because
multi-archetype cards (high affinity in 2+ archetypes) survive longer in the
pool, available as cross-archetype options until the player's profile
definitively commits to one.

**Hidden metadata requirements:** 8-float affinity vector per card (or 4-float
if only adjacent archetypes are non-zero, ~32-64 bits per card). Affinities
derived at card design time from: primary resonance symbol (+0.6 for home
archetype pair, +0.3 for adjacent pairs sharing that resonance), card subtype
(Warriors subtype: +0.2 for Warriors archetype), mechanical keywords (graveyard
trigger: +0.2 for Self-Mill/Sacrifice, -0.1 for Flash), mana cost (ramp cards:
+0.1 for Ramp), power-level estimate. All derivation rules are published.

**Predicted metrics (Graduated Realistic):**
- M3: 2.65-2.80 (matches V8 Narrative Gravity on 40% pool)
- M10: 2.5-3.0 (improved over V8 NG at 3.3 due to smoother contraction)
- M11: 3.1-3.3 (better than 3-bit tag because affinity concentration is higher)
- M6: 82-88% (comparable to V8 NG at 85%)

---

### Proposal B: Affinity-Layered Slot Filling (ALSF)

**One-sentence visible description:** "Your picked cards shape what the game
offers next — commit to a resonance and your packs reflect it."

**Technical description:** Two-layer slot allocation. Layer 1: R1 visible
filtering on 2 of 4 slots (filters by primary resonance symbol, same as V7
SF+Bias). Layer 2: Affinity targeting on 1 additional slot — from the R1-filtered
candidates, select the card with highest affinity score for the player's inferred
archetype. 1 slot remains random (M4 splash). Archetype inference uses the same
running affinity profile as AWNG but without pool contraction. The visible symbols
do the broad filtering (V1 ~50-55%); affinity scores do the fine-grained selection
within that filtered set (the Agent 4 "Layered Salience" idea, but using affinity
rather than a binary tag).

**Hidden metadata requirements:** Same 8-float affinity vector. The layer 1
R1 filter uses visible symbols (no hidden data). The layer 2 affinity selection
uses hidden affinities only within the already-visible-symbol-filtered pool.
Because hidden affinities only break ties within a visible-symbol-filtered set,
the hidden contribution is genuinely secondary.

**Predicted metrics (Graduated Realistic):**
- M3: 2.35-2.50 (better than visible-only ~2.05, worse than AWNG)
- M10: 3.5-5.0 (no contraction means no late-draft improvement mechanism)
- M11: 2.3-2.6 (without contraction, no late-draft amplification)
- M6: 72-80% (slot-filling keeps concentration in check better than contraction)

---

### Proposal C: Affinity-Contracted Late Surge (ACLS)

**One-sentence visible description:** "The further you commit, the better your
packs get — your draft history shapes everything."

**Technical description:** Hybrid of AWNG (affinity-weighted contraction for
picks 1-14) and ALSF-style affinity targeting (2 dedicated affinity-targeted
slots) for picks 15+. Early phase: contraction builds a concentrated pool from
360 down to ~80-120 cards. Late phase: the contracted pool plus explicit
affinity targeting delivers the M11 >= 3.0 target. This addresses M11 directly
by layering two mechanisms in sequence: contraction narrows the field, affinity
targeting within the contracted pool fills slots precisely.

**Hidden metadata requirements:** Same affinity vector. Two-phase algorithm
adds minor complexity but uses the same metadata throughout.

**Predicted metrics (Graduated Realistic):**
- M3: 2.60-2.75
- M10: 2.0-2.5 (contraction phase is smooth; late surge adds quality without
  creating bad streaks)
- M11: 3.3-3.6 (best M11 of the three, targeted to exceed >= 3.0)
- M6: 83-90% (late targeting raises concentration; risk of M6 > 90% at pick 20+)

---

## Champion Selection: Proposal A — Affinity-Weighted Narrative Gravity (AWNG)

**Justification:** AWNG is the correct champion because it best serves V9's
central question. The Honest Affinity System assignment asks: what if hidden
metadata is maximally honest? AWNG answers this by (a) deriving all affinities
from visible card properties, (b) using those affinities for contraction in
a way that exactly replicates what visible symbols would do if they were
more informative, and (c) producing M3/M11 performance matching V8's best
results while using only ~10% visible dual-resonance.

Proposal B (ALSF) achieves higher V1 but fails M11 >= 3.0 because slot-filling
without contraction cannot amplify late-draft quality. Proposal C (ACLS) achieves
better M11 but at the cost of M6 risk and algorithmic complexity. AWNG is the
clean single-mechanism solution.

The key comparison: AWNG at 10% visible dual-res should match V8 Narrative
Gravity at 40% visible dual-res in M3 (both ~2.65-2.80), while scoring V3 = 9/10
(highest possible design integrity) because the affinities are publicly derivable
from each card's visible properties.

---

## Champion Deep-Dive: Affinity-Weighted Narrative Gravity

### How It Works

**Pool setup:** 360 cards, each with an 8-float affinity vector. The vector is
computed from visible properties at card design time and stored in the game
database (publicly accessible, not printed on card face).

**Affinity derivation rules (published):**

| Property | Affinity contribution |
|---|---|
| Primary resonance symbol matches archetype primary | +0.60 |
| Primary resonance matches archetype secondary | +0.30 |
| Secondary resonance (visible dual-res) exact pair match | +0.90 |
| Mechanical keyword: creature subtype matches archetype | +0.20 |
| Mechanical keyword: graveyard trigger | +0.20 to Self-Mill, Sacrifice |
| Mechanical keyword: instant/flash speed | +0.20 to Flash, Storm |
| Mechanical keyword: land/ramp | +0.15 to Ramp |
| Card type: character | +0.05 to Warriors, Sacrifice |
| Mana cost >= 5 | +0.10 to Ramp, -0.05 to Flash |

All affinities are clamped to [0, 1] and normalized so no card exceeds 1.0
for any single archetype.

**Draft algorithm:**

1. Initialize player profile P as zero vector (8 floats).
2. After each pick, update P: P += card.affinity_vector * (1 + 0.1 * pick_number)
   (later picks weight heavier, reflecting commitment).
3. From pick 4: compute relevance(card) = dot(card.affinity, P) for each pool card.
   Generics receive relevance = 0.5 (protected).
4. Remove bottom 12% of pool cards by relevance per pick (same as V8 NG).
5. Pack slots: 3 draw randomly from surviving pool; 1 draws from top-quartile
   by relevance (floor slot, from pick 3).
6. No explicit pair-matching slots. The entire pool improves, benefiting all slots.

### What the Player Sees vs. What the Algorithm Does

**Player sees:**
- A pack of 4 cards with visible resonance symbols.
- Over time, more of those cards match the resonance they have been drafting.
- An occasional dual-resonance signpost card (~10% of pool) that strongly signals
  an archetype pair.
- Quality improving monotonically after committing (picks 6-15 feel progressively
  better).
- The game description: "As you draft, the game removes cards that don't fit your
  style. Your packs reflect your choices."

**Algorithm does:**
- Uses per-card affinity scores (not visible) to compute relevance against the
  player's accumulated draft profile.
- Removes low-relevance cards permanently from the draw pool.
- Offers a guaranteed top-quartile card per pack from pick 3 onward.
- The contraction is entirely driven by affinity scores — single-symbol cards
  are correctly targeted without the player seeing a second symbol.

**The gap between what is visible and what the algorithm does** is precisely the
information content of the affinity vector beyond what the primary resonance
symbol provides. For most cards, the affinity vector's primary archetype match
is predictable from the visible symbol: a Tide card has high affinity for Warriors
and Sacrifice. The hidden detail is which of those two archetypes it fits better.
This is the minimum information needed to distinguish Warriors from Sacrifice among
single-symbol Tide cards — and it is derivable from visible mechanical keywords.

### Example Draft Trace

**Player drafting Warriors (Tide/Zephyr), picks 1-20:**

- **Picks 1-5:** Player takes 3 Tide cards (1 with visible dual-res (Tide, Zephyr))
  and 2 generics. Profile is Tide-heavy but Warriors/Sacrifice unresolved.
  Algorithm begins contraction: Storm, Blink, Flash cards drop off. Pool: ~290 cards.

- **Picks 6-10:** Player commits to Warriors by taking 4 more Tide cards (1 another
  dual-res (Tide, Zephyr)). Profile now clearly Warriors. Algorithm contraction
  accelerates: Self-Mill, Self-Discard, Ramp cards removed. Pool: ~210 cards. M3
  this phase: ~2.4 (Tide-primary cards are abundant, and their affinity scores
  are correctly parsed as Warriors-oriented due to creature subtype keywords).

- **Picks 11-15:** Pool is ~150 cards. Sacrifice cards begin to fall off (they
  have high Tide affinity but Warriors affinity is now clearly preferred by the
  player's profile). Some splash Sacrifice cards with high Warriors affinity
  survive (M4: they appear occasionally in the random slots). The floor slot
  almost always produces a Warriors-affinity card. M3 this phase: ~2.6.

- **Picks 16-20:** Pool is ~80 cards, heavily Warriors + high-affinity adjacents.
  Random slots from this concentrated pool produce Warriors cards ~60% of the
  time even without explicit targeting. M11 this phase: ~3.1-3.3.

**A player who looked up the affinity database during this draft would see:**
Each card they picked has the highest affinity scores for Warriors (0.7-0.9),
confirming the algorithm correctly identified their archetype. The algorithm's
behavior is exactly what the visible description promises. No card is labeled
with an archetype that contradicts its mechanics.

### Failure Modes

**1. Archetype misidentification in picks 1-5:** If the player's first 5 picks
split Tide and Stone evenly (say, building toward Self-Mill but taking one Sacrifice
card), the affinity profile is ambiguous. The contraction algorithm begins removing
Flash/Ramp/Blink correctly, but Warriors vs. Self-Mill stays unresolved. The
player may experience 2-3 packs (picks 6-8) with lower quality as the algorithm
converges. This is the M10 risk — the transition zone still creates occasional
bad streaks, estimated at M10 = 2.5-3.0 (better than V8 NG at 3.3 but still
a marginal failure).

**2. Power-chaser divergence:** Players who ignore visible resonance and draft
by power get correctly targeted by affinities (because high-power cards have
clear archetype affinities from their mechanics) but build incoherent decks.
M6 for power-chasers will be low. This is acceptable — it is a feature, not
a bug. Power-chasers should be guided to resonance strategies.

**3. Affinity derivation inconsistency:** If card designers assign mechanical
keywords inconsistently (e.g., not tagging all graveyard triggers), affinity
scores will be slightly off. This reduces targeting precision modestly but does
not break the system. The derivation rules are a design-time discipline, not
a runtime calculation.

**4. Late-draft pool exhaustion:** Same risk as V8 NG — by pick 25-28, the pool
may contract to 15-20 cards, and repeated offerings feel limiting. The affinity-
weighted contraction is slightly slower than V8's binary symbol removal (multi-
archetype cards survive longer), mitigating this somewhat. A minimum pool floor
of 20 cards is recommended.

### V1-V4 Metrics

**V1 (visible symbol influence):** Estimated 40-45%. If hidden affinities are
stripped and only visible symbols drive contraction, M3 drops to ~2.05 (visible-
only ceiling). AWNG full M3 ~2.70. V1 = (2.05 - baseline) / (2.70 - baseline)
≈ 0.40-0.45. Visible symbols do meaningful work (early phase contraction uses
resonance signature from visible symbols), but affinity refinement contributes
the majority of precision gain. This is below the V9 ideal of >= 60% but above
the V3 design integrity threshold (the hidden metadata is honestly derived).

**V2 (hidden info quantity):** 4-8 floats per card at 4-bit precision = 16-32
bits per card. For 360 cards: 5,760-11,520 bits total. This is substantially
more than a 3-bit archetype tag (1,080 bits total) but substantially less than
the full-precision affinity matrix. The derivation rules reduce this to an
auditable formula, not arbitrary values.

**V3 (reverse-engineering defensibility):** 9/10. A player who inspected the
database would find affinities that match their expectations from reading the
cards. "This Tide character with the 'When a Warrior enters play' trigger has
0.85 Warriors affinity and 0.40 Sacrifice affinity — that's exactly right."
The one point off: some players may feel the exact float values (0.85 vs 0.80)
are arbitrary even if the direction is correct. Publishing the derivation formula
eliminates this concern for most players.

**V4 (visible resonance salience):** Estimated 65-75% of picks align between
"best visible pick" and "best hidden-affinity pick." The 25-35% divergence
occurs when two cards have the same visible symbol but different affinity scores
(e.g., two Tide cards, one with Warriors-relevant mechanics and one with Sacrifice-
relevant mechanics). This is the informative divergence — the player is choosing
between mechanically distinct cards that look similar by symbol, and the algorithm
has a preference the player might not yet have. This divergence is desirable: it
means mechanical evaluation matters even for same-symbol cards.

---

## Pool Specification

### Visible Symbol Distribution

| Symbol Count | Cards | % | Notes |
|:---:|:---:|:---:|---|
| 0 (generic) | 40 | 11.1% | No visible resonance; affinity vector all ~0.1-0.2 |
| 1 visible symbol | 284 | 78.9% | Standard; affinity vector peaks at 1-2 archetypes |
| 2 visible symbols | 36 | 10.0% | Dual-res signposts; affinity = 0.9+ for pair archetype |

Visible dual-res remains at 10% (36 cards). Identical to V9 baseline. The
affinity system adds hidden information to the 284 single-symbol cards, not
additional visible symbols.

### Hidden Metadata Schema

```python
class AffinityVector:
    flash_tempo: float       # [0, 1]
    blink_flicker: float     # [0, 1]
    storm_spell: float       # [0, 1]
    self_discard: float      # [0, 1]
    self_mill: float         # [0, 1]
    sacrifice: float         # [0, 1]
    warriors: float          # [0, 1]
    ramp: float              # [0, 1]

    # Derivation: computed from visible properties at card design time.
    # Rules published in card designer's handbook. No arbitrary assignments.
    # Sparse in practice: most cards have 1-2 non-zero values above 0.3.
```

**In practice, only 2-4 affinities per card will be meaningfully non-zero.**
The full 8-float vector is a design-time artifact; the runtime representation
can use a sparse encoding of ~2-3 pairs (archetype_id, affinity_score), reducing
actual hidden data to ~6-9 bytes per card.

### Card Designer's Brief

For each card, assign affinity scores by answering:
1. Which primary resonance symbol does this card show? (+0.6 to both archetypes
   sharing that resonance as primary or secondary)
2. Does the card's mechanical text reference an archetype-specific mechanic?
   (graveyard, combat, spell, land — add +0.2 to relevant archetypes)
3. Does the card's subtype or creature type align with an archetype's identity?
   (+0.2 to that archetype)
4. Normalize so no value exceeds 1.0.

This takes ~30 seconds per card and requires no judgment calls beyond reading the
card's own text. The derivation is not an art form — it is a checklist.

---

## Post-Critique Revision

**Agent 5, Round 3 Response**

The critic's ranking is fair on V1/V4 salience, and I accept it. AWNG is last
among the six proposals on visible symbol influence at 40-45%, and the caution
flag is appropriate. Let me address each critique directly.

**Accept: The V1 weakness is real.** From pick 6 onward, the contraction is
driven by the 8-float affinity vector, not the visible primary resonance symbol.
The critic is correct that this makes visible symbols feel less causal than in
Designs 4 or 6. I claimed V1 = 40-45% and noted this was below the V9 ideal of
>= 60% — I was honest about this tradeoff when selecting AWNG as champion. The
critique confirms the cost is real.

**Defend: The V3 = 9/10 distinction is also real, and matters.** AWNG is the
only proposal where the hidden information is not just "defensible" but fully
derivable from published rules in 30 seconds per card. The critic awards all
other proposals 8/10 for this dimension. That 1-point gap represents the
difference between "a game-literate player would endorse this tag" and "a player
with a calculator and the rules document would reproduce this score exactly." If
V9's final synthesis is trying to answer whether design integrity and honest
metadata add real value, AWNG is the only proposal that can give a clean answer.

**Accept: The full 8-float vector may be over-engineered.** The critic's Hybrid B
proposal — two-float pair affinity (warriors_affinity, sacrifice_affinity) per
Tide card, per resonance pair — captures the central insight of AWNG at 8 bits
vs. 16-32 bits. The Hybrid B framing is better than my original: the genuine
problem AWNG solves is distinguishing same-primary-resonance siblings (Warriors
vs. Sacrifice within Tide cards), and that problem requires exactly one comparison
between the two archetypes sharing that resonance — not eight values. AWNG over-
parameterized the solution.

**Champion modification: I endorse Hybrid B as the cleaner version of this
design.** If forced to choose between advancing AWNG as specified and advancing
Hybrid B, Hybrid B is the right test. It retains V3 = 9/10 (published two-float
derivation per resonance pair is equally auditable), reduces V2 from 16-32 to
8 bits/card, and preserves the key differentiator: no card is forced into a single
archetype tag when it genuinely serves two.

**One defense I maintain:** The critic ranks AWNG 1st on M3 potential (2.65-2.80)
and last on V1/V4 salience. This tradeoff is real and the ranking reflects it
honestly. But if simulation confirms V1 >= 40% for AWNG and M3 >= 2.65, AWNG
represents a specific point on the design space — maximum targeting precision at
minimum visible symbolism — that is genuinely different from the other proposals.
It should advance to simulation with the V1 monitoring condition the critic
specified, not because I think it will beat Design 4 on salience, but because it
answers a different question: how much M3 and design integrity can be extracted
from a system where visible resonance is a signal but not the primary driver?

**In summary:** I accept the V1 weakness, accept that Hybrid B is the cleaner
version of the key AWNG insight, and maintain that the design advances to
simulation as a boundary condition test rather than a likely overall winner.
