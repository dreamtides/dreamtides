# Domain 5: Conditional Pack Enhancement -- Round 1 Design (V6)

## Key Takeaways

- **V5's pair matching is dead under the 15% constraint.** V5's Pair Cluster
  Bonus relied on ~60% of cards being pair-eligible (2+ symbols with 2 different
  resonance types). With only 54 dual-type cards in V6, there are roughly 6-7
  dual-type cards per archetype. A random 4-card pack almost never contains 2
  cards sharing an ordered pair. Conditional triggers based on pair matching
  cannot fire frequently enough to matter.

- **V4 proved single-resonance triggers cap at ~1.7 S/A.** Any trigger
  condition based on "does this pack cluster with my resonance" inherits the
  50% archetype dilution problem -- half of resonance-matched cards belong to
  wrong archetypes. To cross 2.0, the bonus card selection must be smarter than
  the trigger condition, or the trigger must fire often enough to compensate
  with volume.

- **The addition mechanism is what breaks the ceiling, not the condition.**
  V4's Pack Widening crossed 2.0 by adding cards, not by being clever about
  when to add them. For conditional enhancement to work under V6, the
  enhancement itself (what gets added) must be high-precision, even if the
  trigger condition is low-precision.

- **Conditional triggers provide the best natural variance.** The organic
  fire/no-fire pattern produces genuine pack-to-pack fluctuation that
  deterministic slot locking and auto-widening cannot match. This is the domain's
  unique advantage.

- **Multi-card bonus or multi-condition triggers are needed to reach 2.0.** The
  math from the orchestration plan is instructive: at 60% fire rate with 1 bonus
  card (~50% S/A due to resonance dilution), the enhancement adds only ~0.3 S/A
  on top of the ~1.5 base. Reaching 2.0 requires either (a) adding 2 bonus
  cards per trigger, (b) firing on ~90%+ of packs (becoming unconditional), or
  (c) using the 54 dual-type cards as a targeted bonus pool.

- **The 54 dual-type cards are a precision resource.** Under V6, dual-type
  cards are rare but highly archetype-specific (ordered pair maps to exactly 1
  archetype). Drawing bonus cards exclusively from the dual-type pool gives
  ~95% archetype precision at the cost of a tiny bonus pool (~54 cards). This
  is the key V6-specific insight for conditional enhancement.

- **Layered triggers can combine resonance conditions (high fire rate) with
  dual-type bonus pools (high precision).** The trigger detects resonance
  clustering (easy to hit), and the bonus draws from the dual-type subset
  (precise for archetype). This separation is the core design pattern.

---

## Algorithm 1: Resonance Cluster Add

**One-sentence description:** Draw 4 random cards; if 2 or more share a
primary resonance with your highest-count resonance, add 1 random card of
that resonance to the pack.

**Technical description:** Track the player's weighted resonance counts
(primary=2, secondary/tertiary=1). For each pack, draw 4 random cards. Count
how many have a primary resonance matching the player's top resonance. If 2+
match, draw 1 additional card whose primary resonance matches and add it to
the pack (5 cards total). Player picks 1.

**Goal assessment:**
- Hits: Simplicity (1), No extra actions (2), Variance (natural 4/5 split),
  Open early (8, trigger rare before commitment).
- Misses: Convergence (6) -- single-resonance bonus inherits 50% dilution.
  At ~45% fire rate post-commitment, adds ~0.23 S/A, reaching ~1.73 total.
  Below 2.0.

**Symbol distribution:** 20% 1-symbol, 55% 2-symbol, 25% 3-symbol.

---

## Algorithm 2: Dual-Type Precision Bonus

**One-sentence description:** Draw 4 random cards; if 2 or more share a
primary resonance with your top resonance, add 1 card drawn exclusively from
the dual-type pool matching your top two resonances.

**Technical description:** Track weighted resonance counts. Identify the
player's top resonance and second resonance. When a pack's random 4 cards
contain 2+ sharing the player's top primary resonance, draw 1 bonus card from
the subset of dual-type cards whose ordered pair is (top resonance, second
resonance). This dual-type card maps to exactly 1 archetype with ~95%
precision. The trigger uses resonance (easy to hit, ~45-55% fire rate) while
the bonus uses the dual-type pool (high precision).

**Goal assessment:**
- Hits: Simplicity (1, one sentence), No extra actions (2), Convergence (6,
  bonus cards are ~95% S/A), Variance (4/5 pack split).
- Misses: The dual-type bonus pool is small (~6-7 cards per archetype). With
  replacement, this works but the player may see repeated bonus cards. Fire
  rate at ~50% and ~95% precision yields ~0.48 S/A from bonuses, total ~1.98.
  Still borderline.

