# Results: Pair-Escalation Slots v2 (Agent 2)

## One-Sentence Algorithm

"Track the resonance pair (first, second symbol) of each 2+ symbol card you
draft; each pack slot independently shows a card matching your most common pair
with probability min(that pair's count / 6, 0.65), otherwise a random card."

No player decisions. Visible properties only. Automatic.

## Target Scorecard (Archetype Level)

| Metric | Target | Actual | P/F |
|--------|--------|--------|-----|
| Picks 1-5: unique archetypes w/ S/A | >= 3 | 6.25 | PASS |
| Picks 1-5: S/A for emerging arch | <= 2 | 2.02 | ~FAIL |
| Picks 6+: S/A for committed arch | >= 2 | 3.00 | PASS |
| Picks 6+: off-archetype (C/F) | >= 0.5 | 0.51 | PASS |
| Convergence pick | 5-8 | 5.8 | PASS |
| Deck concentration | 60-90% | 97.1% | FAIL |
| Run-to-run overlap | < 40% | 29.7% | PASS |
| Archetype frequency | 5-20% | 8.6-17.8% | PASS |

**6/8 pass.** Deck concentration fails high (97.1%) -- pair targeting so
precise players rarely pick off-archetype. Early S/A borderline (2.02).

## Variance Report

StdDev: **0.90** (target >=0.8, PASS). Distribution: 0 S/A: 0.8%, 1: 5.2%,
2: 19.8%, 3: 41.0%, 4: 33.1%. Per-slot coin flip produces natural variance --
6% bad packs, 74% great packs.

## Per-Archetype Convergence

| Archetype | PairEsc | LaneLock | PackWiden |
|-----------|---------|----------|-----------|
| Flash/Tempo/Prison | 7.3 | 6.9 | 8.6 |
| Blink/Flicker | 6.9 | 6.6 | 8.8 |
| Storm/Spellslinger | 8.0 | 6.7 | 9.4 |
| Self-Discard | 7.2 | 6.4 | 9.6 |
| Self-Mill/Reanimator | 6.7 | 6.4 | 9.4 |
| Sacrifice/Abandon | 7.1 | 6.5 | 9.3 |
| Warriors/Midrange | 7.0 | 6.3 | 9.5 |
| Ramp/Spirit Animals | 8.1 | 7.5 | 8.6 |

Range 6.7-8.1. Lane Locking faster (6.3-7.5). Pack Widening fails (8.6-9.6).

## V3/V4 Comparison

| Metric | PairEsc | LaneLock | PackWiden | Random |
|--------|---------|----------|-----------|--------|
| Late S/A | **3.00** | 2.34 | 1.96 | 1.34 |
| Off-arch C/F | 0.51 | **0.70** | 1.50 | 1.35 |
| Conv. pick | **5.8** | 6.6 | 9.5 | 19.1 |
| S/A stddev | 0.90 | **0.96** | 1.08 | 0.94 |
| Deck conc. | 97.1% | 95.2% | 90.5% | **79.9%** |
| Overlap | 29.7% | **16.7%** | 13.0% | 9.5% |

Pair-Escalation dominates convergence (3.00 vs 2.34 vs 1.96). Lane Locking
wins splash and overlap. Pack Widening fails 2.0 S/A. Pair matching achieves
76-93% S-tier precision vs ~50% for single-resonance.

## Symbol Distribution & Sensitivity

Default: 15/60/25 (1/2/3-sym).

| Distribution | Late S/A | Off-arch | StdDev |
|-------------|---------|---------|--------|
| 15/60/25 default | 3.00 | 0.51 | 0.90 |
| 30/50/20 more 1-sym | 2.90 | 0.54 | 0.96 |
| 05/70/25 min 1-sym | 2.99 | 0.51 | 0.90 |

Robust: 30% 1-sym only drops S/A to 2.90 (still >2.0).

## Parameter Sensitivity

| K | Cap | Late S/A | Off-arch | StdDev |
|---|-----|---------|---------|--------|
| 5 | 0.65 | 3.00 | 0.51 | 0.90 |
| 6 | 0.65 | 3.00 | 0.51 | 0.90 |
| 7 | 0.70 | 3.07 | 0.47 | 0.91 |
| 6 | 0.50 | 2.61 | 0.71 | 0.97 |
| 6 | 0.80 | 3.31 | 0.35 | 0.85 |

K insensitive (5-7 equivalent). **Cap is the key lever:** 0.50 gives splash
(0.71) at 2.61 S/A; 0.80 maximizes convergence (3.31) but crushes splash.
Default 0.65 balances both.

## Draft Traces

**Trace 1 (Early Committer, Warriors forced):** Pair (Tide,Stone) hits cap by
pick 7. Picks 6-30 average 2.7 S/A. Mostly picks Sacrifice A-tier cards
(same pair). Final: 30/30 S/A.

**Trace 2 (Flexible, natural Flash/Tempo):** Early Blink pick starts
(Zephyr,Ember). Commits pick 5, cap by pick 8. Picks 8-30 average 3.1 S/A.
Final: 29/30 S/A (96.7%).

**Trace 3 (Signal Reader, Blink/Flicker):** Scattered early, commits pick 5.
Pair builds slowly, cap at pick 12. From pick 12: 3.0 S/A. Final: 25/30 S/A
(83.3%).

## Self-Assessment

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | 6/10 | Formula requires math reasoning from players |
| 2. No actions | 10/10 | Fully automatic |
| 3. Not on rails | 5/10 | 65% slots targeted post-commitment |
| 4. No forced decks | 6/10 | Follows emergent pair, not player-forced |
| 5. Flexible archetypes | 5/10 | Single-pair tracking limits hybrids |
| 6. Convergent | 10/10 | 3.00 S/A, pick 5.8, all 8 archetypes 6.7-8.1 |
| 7. Splashable | 5/10 | Off-arch 0.51 barely passes |
| 8. Open early | 7/10 | No targeting first 3-4 picks |
| 9. Signal reading | 3/10 | Player-internal; pool irrelevant |
