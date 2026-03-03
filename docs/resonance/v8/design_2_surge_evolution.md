# Agent 2: Surge Framework Evolution

## Key Takeaways

- **Bimodal quality is Surge's core weakness.** Surge+Floor alternates ~2.5 S/A
  (surge) and ~1.2 S/A (floor). Loss aversion makes floor packs feel twice as
  bad as surge packs feel good. M10 fails at 3-4 consecutive floor packs.
- **Pair-filtering on 40% dual-resonance pool changes Surge's ceiling.**
  Pair-matched filtering achieves ~85% S/A precision under Pessimistic fitness
  vs. R1-filtering's 62.5% (Research Agent A). This is the highest-leverage
  integration.
- **Spreading surge energy requires fractional slot allocation.** Converting
  tokens into per-slot probability (instead of threshold-triggered bulk
  spending) produces unimodal quality distributions, eliminating the bimodal
  problem.
- **Token decay creates natural steady-state.** A small per-pick decay (-0.5)
  prevents runaway convergence and produces a stable targeting probability for
  committed drafters.
- **Per-pair fitness variance demands the pair-matching bypass.** Flash/Ramp at
  10-25% sibling A-tier vs. Warriors/Sacrifice at 35-50% (Agent B). Pair-matched
  slots draw home-archetype cards directly, sidestepping the sibling fitness
  problem entirely.
- **40% dual-resonance is the minimum for pair-matching sustainability.** At 15%
  (V7), only ~7 pair-matched cards per archetype. At 40%, ~18 -- enough for 2-3
  targeted slots per pack across a 30-pick draft.

______________________________________________________________________

## Five Algorithm Proposals

### 1. Continuous Surge (Champion)

**Description:** Each drafted symbol's tokens set a persistent per-slot
probability of pair-matched targeting, replacing threshold-based bulk spending.

**Technical:** Maintain 4 counters (+2 primary, +1 secondary per pick). Each
slot independently targets with P = min(C_max / K, P_max). Decay -0.5/pick
prevents runaway. Targeted slots draw from pair-matched subpool (40% dual-res
pool required). Committed drafters stabilize at ~2.5-3.0 targeted slots/pack.

**Fitness predictions:** Optimistic M3 ~2.4; Graduated Realistic (36% avg) M3
~1.9-2.1; Pessimistic (21% avg) M3 ~1.6-1.8; Hostile M3 ~1.3.

### 2. Graduated Surge

**Description:** Packs gain targeted slots one at a time at successive token
thresholds (T1=2, T2=5, T3=9) rather than jumping from 1 to 3.

**Technical:** Tokens never spent, only accumulated. Monotonic quality ramp
matching "7 Wonders progression." Pair-matching on 40% dual-res pool.

**Fitness predictions:** Optimistic M3 ~2.3; Graduated Realistic M3 ~1.8;
Pessimistic M3 ~1.5. Risk: M9 variance likely too low (no regression = monotonic
= sterile).

### 3. Surge+Bleed

**Description:** Surges spend only 2 of 3 tokens and allocate 2 targeted slots;
the residual token bleeds into the next pack as 1 guaranteed slot.

**Technical:** Creates 2-1-2-1 alternation instead of V7's 3-1-3-1. Gap between
pack types narrows from ~1.3 S/A spread to ~0.7.

**Fitness predictions:** Optimistic M3 ~2.2; Graduated Realistic M3 ~1.7;
Pessimistic M3 ~1.4. Strong M10 but lower ceiling than V7.

### 4. Pair-Escalation Surge (V5+V7 Hybrid)

**Description:** Pair-escalation probability (V5-style) replaces threshold-based
surge, with a permanent 1-slot floor from pick 4.

**Technical:** P(targeted per slot) = min(pair_count / (2 * pick_number), 0.75).
Organic variance from probabilistic allocation. 40% dual-res pool required.

**Fitness predictions:** Optimistic M3 ~2.5; Graduated Realistic M3 ~2.0;
Pessimistic M3 ~1.7. Risk: V5's probabilistic model untested under realistic
fitness; high variance may produce occasional dead packs despite floor.

### 5. Adaptive Surge

**Description:** Surge threshold adjusts per-resonance based on inferred subpool
quality -- low-overlap pairs get more aggressive targeting.

**Technical:** Tracks whether player picks or skips pair-matched cards to
estimate pair quality. Low-quality pairs: T=2, 3 slots. High-quality pairs: T=3,
2 slots. Hidden state required.

**Fitness predictions:** Optimistic M3 ~2.3; Graduated Realistic M3 ~1.9;
Pessimistic M3 ~1.6. Risk: behavioral inference is noisy; false compensation
could degrade experience.

______________________________________________________________________

## Champion: Continuous Surge

**Justification:** The only proposal that fundamentally eliminates bimodal
quality distribution. Graduated Surge sacrifices variance (M9). Surge+Bleed
narrows but does not eliminate the bimodal gap. Pair-Escalation Surge relies on
an unvalidated V5 model. Adaptive Surge adds fragile behavioral inference.
Continuous Surge spreads energy smoothly, uses pair-matching for precision, and
maintains natural variance through probabilistic slot allocation.

### Example Draft (Graduated Realistic, Warriors, K=6, P_max=0.75, decay=0.5)

- **Picks 1-3:** Counter ~1.5. P(targeted)=25%. ~1.0 targeted slot/pack. Mostly
  random, exploratory. ~1.0 S/A.
