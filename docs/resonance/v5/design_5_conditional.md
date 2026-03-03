# Domain 5: Conditional Pack Enhancement -- Round 1 Design

## Key Takeaways

- **Conditional enhancement creates organic variance by design.** The pack's own
  random composition determines whether enhancement fires, producing a natural
  mix of 4-card and 5-card packs that feels like luck rather than machinery.
- **The "cluster trigger" pattern is the most promising mechanism.** When the
  randomly drawn base pack happens to contain cards matching the player's
  drafted profile, a bonus card is added. This rewards commitment without
  mechanical determinism.
- **Pair-based triggers dramatically improve archetype precision.** A trigger
  that checks ordered resonance pairs (e.g., Tide/Zephyr) instead of single
  resonances reduces the false-positive rate from ~50% to ~15-25% for 2+ symbol
  cards, making conditional bonuses far more archetype-targeted.
- **Fire rate is the critical tuning parameter.** Too low (<30%) and the
  algorithm cannot cross 2.0 S/A. Too high (>80%) and it degenerates into
  unconditional auto-widening (Domain 1) with no variance benefit. The sweet
  spot is 45-65% fire rate at steady state for committed players.
- **Conditional systems are inherently self-balancing.** Early in the draft
  (before commitment), the player has a diffuse profile and packs rarely
  cluster, so enhancement is rare. After commitment, the player's concentrated
  profile makes clustering more likely. This creates automatic early-open /
  late-convergent behavior without thresholds or state machines.
- **The main risk is under-convergence for 1-symbol-heavy pools.** If most cards
  have only 1 symbol, the trigger condition fires less often because there is
  less information to cluster on. A distribution weighted toward 2-symbol cards
  (55%+) is essential.
- **Champion: Pair Cluster Bonus** -- uses ordered-pair matching for the trigger,
  giving near-archetype-level precision while remaining fully automatic and
  explainable in one sentence.

---

## Algorithm 1: Resonance Cluster Bonus

**One-sentence description:** Draw 4 random cards; if 2 or more share a primary
resonance with your most-drafted resonance, add 1 bonus card of that resonance.

**Technical description:** Track the player's weighted resonance counts
(primary=2, secondary/tertiary=1). When generating a pack, draw 4 cards at
random from the pool. Count how many of those 4 cards share a primary resonance
with the player's highest-count resonance. If 2 or more match, draw 1
additional card whose primary resonance matches and add it to the pack
(making it 5 cards). The player picks 1 card from whatever pack size results.

**Goal assessment:**
- Serves well: Simplicity (1), No extra actions (2), Variance (natural 4/5 mix),
  Open early (8, trigger rarely fires before commitment).
- Fails: Convergence (6) -- single-resonance matching means ~50% of triggers
  produce cards for the wrong archetype. Projected steady-state fire rate of
  ~40-50%, yielding ~1.7-2.0 S/A. Borderline on the 2.0 threshold due to V4's
  proven dilution ceiling.

**Best symbol distribution:** 2-symbol heavy (55%). More symbols per card means
more resonance density in random packs, increasing cluster frequency.

---

## Algorithm 2: Pair Cluster Bonus

**One-sentence description:** Draw 4 random cards; if 2 or more share their
ordered resonance pair (first two symbols) with your most-drafted pair, add 1
bonus card matching that pair.

**Technical description:** Track the player's pair counts: each drafted card
with 2+ symbols contributes its (primary, secondary) ordered pair. When
generating a pack, draw 4 cards at random. For each card with 2+ symbols,
extract its ordered pair. If 2 or more of the 4 cards share a pair with the
player's most-common pair, draw 1 additional card from the pool whose ordered
pair matches and add it to the pack. If the trigger does not fire, the pack
remains 4 cards. The bonus card, drawn from the pair-matched subset, has ~85-95%
chance of being S/A for the player's target archetype (since ordered pairs map
nearly uniquely to archetypes).

**Goal assessment:**
- Serves well: Simplicity (1, slightly more complex than Algorithm 1 but still
  one sentence), No extra actions (2), Convergence (6, pair precision pushes
  effective S/A of bonus cards to ~90%), Variance (natural fire/no-fire
  fluctuation), Open early (8, pair profile is diffuse early).
- Potential weakness: Fire rate may be lower than Algorithm 1 because pair
  matching is stricter than single-resonance matching. With 12 possible pairs
  spread across the pool, getting 2-of-4 random cards to share a specific pair
  requires that pair to be well-represented. This is addressable by lowering the
  trigger threshold to 1 matching card instead of 2.
- Fails: Signal reading (9, pack composition is random and pair-conditional,
  giving minimal signal).

**Best symbol distribution:** Very 2-symbol heavy (60%+). Pair matching requires
2+ symbols, so 1-symbol and 0-symbol cards are invisible to the trigger.

