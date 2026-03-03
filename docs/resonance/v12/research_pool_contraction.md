# Research Results: Pool Contraction via Physical AI Drafting

## Question

How does AI avoidance + declining refills produce pool contraction, and what
contraction trajectories achieve M3 >= 2.0 with modest oversampling (N = 8-12)?

---

## Findings

### 1. V11 SIM-4: The Baseline Failure and What It Reveals

V11 SIM-4 is the direct predecessor to V12's pool contraction mechanism: 4
rounds (8/8/7/7 picks), declining balanced refills (48/36/21/0), Level 0 AIs.
Result: M3 = 0.83, M11' = 0.71, M6 = 59%.

**What SIM-4's pool trajectory looked like:**

- Round 1 start: 120 cards, ~15 per archetype (1.0x gradient)
- Round 2 start: 120 cards after 48-card balanced refill. Within-round gradient
  built by 5 AIs was ~1.25-1.30x. The balanced refill compressed this almost
  entirely back to ~1.05-1.10x.
- Round 3 start: 108 cards after 36-card balanced refill. Same reset pattern.
  Gradient ~1.1x.
- Round 4 start: ~87 cards after 21-card balanced refill. Gradient ~1.2x.
- End: ~45-52 cards remaining.

**Where concentration failed — pool level AND pack level:**

SIM-4 failed at both layers, but the pack-sampling bottleneck was the binding
constraint. Even though the pool shrank to ~45-52 cards by draft end, S/A
density *declined* monotonically from 25% at pick 9 to 11% at pick 30. This is
the critical finding: without AI avoidance, AIs preferentially drained S/A
cards (they took the best cards for their archetypes), so the shrinking pool
was actually lower quality over time. The player's archetype was 1 of 3 open
lanes (~12-15 cards in a 45-52 card pool = roughly 25-33% density), but with
only 36% sibling A-tier rate, expected S/A per 4-card uniform pack ≈ 0.36-0.48.
M3 = 0.83 matches the theoretical random baseline precisely — zero emergent
concentration.

**The refill reset problem:** Each balanced refill restored all 8 archetypes
equally. A refill of 48 cards adds 6 cards per archetype. The within-round
gradient (AIs drained AI-lanes faster than open lanes = open lanes at ~18,
AI-lanes at ~12) was reset toward uniformity at each boundary. V11's algorithm
overview calls this "a fixed cost per round boundary" — the reset is
proportional to refill volume and cannot be eliminated without bias or
contraction below refill volume.

### 2. Adding AI Avoidance to SIM-4's Refill Schedule: Modeled Trajectory

V12's key departure from SIM-4: AIs avoid the player's archetype once inferred
(~picks 5-8 onward). Two changes compound under avoidance:
1. The player's S/A cards are no longer taken by AIs — only the player depletes
   their own archetype (1 pick per cycle vs. 6 previously).
2. AIs concentrate picks on 7 of 8 archetypes, so non-player archetypes deplete
   faster.

**Round boundaries modeled with avoidance (48/36/21/0, balanced refills):**

- Round 1 end (pre-avoidance): Player archetype ~8-9 cards (from 15). AI
  archetypes ~11-12. Refill restores player's lane to ~14-15. Avoidance begins.
- Round 2 end (full avoidance): AIs take ~40 from 7 non-player lanes; player
  takes ~8. Player archetype ~7 of 72 total = 10% density. Balanced refill
  restores player archetype to ~11.5 of 108 = 10.6%.
- Round 3 end: AIs take ~35 from non-player lanes; player takes ~7. Player
  archetype ~4.5 of 66 = 6.8%. Refill restores to ~7.1 of 87 = 8.2%.
- Round 4 end: ~60 cards remaining; player archetype near-exhausted (0-3 cards).

**The problem:** Balanced refills restore all archetypes proportionally.
Each refill resets the player's archetype back toward parity with non-player
archetypes, undoing one round of avoidance benefit. Archetype density stays
6-11% through late draft — far below the 45-55% needed for M3 >= 2.0.
Crucially, at ~7 player archetype cards with 36% sibling A-tier, only ~2.5
S/A cards exist in the player's lane by Round 4 start.

**The insight for V12:** Balanced refills actively work against avoidance.
A refill biased toward open lanes (or away from restoring the player's lane
at the same rate) would compound avoidance rather than partially resetting it.

