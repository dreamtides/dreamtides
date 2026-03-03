# Design 2: Small Pool, Fast Cycles

## Key Findings

- **Frequent refills make the reset problem worse, not better.** Each balanced
  refill washes out ~half the concentration gradient built during the preceding
  round. With 5-6 rounds instead of 3, the refill reset happens more often,
  compounding the damage. For small-pool/fast-cycle to work at all, per-round AI
  removal must dramatically outpace per-round refill in AI lanes.

- **The pool math is severe at 60 cards.** 6 drafters removing 1 card each per
  pick means 30 cards gone after 5 picks — half the pool. The remaining 30 cards
  are weighted toward C/F cards (AIs preferentially took S/A first). The player
  is picking from a depleted, low-quality residue by pick 4 or 5 of every round.
  This is not scarcity that creates urgency; it is scarcity that creates
  frustration.

- **Short rounds cannot establish stable within-round lane signals.** With only
  5-6 picks per round, the player sees 5 pool snapshots before the pool
  refreshes. That is not enough picks to distinguish "this lane is open" from
  "this lane's S/A cards are gone already." By the time the player reads the
  signal, the round is over.

- **Frequent refills destroy signal coherence across rounds.** Each refill
  partially restores the pool toward its baseline distribution, partially
  erasing the information the player accumulated. In a 6-round draft, pool
  state from round 1 is nearly meaningless by round 4 because 5 refills have
  intervened. A committed signal-reader cannot build a multi-round read.

- **The commitment decision is worse, not better, in short rounds.** With 5-6
  picks per round, a player who commits early in round 1 has ~4-5 remaining
  picks to capitalize on their read before the pool resets. The value of early
  commitment is compressed. Conversely, a player who waits until round 2 to
  commit has lost very little from the round 1 pool, reducing commitment pressure.

- **The key variable is AI aggression per round vs. refill volume.** If each AI
  takes 5 cards in 5 picks (all archetype-aligned), and the refill adds only 5
  cards per archetype back, the round's net depletion is zero in AI lanes. Fast
  cycles with balanced refills can never build concentration — they run in place.
  The only escape is strongly asymmetric refills (favor open lanes) or
  aggressively small refills (partial only).

- **Small-pool fast cycles can be rescued if refills are partial and declining.**
  The structure has one genuine advantage: frequent declining refills create a
  natural concentration ramp with more inflection points than a 3-round design.
  If each refill is 60% of consumed cards, and later refills are smaller, the
  pool concentration can ramp more steeply than a 3-round declining design.

---

## Three Algorithm Proposals

### Algorithm 2A: Pure Fast Cycle (Balanced Refills)

**Description:** 6 rounds of 5 picks each; balanced refills to 60 cards at each
round boundary; Level 0 AIs; baseline test of whether the structure works at all.

**Technical spec:**
- Pool: 60 cards at start of each round (8 archetypes x 7.5 cards each)
- Rounds: 6; Picks/round: 5; Total picks: 30
- Refill: Full replenishment to 60 at each boundary (add 30 cards, balanced)
- AIs: 5, Level 0, each assigned one archetype, pick highest fitness card
- Player info: Archetype availability bars updated per pick

**Predicted metrics:**
- M3: ~0.9-1.1 (worst case; frequent refills actively reset concentration)
- M10: ~5+ (consecutive bad packs expected; pool quality crashes mid-round)
- M11': ~1.0-1.3 (no late-draft concentration accumulation)
- M6: ~40-55% (diluted decks due to poor mid-round picks)
- M12: ~0.1 (signals reset each round; signal-reader gains minimal advantage)

**Verdict:** Does not work. Full balanced refills at this frequency guarantee the
reset problem dominates. This is V10's failure mode with more reset events.

---

### Algorithm 2B: Fast Cycle with Aggressive Partial Refills

**Description:** 6 rounds of 5 picks; refills are partial (50% of consumed) and
declining (round 1: 50%, rounds 2-4: 40%, rounds 5-6: 25%); AI aggression
escalates per round to outpace declining refill volume in AI lanes.

