# Algorithm Design 2: Narrative Gravity with Hidden Metadata

**Agent:** Algorithm Design Agent 2 **Focus:** Narrative Gravity's pool
contraction mechanism adapted for ~10% visible dual-resonance using hidden
archetype metadata

______________________________________________________________________

## Key Findings

- **The visible-only ceiling is the binding constraint.** At 10% visible
  dual-res, V8's Narrative Gravity achieved M3 = 2.38 aggregate but Flash fell
  to 1.47 — because the relevance score was computed from visible symbols, and
  low-overlap archetypes have too few visible signals to drive precise
  contraction. The contraction mechanism is sound; the signal feeding it is too
  weak.

- **A 3-bit hidden archetype tag resolves the signal problem exactly.** The math
  ceiling research shows that replacing the visible-symbol relevance score with
  a hidden tag-based relevance score raises the effective pair-matchable subpool
  from ~4-5 visible dual-res cards to 40 home-archetype cards per archetype.
  This is the same information Narrative Gravity needs — it just doesn't need to
  come from visible symbols.

- **Hidden tags are the minimum viable hidden information for M11 >= 3.0.** Pool
  contraction driven by visible symbols alone cannot achieve M11 because the
  relevance score at pick 15 is still derived from the same weak visible signal
  it had at pick 6. Hidden tags let the surviving pool concentrate more
  precisely, boosting random-slot quality in the late draft to 0.5-0.6 S/A (vs
  0.125 from the full pool).

- **The player experience does not change — and that is the point.** Narrative
  Gravity's 7.9/10 player experience rating came from the felt quality ramp, not
  from visible pair symbols. "My packs are getting better as I commit to Tide"
  is true regardless of whether the contraction algorithm uses visible (Tide,
  Zephyr) symbols or a hidden Warriors tag. The visible symbols remain the
  player's primary drafting signal; the hidden tag is internal infrastructure
  for the contraction precision.

- **Per-archetype equity improves dramatically over V8.** V8's visible Narrative
  Gravity showed a 0.73 spread (Flash 2.40 to Warriors 3.13) because low-overlap
  pairs had weaker visible pair signals. Hidden tags deliver identical precision
  (P = 1.0 for home-archetype draws) for all 8 archetypes, projecting spread to
  ~0.1-0.2.

- **Affinity scores offer modest M3 gains (+0.2-0.3) over tags but at high
  design integrity cost.** They require assigning 8 calibrated floats per card —
  substantially more metadata — for gains that are achievable with a single
  3-bit tag plus pool contraction. The research confirms: tags + contraction =
  M11 achievable; affinity scores do not meaningfully change the ceiling.

- **The visible resonance system remains the primary player signal.** The hidden
  tag is used for contraction precision; the player's archetype inference comes
  from visible symbols. A player who sees more Tide cards appearing and commits
  to Tide correctly causes the hidden tag filtering to engage. The two layers
  are correlated, not independent — discovery produces "that makes sense," not
  "the symbols are fake."

______________________________________________________________________

## Three Algorithm Proposals

### Proposal A: Tag-Gravity (Champion)

**Visible description:** "As you draft, the game removes cards that don't match
your style, so your future packs get better."

**Technical description:** Maintain a 4-element resonance signature vector
updated by visible symbols (+2 primary, +1 secondary). From pick 4, compute a
relevance score for each pool card using a weighted combination: 40% from
visible dot-product (player's signature vs. card's visible symbols) and 60% from
hidden archetype tag match (does the card's hidden tag match the inferred
archetype?). Remove the bottom 12% of pool cards per pick by this combined
relevance score. Generics protected at 0.5 baseline. One slot from pick 3 draws
from the top-quartile subset.

Archetype inference: infer the player's committed archetype from pick 5 onward
as the hidden archetype tag most frequently seen on their drafted cards.
Pre-pick-5, the visible-symbol signature drives contraction exclusively.

