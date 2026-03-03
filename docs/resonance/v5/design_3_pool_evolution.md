# Domain 3: Pool Evolution / Seeding -- Round 1 Design

## Key Takeaways

- **Net-add seeding is essential.** V3's Resonance Swap (3-in / 3-out, net zero)
  peaked at 1.58 S/A because the pool size stayed flat and resonance density
  barely moved. Only net-positive injection can shift the density fast enough
  to cross 2.0 by pick 6-8.
- **Pair-based seeding solves the dilution ceiling.** V4 proved single-resonance
  probabilistic approaches cap at ~1.7 S/A because each resonance spans 4
  archetypes. Seeding cards matched by ordered pair (primary, secondary) instead
  of single resonance achieves near-100% archetype precision for the 75% of
  cards with 2+ symbols, potentially breaking through 2.0 without slot locking
  or bonus-card UI.
- **Pool manipulation is the most invisible mechanism available.** The player
  never sees a slot lock, a token counter, or a bonus card appearing. They just
  notice the pool getting better over time. This is the strongest possible
  candidate for "feels like natural variation."
- **Aggressive seeding rates (3-5 cards per pick) are needed but risk pool
  bloat.** Adding 5 cards per pick for 30 picks grows the pool from 360 to
  510. The added cards must be concentrated enough in the player's archetype to
  outpace the dilution from the growing denominator.
- **Pair-based seeding with proportional injection is the champion.** It
  auto-injects cards matching the player's most-drafted ordered pair, achieving
  high archetype precision with zero player decisions and zero visible
  mechanism. The key tuning lever is injection count per pick.
- **Removal is dangerous but valuable as a secondary lever.** Removing
  off-archetype cards sharpens density faster than pure addition, but
  aggressive removal shrinks the pool and reduces variance. A conservative
  removal rate (1-2 cards per pick) paired with moderate addition (3 per pick)
  offers the best balance.
- **Symbol distribution should favor 2-symbol cards (60%+).** Pair-based
  seeding needs cards with 2+ symbols to identify the pair. A distribution
  heavy on 2-symbol cards maximizes the algorithm's archetype precision
  while keeping 1-symbol cards available for splash and flexibility.

---

## Five Algorithm Proposals

### 1. Primary Resonance Flood

**One-sentence description:** When you draft a card, 3 random cards from a
reserve whose primary resonance matches the drafted card's primary resonance
are added to the pool.

**Technical description:** Maintain a reserve of ~200 cards (50 per resonance).
After each pick, draw 3 cards from the reserve whose primary resonance matches
the picked card's primary resonance and shuffle them into the active pool. The
pool grows by 3 per pick (360 to 450 over 30 picks). Pack generation remains
"draw 4 random cards from the pool."

**Assessment:** Serves simplicity (one operation, no state tracking), no extra
actions, and signal reading (pool asymmetry emerges naturally from picks). Fails
convergence: single-resonance matching means ~50% of injected cards belong to
the wrong archetype (the V4 dilution ceiling). At +3 per pick, pool density for
the target archetype increases from ~12.5% to roughly 15-16% by pick 15 -- not
enough to cross 2.0 S/A. Aggressive +5 might reach ~18%, still likely under 2.0.

**Best symbol distribution:** Distribution-agnostic; works with any mix since
matching is on primary resonance only.

---

### 2. Pair-Matched Injection

**One-sentence description:** When you draft a card with 2+ symbols, 3 random
cards from a reserve whose ordered pair (primary, secondary) matches the drafted
card's pair are added to the pool.

**Technical description:** Maintain a reserve partitioned by ordered pair (12
possible pairs, ~15 cards each). After each pick of a 2+ symbol card, draw 3
cards from the reserve matching that card's (primary, secondary) pair and add
them to the pool. 1-symbol and 0-symbol picks inject nothing. Pack generation
is "draw 4 random from pool."

**Assessment:** Serves convergence strongly -- pair matching is ~100% archetype-
precise for 2+ symbol cards, so nearly all injected cards are S/A for the
target archetype. Serves simplicity (one conditional operation) and no extra
actions. Weakness: 1-symbol picks get no injection, creating dead turns (~20%
of picks if symbol distribution is 20% 1-symbol). Also requires a large
pair-partitioned reserve (~180 cards, 15 per pair), which may be hard to
populate for rarer pairs.

