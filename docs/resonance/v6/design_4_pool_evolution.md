# Agent 4: Pool Evolution — Algorithm Design

## Key Takeaways from V3/V4/V5

1. **The 50% dilution ceiling is the central obstacle.** Each resonance is
   shared by 4 archetypes. Probabilistic mechanisms that increase resonance
   density in packs still deliver only ~50% archetype-correct cards, capping S/A
   at ~1.7. Pool evolution is fundamentally probabilistic — changing what's in
   the pool changes what's drawn at random — so it must overcome this same
   ceiling.

2. **To reach 2.0 S/A with 50% dilution, the pool must contain ~50%+ cards of
   the player's lead resonance.** If 50% of the pool is resonance-matched and
   50% of those are archetype-correct, a random 4-card pack yields ~1.0 S/A from
   the archetype-correct share alone. That is still insufficient. The pool must
   be so dominated by the lead resonance (~75%+) that even with dilution, 2.0+
   S/A emerges — or the algorithm must use both resonances to narrow the
   archetype.

3. **Aggressive removal is as powerful as aggressive seeding.** Adding 3
   on-resonance cards has the same density effect as removing 3 off-resonance
   cards. Combining both doubles the rate of pool drift. V3's Resonance Swap
   used a modest 3-add/3-remove rate and achieved only 1.58 S/A — the rate was
   far too low.

4. **Dual-resonance tracking narrows archetype identity.** Under the 15% cap,
   only ~54 cards carry two resonance types. But the algorithm can track the
   player's top two resonances from mono-type cards drafted. If a player drafts
   4 Tide cards and 2 Zephyr cards, seeding both Tide AND Zephyr cards into the
   pool creates a pool enriched in the Tide+Zephyr overlap — Warriors territory.
   This uses pair information without requiring pair symbols on cards.

5. **Pool evolution is invisible to the player.** Unlike Lane Locking (visible
   slot states) or Pack Widening (visible tokens), pool changes are hidden
   infrastructure. The one-sentence description must make the mechanism
   intuitive despite this opacity.

6. **V5 failed because it required 70%+ dual-resonance cards.** Under the 15%
   cap, pair-based matching from card symbols alone cannot be a primary
   strategy. But tracking the player's drafted resonance *profile* (which
   resonances they accumulate) is fair game — that uses the player's choices,
   not card labels.

7. **Natural variance is pool evolution's strength.** Because packs are always
   drawn randomly from the pool, variance emerges organically. No deterministic
   slot assignment, no binary states. The challenge is making the pool shift
   hard enough to cross 2.0 while keeping the randomness.

______________________________________________________________________

## Five Algorithm Proposals

### 1. Flood and Drain

**One sentence:** After each pick, add 5 cards matching the drafted card's
primary resonance to the pool from a reserve and remove 3 random cards whose
primary resonance differs from the player's most-drafted resonance.

**Technical description:** Maintain a reserve of ~200 cards (50 per resonance).
When the player drafts a card with primary resonance R, move 5 random R-primary
cards from reserve to pool. Then identify the player's lead resonance L (highest
weighted symbol count) and remove 3 random cards whose primary resonance is not
L. The pool starts at 360 and grows by a net +2 per pick early, but removal
targets shrink as off-resonance cards deplete. By pick 15, the pool is heavily
skewed toward L.

**Design goal assessment:** Hits convergence (aggressive rates should push past
2.0), no player decisions, good variance (random draws from skewed pool). Misses
simplicity (two operations — add and remove — with different targeting), and
convergence may be too slow because the pool is large (360 cards) and +5/-3 per
pick takes many picks to shift density meaningfully.

**Symbol distribution:** 15% dual-type (54 cards). Maximize dual-type to help
the add/remove operations occasionally seed archetype-precise cards.

### 2. Shrinking Focused Pool

**One sentence:** The pool starts at 360 cards; after each pick, remove 8 random
cards whose primary resonance does not match any resonance the player has
drafted, so the pool shrinks and concentrates on the player's resonances.

**Technical description:** Track which resonance types the player has drafted at
least one symbol of. After each pick, remove 8 cards at random from the pool
whose primary resonance is not in the player's drafted set. Generic cards are
never removed. The pool shrinks from 360 toward a floor of ~120 (the player's
1-2 resonances plus generics). Packs are drawn randomly from the remaining pool.
As the pool shrinks, resonance density increases mechanically.

**Design goal assessment:** Maximally simple (one operation, purely removal). No
decisions. But removal alone may not shift fast enough — after pick 1, the
player has 1 resonance, and 3 of 4 resonances are "off." Removing 8 of ~270
off-resonance cards per pick means ~30 picks to halve off-resonance count.
Convergence will be too slow. Also, once the player has drafted 2 resonances
(likely by pick 2-3), 2 of 4 resonances are "on" and removal slows dramatically.