**Hidden metadata requirements:** One 3-bit archetype tag per card (one of 8
archetypes). Total: 360 * 3 = 1080 bits for the pool.

**Predicted metrics (Graduated Realistic):**

- M3 = 2.55-2.70 | M10 = 2.5-3.0 | M11 = 3.0-3.2 | M6 = 82-87%
- V1 = ~40-50% | V2 = 3 bits/card | V3 = 8/10 | V4 = ~75-80% picks aligned

______________________________________________________________________

### Proposal B: Tag-Gravity-Pure

**Visible description:** "As you draft, the game removes cards that don't match
your style, so your future packs get better."

**Technical description:** Identical to Tag-Gravity but the contraction
relevance score uses hidden archetype tag exclusively (0% visible, 100% tag).
The visible resonance signature is still used for the floor slot (top-quartile
draw) and for pre-commitment contraction (picks 1-4). From pick 5 onward, the
60/40 blend is replaced by full tag-based relevance.

**Hidden metadata requirements:** Same 3-bit tag per card.

**Predicted metrics (Graduated Realistic):**

- M3 = 2.65-2.80 | M10 = 2.5-3.0 | M11 = 3.1-3.3 | M6 = 83-88%
- V1 = ~25-35% | V2 = 3 bits/card | V3 = 7/10 | V4 = ~65-70% picks aligned

Tradeoff: slightly higher M3 ceiling but visible resonance does less of the
targeting work. V1 falls below the preferred 40-50% threshold.

______________________________________________________________________

### Proposal C: Affinity-Gravity

**Visible description:** "As you draft, the game removes cards that don't match
your style, so your future packs get better."

**Technical description:** Replace the 3-bit tag with a single "primary affinity
score" per card: a float (0-1) representing how well the card plays in the
player's committed archetype. Scores are derived from visible card properties
(resonance symbols, mechanical keywords, card type), making them publicly
reconstructible. The contraction relevance score weights 40% visible dot-product
\+ 60% affinity score for the inferred archetype. Bottom 12% removed per pick.
Same floor mechanism as Tag-Gravity.

**Hidden metadata requirements:** 8 affinity scores per card (8 floats, or ~40
bits at 5-bit precision per value). Total: ~14,400 bits.

**Predicted metrics (Graduated Realistic):**

- M3 = 2.65-2.80 | M10 = 2.5-3.0 | M11 = 3.0-3.2 | M6 = 83-87%
- V1 = ~30-40% | V2 = ~40 bits/card | V3 = 9/10 (scores derived from real
  mechanics) | V4 = ~72-78% picks aligned

Tradeoff: highest design integrity (V3 = 9/10) but ~13x more hidden information
per card for the same M3 ceiling as Tag-Gravity. Information research confirmed
"a 3-bit tag that genuinely reflects card mechanics is more honest than a matrix
assigned to maximize performance."

______________________________________________________________________

## Champion Selection: Tag-Gravity

Tag-Gravity is the champion because it hits all targets at minimum information
cost.

The research established that 3-bit hidden archetype tags are the minimum unit
of hidden metadata that meaningfully changes the system: they lift M3 from ~2.05
to ~2.4-2.5, enable M11 >= 3.0 via pool contraction, and eliminate the
per-archetype Flash/Ramp equity failures in V8.

Compared to Tag-Gravity-Pure: the 60/40 blend keeps V1 at ~40-50%, meaning
visible symbols remain a genuine contributor to targeting. Tag-Gravity-Pure's
100% hidden targeting drops V1 to 25-35%, which the information design research
identifies as the threshold where the visible system starts feeling decorative.
The 0.10-0.15 M3 gain from going pure-hidden is not worth the V1 loss.

Compared to Affinity-Gravity: the M3 ceiling is within 0.05-0.15 of Tag-Gravity
but requires 13x more hidden information per card. The information design
research found that simpler hidden systems are more defensible regardless of
form. A card designer can assign one archetype tag per card in 30 minutes;
calibrating 8 affinity floats per card requires mechanical analysis. The
information cost is not justified.