**Technical spec:**
- Pool start: 72 cards (9 per archetype; slightly larger to buffer round 1)
- Rounds: 6; Picks/round: 5; Total picks: 30
- Refill schedule: After round 1: +15 cards (50% of 30 consumed); after rounds
  2-4: +12 cards (40%); after rounds 5-6: +7 cards (25%)
- Refill bias: Balanced across 8 archetypes; no lane preference
- AIs: 5, Level 0 with escalating focus; pick top-2 fitness in rounds 1-3, top-1
  fitness only in rounds 4-6 (simulating late-draft tightening)
- Player info: Round-start snapshot + per-pick archetype bars

**Predicted metrics:**
- M3: ~1.4-1.7 (partial refills reduce reset; declining curve helps late rounds)
- M10: ~3-4 (bad packs possible in round 4-5 as pool thins)
- M11': ~1.8-2.1 (final rounds more concentrated but pool is thin)
- M6: ~60-70% (better than 2A; deck quality improves in later rounds)
- M12: ~0.2-0.3 (round-start snapshots create some signal-reader advantage)

**Verdict:** Better but still unlikely to reach M3 >= 2.0. Pool becomes too thin
by rounds 4-5 (estimated 30-35 cards in round 5 pool), risking thin-pool
frustration. Achieves M11' near target by accident (thin pool = fewer C/F
cards remaining), but for the wrong reason.

---

### Algorithm 2C: Fast Cycle with Partial Refills + Open-Lane Bias

**Description:** 5 rounds of 6 picks; partial declining refills (70%/55%/40%);
refills biased toward the 3 open (uncontested) archetypes; Level 1 pool-reactive
refill sourcing justified as "market favors unsought goods."

**Technical spec:**
- Pool start: 66 cards (8 archetypes x ~8 cards; dual-symbol cards counted once)
- Rounds: 5; Picks/round: 6; Total picks: 30
- Refill schedule: After round 1: +24 cards (70%); after rounds 2-3: +20 cards
  (55%); after round 4: +14 cards (40%); no round 5 refill (final round)
- Refill bias: Track 5 "hot" archetypes (fastest depletion rate = AI lanes).
  Refill allocates 70% of new cards to the 3 "cold" archetypes (slow depletion
  = open lanes), 30% spread across hot archetypes. This is Level 1 pool-reactive
  (reads depletion rate, not player picks).
- AIs: 5, Level 0, pick highest fitness. Saturation mechanic: after 8 on-archetype
  picks, AI fills generics and adjacents.
- Player info: Round-start snapshot showing "busy" vs "quiet" archetype bars;
  depletion trend arrows (up = being taken fast, down = slow)

**Predicted metrics:**
- M3: ~1.8-2.2 (open-lane bias directly counteracts refill reset)
- M10: ~2-3 (some bad packs as pool thins in rounds 4-5; manageable)
- M11': ~2.2-2.6 (final round pool is thinner and more concentrated)
- M6: ~68-78% (better deck construction from open-lane bias)
- M12: ~0.3-0.4 (depletion trend arrows give signal-readers actionable info)

**Verdict:** The strongest proposal in this space. Open-lane bias directly targets
the refill reset problem; declining volume creates concentration ramp; 5 rounds
is more structured than 6 but still "fast cycle." The primary risk is the Level
1 pool-reactive refill bias being perceived as a rigged mechanic if the depletion
tracking is visible to the player.

---

## Champion: Algorithm 2C

**Justification:** 2A fails outright. 2B improves on 2A but cannot reach M3 >= 2.0
without open-lane concentration pressure. 2C's refill bias is the minimum
additional mechanism needed to counteract the refill reset problem that balanced
refills structurally cannot solve. The bias is Level 1 pool-reactive (reads
depletion rate, not player identity), which is explicitly permitted by the V11
brief. The 5-round structure is simpler than 6 rounds and creates a cleaner
exploration/commitment/consolidation/execution arc with a brief finale.

---

## Champion Deep-Dive: Algorithm 2C

### Round-by-Round Walkthrough

**Round 1 (Picks 1-6) — Open Exploration**

Pool opens at 66 cards. Each archetype has 8 cards on average (mix of S/A and
C/F). All 5 AIs pick their archetype's best card; player sees 6 choices. By pick
3-4, AIs have taken 3 S/A cards from each of their 5 lanes; the 3 open lanes
remain untouched (except by the player). The player can observe archetype bars
beginning to diverge: Tide/Ember/Stone AI lanes are visibly shrinking faster than
Zephyr/Stone/Ember open lanes (depends on which 5 archetypes have AIs).