### 3. Contraction Targets: What Pool State Achieves M3 >= 2.0

The V12 orchestration plan's lookup table (assuming 5 S/A in the player's
archetype portion of the pool):

- Pool = 20 cards, 55% archetype density, 5 S/A: N=4 → 1.00, N=8 → 2.00, N=12 → 3.00
- Pool = 25 cards, 50% density, 5 S/A: N=4 → 0.80, N=8 → 1.60, N=12 → 2.40
- Pool = 30 cards, 45% density, 5 S/A: N=4 → 0.67, N=8 → 1.33, N=12 → 2.00

**For N = 8:** Pool must reach ~20 cards with 55% archetype density. Requires
aggressive contraction and strong avoidance.

**For N = 12:** Pool must reach ~30 cards with 45% density. More forgiving.

**S/A supply constraint:** Starting ~5.4 S/A per archetype (15 cards × 36%
A-tier). Imperfect pre-avoidance (picks 1-7) drains 1-2 S/A from the player's
lane. Biased refills partially replenish. Achieving 5 S/A in a 20-30 card pool
requires either: open-lane biased refills that add S/A back to the player's
archetype, or avoidance activating before most S/A is drained. With balanced
refills, S/A added per refill = total × (1/8) × 0.36 — barely enough to
replace what was lost pre-avoidance.

### 4. Pool Exhaustion Risk

With 6 removals per pick cycle (5 AIs + 1 player) and no refills:
- Starting pool of 120: exhausted by pick 20 (120 / 6 = 20 picks to empty).
- With refills (48/36/21/0): 120 + 48 + 36 + 21 = 225 total cards available.
  30 picks × 6 removals = 180 total removals. Surplus = 45 cards. Pool
  exhaustion is not a primary risk with this schedule.

With steeper declines (60/30/0, 3 rounds): 120 + 60 + 30 = 210 available.
180 removals needed. Surplus = 30 cards. Tight but viable, requiring AI
saturation in Round 3 (~5 picks × 6 = 30 removals from ~30-card pool).

**Minimum safe pool size:** At 4 cards shown per pick, the pool needs at least
~8-12 cards to avoid showing duplicates or exhausting options. With AI
saturation logic (AIs stop taking cards after hitting a threshold), the
practical floor is ~15-20 cards.

**Key risk:** With AI avoidance, AIs concentrate picks on 7 of 8 archetypes.
If some archetypes have fewer than 5 cards remaining, AIs assigned to those
archetypes will hit saturation early and may start taking any available card,
potentially including the player's archetype. Saturation thresholds (e.g., stop
at 4 cards remaining per lane) prevent this.

### 5. Refill Bias: Balanced vs. Biased Under Avoidance

V11 Finding 4 confirmed open-lane-biased refills are Level 0 (determined by
pre-draft AI assignment, not player behavior).

**Balanced refills** partially reset the avoidance effect each round (as shown
in Section 2). This is the worst case for V12 — the system works against itself.

**Open-lane-biased refills** add more cards to the player's archetype (one of 3
open lanes) than to AI-lane archetypes. This compounds avoidance: AIs drain
non-player archetypes while refills preferentially restore open lanes. V11 SIM-2
achieved M3 = 0.87 with 1.7x open-lane bias and Level 0 AIs. With avoidance
added, open-lane bias should amplify concentration further. The player's
archetype gets 1.7x as many refill cards as each AI-lane archetype without
any player-reactive information — the bias is pre-determined by AI assignment.

**Recommendation:** Open-lane-biased refills are the natural pairing with
avoidance. Balanced refills should be avoided in V12 — they compound the
avoidance-reset problem identified in Section 2.

### 6. Starting Pool Size

V12 baseline: 120 cards (15 per archetype). A smaller pool (80-100) contracts
faster but risks archetype exhaustion by Round 3 and feels thin in early
exploration (only 10-12 cards per archetype at pick 1). A larger pool (180-240)
provides better early variety (M1 metric) but requires steeper declines to
reach 20-30 cards by late draft — or higher N oversampling to compensate for
the larger remaining pool.