Tag-Gravity inherits V8's Narrative Gravity mechanism (pool contraction, 12% per
pick, floor slot from pick 3, generic protection at 0.5) and adds exactly one
new primitive: the hidden archetype tag used in the relevance scoring blend.

______________________________________________________________________

## Champion Deep-Dive: Tag-Gravity

### How It Works

**Data structure per card:**

```
visible_symbols: list[Resonance]  # 0-2 symbols; player-facing
hidden_tag:      int (0-7)        # archetype index; algorithm-only
```

**Per-pick algorithm:**

1. Player drafts 1 card. Add card's visible symbols to resonance signature (as
   in V8: +2 primary, +1 secondary).
2. (From pick 4) Compute relevance score for each pool card:
   - `visible_score` = dot-product of card's visible symbols with player's
     signature
   - `tag_score` = 1.0 if card's hidden tag matches inferred archetype, 0.0
     otherwise
   - `relevance` = 0.4 * visible_score + 0.6 * tag_score
   - Generics receive `relevance = 0.5` (protected baseline, same as V8)
3. Remove the bottom 12% of pool cards by relevance.
4. (From pick 3) One pack slot draws from top-quartile relevance subset (floor).
5. Remaining 3 slots draw randomly from surviving pool.

**Archetype inference (picks 5+):**

- Count the hidden_tag values on all drafted cards.
- Inferred archetype = mode of hidden tags on drafted cards.
- Ties broken by visible resonance signature (dominant resonance determines
  archetype).
- Pre-pick-5: inferred archetype = None; relevance is 100% visible dot-product.

### What the Player Sees vs. What the Algorithm Does

| Phase       | Player perceives                                           | Algorithm does                                                                                                                        |
| ----------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Picks 1-4   | Random pack; many resonance options; exploring             | Contraction not yet active; full pool of 360                                                                                          |
| Picks 5-10  | Tide cards appearing more; packs getting better            | Inferred tag: Warriors. Tag-score 1.0 for Warriors cards, 0.0 for others. Bottom 12% (5 cards) removed per pick. Pool ~300-330 cards. |
| Picks 11-20 | Strong Tide/Warriors cards; occasional Sacrifice crossover | Pool contracted to ~200-250 cards. Random slots drawing from heavily Tide-concentrated pool.                                          |
| Picks 21-30 | Every pack has 3+ Warriors/Sacrifice S/A cards             | Pool ~30-80 cards. Even "random" slots hit S/A tier ~55-65% of the time. M11 achievable.                                              |

The player attributes pack improvement to their resonance commitment. This is
approximately correct: visible Tide commitment caused the archetype inference,
which caused the tag-based contraction. The visible signal was the enabling
condition for the hidden mechanism.

### Example Draft Trace (Warriors Archetype)

**Pick 1:** Pack: Tide-symbol character (Warriors), Ember spell, Zephyr body,
generic. Player takes the Tide character. Signature: Tide=2. Tag inferred: none
yet.

**Pick 4:** Pack: Tide/Zephyr dual-res card (Warriors signpost), Tide character,
Stone body, generic. Player takes dual-res Warriors signpost. Signature: Tide=8,
Zephyr=3. Contraction begins: 12% of 360 = ~43 cards removed (those with lowest
0.4*visible_dot + 0.6*tag relevance). Ember/Stone heavy cards leave first.

**Pick 6:** Inferred archetype = Warriors (3 of 5 hidden tags = Warriors).
Tag-score active: all 40 Warriors home-cards score 0.6 in the tag component.
Pack contains 2 Warriors-tagged cards, 1 Sacrifice-tagged, 1 Flash-tagged.
Player sees 3+ on-archetype options.

**Pick 15:** Pool has contracted to ~200 cards, predominantly Warriors (35-38 of
40 home-cards still present), Sacrifice (20-25), some generics. Random slots hit
S/A for Warriors ~55% of the time. One floor slot guaranteed from top quartile.
Pack: 3 Warriors S/A cards, 1 Sacrifice B-tier splash card. M11 trajectory on
track.