After pick 6: 30 cards consumed (5 AIs + 1 player x 5 picks... wait: 6 picks x 6
drafters = 36 cards consumed). Pool has 30 cards remaining. The player has 6 picks
and should be able to identify 2-3 candidate archetypes from the round-start
snapshot + observed depletion.

Refill: 70% of 36 = ~25 cards. Bias applies: 17-18 of 25 go to the 3 cold
(open-lane) archetypes (~5-6 each), 7-8 spread across the 5 hot (AI) archetypes.
Round 2 pool: 30 + 25 = 55 cards. Open lanes now have noticeably more cards than
AI lanes.

**Round 2 (Picks 7-12) — Emerging Commitment**

Round-start snapshot shows the concentration gap widening: cold archetypes have
6-7 cards each; hot archetypes have 3-4 each. A signal-reading player can now
identify which archetypes are truly open. Commitment pressure is real: the player
has used 6 of 30 picks and should commit by pick 8-10 (M5 target).

AIs continue draining their lanes. After 6 more picks (36 total, 6 per round),
pool drops to ~19 cards. Open-lane concentration is noticeable: the pool is
skewing toward cold-lane cards because AIs keep removing hot-lane cards.

Refill: 55% of 36 = ~20 cards. Bias: 14 to cold lanes (~4-5 each), 6 to hot
lanes. Round 3 pool: 19 + 20 = 39 cards.

**Round 3 (Picks 13-18) — Commitment Locked**

Pool is 39 cards. Player is committed. The open-lane concentration is now clearly
visible: each open archetype has 8-10 cards; each AI archetype has 4-5. A player
committed to their open lane sees 2-3 S/A cards per round from their lane plus
1-2 useful adjacents.

After 6 picks: pool at 3 cards. Refill: 40% of 36 = ~14 cards. Bias: 10 to cold,
4 to hot. Round 4 pool: 3 + 14 = 17 cards. Pool is thin and concentrated.

**Round 4 (Picks 19-24) — Concentration Zone**

17-card pool is majority cold-lane cards. AI archetypes are now S/A-depleted;
AIs are picking generics and adjacents (saturation mechanic triggers at 8
on-archetype picks, which AIs hit around pick 18-20). The pool's quality is
concentrated in open-lane S/A cards. Player picks 2-3 S/A cards per round from
their lane. M11' picks begin here.

After 6 picks: pool nearly exhausted. No refill for round 5 — this is the
intended design.

**Round 5 (Picks 25-30) — Final Consolidation**

No refill. Pool has ~5-8 cards remaining (the residue from round 4 plus whatever
the AIs left). AIs are fully saturated and picking generics. The player picks
whichever open-lane cards remain plus filler. Pool quality is highest here in
relative terms (open-lane S/A cards have accumulated) but lowest in absolute card
count. Urgency is natural.

### What the Player Sees

- Round-start snapshot: bar chart showing archetype card counts in the current
  pool (relative, not exact). "Tide: many available. Ember: few available."
- Depletion trend arrows: updated after each pick. "Tide: shrinking fast (AI
  activity). Zephyr: stable (no AI pressure)."
- After each of their picks, the player sees the bars update. By mid-round 2,
  the signal is clear without being trivially obvious.

### Pool Composition Evolution

| Round Start | Total Cards | AI-Lane/Arch | Open-Lane/Arch |
|:-----------:|:-----------:|:------------:|:--------------:|
| Round 1     | 66          | 8.0          | 8.0            |
| Round 2     | 55          | 4.8          | 7.5            |
| Round 3     | 39          | 3.2          | 5.8            |
| Round 4     | 17          | 1.8          | 3.4            |
| Round 5     | ~6          | 0.5          | 1.0            |

Open-lane archetypes maintain ~2-3x more cards than AI-lane archetypes by round 4.

### Failure Modes

**F1: Round 3-4 quality crash.** If the player picks slowly in their lane or
commits late, they arrive at round 4's thin pool without enough S/A cards built.
The pool at round 4 has only 17 cards; a bad seed could have 0 S/A in the
player's lane.