**Symbol distribution:** 15% 1-symbol, 60% 2-symbol, 25% 3-symbol. All 54
dual-type cards allocated evenly (6-7 per archetype).

---

## Algorithm 3: Double Enhancement

**One-sentence description:** Draw 4 random cards; if 2 or more share a
primary resonance with your top resonance, add 2 cards of that resonance to
the pack.

**Technical description:** Same trigger as Algorithm 1 (2-of-4 primary
resonance match), but the enhancement adds 2 bonus cards instead of 1, both
drawn randomly from cards whose primary resonance matches the player's top.
This produces packs of either 4 (no trigger) or 6 (trigger fires). The higher
bonus count compensates for resonance-level dilution: 2 bonus cards at ~50%
archetype precision yields ~1.0 expected S/A per trigger.

**Goal assessment:**
- Hits: Convergence (6, at ~50% fire rate post-commitment: 0.50 x 1.0 = +0.50
  S/A, total ~2.0), No extra actions (2), Variance (4/6 split creates dramatic
  pack-to-pack variation).
- Misses: Simplicity (1, borderline -- 6-card packs feel unusual), Not on
  rails (3, 6-card packs with 3+ on-archetype cards may feel forced),
  Splashable (7, 6-card packs are archetype-heavy when trigger fires).

**Symbol distribution:** 20% 1-symbol, 55% 2-symbol, 25% 3-symbol.

---

## Algorithm 4: Replace-Worst Enhancement

**One-sentence description:** Draw 4 random cards; if 2 or more share a
primary resonance with your top resonance, replace the card with the fewest
matching symbols with a random card of your top resonance.

**Technical description:** Same trigger condition (2-of-4 primary resonance
match). Instead of adding a bonus card, identify the card in the pack whose
symbols have the least overlap with the player's resonance profile and replace
it with a card whose primary resonance matches the player's top. Pack size
stays at 4. The replacement increases the proportion of on-resonance cards
from 2/4 to 3/4 without changing pack size.

**Goal assessment:**
- Hits: Simplicity (1), No extra actions (2), Pack size consistency (always 4).
- Misses: Convergence (6, replacing 1 card with a ~50% S/A card adds ~0.25
  effective S/A per trigger, total ~1.75. Below 2.0), Splashable (7, the
  replaced card was the best splash candidate), Variance (less dramatic than
  add-based approaches since pack size is constant).

**Symbol distribution:** 20% 1-symbol, 55% 2-symbol, 25% 3-symbol.

---

## Algorithm 5: Cascading Resonance Enhancement

**One-sentence description:** Draw 4 random cards; for each card whose primary
resonance matches your top resonance, roll a 40% chance to add 1 bonus card
of your top two resonances' dual-type pool.

**Technical description:** Track the player's weighted resonance counts.
Identify the player's top and second resonance. For each pack, draw 4 random
cards. For each of the 4 cards whose primary resonance matches the player's
top resonance, independently roll a 40% chance. On success, draw 1 card from
the dual-type pool matching the ordered pair (top, second). Pack size ranges
from 4 to 8 in theory, but typically 4-6. Each bonus card drawn from the
dual-type pool has ~95% archetype precision. At steady state with ~2 matching
cards per pack and 40% per-card trigger: ~0.8 bonus cards per pack, ~0.76 S/A
added, total ~2.26.

**Goal assessment:**
- Hits: Convergence (6, reliably crosses 2.0), No extra actions (2), Variance
  (variable bonus count 0-3 creates organic fluctuation).
- Misses: Simplicity (1, per-card probability roll is harder to explain than
  a threshold check), Not on rails (3, occasionally 7-8 card packs dominate),
  the dual-type pool constraint (only 6-7 cards per archetype means repetition).

**Symbol distribution:** 15% 1-symbol, 60% 2-symbol, 25% 3-symbol. Maximize
dual-type allocation at 54 cards (full 15%).

---

## Champion Selection: Algorithm 5 -- Cascading Resonance Enhancement

**Why Cascading Resonance Enhancement wins:**

The core problem for conditional enhancement under V6 is crossing 2.0 S/A
without pair matching as a primary tool. Each algorithm above faces this
tension differently:

- Algorithms 1 and 4 cannot cross 2.0: single-resonance bonuses and
  replacement both inherit the 50% dilution ceiling, and the conditional
  trigger's sub-100% fire rate makes it worse.
- Algorithm 2 is tantalizingly close (1.98) but depends on a tiny dual-type
  pool and still falls short.
- Algorithm 3 crosses 2.0 via brute force (2 bonus cards) but creates awkward
  4/6 pack splits and feels heavy-handed.
