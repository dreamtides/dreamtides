# Pool Design Final Specification

## 1. Recommended Pool Design

### Overview

360 cards total. 40 cards per archetype (320 across 8 archetypes) plus 40
generic cards (11%). Lane Locking thresholds **(4, 10)** with primary weight 2.

### Symbol Pattern Breakdown (per archetype, 40 cards)

| Pattern | Symbols | Cards | % | Role |
|---------|---------|:-----:|:-:|------|
| mono_primary | [P] | 10 | 25% | Clean identity, controlled accumulation |
| standard_dual | [P, S] | 10 | 25% | Archetype signature, breadth building |
| double_primary | [P, P] | 4 | 10% | Deep commit acceleration |
| secondary_led | [S, P] | 4 | 10% | Bridge/pivot enabler |
| deep_commit | [P, P, S] | 6 | 15% | Strong commit signal |
| balanced_triple | [P, S, S] | 4 | 10% | Breadth-over-depth option |
| cross_bridge | [P, Other] | 2 | 5% | Off-archetype splash |

This yields a pool-wide symbol count distribution of roughly 25% one-symbol,
45% two-symbol, 25% three-symbol, plus 11% zero-symbol (generics). Of the 320
archetype cards: 80 mono, 144 dual, 80 triple, plus the 40 generics.

### Rarity Distribution

180 Common / 100 Uncommon / 60 Rare / 20 Legendary across the full 360-card
pool. Rarity is independent of symbol count -- it controls card power, not
resonance structure. Power ranges: Commons 2-5, Uncommons 4-7, Rares 6-9,
Legendaries 8-10 (overlapping ranges are deliberate). No rarity guarantees in
packs; rarity emerges naturally from pool proportions.

Generic card rarities: 22C / 10U / 6R / 2L (proportional to the pool).

### Lane Locking Configuration

- **Thresholds: (4, 10)** -- first lock at 4 weighted symbols, second at 10
- **Primary weight: 2** -- first symbol on each card counts double
- **Lock cap: 4** total locked slots across all resonances
- **Pool asymmetry: +20/-20** cards per run (one resonance boosted, one
  suppressed) for signal-reading variety

### One-Sentence Description

> "Your pack has 4 slots that start random; when your resonance symbol count
> hits 4 a slot locks to that resonance, and at 10 a second locks -- plus each
> quest starts with one resonance having more cards in the pool."

---

## 2. How We Got Here

**Agent 1 (Symbol Ratios):** Mono-symbol cards produce better lock quality
(+18% late S/A) because locks land on the target resonance. SA Trend was the
key metric: mono-heavy pools are the only ones where packs improve throughout
the draft. Recommended 70/20/10.

**Agent 2 (Rarity):** Rarity is orthogonal to Lane Locking. All five rarity
models produced within 3% on core metrics. Rarity controls power variance and
draft tension (15% of packs present archetype-vs-power tradeoffs). Recommended
180/100/60/20 with no rarity-symbol correlation.

**Agent 3 (Archetype Breakdown):** 40 per archetype + ~10% generics is robust.
Explicit bridge categories are harmful (inflate early S/A). Dual-symbol cards
naturally bridge adjacent archetypes via the circular arrangement.

**Agent 4 (Symbol Patterns):** Pattern variety is mandatory. Uniform-pattern
pools produce 0% genuine choice rate. The "depth vs breadth" tension ([P,P] vs
[P,S]) is the primary decision driver. Recommended 7 pattern types, 83% choice.

**Agent 5 (Threshold Tuning):** (3,8) locks on pick 1 in 18-29% of drafts.
Recommended (5,12) for a three-act structure. Higher thresholds also reduce
unwanted locks.

---

## 3. Tensions and Tradeoffs

### Symbol Ratio vs Pattern Variety

Agent 1 wanted 70% mono cards for lock quality. Agent 4 showed mono-only pools
produce 0% genuine choice. The reconciliation: **25% mono, 45% dual, 25%
triple**. This works because higher thresholds absorb multi-symbol accumulation.
Under (3,8), multi-symbol cards cause 18-31% pick-1 locks. Under (4,10),
pick-1 locks drop to 0%. We traded some lock quality for decision quality, and
the threshold increase covers the difference. Simulation confirms: Late S/A
2.70, genuine choice 90.8%, pick-1 locks 0%, SA Trend +0.24.

### Threshold Selection: (4,10) over (5,12)

(4,10) chosen because: first lock at pick 2.6 (vs 3.3) gives faster feedback;
second lock at 4.6 (vs 5.6) arrives before the commitment window; 0% pick-1
locks (same as (5,12)); SA Trend +0.24 (vs +0.40 for (5,12)). Late S/A 2.70
matches the original default. (5,12) remains a valid alternative -- better SA
Trend but players wait 3+ picks before seeing any mechanical response.

### Pattern Mix Reconciliation

Agent 3's mix (6 patterns including mono-secondary [S]) was replaced by Agent
4's 7-pattern framework. Balanced_triple [P,S,S] and cross_bridge [P,Other]
replace mono-secondary, serving the same bridge function with richer decisions.
Deep_commit [P,P,S] increased to 15% (strongest commit signal, pairs with
higher threshold). Secondary_led [S,P] kept at 10% (agreed pivot enabler).