**F2: Open-lane bias miscalibration.** If the "cold/hot" detection algorithm
misclassifies a lane (e.g., player's lane appears cold because the player is
picking from it slowly), the refill bias could under-serve the player's actual
lane. Solution: bias on archetype-level depletion rate, not individual card
counts.

**F3: Cognitive overload from frequent round boundaries.** 5 rounds means 4
refill events, each with a new snapshot to process. Players may experience
decision fatigue. Mitigation: keep the round-start snapshot simple (8 bars, not
detailed counts).

---

## Complete Specification

| Parameter | Value |
|-----------|-------|
| Pool size (round 1) | 66 cards |
| Cards per archetype | ~8.25 (66 / 8) |
| Rounds | 5 |
| Picks per round | 6 |
| Total picks | 30 |
| Refill schedule | Round 1: +25; Rounds 2-3: +20; Round 4: +14; Round 5: none |
| Refill bias | 70% to cold lanes (3 slowest-depleting archetypes), 30% to hot |
| Bias detection | Depletion rate = cards removed per archetype since last refill |
| AI count | 5 |
| AI pick logic | Level 0; highest fitness in own archetype; saturation at 8 picks |
| AI saturation | After 8 on-archetype picks: pick highest generic/adjacent fitness |
| Player information | Round-start snapshot (8 archetype bars) + per-pick depletion arrows |
| Quality visible | No — bars show card counts, not S/A density |
| Visible symbol distribution | 11% generic, 79% single, 10% dual (per refill and starting pool) |
| Fitness model | Graduated Realistic |
| Reactivity level | Level 1 (pool-reactive refill bias) — justified by depletion tracking |

---

## Post-Critique Revision

### Accepting the Last-Place Ranking

The 6th-place ranking is correct. The structural argument is sound and I
accept it: 4 refill events is worse than 2 refill events, not better. Each
refill event is a partial concentration reset, and paying that cost 4 times
means the design enters each round having partly erased what the previous round
built. Open-lane bias (70%) can slow the erosion but cannot eliminate it. The
concentration gradient in Algorithm 2C is built against headwinds that 3-round
designs do not face.

### On Cognitive Overload

I flagged F3 (cognitive overload) as a failure mode in the original design but
did not weight it heavily enough. The critic is right: 5 rounds x 6 picks x 4
refill snapshots x depletion-rate arrows is a meaningful UI burden compared to
a 3-round design where the player processes 2 refill events. For a roguelike
draft where the player is also managing their deck construction decisions, the
added cognitive surface is a real cost, not just a perception problem.

The mitigation I proposed (keep bars simple) does not address the underlying
issue. Simpler bars reduce information density but do not reduce the number of
times the player must update their mental model of pool state.

### On the 4 Resets vs. 2 Resets Argument

The critique's structural argument holds: even with 70% open-lane bias per
refill, 4 resets accumulate more total dilution than 2 resets would. The open-
lane bias in 2C reduces each reset's damage, but "4 smaller resets" cannot
reliably beat "2 equivalent resets" unless the bias is strong enough to
compound — i.e., each open-lane refill must more than recover the concentration
the reset erased. The numbers in Algorithm 2C do not achieve this: at 70% bias
toward 3 of 8 archetypes, each cold-lane refill adds ~6 cards to a lane that
lost ~2-3, which is a net positive but not a compounding one.

### Would I Modify to a 3-Round Variant?

Yes. Given the structural critique, the correct move is to take 2C's best
mechanism — open-lane bias in refills — and port it to a 3-round structure
where it can work with the round boundary schedule rather than against it. A
3-round design with 10 picks per round, open-lane biased partial refills at
round boundaries, and depletion-rate arrows would preserve the signal
infrastructure of 2C while paying the reset cost only twice. The fast-cycle
frame does not offer enough unique upside (M3 delta is marginal per the critic)
to justify its structural penalties. The open-lane bias mechanism is worth
preserving; the 5-round cadence is not.

The champion algorithm 2C is the best this design space can do and it is still
not competitive. The lesson is that mechanism quality (bias) matters less than
structural reset count at this parameter range.
