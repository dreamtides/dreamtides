# Algorithm Design 3: CSCT Detuned with Visible Emphasis

**Agent 3 — Round 2 Design** **Focus:** Detuned CSCT with hidden archetype tags
for precision and visible resonance as the player-facing signal.

______________________________________________________________________

## Key Findings

- **CSCT has a 0.92 M3 surplus above the 2.0 target.** Trading that headroom for
  M6/M9 compliance is the central bargain. The math shows that capping
  pair-matched slots at 2 and adding a random slot projects M3 ~ 2.3-2.5, well
  above 2.0 while fixing concentration.
- **CSCT's fitness immunity is its hidden strength.** Only 0.22 M3 degradation
  from Optimistic to Hostile — this robustness is structural, not coincidental.
  It comes from the commitment ratio's noise resistance, not from the
  pair-matched slot count. Detuning slots preserves this advantage.
- **The 10% visible dual-res subpool is too shallow for CSCT's pair-matching
  mechanism.** At only ~4-5 visible dual-res cards per archetype, CSCT cannot
  sustain even 1 pair-matched slot per pack without unacceptable repetition.
  Hidden archetype tags are not optional — they are required for CSCT to
  function at this pool composition.
- **Research Agent B's math ceiling establishes the floor:** with a 3-bit
  archetype tag, 40 home-tagged cards per archetype are available for targeting,
  removing the subpool exhaustion problem entirely. This is the minimum hidden
  metadata CSCT needs.
- **CSCT's M6 failure is mechanistic, not incidental.** M6 = 99% is caused by 3+
  pair-matched slots delivering home-archetype cards at ~87% precision for 25+
  picks. Capping at 2 slots with 1 random slot projects M6 ~ 75-82% — inside the
  60-90% target.
- **M9 (variance) is recoverable.** CSCT's M9 = 0.68 came from the "every pack
  looks identical" effect when 3 targeted slots deliver near-identical
  precision. With 2 targeted + 1 R1-filtered + 1 random, the variance
  reintroduced by the random slot should bring M9 to approximately 0.85-0.95.
- **Hidden tags keep visible resonance as the archetype signal, not the
  targeting signal.** The player sees visible symbols as evidence of their
  archetype identity. The algorithm uses hidden tags for pack precision. These
  roles are aligned but not identical — which is exactly the V9 design goal.

______________________________________________________________________

## Three Algorithm Proposals

### Proposal A: CSCT-2 (Hard Cap, Archetype Tag)

**Visible description:** "The better you commit to your archetype, the more
guaranteed on-archetype cards you see per pack — up to 2 per pack."

**Technical description:** Maintain the commitment ratio R = (pair-aligned
picks) / (total picks) using the player's inferred archetype pair from visible
resonance. Fill exactly min(floor(R * 3), 2) pack slots from cards tagged to the
committed archetype (hidden 3-bit tag). One additional slot uses R1-visible
filtering (primary resonance only). One slot is fully random. Commitment
inferred at pick 3 from the leading resonance type; resets if the player pivots
by 2+ consecutive off-resonance picks.

**Hidden metadata:** 3-bit archetype tag per card (8 archetypes). Applied to all
320 non-generic cards. For the ~36 visible dual-res cards, the tag matches the
primary archetype implied by the symbol order. For the ~284 single-symbol cards,
the tag identifies best-fit archetype based on card mechanics.

**Predicted metrics (Graduated Realistic):**

- M3: 2.35-2.50 (2 tagged slots at P=1.0 + 1 R1-filtered at P=0.58 + 1 random at
  P=0.125)
- M10: 3-5 (commitment ratio needs 3-4 picks to distinguish archetypes;
  transition zone risk)
- M11: 2.6-2.8 (no late-draft pool improvement; tagged slots plateau at 2)
- M6: 75-82% (2 perfect-precision slots + 1 filtered + 1 random prevents
  over-concentration)

**Weakness:** M11 does not reach 3.0 — two targeted slots cannot deliver 3 S/A
cards late without pool concentration.

______________________________________________________________________

### Proposal B: CSCT-2+C (Hard Cap + Contraction)

**Visible description:** "As you draft your archetype, your packs focus — and
late-draft packs become even stronger."