- Algorithm 5 crosses 2.0 (projected ~2.26) by combining two V6-compatible
  insights: (a) use the per-card resonance match as a granular trigger that
  scales naturally with commitment, and (b) draw bonuses from the dual-type
  pool for ~95% archetype precision.

The per-card trigger is the key innovation. Rather than a binary
pack-level condition (2-of-4 match or not), each on-resonance card in the
base pack independently has a chance to spawn a precision bonus. This creates
a smooth distribution: packs with 0 on-resonance cards get 0 bonuses (fully
random), packs with 1 get 0-1, packs with 2 get 0-2, etc. The fire rate
scales proportionally with how resonance-dense the random pack happens to be,
producing organic variance that is impossible with a binary trigger.

The dual-type bonus pool solves the precision problem. Under V6's 15%
constraint, there are ~54 dual-type cards. Drawing bonuses exclusively from
this pool achieves ~95% S/A precision for the target archetype. The pool is
small (6-7 per archetype), so bonus cards are drawn with replacement,
meaning the player may see the same dual-type card offered multiple times.
This is acceptable: dual-type cards are rare and powerful, so seeing them
repeatedly reinforces the archetype identity rather than feeling repetitive.

**One-sentence:** "Draw 4 random cards; for each card whose primary resonance
matches your top resonance, roll a 40% chance to add 1 bonus card from the
dual-type cards matching your top two resonances."

---

## Champion Deep-Dive: Cascading Resonance Enhancement

### Example Draft Sequences

**Early committer (commits to Warriors/Tide-Zephyr by pick 5):**

- **Picks 1-3:** Drafts 2 Tide cards, 1 generic. Weighted Tide count = ~5
  (assuming 2-symbol cards giving primary=2 + secondary=1). Tide is top
  resonance. Random packs contain ~1.0 Tide-primary cards on average (Tide
  primary is ~80/360 of pool). Per-card trigger: 40% each. Expected bonuses
  per pack: 1.0 x 0.4 = 0.4. Most packs remain 4 cards; occasionally 5.
  Bonuses are drawn from (Tide, Zephyr) dual-type pool = ~7 cards, ~95% S/A
  for Warriors. Early packs feel mostly random with occasional pleasant
  surprises.

- **Picks 4-6:** Commits harder. Tide count grows. Packs still average ~1.0
  Tide-primary cards (pool hasn't changed). Expected bonuses still ~0.4 per
  pack. But the base pack also starts contributing: the random 1.0
  Tide-primary card has ~50% chance of being Warriors S/A = 0.5 base S/A, plus
  ~0.38 from bonuses (0.4 x 0.95). Total ~0.88 + random other S/A ~0.7 =
  ~1.58 S/A. Not yet at 2.0 but climbing.

- **Picks 7-15:** The player has been removing non-Tide cards from the pool
  and the remaining random draws include Tide-primary cards at slightly higher
  rates as other resonance cards are depleted. Typical pack has 1-2
  Tide-primary cards. Expected bonuses: 1.3 x 0.4 = 0.52 per pack, each at
  ~95% S/A = 0.49 bonus S/A. Base S/A (random Tide cards that are Warriors):
  ~0.65. Other random S/A: ~0.55. Total: ~1.69 S/A. With variance, many packs
  hit 2+ but the average is below 2.0.

**Observation:** Even with 40% per-card probability and dual-type precision,
the average is below 2.0. The problem: in a 360-card pool, Tide-primary cards
are ~22% of the pool, so only ~0.9 of 4 random cards are Tide-primary on
average. This limits trigger opportunities.

**Adjustment needed:** Increasing the per-card probability to 60% would yield
~1.3 x 0.6 = 0.78 bonuses per pack, adding ~0.74 S/A. With base ~1.2 S/A,
total reaches ~1.94. Still borderline. At 80%: ~1.3 x 0.8 = 1.04 bonuses,
adding ~0.99 S/A, total ~2.19. This works but 80% feels near-deterministic.

**Alternative: count all resonance matches, not just primary.** If we trigger
on any card that has the player's top resonance anywhere in its symbols (not
just as primary), the base rate rises to ~35% of the pool (primary for 2
archetypes, secondary/tertiary for 2 more). Expected matching cards per pack:
~1.4. At 50% per-card probability: 0.7 bonuses per pack, ~0.67 S/A from
bonuses, total ~1.87. At 60%: 0.84 bonuses, ~0.80 S/A, total ~2.0. This
crosses the threshold.

**Revised one-sentence:** "Draw 4 random cards; for each card containing your
top resonance in any symbol position, roll a 60% chance to add 1 bonus card
from the dual-type cards matching your top two resonances."

**Signal-reader (notices Tide appearing frequently):**

