# Domain 5: Pool Manipulation Mechanisms

## Key Takeaways

- **Pool manipulation is inherently transparent.** Changing what cards exist
  (rather than sampling weights) produces observable frequency shifts -- players
  notice "I keep seeing Tide cards" without understanding hidden weights.
- **Additive manipulation converges more reliably than subtractive.** Adding
  cards of your drafted resonance increases future frequency naturally; removing
  undrafted cards creates depletion problems V2 already identified.
- **Constant-size pool mechanisms avoid probability dilution.** Algorithms that
  grow the pool dilute existing cards, weakening convergence. Swap mechanisms
  (remove one, add one) keep density predictable.
- **Pool mechanisms create natural signal reading.** If the starting pool is
  asymmetric, players see this directly in early packs without special
  detection heuristics.
- **The critical tuning parameter is swaps per pick.** Too few and convergence
  is too slow; too many and the pool becomes mono-resonance by mid-draft.
- **Card-level duplication is the wrong granularity.** Copying exact drafted
  cards creates "rich get richer" loops; resonance-level pool shifts produce
  broader archetype support.

---

## Proposal 1: Resonance Reinforcement

**Description:** "When you draft a card, 3 random cards from a reserve that
share its primary resonance are added to the draft pool."

**Technical:** A ~120-card reserve (30 per resonance) exists alongside the
360-card active pool. Drafting a card with primary resonance R moves 3
R-primary reserve cards into the active pool. Packs are 4 random cards from
the active pool.

**Assessment:** Very simple (9/10). Convergence is slow -- adding 3 to a
360-card pool shifts probability ~0.8% per pick. After 10 Tide picks, pool
has grown to 390 with ~120 Tide cards (30.7%, up from 25%). Splashable since
originals remain. Fails convergence speed target without a very large reserve.

**Best symbol distribution:** Mostly 2-symbol [Primary, Secondary].

---

## Proposal 2: Pool Filtration

**Description:** "Before each pack, temporarily remove all cards whose primary
resonance matches a resonance you have zero drafted symbols in, then draw 4."

**Technical:** Track symbol counts per resonance. Before each pack, filter out
cards whose primary resonance the player has zero accumulated symbols in. Draw
4 from filtered pool. No filtering before first pick.

**Assessment:** Strong convergence (9/10) but catastrophically fails flexibility
(3/10). After 3 mono-resonance picks, up to 75% of the pool is filtered away.
Pivoting becomes impossible. Too aggressive -- creates a cliff rather than a
gradient.

**Best symbol distribution:** 3-symbol cards to slow filtration by spreading
symbols across more resonances.

---

## Proposal 3: Drafted Resonance Duplication

**Description:** "After each pick, for every 3 resonance symbols you have in
a type, duplicate one random card of that type in the pool."

**Technical:** Maintain running symbol totals (primary=2, secondary/tertiary=1).
After each pick, for each resonance R, add floor(total_R / 3) copies of random
R-primary cards to the pool. Pool grows over time; heavily-drafted resonances
grow faster due to quadratic accumulation.

**Assessment:** Clear threshold mechanic (8/10 simplicity). Good convergence
(8/10). Risk of exponential pool growth -- a committed player at pick 15 might
trigger 6+ duplications per pick, overwhelming the pool with one resonance and
failing splashability.

**Best symbol distribution:** 2-symbol [Primary, Secondary] for moderate
accumulation speed.

---

## Proposal 4: Resonance Swap

**Description:** "When you draft a card, replace 2 random cards in the pool
that don't share its primary resonance with 2 reserve cards that do."

**Technical:** A ~200-card reserve (50 per resonance) exists alongside the
360-card pool. Drafting a card with primary resonance R removes 2 random
non-R-primary cards from the pool and adds 2 R-primary cards from the reserve.
Pool size stays at 360. Packs are 4 random cards from the pool.

**Assessment:** Simple (8/10) -- "replace 2 non-matching with 2 matching" is
one concept. Strong convergence (8/10) from the double effect (add matching +
remove non-matching). Good signal reading from starting pool asymmetries.
Moderate splashability risk (6/10) since removal actively reduces off-resonance
cards. Pivoting is possible but costly.

**Best symbol distribution:** 2-symbol [Primary, Secondary].

---

## Proposal 5: Resonance Seeding

**Description:** "When you draft a card, add 2 copies of that exact card back
to the pool."

**Technical:** Standard 360-card pool. After each pick, 2 copies of the picked
card are added. Pool grows by 1 per pick (1 removed by drafting, 2 added).
Over time, drafted cards become increasingly common.

**Assessment:** Maximum simplicity (10/10). But convergence operates at
card-level, not resonance-level -- drafting 5 different Tide cards seeds 5
specific cards, not "Tide" broadly. Creates erratic resonance representation
and "rich get richer" loops where powerful cards dominate. Fails convergence
at the archetype level (6/10).