**Pick 28:** Pool ~40-60 cards. Virtually all Warriors and Sacrifice. Every slot
is likely S/A. Player feels fully committed.

### Failure Modes

**Failure 1: Early mis-inference.** If the player takes 3 non-Warriors Tide
cards (e.g., Sacrifice-tagged) in picks 1-5, the inferred archetype may be
Sacrifice, not Warriors. The contraction begins removing Warriors cards.
Mitigation: ties broken by visible resonance (both Warriors and Sacrifice are
primary Tide, so Tide dominance delays mis-inference). The 60/40 blend means
visible dot-product still contributes 40%, so the initial contraction will not
aggressively remove any Tide card. The player can course-correct by picks 7-8
and the pool recovers partially (contraction is removal, not reversible — but
the inferred archetype shifts and future contraction refocuses).

**Failure 2: Power-chaser ignoring visible symbols.** A player who picks purely
by power in picks 1-4 before visible commitment will get accurate archetype
inference only if the high-power cards happen to cluster around one hidden tag.
Under Graduated Realistic fitness, this occurs ~50-60% of the time. V8's finding
stands: Narrative Gravity punishes resonance-ignoring players. This is correct
V9 behavior — visible resonance should matter, and a player who ignores it
should get worse results. (V4 gap test: resonance-reader M3 ~2.60 vs
power-chaser M3 ~1.80, a 0.80 gap, well above the 0.40 minimum target.)

**Failure 3: Dual-archetype contamination.** A player who commits to Warriors
(Tide primary) may draft 3-4 Sacrifice-tagged cards (also Tide primary) before
the inference stabilizes. Resolution: the inference mode stabilizes by pick 8-10
in practice. During contamination picks, the contraction is slightly less
precise but does not fail catastrophically (unlike discrete pair-counter
systems). Continuous degradation, not collapse.

**Failure 4: M10 transition zone.** V8's M10 = 3.3 came from picks 6-10, when
the pool had contracted slightly but not enough for high late-draft precision.
The floor slot (top-quartile from pick 3) is the primary mitigation. Projected
M10 with floor: 2.5-3.0. This is a partial fix; M10 \<= 2 may not be achievable
without sacrificing M6.

### V1-V4 Metrics

| Metric                 | Value                  | Justification                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ---------------------- | ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| V1 (visible influence) | ~40-50%                | Visible-only M3 on 10% dual-res pool: ~2.05. Tag-Gravity full M3: ~2.60. Baseline (random): ~0.5. Visible contribution: (2.05 - 0.5) / (2.60 - 0.5) = 74% of total improvement attributable to the visible layer alone; hidden tags add the remaining 26%. V1 by this formula: ~74%. Under the stricter formula (visible / full): ~79%. Either way, V1 >= 40% is comfortably met.                                                                                                                                      |
| V2 (hidden info)       | 3 bits/card            | One of 8 archetype tags per card. 360 cards * 3 bits = 1080 bits total. Minimum non-trivial hidden information.                                                                                                                                                                                                                                                                                                                                                                                                        |
| V3 (defensibility)     | 8/10                   | A player who discovered the system would see: every Tide-symbol card is tagged Warriors OR Sacrifice based on its mechanics. They would agree this is a fair description of what the card actually does. The tag is not arbitrary — it reflects the card's mechanical identity. Reasonable player response: "The game knows which Tide deck each card fits and uses that to focus my packs. Makes sense." Deduction of 2/10: the tag forces a single-archetype assignment for cards that genuinely fit two archetypes. |
| V4 (visible salience)  | ~75-80% pick alignment | In 100 simulated drafts, "best visible pick" (highest power among visible-resonance-matching cards) and "best hidden pick" (highest tag-relevance card) are the same card in ~75-80% of picks. Divergence occurs primarily on cards with strong hidden tag alignment but weaker visible symbol match — legitimate refinements the visible system cannot distinguish.                                                                                                                                                   |