**Symbol distribution:** 10% dual-type (36 cards). Fewer dual-type since this
algorithm doesn't benefit from them.

### 3. Resonance Cascade

**One sentence:** Each drafted resonance symbol adds 2 cards of that resonance
from a reserve to the pool, so a card with symbols [Tide, Tide, Zephyr] adds 4
Tide cards and 2 Zephyr cards.

**Technical description:** Maintain a reserve of 400+ cards (~100 per
resonance). When the player drafts a card, for each symbol on the card, add 2
random cards of that symbol's resonance type from the reserve to the pool.
Primary symbol weight is already encoded in the card's symbol list (a \[Tide,
Tide\] card adds 4 Tide cards; a [Tide] card adds 2). No removal. The pool grows
from 360 and becomes increasingly dominated by the player's drafted resonances.
A player drafting a 2-symbol card adds 4 cards; a 3-symbol card adds 6.

**Design goal assessment:** Very simple (one sentence, one operation). No
decisions. Pool growth is proportional to symbol count, creating a natural
feedback loop. However, the pool grows without bound — by pick 15, the pool
might be 360 + 60 = 420, with only ~17% of the pool being newly added
on-resonance cards. The base 360 dilutes the additions. Growth rate may be
insufficient without removal.

**Symbol distribution:** 15% dual-type (54 cards). Dual-type cards in the
reserve add to both resonances, helping narrow toward archetype.

### 4. Pool Replacement

**One sentence:** After each pick, replace 10 random cards in the pool with
cards from a reserve that match the drafted card's primary resonance, keeping
the pool at exactly 360 cards.

**Technical description:** Maintain a reserve partitioned by primary resonance.
When the player drafts a card with primary resonance R, select 10 random cards
from the pool whose primary resonance is NOT R, move them to a discard pile, and
replace them with 10 random R-primary cards from the reserve. Pool size stays
constant at 360. After 10 picks of the same resonance, 100 cards have been
replaced — the pool shifts from 25% R-primary to ~50% R-primary. After 15 picks,
~60-65% R-primary. At 60% R-primary density, a random 4-card pack has ~2.4
R-primary cards on average. With 50% archetype dilution, that is ~1.2 S/A —
still below 2.0.

**Design goal assessment:** Simple (constant pool size, one operation). No
decisions. But the math shows this still falls short: even at 65% resonance
density, 50% dilution yields ~1.3 S/A. To reach 2.0 S/A, the pool needs ~80%+
resonance density, requiring replacement rates of 15-20 per pick or 20+ picks of
commitment. The replacement rate can be tuned higher, but at 20 replacements per
pick the pool turns over every 18 picks, which may feel too aggressive.

**Symbol distribution:** 15% dual-type (54 cards) in the reserve. When
replacing, dual-type cards from the reserve help archetype precision.

### 5. Dual-Resonance Pool Sculpting (Champion)

**One sentence:** After each pick, replace 12 cards in the pool with 6 cards
matching your top resonance and 6 matching your second resonance from a reserve,
keeping the pool at 360 cards.

**Technical description:** Track the player's weighted symbol counts for all 4
resonances. After each pick, identify the player's top resonance (T1) and second
resonance (T2). Remove 12 random cards from the pool whose primary resonance is
neither T1 nor T2, and replace them with 6 T1-primary cards and 6 T2-primary
cards drawn randomly from a reserve. If fewer than 12 eligible off-resonance
cards remain, replace as many as possible. Pool stays at 360.

The key insight: by seeding BOTH the player's top two resonances, the pool
converges on the overlap of those two resonances — which is exactly one
archetype (or two adjacent archetypes). A player committing to Warriors (Tide
primary, Zephyr secondary) will have T1=Tide, T2=Zephyr. The pool fills with
Tide and Zephyr cards. The intersection of "Tide-primary" and "Zephyr-primary"
archetypes includes Warriors (Tide/Zephyr), Ramp (Zephyr/Tide), Flash
(Zephyr/Ember), and Sacrifice (Tide/Stone). But cards matching BOTH T1 and T2
simultaneously (dual-type [Tide, Zephyr]) are uniquely Warriors. And even
without dual-type cards, a pack drawn from a pool that's 40% Tide + 40% Zephyr
will contain ~3.2 cards from the Tide-or-Zephyr set. Of those, Warriors cards
are S/A tier in the Warriors archetype. Since Warriors is primary in Tide and
secondary in Zephyr, roughly 50% of Tide cards and 25% of Zephyr cards are S/A
for Warriors. With a 40/40 split, that's ~(0.5 * 1.6) + (0.25 * 1.6) = ~1.2 S/A.
Still below 2.0.