**Technical description:** CSCT-2 mechanism from Proposal A, plus a pool
contraction layer. From pick 6, remove the bottom 8% of pool cards per pick
using a relevance score that combines visible resonance signature (dot-product,
+2 primary +1 secondary) and hidden archetype tag alignment (binary: tagged for
committed archetype = 1.0, else = 0.0; weight 0.6 relevance score, 0.4 visible
resonance). Generics protected at 0.5 baseline. The 2 CSCT-targeted slots
continue to use archetype tags directly; the contraction acts on the remaining 2
slots by improving the quality of the pool they draw from.

**Hidden metadata:** 3-bit archetype tag (same as Proposal A). No additional
hidden data required — the relevance score blends tag alignment with visible
resonance additively.

**Predicted metrics (Graduated Realistic):**

- M3: 2.55-2.75 (2 tagged slots P=1.0 + contraction raises "random" slots from
  P=0.125 to P~0.45-0.60 by pick 15+)
- M10: 2-4 (commitment ratio still needs bootstrapping; floor from tagged slots
  prevents worst droughts)
- M11: 3.0-3.3 (contraction combined with tagged slots delivers 3 S/A late; pool
  by pick 15 is ~80-100 cards, majority home-archetype)
- M6: 80-88% (tagged slot precision + contraction pushes toward upper range;
  needs monitoring)

**This is the champion proposal.**

______________________________________________________________________

### Proposal C: CSCT-Graduated (Continuous Cap, No Contraction)

**Visible description:** "Your commitment ratio determines how many on-archetype
cards appear in each pack, from 1 to 2."

**Technical description:** Replace the hard cap at 2 with a continuous scale:
targeted slots = 1 + min(R, 1.0), where R is the commitment ratio. This delivers
1.0 targeted slots at zero commitment and 2.0 at full commitment — a smoother
ramp that never reaches the 3-slot density that caused CSCT's original M6
failure. The fractional implementation uses weighted slot sampling: each slot
independently draws from the archetype-tagged pool with probability equal to
(targeted-slot-count / 4). One slot is always fully random (minimum floor).

**Hidden metadata:** Same 3-bit archetype tag. Additionally, a 1-bit "late-pick
value" flag on each card (high = card has increasing marginal value in committed
decks; this is derivable from mechanic text and requires no additional design
work beyond tagging). The late-pick flag allows graduated weighting toward
synergy-dense cards in picks 15+.

**Predicted metrics (Graduated Realistic):**