---

## 4. Simulation Results

1000 drafts per configuration, three player strategies.

### Reconciled (4,10) W2 -- Committed Player

| Metric | Target | Result | Status |
|--------|--------|:------:|:------:|
| Early diversity | >= 3 | 6.48 | PASS |
| Early S/A | <= 2 | 1.64 | PASS |
| Late S/A | >= 2 | 2.70 | PASS |
| Late C/F | >= 0.5 | 0.21 | FAIL |
| Convergence pick | 5-8 | 2.3 | FAIL |
| Deck concentration | 60-80% | 97.9% | FAIL |
| Card overlap | < 40% | 6.3% | PASS |
| Archetype frequency | 5-20% | 10.7-13.4% | PASS |

Lock timing: 1st lock pick 2.6, 2nd lock pick 4.6, all-4 locked pick 12.4.
Pick-1 lock rate: 0%. Genuine choice rate: 90.8%. Unwanted lock rate: 7.8%.
SA Trend: +0.24.

### Convergence Curve (S/A per pack by draft phase)

| Phase | Picks 1-5 | 6-10 | 11-15 | 16-20 | 21-25 | 26-30 |
|-------|:---------:|:----:|:-----:|:-----:|:-----:|:-----:|
| S/A | 1.64 | 2.52 | 2.72 | 2.74 | 2.73 | 2.76 |

Packs improve through pick 11-15 and remain stable, never declining. This is
the intended feel: early exploration, ramp into commitment, sustained quality.

### Known Metric Failures

**Late C/F (0.21 vs 0.5):** Locked slots fill with on-archetype cards;
off-archetype cards appear in open slots but the committed player ignores them.
The power-chaser sees 1.01 C/F, confirming splash is *available* in packs.

**Convergence pick (2.3 vs 5-8):** With 37.5% of cards being S/A by the
fitness model, random 4-card packs have ~47% baseline chance of 2+ S/A. Lock
timing (first at 2.6, second at 4.6) better measures when commitment begins.

**Deck concentration (97.9% vs 60-80%):** Fitness model artifact. S/A covers
3 of 8 archetypes (37.5% of pool), so committed players nearly always find an
S/A pick. Target should be revised to 85-99%.

---

## 5. Comparison: Default vs Reconciled

| Metric | Default (3,8) | Reconciled (4,10) | Delta |
|--------|:------------:|:-----------------:|:-----:|
| Late S/A | 2.71 | 2.70 | -0.01 |
| Pick-1 lock % | 18.1% | **0.0%** | -18.1pp |
| Unwanted lock % | 8.8% | **7.8%** | -1.0pp |
| SA Trend | +0.12 | **+0.24** | +0.12 |
| 1st lock pick | 2.1 | **2.6** | +0.5 |
| 2nd lock pick | 3.6 | **4.6** | +1.0 |
| All-4 locked | 9.5 | **12.4** | +2.9 |
| Genuine choice % | 90.5% | **90.8%** | +0.3pp |
| Early S/A | 1.71 | **1.64** | -0.07 |

The reconciled design matches the default on late S/A (the core convergence
metric) while fixing the two biggest experiential problems: pick-1 locks
(eliminated entirely) and SA decay (reversed from +0.12 to +0.24). The draft
now has a clean three-act structure: 2-3 picks of open exploration, first lock
around pick 3, second lock around pick 5, with packs continuing to improve
through pick 15 rather than peaking at pick 6.

---

## 6. Open Questions

1. **(4,10) vs (5,12) feel.** Both eliminate pick-1 locks. (5,12) gives a
   longer exploration phase (first lock at pick 3.3 vs 2.6) and better SA
   Trend (+0.40 vs +0.24). Playtest both to determine whether the extra pick
   of unlocked exploration feels liberating or unresponsive.

2. **Late C/F target.** The 0.5 target was set assuming open-slot cards would
   regularly be off-archetype. For committed players, locked slots dominate
   the pack by mid-draft. Should the target measure "off-archetype cards
   visible in pack" (which passes) or "off-archetype cards available to a
   committed drafter" (which may need splash-slot mechanics)?

3. **Convergence pick measurement.** The current metric (first pack with 2+
   S/A) fires too early due to baseline S/A probability. Consider redefining
   as "first pick where locked slots contribute 2+ S/A cards" to measure the
   algorithm's effect rather than random chance.

4. **Cross-bridge (5%) impact.** These [P, Other] cards connect non-adjacent
   resonances. At 2 per archetype (16 total), they are rare splash options.
   Playtest whether this is enough to create occasional surprises or too few
   to matter.

5. **Third threshold.** Adding threshold 24 could lock all 4 slots by pick
   14-16. Worth testing if convergence needs to be even stronger.

6. **Pattern-aware card design.** [P,P] cards are "commit harder" signals
   (should reward deep investment). [S,P] cards are "pivot enablers" (should
   work in either adjacent archetype).