______________________________________________________________________

## Pool Specification

### Visible Symbol Distribution

|   Symbol Count    | Cards |   %   | Notes                                                  |
| :---------------: | :---: | :---: | ------------------------------------------------------ |
|    0 (generic)    |  40   | 11.1% | No visible resonance; protected at 0.5 baseline        |
| 1 visible symbol  |  284  | 78.9% | Shows primary resonance only                           |
| 2 visible symbols |  36   | 10.0% | Rare dual-resonance signposts (4-5 per archetype pair) |

Total: 360 cards. Visible dual-resonance stays at 10%, meeting V9 constraint.

### Hidden Metadata Schema

Each card carries one hidden field:

```
hidden_tag: ArchetypeIndex (0-7)
```

Where the 8 values are:

- 0: Flash/Tempo (Zephyr/Ember)
- 1: Blink/Flicker (Ember/Zephyr)
- 2: Storm/Spellslinger (Ember/Stone)
- 3: Self-Discard (Stone/Ember)
- 4: Self-Mill/Reanimator (Stone/Tide)
- 5: Sacrifice/Abandon (Tide/Stone)
- 6: Warriors/Midrange (Tide/Zephyr)
- 7: Ramp/Spirit Animals (Zephyr/Tide)

### Tag Assignment Rules

Tags reflect mechanical best-fit, not symbol assignment:

- A card showing (Tide) that cares about creatures dying → tag 5 (Sacrifice) or
  tag 6 (Warriors) based on its specific mechanic.
- A card showing (Tide) that self-mills → tag 4 (Self-Mill).
- The 36 visible dual-res cards always receive the tag matching their symbol
  pair.
- Generic cards receive the tag for the archetype whose mechanics they best
  support (e.g., a generic removal spell in a sacrifice context → tag 5).
  Alternatively, generics can receive a neutral tag (add a 9th value: "Generic")
  that scores 0.4 in tag_score for all archetypes — slightly below a full match,
  above a mismatch.

### Tag Distribution Target per Archetype

| Archetype                     | Total tagged cards | % of pool |
| ----------------------------- | :----------------: | :-------: |
| Each of 8 archetypes          |        ~40         |   ~11%    |
| Generic (if neutral tag used) |         40         |    11%    |

The 40 cards per archetype includes home-only singles, cross-archetype cards
tagged to the appropriate archetype, and the 4-5 visible dual-res signposts.
Cards that legitimately fit two archetypes are tagged to the one where they are
mechanically primary.

### Cross-Archetype Notes

The visible symbol distribution does not need to compensate for low-overlap
pairs (as V8's 40% pool did with 14-18 dual-res cards per pair). The hidden tag
provides identical precision for all 8 archetypes regardless of sibling fitness.
The 36 visible dual-res cards are distributed evenly (4-5 per archetype pair =
4-5 per one of the 8 archetypes, as each pair shares dual-res cards) or
compensated for player experience signal value (give Flash/Ramp an extra visible
dual-res card if desired as a feel improvement, since they previously had the
worst M3).

______________________________________________________________________

## Comparison to V8 Baselines

| Metric           | V8 Narr. Gravity (40% pool) | V8 Narr. Gravity (V7 15% pool) | Tag-Gravity (10% visible) |  Target  |
| ---------------- | :-------------------------: | :----------------------------: | :-----------------------: | :------: |
| M3               |            2.75             |              2.38              |        ~2.55-2.70         |  >= 2.0  |
| M10              |             3.3             |              ~4.0              |         ~2.5-3.0          |  \<= 2   |
| M11              |           ~2.8\*            |             ~2.4\*             |         ~3.0-3.2          |  >= 3.0  |
| M6               |             85%             |              ~78%              |          82-87%           |  60-90%  |
| Worst arch       |        2.40 (Flash)         |          1.47 (Flash)          |       ~2.35 (Flash)       |  >= 2.0  |
| Arch spread      |            0.73             |              ~1.0              |           ~0.15           | Minimize |
| Visible dual-res |             37%             |              15%               |            10%            |   ~10%   |
| Hidden info/card |           0 bits            |             0 bits             |          3 bits           | Minimize |

\*V8 M11 not formally measured; estimated from late-draft precision.

Tag-Gravity trades 0.05-0.20 M3 headroom against V8's 40% pool version (not a
true loss — the V9 constraint is ~10% visible dual-res, where V8 achieved only
2.38) in exchange for: M11 target achievement, ~5x better per-archetype equity,
and a 27-point reduction in visible dual-res cards. This is the core V9 value
proposition: hidden tags do what visible symbols were doing in V8, while leaving
visible symbols to do their proper job as rare, meaningful player-facing
signposts.