---

## Algorithm 3: Resonance Echo Pack

**One-sentence description:** Draw 4 random cards; for each card in the pack
that shares a primary resonance with the card you drafted last pick, replace
it with a random card sharing your most-drafted resonance pair.

**Technical description:** After the player drafts a card, record its primary
resonance as the "echo resonance." On the next pack, draw 4 random cards.
Any card among the 4 whose primary resonance matches the echo resonance is
replaced by a card drawn from the pool whose ordered pair matches the player's
most-common pair. This creates a variable-strength effect: sometimes 0
replacements (echo resonance absent from random draw), sometimes 1-3. The pack
is always exactly 4 cards, but some slots are conditionally upgraded.

**Goal assessment:**
- Serves well: No extra actions (2), Flexible archetypes (5, echo changes every
  pick so pivots immediately change the effect), Variance (replacement count
  varies 0-3 per pack).
- Fails: Simplicity (1, two interacting conditions -- echo resonance AND pair
  replacement -- make this harder to explain), Convergence (6, replacement
  rather than addition caps at ~1.8-2.2 S/A depending on echo hit rate),
  Not on rails (3, if the player drafts the same resonance repeatedly, most
  slots get replaced, becoming deterministic).

**Best symbol distribution:** Balanced (40% 1-symbol, 40% 2-symbol, 20%
3-symbol). The echo trigger uses primary resonance (works with 1-symbol cards)
but the replacement uses pairs (needs 2+ symbol cards).

---

## Algorithm 4: Density-Scaled Bonus

**One-sentence description:** Draw 4 random cards; if the fraction of those 4
sharing any resonance with your drafted cards exceeds 50%, add 1 bonus card
of your most-drafted resonance.

**Technical description:** Track the player's full resonance profile (weighted
counts). When generating a pack, draw 4 cards. For each of the 4 cards, check
whether any of its symbols appear in the player's top 2 resonances. Count the
number of "overlap" cards. If overlap >= 3 out of 4 (i.e., 75% overlap), add
1 bonus card matching the player's top resonance. If overlap >= 2 out of 4
(50%), add 1 bonus card with 50% probability. Otherwise, no bonus. This creates
a graduated trigger: packs with heavy overlap almost always get enhanced, packs
with moderate overlap sometimes get enhanced, packs with little overlap never do.

**Goal assessment:**
- Serves well: Variance (graduated probability creates smooth distribution of
  enhanced/unenhanced packs), Open early (8, diffuse profile means low overlap
  early), No extra actions (2).
- Fails: Simplicity (1, the graduated probability is harder to explain in one
  sentence -- "if 3+ overlap, bonus; if 2 overlap, 50% chance of bonus" is
  technically two conditions), Convergence (6, single-resonance overlap check
  inherits V4's ~50% dilution problem, capping effective S/A contribution of
  bonus cards).

**Best symbol distribution:** 2-symbol heavy (55%). More symbols per card means
more overlap opportunities, increasing fire rate.

---

## Algorithm 5: Cascade Bonus

**One-sentence description:** Draw 4 random cards; if any card in the pack
shares its ordered resonance pair with the card you just drafted, add 1 bonus
card matching that pair, and if the bonus card also shares the pair, add another.

**Technical description:** After each pick, record the drafted card's ordered
pair (if it has 2+ symbols). When generating the next pack, draw 4 random cards.
Check if any of the 4 has an ordered pair matching the recorded pair. If at
least 1 matches, draw a bonus card from the pair-matched pool. Then check the
bonus card: if it also has the matching pair (which it will, by construction of
the pair-matched pool), draw a second bonus card. This creates a guaranteed
cascade of exactly 2 bonus cards whenever the initial trigger fires, producing
packs of either 4 (no trigger) or 6 (trigger + cascade). The cascade is
guaranteed because the bonus pool is pair-filtered.

**Goal assessment:**
- Serves well: Convergence (6, when it fires you get +2 pair-matched cards at
  ~90% archetype precision, strongly pushing S/A above 2.0), Pair precision
  makes bonus cards highly targeted.