- M3: 2.20-2.40 (lower than Proposal B; average ~1.5 targeted slots vs Proposal
  B's ~1.8-2)
- M10: 4-6 (smooth ramp means slow convergence; no hard floor)
- M11: 2.4-2.6 (without contraction, late-draft quality does not improve beyond
  M3 baseline)
- M6: 70-78% (continuous scale spreads concentration; lowest M6 of the three
  proposals)

**Weakness:** Smoother but weaker. M11 cannot reach 3.0 without contraction. The
additional 1-bit flag adds complexity without fixing the core M11 problem.

______________________________________________________________________

## Champion Selection: Proposal B (CSCT-2+C)

**Justification:**

Proposal A solves the M6/M9 problems but cannot reach M11 >= 3.0. Without
late-draft pool concentration, two perfect-precision targeted slots plateau at
M11 ~ 2.65. The V9 orchestration plan lists M11 >= 3.0 as a primary target
alongside M3 >= 2.0.

Proposal C solves M6 most cleanly but at the cost of M3 and M11. It is the
lowest-risk proposal by a narrow definition but does not deliver the late-draft
quality the plan requires.

Proposal B combines CSCT's proven commitment-ratio mechanism (fitness-immune,
smooth M10, minimal alignment catastrophe risk) with targeted pool contraction
(Narrative Gravity's M11 engine). The blend is mathematically coherent: CSCT's 2
targeted slots provide the reliable floor (M10 protection) while contraction
improves the remaining 2 slots progressively (M11 engine). Neither mechanism
contradicts the other.

The 3-bit archetype tag is the minimum hidden information required, keeping V2
low. The tag is mechanically derived (a card's best-fit archetype), maintaining
V3 defensibility. The visible resonance symbols provide the archetype inference
for the commitment ratio and remain the player's primary signal, keeping V1 and
V4 at target.

Proposal B's predicted M6 range (80-88%) sits within the 60-90% target at
moderate contraction rates and slightly above at aggressive rates — manageable
through contraction rate tuning.

______________________________________________________________________

## Champion Deep-Dive: CSCT-2+C

### How It Works

**Commitment Ratio Engine:** The algorithm tracks R = (picks with the player's
leading visible resonance) / (total picks since pick 3). The leading visible
resonance is inferred from the player's draft history — whichever single
resonance type (Ember, Stone, Tide, Zephyr) appears most frequently on drafted
cards. This infers archetype family (e.g., Tide = Warriors or Sacrifice) but not
the specific pair. The specific pair is inferred at pick 5-6 when enough data
distinguishes Warriors from Sacrifice within the Tide commits.

**Targeted Slots (2 per pack, post-commitment):** Draws exclusively from cards
with the hidden archetype tag matching the player's committed archetype. Tag
precision is P = 1.0: all tagged cards are home-archetype S-tier. Slot count =
min(floor(R * 3), 2) — reaches 2 at R = 0.67 (roughly pick 7-8 for a committed
drafter).

**R1 Slot (1 per pack):** Draws from cards whose visible primary resonance
matches the player's leading resonance. This maintains visible resonance as a
real targeting signal: the player's drafted symbols actively narrow this slot.
Precision P ~ 0.58-0.75 depending on archetype fitness.

**Contraction (from pick 6, weight 0.6 tag + 0.4 visible):** Remove bottom 8% of
pool by blended relevance score each pick. The tag component ensures contraction
tracks the committed archetype precisely even when visible symbols are
ambiguous. The visible component preserves the visible symbols' causal role —
packs that visibly include the player's resonance score higher on the visible
component and survive contraction longer. The player perceives their visible
resonance cards as "sticking around" and off-resonance cards disappearing.

**Random Slot (1 per pack, always):** Always present. Draws from the surviving
contracted pool. In early draft, this is nearly random; by pick 20, the pool is
concentrated enough that the "random" slot delivers P ~ 0.50-0.65, boosting M11
without targeted slot count.

### What the Player Sees vs. What the Algorithm Does

| Player experience                                                        | Algorithm reality                                                                                                             |
| ------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| "I drafted mostly Tide cards and now my packs are full of Tide cards"    | Commitment ratio reached 0.67, activating 2 archetype-tagged slots                                                            |
| "Finding that (Tide, Zephyr) card really focused my direction"           | Visible dual-res card boosted both the commitment ratio AND visible relevance score for contraction                           |
| "The packs are getting better as I commit deeper"                        | Contraction removes low-tag-alignment cards; random slot draws from increasingly clean pool                                   |
| "I see mostly Warriors cards now with occasional generics and surprises" | 2 tagged (Warriors) + 1 R1 (Tide) + 1 random-from-contracted-pool                                                             |
| "I had agency — I chose Warriors over Sacrifice"                         | Algorithm inferred Warriors from pair-distinguishing visible picks 5-7; player's visible choices genuinely drove the decision |

### Example Draft Trace (Warriors Committed Player)

**Picks 1-5 (exploration):** Algorithm provides 0 targeted slots + 1 R1-Tide + 3
random. Player sees variety. Draft: 2 Tide cards, 1 Zephyr, 1 Ember, 1 generic.
R = 2/5 = 0.40. Targeted slots = floor(0.40 * 3) = 1. Player infers they're in
Tide territory.

**Picks 6-10 (commitment):** Algorithm activates 1-2 targeted slots once
Warriors vs Sacrifice disambiguation happens at pick 6. Pack quality noticeably
improves. Contraction begins, removing obvious non-Tide cards. Player picks
Warrior tribal card, sees another Warrior card in next pack, picks it. Feels
like draft momentum.

**Picks 11-15 (consolidation):** 2 tagged slots locked. Pool has contracted
~35-40% from original size. R1 slot consistently delivers Tide cards. Random
slot draws from a pool of ~200-220 cards, now weighted toward Tide and Warriors.
Player sees occasional Sacrifice cards (sibling archetype, high tag overlap),
1-2 generics, and sometimes a Zephyr card.

**Picks 16-25 (late draft):** Pool contracted to ~130-150 cards. Tagged slots
deliver high-powered Warriors cards. Random slot yields M3 contribution of
~0.55-0.65. M11 (picks 15+ average) = approximately 3.1-3.3. Player feels deck
is "coming together" rather than "still searching."

### Failure Modes

**Failure 1: Early pivot costs commitment ratio.** A player who commits 4 picks
to Tide then pivots to Ember will have R reset toward 0, losing the targeted
slots. The algorithm will follow the pivot — this is intentional and fair. But
it costs ~2-3 picks of suboptimal pack quality during re-convergence.
Mitigation: commitment ratio uses a rolling window of last 8 picks rather than
all-time ratio, making pivots recoverable in ~4 picks.

**Failure 2: Archetype inference error at the pair level.** If the algorithm
infers Warriors but the player is building Sacrifice (both Tide-primary), the
hidden archetype tag targeting is wrong until pick 6-7 disambiguation. The R1
slot (Tide) remains valid throughout — only the tagged slots are misdirected.
The 1-2 misdirected picks deliver Tide-family cards that are often playable
regardless. Structural mitigation: all tagged-slot draws in picks 3-5 use the
resonance-family tag (Tide-family), not the pair-specific tag. Pair-specificity
only engages after pick 5.

**Failure 3: M6 creep at aggressive contraction rates.** If contraction removes
10%+ per pick rather than 8%, the random slot's effective precision rises above
0.65 late, pushing M6 toward 88-92%. Must be validated in simulation.
Mitigation: set contraction floor at 80 cards (never contract below this); this
bounds the random slot precision at ~0.55.

**Failure 4: Power-chaser invisibility.** A player who ignores visible resonance
entirely and drafts by power will have low commitment ratio, fewer targeted
slots, and no credit in the contraction score from visible resonance. Their M3
will be approximately 1.5-1.8 (mostly random + R1 slot from incidental picks).
This is correct behavior — the system rewards resonance commitment — but must
not be so punishing that new players feel the game is broken.

### V1-V4 Metrics

**V1 (Visible symbol influence):** Strip hidden tags; run algorithm using only
visible resonance for commitment ratio and R1 slot. Contraction uses only
visible relevance score (no tag component). Expected visible-only M3:
approximately 2.05-2.15 (commitment ratio can still trigger 2 slots, but without
tags the slots draw from R1-filtered pool at P ~ 0.58, and contraction is
weaker). Full M3: 2.55-2.75. V1 = (2.10 - 0.125) / (2.65 - 0.125) = 1.975 /
2.525 = **~78%.** This is high — nearly 80% of the gain over baseline comes from
visible symbols. The hidden tags add precision to the 2 targeted slots (P: 0.58
→ 1.0) but the overall pack quality is driven primarily by the commitment ratio
mechanism (visible) and contraction (70% visible-weighted).

**V2 (Hidden info quantity):** 3-bit archetype tag per card. 360 * 3 = 1080 bits
total. This is the minimum unit identified by Research Agent B. **V2 = 3
bits/card.** (Target: low. This is optimal.)

**V3 (Reverse-engineering defensibility):** The hidden tag is each card's
best-fit archetype derived from its mechanical design and resonance symbol. A
player examining the tag database would see: "this Tide card tagged Warriors has
'Warrior creature types matter' text — yes, that's Warriors." No card is tagged
to an archetype that contradicts its visible identity. Discovery reaction: "I
see, so the game knows more precisely which Tide cards are Warriors vs
Sacrifice, and uses that for the targeting. Fair enough." **V3 = 8/10.** One
point off because the tag forces one-archetype assignment for cards that
genuinely fit two (e.g., high-overlap Warriors/Sacrifice bridges); a
multi-archetype card getting one label could feel like a simplification to a
player who examines it.

**V4 (Visible resonance salience):** The "best visible pick" (highest visible
resonance match for the player's leading resonance) differs from the "best
hidden pick" (highest archetype tag match for committed archetype) in
approximately 15-20% of picks. The divergence occurs primarily in picks where a
card has the right visible symbol but is tagged for the sibling archetype (e.g.,
a Tide card tagged Sacrifice appearing in a Warriors draft). In these cases, the
algorithm's R1 slot would select it anyway (Tide symbol matches), but the tagged
slot would not. **V4: divergence ~ 15-20% of picks.** Within the < 20% target
from Research Agent C.

______________________________________________________________________

## Pool Specification

### Visible Symbol Distribution

|   Symbol count    | Cards |   %   | Role                                                       |
| :---------------: | :---: | :---: | ---------------------------------------------------------- |
|    0 (generic)    |  40   | 11.1% | Protected in contraction; splash options                   |
| 1 visible symbol  |  284  | 78.9% | Primary resonance; R1 slot targets; commitment ratio input |
| 2 visible symbols |  36   | 10.0% | Signpost cards; visible archetype-pair anchors             |

This matches the V9 baseline exactly. No additional visible dual-resonance is
required.

### Visible Dual-Resonance Breakdown (36 cards)

| Archetype pair       | Cards | Notes                                                                      |
| -------------------- | :---: | -------------------------------------------------------------------------- |
| Flash (Ze/Em)        |  4-5  | Tempo-identity cards; seen rarely but definitively                         |
| Blink (Em/Ze)        |  4-5  | Flicker-identity; mechanically distinct from Flash despite same resonances |
| Storm (Em/St)        |  4-5  | Spellslinger payoffs                                                       |
| Self-Discard (St/Em) |  4-5  | Graveyard-matters with Ember secondary                                     |
| Self-Mill (St/Ti)    |  4-5  | Reanimator anchors                                                         |
| Sacrifice (Ti/St)    |  4-5  | Abandon/ETB synergy cards                                                  |
| Warriors (Ti/Ze)     |  4-5  | Combat-matters tribal anchors                                              |
| Ramp (Ze/Ti)         |  4-5  | Spirit animal payoffs, mana acceleration                                   |

**Design principle for signpost cards (per Research Agent C):** Each visible
dual-res card should be a mechanical hinge — its text references the archetype's
defining theme, not merely having two symbols. It should be slightly above
average power (decision-forcing, not auto-pick). Distribute across draft phases:
~2 per archetype in picks 1-10, ~1-2 in picks 11-20, ~1 in picks 21-30
(late-synergy payoff).

### Hidden Metadata Schema

Each card in the pool receives exactly one hidden field:

```
archetype_tag: Archetype  // one of 8 values; 3 bits per card
```

Where `Archetype` is one of: Flash, Blink, Storm, SelfDiscard, SelfMill,
Sacrifice, Warriors, Ramp.

**Assignment rules:**

1. **Generic cards (40):** No archetype tag. Algorithm skips them in tagged
   slots; they remain in pool for R1 and random slots; protected from early
   contraction.

2. **Visible dual-res cards (36):** Tag matches the primary archetype of the
   symbol pair. (Tide, Zephyr) = Warriors. (Zephyr, Tide) = Ramp. These are the
   highest-confidence assignments — the card's mechanical text is explicitly
   designed for that archetype.

3. **Single-symbol cards (284):** Tag is the card's best-fit archetype
   determined by card design. For unambiguous cards (e.g., a Tide card with
   "your Warrior characters get +1 Spark"), tag = Warriors. For contested cases
   (e.g., a Tide card with a generic ETB that works in both Warriors and
   Sacrifice), tag = the archetype where the card is S-tier (not merely A-tier).
   For ambiguous generics that happen to have a resonance symbol, tag = the
   archetype whose secondary resonance matches the visible symbol.

**Honesty constraint:** No card should be tagged to an archetype that
contradicts its visible resonance symbol. A Tide card can only be tagged
Warriors or Sacrifice (both Tide-primary). A Zephyr card can only be tagged
Flash or Ramp. This preserves the visible-to-hidden alignment that Research
Agent A identifies as the critical condition for design integrity.

**Total hidden information:** 320 cards with tags * 3 bits = 960 bits + 40
generic cards with 0-bit null tag. Well within the "minimum viable" envelope
defined by Research Agent B.

### Contraction Relevance Score (Algorithm-Internal, Not a Card Property)

The contraction relevance score is computed at runtime from:

```
score = 0.6 * tag_match + 0.4 * visible_dot_product
```

Where:

- `tag_match` = 1.0 if card's archetype_tag == player's committed archetype,
  else 0.0
- `visible_dot_product` = dot product of card's visible symbol vector with
  player's signature vector (standard V8 weights: +2 primary, +1 secondary)

This is not stored per-card — it is computed dynamically. The visible resonance
signature drives 40% of contraction decisions, ensuring visible picks cause
visible pack changes.

______________________________________________________________________

## Post-Critique Revision

### Summary of Critique

The critic ranked CSCT-2+C last among the six champions, citing three specific
concerns: (1) the V1 = 78% claim is misleading because the most consequential
improvement — doubling slot precision from P=0.58 to P=1.0 via hidden tags — is
not adequately reflected in the V1 accounting; (2) three independent sub-systems
running simultaneously is the most complex of all proposals, violating V9's
preference for minimal hidden mechanism; (3) M10 = 2-4 is a wide, dishonest
range whose honest upper bound may be 4, not 2. The critic also displaced
CSCT-2+C from simulation slots, replacing it with Hybrid B (Affinity-Tagged
Gravity) as a cleaner test of a similar design space.

### Accepted Criticisms

**V1 accounting is overstated.** The critic is correct. The V1 calculation
treated the targeted-slot mechanism as primarily driven by visible commitment
ratio — because it is the ratio that activates the slots — but buried the key
hidden contribution: once slots activate, the precision jump from P=0.58 to
P=1.0 is entirely from hidden tags, not from visible signals. The correct way to
measure V1 is to run the algorithm with hidden tags stripped and measure the
resulting M3. A visible-only CSCT-2 (using R1-filtered pool, P=0.58, for the
targeted slots) achieves M3 approximately 2.05-2.15. Full M3 is 2.55-2.75. The
honest V1 estimate is closer to 74-76%, not 78% — a meaningful overstatement in
context.

**M10 range is too optimistic.** M10 = 2-4 should be reported as M10 = 3-4 in
the Graduated Realistic scenario. The commitment ratio engine needs 3-5 picks to
activate 2 targeted slots, and V8 established that the transition zone (picks
4-9) is structurally resistant to improvement without a quality floor slot.
CSCT-2+C does not include a dedicated quality floor slot — the random slot is
fully random, not top-quartile filtered. This is a real M10 weakness, and the
lower-bound estimate of 2 is only achievable in Optimistic fitness, not
Graduated Realistic.

### Defended Positions

**Complexity concern is overstated in one respect.** The critic characterizes
CSCT-2+C as having three independent sub-systems. This is fair as a count, but
the systems are not orthogonal — the commitment ratio directly governs both
targeted slot count and the primary-resonance inference that feeds the
contraction relevance score's visible component. They share a single inference
state (the player's inferred archetype). The practical implementation is two
tracked values (commitment ratio R, inferred archetype) plus the pool
contraction list. This is more complex than Design 4's two-stage filter, but it
is not three genuinely independent systems.

**The V1 overstatement does not invalidate the champion selection.** Even
corrected to V1 ~ 74-76%, CSCT-2+C satisfies V9's visible-primary requirement.
The correction weakens the margin but does not cross the threshold. The design's
core claim — that the commitment ratio engine makes visible resonance causally
necessary, not merely correlated — is unaffected by the precision-jump
accounting.

### Modified Champion: CSCT-2+C with Floor Slot

The one structural change the critique warrants: add a quality floor slot to
address the honest M10 weakness. Replace the fully random slot with a
top-quartile draw from the contracted pool (same mechanism as Designs 1, 2, 5,
and 6). This is directly convergent with the critic's observation that all five
other proposals include this mechanism as an M10 floor.

**Revised slot structure (post-pick 5):**

- 2 slots: archetype-tagged draws (P = 1.0, home archetype)
- 1 slot: R1-visible filtered draw (P ~ 0.58-0.75)
- 1 slot: top-quartile draw from contracted pool (floor mechanism; P ~
  0.35-0.50)

This change reduces M10 to 2-3 (honest Graduated Realistic range) by
guaranteeing the worst pick is at least within the top quartile of available
pool quality. It marginally increases V2 (the floor slot now has implicit hidden
information from the contracted pool's tag distribution) but does not add any
new hidden field.

**Revised predicted metrics (Graduated Realistic, with floor slot):**

- M3: 2.50-2.70 (floor slot contributes P ~ 0.40 instead of P ~ 0.125; modest
  lift)
- M10: 2-3 (floor slot prevents worst droughts; honest range)
- M11: 3.0-3.3 (unchanged; contraction engine is the M11 driver)
- M6: 80-88% (unchanged; floor slot slightly increases concentration risk at
  upper range)
- V1: ~74-76% (corrected from original 78% estimate)

### On Displacement from Simulation Slots

The critic's decision to displace CSCT-2+C from simulation in favor of Hybrid B
is reasonable given simulation resource constraints. Hybrid B directly tests
whether two-float affinity (8 bits/card) beats a binary tag (3 bits/card) for
V3, which is a cleaner experimental question. CSCT-2+C's contribution —
commitment-ratio mechanics as a visible-first activation gate — is a real but
distinct question that adds mechanism complexity to what should be a clean V1/V3
test.

If simulation slots expand beyond six, CSCT-2+C with the floor slot modification
is the right addition. Its commitment ratio gate is structurally different from
all other proposals: it is the only algorithm where the player's *rate* of
visible commitment, not just the *direction*, determines pack quality. That is a
genuine design property no other proposal tests.