- **Picks 4-7:** Counter ~3.0. P=50%. ~2.0 targeted slots. Quality ramp visible.
  ~1.6 S/A.
- **Picks 8-15:** Counter ~4.5 (steady state). P=75%. ~3.0 targeted slots.
  ~2.0-2.1 S/A.
- **Picks 16-30:** Steady state. Some packs: 4 targeted (~2.5 S/A). Some: 2
  targeted (~1.5 S/A). Natural binomial variance, no cyclical pattern.

### Failure Modes

1. **M5 too early:** If K too low, convergence at pick 4. Mitigate with K=6,
   decay=0.5.
2. **M9 too low:** Binomial variance at p=0.7 with 4 slots: stddev ~0.92
   targeted slots, translating to M9 ~0.8-1.0. Marginal but passing.
3. **Pair subpool exhaustion:** 18 cards viewed ~3.5 times each over 25 picks.
   At upper bound; fallback to R1-only filtering when pair pool is locally
   depleted.

### Parameter Variants

| Variant      |  K  | P_max | Decay | Steady-State Slots | Convergence Pick |
| ------------ | :-: | :---: | :---: | :----------------: | :--------------: |
| Conservative |  8  | 0.65  |  0.5  |      2.2-2.6       |      ~9-10       |
| Balanced     |  6  | 0.75  |  0.5  |      2.7-3.0       |       ~7-8       |
| Aggressive   |  5  | 0.80  |  0.3  |      3.0-3.2       |       ~5-6       |

### Fitness Models for Testing

1. **V7 Moderate (50% uniform):** Backward comparison. Expected M3 ~2.0.
2. **Graduated Realistic (50/40/30/25% per pair):** Primary target. Expected M3
   ~1.9 avg, ~1.6 worst-archetype.
3. **Pessimistic (35/25/15/10%):** Stress test. Expected M3 ~1.6 avg.
4. **Hostile (0-10% uniform):** Floor test. Expected M3 ~1.3.

______________________________________________________________________

## Set Design Specification

### 1. Pool Breakdown by Archetype

| Archetype            |  Total  | Home-Only (1 symbol) | Cross-Archetype (dual-res) | Generic |
| -------------------- | :-----: | :------------------: | :------------------------: | :-----: |
| Each of 8 archetypes |   40    |          22          |             18             |   --    |
| Generic              |   40    |          --          |             --             |   40    |
| **Total**            | **360** |       **176**        |          **144**           | **40**  |

### 2. Symbol Distribution

|      Symbol Count       | Cards |  %  | Example        |
| :---------------------: | :---: | :-: | -------------- |
|       0 (generic)       |  40   | 11% | No symbols     |
|     1 (single-res)      |  176  | 49% | (Tide)         |
| 2 (different, dual-res) |  144  | 40% | (Tide, Zephyr) |

### 3. Dual-Resonance Breakdown

| Type                               | Cards |  %  | Filtering Effect                               |
| ---------------------------------- | :---: | :-: | ---------------------------------------------- |
| Single-resonance                   |  176  | 49% | 2 archetypes on R1 filter                      |
| Dual-resonance (archetype-aligned) |  144  | 40% | 1 archetype on pair filter, ~85% S/A precision |
| Generic                            |  40   | 11% | No filtering benefit                           |

Each archetype-aligned pair gets 18 dual-res cards. (Tide, Zephyr) and (Zephyr,
Tide) are distinct pools -- primary symbol determines home archetype.

### 4. Per-Resonance Pool Sizes

| Resonance |         R1 Filter Pool          | Home-Archetype % (R1 only) | Pair-Filter Pool (per archetype) |
| --------- | :-----------------------------: | :------------------------: | :------------------------------: |
| Each of 4 | 116 cards (80 single + 36 dual) |      50% (same as V7)      |      18 cards (~100% home)       |

When pair-filtering for Warriors (Tide, Zephyr): 18 cards, all designed as
Warriors-home. Pair-matched slots bypass sibling fitness entirely.

### 5. Cross-Archetype Requirements

Under Graduated Realistic fitness, sibling A-tier rates for the 80
single-resonance cards per resonance determine R1-fallback slot quality. But
pair-matched slots (the majority of targeted slots) draw from the 18
home-archetype dual-res cards, requiring NO cross-archetype fitness. The
algorithm's fitness sensitivity is reduced to only the random and R1-fallback
slots.

Per-pair cross-archetype targets (for R1-fallback only):

- Warriors/Sacrifice: 50% (20 of 40 cards)
- Self-Discard/Self-Mill: 40% (16 of 40)
- Blink/Storm: 30% (12 of 40)
- Flash/Ramp: 25% (10 of 40)

### 6. Card Designer Guidance

**Key change from V7:** Create 144 dual-resonance cards (up from 54). Per
archetype, 18 of 40 cards carry both primary and secondary resonance symbols.

**The secondary symbol is a filtering tag, not a fitness promise.** A Warriors
card with (Tide, Zephyr) need not be playable in Ramp. It must be a good
Warriors card that has some thematic connection to Zephyr (speed, agility,
wind). The connection can be loose.

**Cross-archetype fitness burden is lighter than V7.** V7 required 50-65%
sibling A-tier across all cards. Continuous Surge with pair-matching only needs
fitness on R1-fallback slots (minority of targeted draws). The designer can
focus energy on making each archetype's 40 cards individually excellent rather
than universally cross-playable.

**Generics increase from 36 to 40** for slightly more splash availability.