**120 is a reasonable starting point:** 15 cards per archetype supports early
exploration, can contract to 20-30 cards with steep declining refills, and
makes N = 8-12 oversampling tractable (25-60% of the late pool). If simulation
shows late-draft pool stays too large, increasing the starting pool and steepening
the decline schedule is preferable to shrinking starting pool size.

### 7. Round Structure: 3 vs 4 Rounds

V11's structural finding: fewer rounds with larger refills outperform more
rounds with smaller refills because the refill reset is a fixed cost per
boundary. Under AI avoidance with open-lane-biased refills, this may invert:
more boundaries mean more compounding opportunities.

**3-round steep decline (e.g., 60/30/0):** Pool contracts to ~20-25 by pick
25. Fewer resets. Requires early avoidance (picks 3-5) to build density before
the pool shrinks. Risk: Round 3 pool is thin, requiring AI saturation management.

**4-round gradual decline (48/36/21/0):** Each reset is smaller, but with
open-lane bias each refill adds to the player's lane. Late-draft pool ~45-50 —
still large, requiring N = 12 to approach M3 = 2.0.

**For V12:** 3-round steep decline is preferable when avoidance starts early.
4-round gradual works when avoidance is delayed (pick 8+).

### 8. Existing Draft Formats and Pool Shrinkage

**MTG Rochester Draft:** Face-up shared pool, no refills, shrinks to zero.
Signal reading from visible picks. Concentration is natural: whatever nobody
wants persists. No oversampling — players pick freely from all visible cards.

**Ascension Market Row:** 6-card face-up row replaced immediately on purchase.
Pool stays constant size — the continuous-market model (V12 Variable 3-E).
No net shrinkage; not applicable to declining-refill contraction.

**7 Wonders:** Each player's hand shrinks from 7 to 1 over an age. Late-age
scarcity and signal reading emerge naturally from the passing mechanism. Pool
is distributed rather than shared, but the concentration mechanic — competitive
demand on a shrinking supply — is structurally analogous to V12.

**Key insight:** Existing formats concentrate supply through competitive demand
on finite supply; no format uses partial replenishment as V12 does. V12's
declining-refill approach is structurally novel. The closest analogue is a
booster draft where pack supply runs out and late drafters take from the dregs.

---

## Connections

**Research Agent A (AI Avoidance):** Balanced refills actively counteract
avoidance. AI avoidance alone without biased or steep declining refills will
partially self-cancel at each round boundary. Agent A should factor this into
avoidance timing recommendations.

**Research Agent C (Concentration Math):** Section 3 provides the round-by-round
pool model for Agent C to formalize. Key inputs: starting S/A per archetype
(~5.4), S/A lost pre-avoidance (1-2 cards), S/A added by biased refills
(function of bias and volume). Agent C should verify whether 5 S/A in a 20-30
card late pool is achievable.

**Algorithm Design Agents:** The core finding: balanced refills counteract
avoidance; open-lane-biased refills compound it. Design agents should combine
avoidance with biased refills, not balanced. V11 Design 3's post-critique
revision (reverting to pure balanced) may need reconsideration for V12 — the
avoidance interaction changes the calculus.

**Agent 1 (Isolation Test, N=4):** Avoidance + balanced declining refills
produces ~8-15% archetype density in late draft (Section 2 model). This
predicts Agent 1's N=4 result will be well below M3 = 2.0 and sets the
baseline for measuring oversampling's contribution.

---

## Open Questions

1. **What is the S/A trajectory under avoidance + biased refills?** Section 3
   shows 5 S/A in a 20-30 card late pool requires biased refills or high
   starting S/A. How much S/A is lost in picks 1-7 before avoidance stabilizes?

2. **Does open-lane bias help or hurt when avoidance is added?** Open-lane
   bias adds cards to all 3 open lanes (including player's), but also to 2
   other open lanes that AIs don't contest. Does the 3-lane bias give the
   player proportionally less benefit than a 1-lane (player-specific) bias?

3. **What avoidance onset pick is compatible with 3-round vs. 4-round
   structures?** 3-round steep decline needs early avoidance (picks 3-5). Can
   AIs infer the player's archetype that early from depletion patterns with 5
   other AIs creating confounding signal?

4. **Is 120 the right starting pool size?** A larger pool (150-180) with
   steeper decline may provide better early variety (M1) while still
   contracting to 20-30 cards late.