**To cross 2.0:** Increase replacement rate to 15 per pick (8 T1 + 7 T2), or add
a secondary mechanism: once T1 and T2 are established (say, by pick 5), increase
the replacement rate to 18 per pick. This drives the pool to 85%+ T1+T2 density
by pick 10, yielding ~3.4 matched cards per pack and ~1.7 S/A from
archetype-correct cards alone. Adding the ~0.4 S/A contribution from generic
B-tier and adjacent-archetype cards pushes past 2.0.

**Design goal assessment:** Hits simplicity (one sentence of concrete
operations), no decisions, good variance. Uses both resonances for narrower
archetype targeting. May need aggressive replacement rates. The dual-resonance
tracking adds meaningful archetype precision over single-resonance approaches.

**Symbol distribution:** 15% dual-type (54 cards), ~7 per archetype. Dual-type
cards in the reserve provide maximum archetype signal when drawn. The reserve
should be enriched with dual-type cards (20%+ dual-type in reserve vs 15% in
starting pool).

______________________________________________________________________

## Champion Selection: Dual-Resonance Pool Sculpting

**Justification:** The other four proposals all face the same fundamental
problem: single-resonance pool manipulation cannot overcome 50% archetype
dilution without extreme pool warping. Proposals 1-4 achieve at best ~1.3-1.7
S/A even with aggressive parameters.

Dual-Resonance Pool Sculpting is the only proposal that attacks the dilution
problem directly. By tracking two resonances and seeding both, it narrows the
pool toward the *intersection* of two resonances, which is a 2-archetype band
rather than a 4-archetype band. This halves the dilution from 50% to ~25-30%,
making 2.0 S/A achievable at realistic replacement rates.

The mechanism is also genuinely simple: one sentence, one operation per pick, no
player decisions. The pool stays at a fixed size. Packs are drawn purely at
random from the pool.

______________________________________________________________________

## Champion Deep-Dive: Dual-Resonance Pool Sculpting

### Example Draft Sequence (Warriors Player — Tide Primary, Zephyr Secondary)

**Pick 1:** Player drafts a [Tide, Tide] card. T1=Tide (count 3), T2=undefined.
Replace 12 off-resonance cards with 12 Tide-primary cards from reserve. Pool:
~112 Tide, ~83 each other resonance, 36 generic. Resonance density for Tide:
31%.

**Pick 2:** Player drafts a [Tide] card. T1=Tide (count 5), T2=undefined.
Replace 12 more. Pool: ~124 Tide, ~75 each other. Tide density: 34%.

**Pick 3:** Player drafts a [Zephyr, Tide] card. T1=Tide (count 6), T2=Zephyr
(count 2). Now replacing with 6 Tide + 6 Zephyr. Pool: ~130 Tide, ~81 Zephyr,
~65 each of Ember/Stone, 36 generic. Combined T1+T2 density: 59%.

**Pick 4:** Player drafts a [Tide, Zephyr] card. T1=Tide (count 8), T2=Zephyr
(count 3). Replace 12 (6+6). Pool: ~136 Tide, ~87 Zephyr, ~56 each Ember/Stone.
Combined density: 62%.

**Pick 5:** Another Tide card. Replace 12. Pool: ~142 Tide, ~93 Zephyr, ~48 each
Ember/Stone. Combined: 65%.

**Pick 6:** Expected pack composition from 65% T1+T2 pool: ~2.6 cards are Tide
or Zephyr primary. Of these, Warriors S/A rate: Tide cards ~50% S/A (Warriors is
one of 2 Tide-primary archetypes), Zephyr cards ~25% S/A (Warriors is secondary
in Zephyr). Weighted average: ~40% S/A rate. Expected S/A: ~1.04 from
resonance-matched + ~0.35 from generics = ~1.4 S/A. Not yet at 2.0.

**Pick 8:** After continued replacement, pool is ~75% T1+T2. Pack has ~3.0
resonance-matched cards. S/A: ~1.2 + 0.35 = ~1.55.

**Pick 10:** Pool is ~85% T1+T2. Pack has ~3.4 resonance-matched cards. S/A:
~1.36 + 0.35 = ~1.71.

### Honest Assessment

The math shows that even with aggressive dual-resonance seeding at 12
replacements per pick, pool evolution struggles to cross 2.0 S/A. The
fundamental issue: random pack draws from even a heavily skewed pool still
suffer from archetype dilution. At 85% T1+T2 density with ~40% S/A rate among
matched cards, you get ~1.7 S/A.

