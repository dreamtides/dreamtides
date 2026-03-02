# Results -- Lane Locking (Corrected Archetype-Level Metrics)

## One-Sentence Algorithm

"Your pack has 4 slots; when your symbol count in a resonance first reaches 3,
one open slot locks to that resonance; when it first reaches 8, a second slot
locks." Primary symbol = 2, secondary/tertiary = 1 each. Max 4 locked slots.

## Correction Summary

Prior simulation measured openness as "unique resonances per pack" and
splashability as "B-tier cards." Both operate at the resonance level -- a
resonance like Tide is shared by 4 archetypes, so matching a resonance is far
easier than matching a specific archetype. This version measures everything at
the archetype level: S/A-tier for target archetype, C/F-tier for off-archetype,
and unique archetypes with S/A representation for openness.

Fitness model also corrected: A-tier now correctly covers both adjacent
archetypes sharing the home's primary resonance. Each card: S=1, A=2, B=2,
C=1, F=2 across 8 archetypes. Pool baseline: ~122/360 S/A-tier cards per
archetype (34%), giving 1.36 expected S/A per random 4-card pack.

## Target Scorecard (archetype_committed, 1000 drafts)

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Picks 1-5: unique archetypes w/ S/A per pack | >= 3 | 6.49 | PASS |
| Picks 1-5: S/A cards for archetype per pack | <= 2 | 1.93 | PASS |
| Picks 6+: S/A cards for archetype per pack | >= 2 | 2.72 | PASS |
| Picks 6+: C/F-tier cards per pack | >= 0.5 | 0.84 | PASS |
| Convergence pick | 5-8 | 6.1 | PASS |
| Deck concentration | 60-80% | 99.0% | FAIL |
| Card overlap (Jaccard) | < 40% | 5.4% | PASS |
| Archetype frequency | 5-20% each | 8.2-18.8% | PASS |

**7/8 passed** (up from 5/8 in prior version). The deck concentration failure
occurs because committed players find S/A options in nearly every pack (34%
baseline, boosted by locks). Power-chasers hit 61.4% -- the target suits
mixed-strategy drafting rather than fully committed play.

## Parameter Sensitivity

**Threshold pairs** (archetype_committed):

| Pair | Late S/A | Conv. | Conc. | Off C/F |
|------|----------|-------|-------|---------|
| (2,6) | 2.70 | 6.1 | 99.4% | 0.87 |
| (3,8) | 2.67 | 6.1 | 98.8% | 0.84 |
| (4,10) | 2.67 | 6.2 | 98.5% | 0.83 |
| (5,12) | 2.64 | 6.4 | 98.1% | 0.83 |

All pairs perform similarly. (3,8) recommended.

**Single threshold at 3:** Late S/A = 1.95 (below 2.0 target). The second lock
at threshold 8 is necessary for convergence.

**Lock cap 4 vs 6:** Identical results (2.67 vs 2.69). Cap 4 is simpler.

**Symbol distribution:** Minimal impact across all distributions tested
(60/25/15 through 15/35/50). Thresholds are reached quickly regardless.

## Draft Traces

**Early Committer (Self-Mill):** Stone and Tide lock at pick 2 (counts hit 3).
Second Stone lock at pick 4 (count hits 8). Sees 2-4 S/A cards per pack from
pick 3 onward. Final: 100% S/A, convergence at pick 6.

**Power Chaser:** Picks scatter across archetypes. Locks trigger for Zephyr
(pick 3), Tide (pick 4), Stone (pick 6), second Tide (pick 7). Ends at 60% S/A
for best archetype (Sacrifice). Convergence at pick 6.

**Signal Reader:** Detects Zephyr, commits to Ramp. Zephyr locks at pick 1
(one [Zephyr/Zephyr] card = count 3 instantly). Stone locks incidentally at
pick 3. All 4 slots locked by pick 4. Final: 96.7% S/A, convergence at pick 6.

## Self-Assessment

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | 9 | One sentence captures complete algorithm; player predicts pack structure from symbol counts. |
| 2. Not on rails | 7 | 2.72 S/A per 4-card pack leaves ~1.3 slots for choice; early packs wide open (6.5 archetypes). |
| 3. No forced decks | 8 | 5.4% Jaccard overlap; archetype frequency 8-19% across all 8. |
| 4. Flexible archetypes | 7 | A-tier adjacency makes neighboring archetypes viable. |
| 5. Convergent | 9 | 2.72 S/A post-commitment, pick 6.1 convergence, 100% convergence rate. |
| 6. Splashable | 8 | 0.84 C/F cards per pack; off-archetype options appear regularly. |
| 7. Open early | 9 | 6.49 archetypes represented in early packs; emerging S/A capped at 1.93/4. |
| 8. Signal reading | 5 | Algorithm responds to drafted cards, not offered cards; limited signal advantage. |