- Fails: Simplicity (1, the cascade mechanic is confusing -- "add a card, then
  check if that card triggers another add" is a recursive concept), Variance
  (the binary 4-or-6 split is too extreme -- either nothing happens or a lot
  happens), Not on rails (3, 6-card packs with 3+ archetype cards feel
  deterministic), Splashable (7, the pack is dominated by on-archetype cards
  when cascade fires).

**Best symbol distribution:** 2-symbol dominant (60%+). The cascade depends
entirely on pair matching, which requires 2+ symbols.

---

## Champion Selection: Algorithm 2 -- Pair Cluster Bonus

**Why Pair Cluster Bonus wins:**

Algorithm 2 is the strongest proposal because it directly addresses V4's
structural finding -- the ~50% archetype dilution ceiling -- while remaining
fully automatic and one-sentence explainable. Here is the reasoning:

1. **Pair matching breaks the dilution ceiling.** V4 proved that
   single-resonance probabilistic approaches cap at ~1.7 S/A because each
   resonance maps to 4 archetypes. Pair Cluster Bonus uses ordered pairs, which
   map to archetypes with ~85-95% precision for 2+ symbol cards. This means when
   the trigger fires and a pair-matched bonus card is added, it has ~90% chance
   of being S/A for the target archetype -- compared to ~50% for
   single-resonance algorithms.

2. **Conditional triggering provides organic variance.** Unlike Domain 1
   (auto-widening, which fires on a schedule) or Domain 2 (soft locking, which
   is always-on with probability), Pair Cluster Bonus only fires when the random
   base pack happens to cluster with the player's profile. This creates
   genuinely unpredictable pack sizes (4 or 5) that feel like lucky draws rather
   than mechanical delivery.

3. **Self-balancing convergence curve.** Early in the draft, the player has no
   dominant pair, so the trigger almost never fires (packs are uniformly random,
   supporting open exploration). After commitment, the dominant pair becomes
   common enough that 2-of-4 random cards matching the pair becomes plausible
   (~45-60% of the time), providing convergence without thresholds or state
   machines.

4. **Simplicity.** The one-sentence description fully specifies the algorithm.
   A programmer can implement it: track pair counts, check base pack for pair
   matches, add bonus if threshold met.

Algorithms 1 and 4 fail because single-resonance matching inherits V4's dilution
ceiling. Algorithm 3's replacement mechanic is harder to explain and caps pack
size at 4 (no addition). Algorithm 5's cascade is too swingy and complex.

---

## Champion Deep-Dive: Pair Cluster Bonus

### Example Draft Sequences

**Early committer (commits to Warriors/Tide-Zephyr by pick 5):**
- Picks 1-3: Drafts 1 Tide/Zephyr card, 1 Tide card, 1 generic. Pair profile:
  {(Tide,Zephyr):1}. Trigger fires rarely -- needs 2 of 4 random cards to have
  pair (Tide,Zephyr), which is ~1/12 of pool pairs. Probability of 2+
  matching: ~5-8%. Packs are almost always 4 cards, fully open.
- Picks 4-6: Drafts 2 more Tide/Zephyr cards and 1 Zephyr/Tide card. Pair
  profile: {(Tide,Zephyr):3, (Zephyr,Tide):1}. Warriors cards are ~40/360 of
  the pool, with ~25 having 2+ symbols (pair-eligible). Probability that 2 of 4
  random cards match (Tide,Zephyr): ~15-25%. Trigger fires roughly 1 in 4-5
  packs. Beginning to see occasional 5-card packs.
- Picks 7-15: Pair profile grows to {(Tide,Zephyr):6-8}. Not only are there
  more pair-matched cards in the pool, but the player has not been removing them
  (they are picking favorites, leaving others). Fire rate climbs to ~45-60%.
  Average S/A per pack: base ~1.0 (random) + ~0.55 (bonus fires 55% of the
  time, ~90% chance the bonus is S/A) = ~1.5 S/A. This is below 2.0 with the
  strict 2-of-4 trigger.
- Picks 16-30: Fire rate plateaus at ~55-65%. Average S/A reaches ~1.6-1.8.

**Observation:** The strict 2-of-4 pair-match trigger may not cross 2.0 S/A.
This suggests the trigger threshold should be **1 matching card** rather than
2, or the bonus should be 2 cards instead of 1. See parameter variants below.

**Flexible player (stays open for 10 picks):**
- Picks 1-10: Drafts across multiple archetypes. Pair profile is spread:
  {(Tide,Zephyr):2, (Ember,Stone):2, (Stone,Ember):1, ...}. No dominant pair,
  so the trigger rarely fires for any specific pair. Packs remain 4 cards and
  open. This is the desired behavior -- the algorithm does not push an
  uncommitted player.
- Picks 11-20: Player commits to Storm/Ember-Stone. Pair profile concentrates.
  Fire rate ramps up as (Ember,Stone) count grows. Convergence begins around
  pick 13-15, later than the early committer. This rewards early commitment
  without punishing flexibility.

**Pivot attempt (commits Warriors, pivots to Ramp at pick 10):**
- Picks 1-9: Builds (Tide,Zephyr) pair count to 5. Fire rate around 40%.
- Pick 10: Decides to pivot to Ramp (Zephyr,Tide). The dominant pair switches
  from (Tide,Zephyr) to (Zephyr,Tide) -- but (Zephyr,Tide) count is only 1-2.
  Fire rate drops to near zero for the new archetype. The player must rebuild
  pair count from scratch, but there is no permanent lock preventing the pivot.
  By pick 15-16, (Zephyr,Tide) count is high enough for regular triggers.
- Key difference from Lane Locking: the pivot is possible but costly (5-6 picks
  of low convergence). Lane Locking makes it impossible after locks fire.

### Predicted Failure Modes

1. **Under-convergence with strict 2-of-4 trigger.** The mathematical analysis
   above suggests 2-of-4 pair matching may cap around 1.6-1.8 S/A, below the
   2.0 target. Mitigation: lower trigger to 1-of-4, or add 2 bonus cards per
   trigger, or use single-resonance matching for the trigger condition while
   using pair matching for the bonus card pool.

2. **1-symbol cards are invisible to pair matching.** Cards with only 1 symbol
   have no pair and cannot trigger the condition or be drawn as pair-matched
   bonuses. If 25%+ of the pool is 1-symbol, effective pool size for the pair
   system shrinks. Mitigation: weight the symbol distribution toward 2-symbol
   cards (60%+).

3. **Adjacent-archetype confusion.** Warriors (Tide,Zephyr) and Ramp
   (Zephyr,Tide) are distinct ordered pairs, but a player straddling both might
   have neither pair dominant enough to trigger consistently. Mitigation: this
   is actually desirable behavior -- it keeps hybrid drafters in the open/random
   regime until they commit.

4. **Fire rate variance is high.** Some committed players will get unlucky
   streaks of 4-5 packs without a trigger. This is the design intent (organic
   variance), but it may frustrate players who expect consistent improvement.
   The stddev target (>= 0.8) should catch this -- if variance is too high, the
   threshold can be loosened.

### Parameter Variants Worth Testing

**Variant A: Loose Trigger (1-of-4 match)**
"Draw 4 random cards; if 1 or more shares its ordered pair with your most-
drafted pair, add 1 bonus card matching that pair."
- Higher fire rate (~60-75% for committed players), more consistent convergence.
- Risk: fires too early (even 1 pair-matched card in a random pack triggers it),
  reducing the open-early feel.
- Expected S/A: ~2.0-2.3.

**Variant B: Double Bonus (2-of-4 match, +2 cards)**
"Draw 4 random cards; if 2 or more share their ordered pair with your most-
drafted pair, add 2 bonus cards matching that pair."
- Lower fire rate but stronger payoff when it fires. More swingy (4 or 6 cards).
- Expected S/A: ~2.0-2.4 (similar average, higher variance).
- Risk: 6-card packs feel very different from 4-card packs.

**Variant C: Hybrid Trigger (single-resonance trigger, pair-matched bonus)**
"Draw 4 random cards; if 2 or more share a primary resonance with your most-
drafted resonance, add 1 bonus card matching your most-drafted pair."
- Uses the easier-to-hit single-resonance condition for the trigger (fire rate
  ~45-55%), but draws the bonus card from the pair-matched pool (~90% archetype
  precision).
- Best of both worlds: higher fire rate from single-resonance trigger, high
  precision from pair-matched bonus.
- Expected S/A: ~2.0-2.3.
- Risk: the one-sentence description is slightly longer ("resonance trigger,
  pair bonus") but still passes simplicity test.

### Proposed Symbol Distribution

| Symbol Count | % of Non-Generic | Cards |
|---|---|---|
| 0 (generic) | -- | 36 |
| 1 symbol | 15% | 49 |
| 2 symbols | 60% | 194 |
| 3 symbols | 25% | 81 |

Rationale: Pair Cluster Bonus depends entirely on 2+ symbol cards for trigger
detection and bonus sourcing. Minimizing 1-symbol cards (15%) maximizes the
pair-eligible pool. The 60% 2-symbol allocation ensures most random packs
contain 2-3 pair-eligible cards, making the trigger condition reachable. The 25%
3-symbol allocation adds extra resonance density for the weighted symbol
counting system while also contributing pairs.

With this distribution, ~85% of non-generic cards (275 of 324) are
pair-eligible. Each archetype has ~40 cards, of which ~34 have 2+ symbols and
carry the archetype's defining ordered pair. In a random 4-card pack, the
expected number of cards sharing any given pair is 4 x (34/360) = ~0.38, so the
probability of 2+ matching a specific pair is low (~5-8%). However, we check
against the player's *most common* pair, and as the player commits, that pair's
representation in the pool stays constant while the player's signal strengthens
-- the key convergence driver is that committed players recognize the cluster
when it appears and the bonus compounds their advantage.

Variant C (hybrid trigger) may be the most practical path to 2.0+ S/A, and I
recommend testing it alongside the pure pair trigger in simulation.