**Best symbol distribution:** Heavily 2-symbol (65%+) to maximize the fraction
of picks that trigger injection.

---

### 3. Proportional Pair Seeding (Champion)

**One-sentence description:** After each pick, cards are added to the pool
from a reserve, distributed across your drafted resonance pairs in proportion
to how many times you have drafted each pair -- 4 total cards injected per pick.

**Technical description:** Track a pair profile: a dictionary mapping each
ordered pair to the count of times the player has drafted a card with that pair.
After each pick, inject 4 cards total from the reserve, allocated proportionally
across pairs with nonzero counts. For example, if the profile is
{(Tide,Zephyr): 5, (Zephyr,Tide): 2, (Ember,Stone): 1}, inject 2-3 cards
matching (Tide,Zephyr), 1 matching (Zephyr,Tide), and 0-1 matching
(Ember,Stone). Cards from 1-symbol or 0-symbol picks do not contribute pairs
but the injection still fires using the existing profile. Round allocations
probabilistically (fractional shares become probabilities).

**Assessment:** Strongest convergence candidate. Proportional allocation means
the dominant pair gets the lion's share of injection, concentrating the pool
toward the target archetype with ~100% precision for 2+ symbol injected cards.
Serves all core goals: no player decisions, invisible mechanism, natural
variance (random draw from an evolving pool), supports splash (secondary pairs
still get some injection), open early (profile is spread until commitment).
Potential weakness: pool bloat (360 + 4*30 = 480 cards) dilutes the density
gain somewhat; must verify injection outpaces dilution. Also, the proportional
allocation rule is slightly more complex than simple flat injection.

**Best symbol distribution:** 60% 2-symbol, 20% 1-symbol, 20% 3-symbol. The
2-symbol majority ensures most picks contribute pairs and most injected cards
are pair-precise.

---

### 4. Flood and Drain

**One-sentence description:** When you draft a card, 3 cards matching its
primary resonance are added to the pool from a reserve, and 2 cards whose
primary resonance differs from any of your drafted resonances are removed from
the pool.

**Technical description:** After each pick, add 3 cards from the reserve
matching the picked card's primary resonance. Then remove 2 cards from the pool
whose primary resonance does not match any resonance the player has drafted (if
such cards exist; otherwise skip removal). Net pool change is +1 per pick
(360 to 390 over 30 picks). Removal targets "completely unrelated" cards,
preserving splash options in adjacent resonances.

**Assessment:** Strongest density shift per pick because it both adds on-type
and removes off-type. Serves convergence well. Maintains pool size better than
pure addition (net +1 vs +3-5). Weakness: uses single-resonance matching for
addition (the V4 dilution problem), partially offset by the removal sharpening
the denominator. Removal also reduces early variety -- a player who drafts Tide
and Zephyr early causes Ember and Stone cards to drain, narrowing options before
commitment. This conflicts with Goal 8 (open early) and Goal 5 (flexible
archetypes).

**Best symbol distribution:** Distribution-agnostic for the addition; removal
benefits from more 2+ symbol cards (easier to identify "completely unrelated").

---

### 5. Escalating Pair Injection

**One-sentence description:** After each pick, a number of cards equal to your
most-drafted pair's count (capped at 5) matching that pair are added to the
pool from a reserve.

**Technical description:** Track pair counts as in Proposal 3. After each pick,
identify the pair with the highest count. Inject min(top_pair_count, 5) cards
from the reserve matching that top pair. Early in the draft (top pair = 1),
inject 1 card. By mid-draft (top pair = 4-5), inject 4-5 cards per pick. This
creates exponential-feeling acceleration: slow seeding early (preserving
variety) and heavy seeding late (driving convergence).