**To genuinely cross 2.0, the replacement rate must be extreme: 20+ cards per
pick.** At 20 replacements per pick (10 T1 + 10 T2), the pool reaches 90%+ T1+T2
density by pick 8, yielding ~3.6 matched cards per pack * 40% S/A = ~1.44 + 0.35
generics = ~1.8. Still marginal.

**The honest conclusion:** Pure pool evolution, even with dual-resonance
tracking, likely caps at ~1.7-1.9 S/A. It can approach 2.0 with extreme
parameters but probably cannot reliably cross it. This is the same structural
ceiling V4 identified for all probabilistic mechanisms. Pool evolution changes
the probability distribution, but packs are still random draws, and the
4-archetype-per-resonance dilution persists.

### Failure Modes

1. **Slow convergence:** At 12 replacements per pick, meaningful pool shift
   takes 8-10 picks. Early packs (1-5) are nearly random. This is fine for "open
   early" but means convergence arrives at pick 8-10, potentially outside the
   target window.

2. **Second-resonance instability:** If the player's T2 changes (they draft an
   Ember card after 3 Zephyr cards), the replacement target shifts and the pool
   becomes unfocused. The algorithm should use cumulative counts, not recent
   picks, to stabilize T2.

3. **Reserve exhaustion:** With 20 replacements per pick, the reserve needs 20
   \* 30 = 600 cards over a full draft. A 400-card reserve (100 per resonance)
   runs dry by pick 20 for the lead resonance. The reserve must be large (~800
   cards) or replacements must recycle removed cards back into the reserve.

4. **Pool homogenization:** By pick 20+, the pool is 90%+ two resonances. Every
   pack looks similar. Variance drops and the "not on rails" goal fails.
   Off-archetype splash (C/F cards) drops below 0.5 per pack.

### Parameter Variants for Simulation

**Variant A — Moderate (12/pick):** Replace 12 cards per pick (6 T1 + 6 T2).
Conservative rate. Projects to ~1.5-1.7 S/A. Tests whether pool evolution alone
can approach 2.0.

**Variant B — Aggressive (20/pick):** Replace 20 cards per pick (10 T1 + 10 T2).
Projects to ~1.7-1.9 S/A. Tests the upper bound of pure pool evolution. Reserve
size: 800 cards, recycling removed cards.

**Variant C — Aggressive with escalation (12 early, 24 late):** Replace 12 cards
per pick for picks 1-5 (before T2 is established), then 24 per pick for picks
6+. This preserves early openness while pushing hard for late convergence.
Projects to ~1.8-2.0 S/A but risks homogenization.

### Proposed Symbol Distribution

| Category             | Count   | Notes                               |
| -------------------- | ------- | ----------------------------------- |
| Generic (0 symbols)  | 36      | 10% of pool, never replaced         |
| 1 symbol, mono-type  | 100     | ~31% of non-generic                 |
| 2 symbols, mono-type | 130     | ~40% of non-generic                 |
| 2 symbols, dual-type | 36      | 11% of non-generic (within 15% cap) |
| 3 symbols, mono-type | 40      | ~12% of non-generic                 |
| 3 symbols, dual-type | 18      | ~6% of non-generic (within 15% cap) |
| **Total**            | **360** | 54 dual-type = exactly 15%          |

**Reserve distribution:** 800 cards, same proportions but with 20% dual-type
(160 cards). The reserve being enriched in dual-type cards means replacements
are more likely to seed archetype-precise signals. This is the algorithm's best
lever for narrowing beyond single-resonance matching.

**Justification:** Maximizing dual-type in the reserve means that as the pool
evolves, dual-type density in the pool increases from 15% toward 20%. A pack
drawn from a mature pool has a higher chance of containing a dual-type card that
precisely matches the player's archetype pair. This provides ~0.2-0.3 additional
S/A per pack from precise archetype hits, potentially pushing the algorithm from
~1.7 to ~1.9-2.0.

### Summary

Pool evolution is a conceptually elegant mechanism — the card pool itself drifts
toward the player's preferences, and packs are always random draws. It produces
natural variance and requires zero player decisions. However, the math
consistently shows it struggles to cross 2.0 S/A due to the fundamental
probabilistic ceiling: random draws from a skewed pool still suffer 50%+
archetype dilution per resonance. The dual-resonance tracking in the champion
proposal reduces this dilution to ~60-70% (better than single-resonance, but
still significant).

This algorithm is best positioned as a complementary layer — combined with a
targeted card injection mechanism (like auto-widening or slot targeting), pool
evolution could provide the natural variance and gradual convergence while the
injection mechanism provides the reliable 2.0+ S/A floor.