**Best symbol distribution:** Irrelevant -- algorithm copies exact cards.

---

## Champion Selection: Proposal 4 -- Resonance Swap

Resonance Swap wins for three reasons:

1. **Constant pool size.** No probability dilution from pool growth. After 15
   picks, probabilities are calculated against exactly 360 cards, making the
   system predictable.
2. **Double convergence effect.** Adding matching cards AND removing
   non-matching creates stronger probability shifts per pick than pure
   addition, achieving target convergence with only 2 swaps per pick.
3. **Appropriate pivot cost.** A player who committed to Ember for 10 picks
   and pivots to Tide faces a degraded Tide pool (~80 cards instead of ~90)
   but can rebuild. Pivoting is costly but not impossible.

I chose this over Reinforcement (Proposal 1) because constant pool size avoids
dilution, and over Seeding (Proposal 5) because resonance-level manipulation
produces broader archetype support than card-level copying.

---

## Champion Deep-Dive: Resonance Swap

### Example: Early Committer (Tide/Zephyr Warriors)

- **Picks 1-5:** Takes Tide-primary cards. Each pick swaps 2 non-Tide cards
  out and 2 Tide-reserve cards in. After 5 picks: pool has ~100 Tide-primary
  cards of 360 (27.8%, up from ~25%). Subtle but growing.
- **Picks 6-15:** Continued Tide commitment. After 15 picks: ~30 swaps total,
  pool has ~120 Tide-primary (33.3%). Expected Tide-primary cards per 4-card
  pack: ~1.33, plus Tide-secondary cards. Convergence target of 2+ fitting
  cards per pack is reachable.
- **Picks 16-30:** Pool reaches ~140-150 Tide-primary (39-42%). Strong
  convergence, with 1.5-1.7 Tide-primary cards per pack on average.

### Example: Flexible Player (explores through pick 8)

- **Picks 1-8:** Drafts across Ember, Tide, Stone. Swaps partially cancel
  (adding Tide sometimes removes a previously-added Ember card). Pool stays
  close to original distribution.
- **Picks 9-30:** Commits to Stone. With 22 remaining picks (~16 Stone-focused),
  ~32 swaps toward Stone. Pool ends with ~108 Stone-primary (30%). Convergence
  is weaker than early committer but still meaningful.

### Example: Pivot at Pick 12 (Ember to Tide)

- **Picks 1-11:** Ember-focused. ~22 swaps toward Ember. Pool: ~112 Ember,
  ~68 Tide (some Tide cards swapped out).
- **Picks 12-30:** Switches to Tide. ~26 swaps toward Tide. Tide recovers to
  ~90 cards. Pivot works but the player ends with a worse Tide pool than
  someone who committed from pick 1. Appropriate cost.

### Predicted Failure Modes

1. **Splashability erosion.** After 20+ committed picks, ~40 off-resonance
   cards have been removed. The 0.5+ off-archetype cards per pack target may
   fail in late picks. Mitigation: removed cards could recycle into other
   resonance reserves.
2. **Reserve exhaustion.** If all 50 reserve cards of one resonance are used,
   further swaps cannot add new cards. Need either larger reserves or recycling
   removed cards into the reserve of their resonance.
3. **Secondary resonance neglect.** The algorithm keys off primary resonance
   only. A Warriors player drafting [Tide, Zephyr] cards swaps in Tide cards,
   not Zephyr. Secondary resonance benefits only if the player also drafts
   Zephyr-primary cards.

### Parameter Variants for Testing

1. **Swap count: 1 vs 2 vs 3 per pick.** 1 swap: 30 total over a draft (8%
   pool turnover). 3 swaps: 90 total (25% turnover). Recommend testing all
   three to find the convergence sweet spot.
2. **Reserve size: 100 vs 200 vs 300.** Smaller reserves cap convergence via
   exhaustion; larger reserves risk over-concentration. Start with 200.
3. **Removal targeting: random vs least-drafted resonance first.** Random is
   simpler. Targeted removal from the player's least-drafted resonance
   accelerates convergence. Test whether the difference is meaningful.

### Proposed Symbol Distribution

- **1-symbol:** 30% (~97 cards). [Primary] only.
- **2-symbol:** 50% (~162 cards). [Primary, Secondary].
- **3-symbol:** 20% (~65 cards). [Primary, Primary, Secondary] or [Primary,
  Secondary, Tertiary].
- **Generic:** 36 cards (10%, 0 symbols).

The 50% 2-symbol concentration ensures most picks provide a clear primary
resonance signal for the swap while also accumulating secondary resonance
for archetype-level tracking.