**Assessment:** Best temporal profile of any proposal -- minimal interference
early, strong convergence late. Serves Goal 8 (open early) and Goal 6
(convergent late) simultaneously. Pair matching gives high archetype precision.
Weakness: the cap at 5 is an arbitrary parameter, and without a cap the
injection becomes extreme (10+ cards per pick by pick 15). Pool bloat is
moderate early but severe late: by pick 30, the pool could be 360 + sum(1..5
then 5*25) = 360 + 15 + 125 = ~500 cards. The escalation also makes the
algorithm slightly harder to explain in one sentence (the "equal to your count,
capped at 5" clause adds complexity).

**Best symbol distribution:** 60%+ 2-symbol to ensure fast pair count growth.

---

## Champion Selection: Proportional Pair Seeding

Proportional Pair Seeding (Proposal 3) is the champion because it combines the
strongest set of properties:

1. **Pair-based matching breaks the dilution ceiling.** V4 proved
   single-resonance probabilistic approaches cap at ~1.7 S/A. Pair matching
   achieves ~100% archetype precision for 2+ symbol cards. This is the single
   most important structural advantage.

2. **Proportional allocation is self-balancing.** Unlike top-pair-only injection
   (Proposals 2, 5), proportional allocation seeds secondary pairs too, naturally
   supporting splash and adjacent archetypes. A player drafting mostly Warriors
   (Tide,Zephyr) with some Ramp (Zephyr,Tide) splash gets injection in both,
   maintaining flexibility.

3. **Constant injection rate (4/pick) is predictable.** Unlike Escalating (slow
   then fast) or Flood-and-Drain (add and remove), a flat 4 cards per pick
   creates steady, predictable pool evolution. This makes the algorithm easier to
   reason about and tune.

4. **Zero visible mechanism.** No tokens, no slots, no counters in the UI. The
   player drafts cards and the pool quietly shifts. This is the strongest
   candidate for "feels like natural variation."

5. **Handles 1-symbol and generic picks gracefully.** When a player picks a
   1-symbol or generic card, no new pair is added to the profile, but the
   existing profile still drives injection. The pool keeps evolving based on
   prior commitment signals.

Proposals 1 and 4 use single-resonance matching, which V4 proved is
structurally capped. Proposal 2 has dead turns on 1-symbol picks. Proposal 5
has the best temporal profile but is harder to explain and risks extreme pool
bloat late. Proportional Pair Seeding strikes the best balance.

---

## Champion Deep-Dive: Proportional Pair Seeding

### Example Draft Sequences

**Early committer (Warriors, Tide/Zephyr):**
- Pick 1: Drafts [Tide, Zephyr] Warriors card. Profile: {(T,Z):1}. Inject 4
  cards matching (T,Z). Pool: 364, with 4 extra Warriors-archetype cards.
- Pick 3: Profile: {(T,Z):3, (Z,T):1}. Inject 3 matching (T,Z), 1 matching
  (Z,T). Pool: 372. Warriors density rising.
- Pick 6: Profile: {(T,Z):5, (Z,T):2, (T,S):1}. Inject ~2-3 (T,Z), ~1 (Z,T),
  ~0-1 (T,S). Pool: 384. Warriors cards now ~14% of pool (up from ~11%).
- Pick 15: Profile: {(T,Z):10, (Z,T):3, others small}. Pool ~420. Warriors
  cards ~18-20% of pool. Expect ~0.72-0.80 Warriors cards per random slot,
  so ~2.9-3.2 across 4 slots. With S/A fitness overlap from adjacent archetypes,
  should see 2.0+ S/A reliably.

**Flexible player (no commitment until pick 8):**
- Picks 1-7: Drafts across 3-4 different pairs. Profile: {(T,Z):2, (E,S):2,
  (S,T):2, (Z,T):1}. Injection spread evenly. Pool: 388. No single archetype
  dominates -- density barely moves for any one archetype. Packs remain diverse.
- Pick 8: Commits to Storm (E,S). Profile starts concentrating.
- Pick 15: {(E,S):6, (T,Z):2, (S,T):2, others}. Injection now favors (E,S).
  Pool ~420. Storm density climbing, but later start means lower final density
  than the early committer. Convergence around pick 10-11.

**Pivot attempt (starts Warriors, pivots to Sacrifice at pick 8):**
- Picks 1-7: Profile: {(T,Z):5, (Z,T):2}. Pool seeded toward Warriors.
- Pick 8: Starts picking Sacrifice cards (T,S). New pair enters profile.
- Pick 12: Profile: {(T,Z):5, (T,S):4, (Z,T):2}. Injection now splits
  between Warriors and Sacrifice. The pool contains extra cards for both.
  Because both share Tide primary, there is overlap. Sacrifice density grows
  but competes with the Warriors legacy in the pool. Convergence is slower
  (~pick 12-14) but achievable because injected (T,S) cards are ~100%
  Sacrifice-archetype.

### Predicted Failure Modes

1. **Pool bloat dilution.** At 4 cards per pick for 30 picks, the pool grows
   to 480. If injected cards are 100% archetype-precise but the pool grows
   33%, the density gain is: (40 + 4*30*0.65) / 480 = (40 + 78) / 480 = 24.6%
   (where 0.65 is the share going to top pair for a committed player). Baseline
   was 40/360 = 11.1%. So density roughly doubles -- but 24.6% across 4 random
   slots yields ~1.0 expected archetype-specific cards. Adding A-tier from
   adjacent archetypes (which also get seeded via secondary pairs), this could
   reach 1.8-2.2. Tight. Must verify in simulation.

2. **Reserve exhaustion.** A pair-partitioned reserve of ~15 cards per pair (180
   total) empties quickly. A committed player injecting ~2-3 of their top pair
   per pick exhausts the reserve for that pair by pick 6-8. Mitigation: use a
   larger reserve (30+ per pair) or allow recycling (drafted cards return to
   reserve after use). Alternatively, generate reserve cards dynamically.

3. **1-symbol card dead weight.** With 20% 1-symbol cards, ~1 in 5 picks adds
   no new pair to the profile. The existing profile still drives injection, but
   the player feels no "progress" from those picks. This is acceptable -- it
   creates natural variance in pool evolution speed.

4. **Flexible players get almost no benefit.** A player drafting 4+ different
   pairs through pick 10 spreads injection so thin that no archetype's density
   meaningfully increases. Packs remain baseline-random. This may actually be
   desirable (Goal 8: open early), but if the player never commits, they never
   converge. The algorithm rewards commitment, which is the correct incentive.

### Parameter Variants Worth Testing

**Variant A: Injection rate 3 vs 4 vs 5 per pick.**
- 3/pick: Pool reaches 450. More conservative density shift. May fall short
  of 2.0 S/A. Safer against pool bloat.
- 4/pick: Pool reaches 480. The baseline proposal. Expected to be near the
  2.0 threshold.
- 5/pick: Pool reaches 510. Strongest density shift but most dilution from
  pool size. May overshoot or may be needed to reliably cross 2.0.

**Variant B: Pair-only injection vs pair + primary resonance hybrid.**
- Pure pair: Only inject cards matching the exact ordered pair. Maximum
  archetype precision (~100%) but limited reserve depth per pair.
- Hybrid: Inject 3 pair-matched + 1 primary-resonance-matched per pick.
  Slightly lower precision (~87.5%) but deeper reserve and faster density
  growth. The primary-only card has ~50% chance of being right-archetype.

**Variant C: With conservative removal (inject 4, remove 1 off-pair).**
- After injection, remove 1 card from the pool whose pair does not match any
  pair in the player's profile. Net +3 per pick (pool reaches 450). Sharper
  density shift without as much bloat. Risk: premature narrowing of variety.

### Proposed Symbol Distribution

| Symbol Count | % of Non-Generic | Cards | Rationale |
|---|---|---|---|
| 0 (generic) | -- | 36 | Standard 10% neutral |
| 1 symbol | 15% | 49 | Minimal; splash/flexibility role only |
| 2 symbols | 65% | 211 | Maximizes pair-matchable cards |
| 3 symbols | 20% | 64 | Provides depth; pair = (1st, 2nd) |

The 65% 2-symbol ratio is higher than V3/V4's 55% because pair-based seeding
specifically benefits from more cards having identifiable pairs. The reserve
should follow the same distribution, ensuring pair-matched reserve cards are
abundant. Each ordered pair gets roughly 211/12 + 64/12 = ~23 cards in the
main pool and a comparable number in the reserve, providing adequate depth for
sustained injection over 30 picks.