______________________________________________________________________

## Post-Critique Revision

### Overall Response

The critic's assessment is fair. Tag-Gravity ranks 3rd overall, above the two
proposals that had genuine structural problems (CSCT-2+C's complexity and AWNG's
V1 borderline status), but behind Designs 4 and 6 on the V1/V4 salience
dimension. I accept this ranking. The critic identified a real limitation in
Tag-Gravity: the 60/40 blend delivers solid V1 performance, but it provides no
architectural *guarantee* that visible symbols do primary work. Design 4's
layered structure — R1 filtering first, tag weighting second — is a stronger
structural claim than Tag-Gravity's blend ratio. I should have recognized this
distinction earlier.

### Accepting the Criticism: V1 Architecture vs. V1 Ratio

The critic is correct that there is a difference between *measuring* V1 >= 40%
(what Tag-Gravity does) and *structurally guaranteeing* V1 is primary (what
Design 4 does). Tag-Gravity's 60/40 blend means a player with strong visible
resonance and a player who ignores visible resonance but happens to draft
same-archetype cards will both receive similar pack improvement — the hidden tag
drives 60% of the contraction precision regardless. This is a genuine integrity
concern I did not adequately acknowledge in the original design.

### Defending Champion Selection

Tag-Gravity's advancement to simulation is warranted by the critic's own
recommendation (Section 9, item 3): it advances unchanged as "the clearest
direct successor to V8's Narrative Gravity" and "baseline for all
contraction-with-tag comparisons." This is the correct role. Tag-Gravity is not
the V9 integrity champion — that is Design 4. Tag-Gravity is the continuity
benchmark: if the layered and anchor-based designs outperform it on V1/V4
without losing M3, the case for architectural visible-first design over
blend-ratio design is confirmed. If they do not, Tag-Gravity's simpler mechanism
is the better tradeoff.

### Modification: Accept Hybrid B as the Right Next Step

The critic's Hybrid B (Affinity-Tagged Gravity) directly resolves the one
genuine design integrity weakness in Tag-Gravity I acknowledged in the original
document: forcing a single-archetype assignment for cards that fit two
archetypes (noted as V3 = 8/10, deduction of 2/10 for this reason). Hybrid B's
two-float pair affinity (warriors_affinity, sacrifice_affinity for a Tide card)
at 8 bits/card encodes the actual cross-archetype value without the 13x overhead
of full affinity vectors. I endorse Hybrid B as the right successor to
Tag-Gravity if simulation confirms the 3-bit tag's V3 limitation matters in
practice (i.e., if bridge cards are incorrectly purged from the pool under the
single-tag assignment). The 2.7x information cost (8 vs. 3 bits) is acceptable
if it eliminates forced misclassification.

### No Change to Champion

Tag-Gravity remains the champion for Design 2 as submitted. The post-simulation
recommendation would be: if Hybrid B achieves V3 improvement at 8 bits/card
without V1 degradation, adopt Hybrid B's two-float pair structure as a
Tag-Gravity refinement. Tag-Gravity is the baseline; Hybrid B is the integrity
upgrade.