- Picks 1-5: Sees Tide cards slightly more often in packs (pool asymmetry or
  random variance). Drafts 2 Tide cards, 1 Zephyr, 2 open. Tide becomes top
  resonance. Trigger fires occasionally (0-1 bonus per pack). Signal is subtle
  but present: enhanced packs suggest "the system likes that I'm drafting Tide."
- Picks 6-10: Commits to Tide-based archetype. Enhancement fires more often.
  The bonus cards being dual-type (Tide, Zephyr) or (Tide, Stone) reveal which
  specific archetype the system is matching. This provides indirect signal
  reading -- the bonus cards themselves hint at the archetype.

### Failure Modes

1. **Under-convergence with conservative parameters.** If the per-card
   probability is too low (30-40%) or the resonance match rate is too narrow
   (primary-only), the algorithm adds insufficient S/A to cross 2.0. This is
   the primary risk and the reason simulation must test 40%/50%/60%/80%.

2. **Dual-type pool exhaustion feels repetitive.** With only 6-7 dual-type
   cards per archetype, players see the same bonus cards repeatedly. This is
   a presentation issue, not a mechanical one. Mitigation: mix dual-type and
   mono-type bonus cards (e.g., 70% dual-type, 30% mono-type of top
   resonance) to increase variety at the cost of some precision.

3. **Variable pack size is confusing.** Packs range from 4 to 7+ cards. Most
   players expect consistent pack sizes. Mitigation: cap bonus cards at 2 per
   pack (pack size 4-6 only). This barely affects the math since 3+ bonuses
   are rare.

4. **Early noise from accidental triggers.** A player's first pick
   establishes a "top resonance," causing triggers on pick 2 even before any
   commitment. Mitigation: require minimum 4 weighted symbols in the top
   resonance before the trigger activates. This delays activation to pick 2-3
   for focused drafters.

### Parameter Variants to Test

**Variant A: Conservative (primary-only trigger, 50% roll, dual-type bonus)**
- Trigger: each card whose primary resonance = player's top resonance
- Roll probability: 50%
- Bonus pool: dual-type cards of (top, second) resonance
- Activation threshold: 4 weighted symbols
- Expected S/A: ~1.75-1.85. May not cross 2.0. Tests the precision-vs-volume
  tradeoff.

**Variant B: Standard (any-symbol trigger, 60% roll, dual-type bonus)**
- Trigger: each card containing player's top resonance in any symbol position
- Roll probability: 60%
- Bonus pool: dual-type cards of (top, second) resonance
- Activation threshold: 4 weighted symbols
- Bonus cap: 2 per pack
- Expected S/A: ~2.0-2.2. The target variant.

**Variant C: Aggressive (any-symbol trigger, 60% roll, mixed bonus pool)**
- Trigger: each card containing player's top resonance in any symbol position
- Roll probability: 60%
- Bonus pool: 60% from dual-type (top, second), 40% from mono-type (top
  resonance primary)
- Activation threshold: 4 weighted symbols
- Bonus cap: 2 per pack
- Expected S/A: ~2.1-2.3 (higher fire rate from larger bonus pool, ~75%
  precision instead of ~95%). Tests whether variety in bonus cards matters more
  than precision.

### Proposed Symbol Distribution

| Symbol Count | % of Non-Generic | Cards | Notes |
|---|---|---|---|
| 0 (generic) | -- | 36 | ~10% of total pool |
| 1 symbol | 15% | 49 | Minimal; invisible to pair-based bonus pool |
| 2 symbols (mono) | 45% | 146 | e.g., [Tide, Tide] -- contributes to resonance triggers |
| 2 symbols (dual) | 12% | 39 | e.g., [Tide, Zephyr] -- part of the 54 dual-type cap |
| 3 symbols (mono) | 15% | 49 | e.g., [Tide, Tide, Tide] -- heavy resonance weight |
| 3 symbols (dual) | 13% | 5 per arch (40 total) | e.g., [Tide, Tide, Zephyr] -- dual-type, high symbol weight |
| **Dual-type total** | | **54** (39 + ~15) | **Exactly 15% cap** |

**Justification:** The algorithm needs two things from the card pool: (1) high
resonance density in random packs to trigger frequently, and (2) a healthy
dual-type bonus pool for precision. The 60% 2+ symbol allocation ensures most
random packs contain multiple cards with the player's top resonance. The full
54 dual-type allocation maximizes the precision bonus pool (~7 dual-type cards
per archetype). Distributing dual-type cards as ~5 with 2 symbols and ~2 with
3 symbols per archetype gives each archetype a reasonable bonus pool while
keeping most cards mono-type for the resonance ambiguity that makes drafting
interesting.

The 15% 1-symbol allocation ensures some "slow" picks that do not advance the
trigger, maintaining natural variance in convergence speed.
