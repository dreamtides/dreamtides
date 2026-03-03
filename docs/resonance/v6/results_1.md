# Agent 1 Results: Lane Locking + Auto-Spend Pack Widening

## One-Sentence Algorithms

**Lane Locking:** "Your pack has 4 slots; when your weighted symbol count in a
resonance first reaches 3, one open slot locks to that resonance and always
shows a card with that primary resonance; a second slot locks at 8."

**Auto-Spend Pack Widening:** "Each symbol you draft adds tokens (+2 primary,
+1 each secondary/tertiary); when any resonance reaches 3 tokens, 3 are
auto-spent and 1 bonus card of that primary resonance is added to the pack."

## Scorecard

| Metric | Target | Lane Locking | Auto-Spend PW |
|--------|--------|:-:|:-:|
| M1: Unique archs w/ S/A (1-5) | >= 3 | **5.12 PASS** | **5.54 PASS** |
| M2: S/A emerging (1-5) | <= 2 | **1.60 PASS** | **1.06 PASS** |
| M3: S/A committed (6+) | >= 2 | **2.11 PASS** | 1.62 FAIL |
| M4: Off-arch C/F (6+) | >= 0.5 | **0.64 PASS** | **1.41 PASS** |
| M5: Convergence pick | 5-8 | 3.3 FAIL (too fast) | **8.3 BORDERLINE** |
| M6: Deck concentration | 60-90% | **81.9% PASS** | **73.0% PASS** |
| M7: Card overlap | < 40% | **29.0% PASS** | **23.8% PASS** |
| M8: Arch freq range | 5-20% | **6.9-18.7% PASS** | **5.4-19.4% PASS** |
| M9: S/A stddev (6+) | >= 0.8 | 0.49 FAIL | **0.95 PASS** |

**Lane Locking:** 6/9 pass, 2 fail (convergence too fast, variance too low).
**Auto-Spend PW:** 7/9 pass, 1 fail (M3 below 2.0), 1 borderline (M5 at 8.3).

## Critical Finding: 100% S/A in Locked Slots

The Tide-primary pool contains only Warriors-home (S-tier) and Sacrifice-home
(A-tier) cards. For a Warriors player, a Tide-locked slot delivers **100% S/A
cards**, not the ~50-65% estimated in design. This applies to all resonances
symmetrically. The implication: slot-locking approaches have a structural
advantage because each resonance's primary pool contains exactly 2 archetypes
that are S+A for each other. The ~50% dilution fear from V3/V4 was wrong at
archetype level -- it only applies at the S-tier-only level.

## Pack Quality Variance

**Lane Locking** (picks 6+): 0 S/A: 0.1%, 1: 5.5%, **2: 78.5%**, 3: 14.8%,
4: 1.2%. StdDev 0.49. Extremely peaked at exactly 2 -- the locked slots
mechanically guarantee 2 S/A, with variance only from the open slots.

**Auto-Spend** (picks 6+): 0 S/A: 10.3%, 1: 37.3%, 2: 35.4%, 3: 14.5%,
4: 2.5%, 5: 0.2%. StdDev 0.95. Much wider spread -- healthy variance but too
many 0-1 S/A packs.

## Per-Archetype Convergence

| Archetype | LL Pick | LL N | PW Pick | PW N |
|-----------|:-:|:-:|:-:|:-:|
| Flash | 3.4 | 119 | 8.6 | 125 |
| Blink | 3.6 | 64 | 8.0 | 73 |
| Storm | 3.1 | 114 | 8.0 | 109 |
| Self-Discard | 3.1 | 83 | 8.4 | 83 |
| Self-Mill | 3.7 | 41 | 9.3 | 36 |
| Sacrifice | 3.3 | 89 | 7.2 | 86 |
| Warriors | 3.5 | 72 | 8.3 | 75 |
| Ramp | 3.3 | 80 | 9.4 | 67 |

Lane Locking converges by pick 3-4 for all archetypes -- too fast. Auto-Spend
converges around pick 7-9, with Self-Mill and Ramp slightly slow.

## Parameter Sensitivity (Lane Locking, committed-only)

| Thresholds | S/A (6+) | Off-arch | Conv. Pick | Deck Conc. | StdDev |
|:----------:|:--------:|:--------:|:----------:|:----------:|:------:|
| 2, 6 | 1.99 | 0.76 | 3.0 | 94.1% | 0.41 |
| **3, 8** | **2.11** | **0.66** | **3.9** | **93.9%** | **0.44** |
| 4, 10 | 2.18 | 0.62 | 4.8 | 92.7% | 0.51 |

Higher thresholds delay convergence (good) and slightly improve S/A and
variance. Thresholds 4/10 put convergence closer to the target window but
variance remains far below 0.8. The committed-only deck concentration (93-94%)
is much higher than the mixed-strategy average (81.9%).

## Draft Traces (Summary)

**Early Committer (Lane Locking, Self-Mill):** Picks first Stone card at pick 2
(weighted 3 -> slot locks). By pick 5, two Stone slots locked. From pick 6
onward, 2 of 4 pack cards are always Stone-primary (Self-Mill S or
Self-Discard A). Highly consistent 2 S/A per pack.

**Power Chaser (Lane Locking, Storm):** Picks highest power regardless. Ember
slots lock by pick 5 from incidental symbol accumulation. Despite not
optimizing for archetype, still sees 2+ Storm/Blink S/A cards per pack late.
Picks many B-tier Stone cards for power.

**Signal Reader (Lane Locking, Warriors):** Tracks Tide appearing most in early
packs, commits to Warriors. Two Tide slots lock quickly. From pick 7 onward,
locked slots show 4 S/A Warriors/Sacrifice cards in one pack (all Tide-primary
= 100% S/A). Extremely strong convergence.

## Symbol Distribution Used

36 generic (0 symbols), 80-84 mono-1, 136-140 mono-2, 48 mono-3, 30-32 dual-2,
24 dual-3. Total dual: 54 (15.0%). Heavy mono-2 ensures ~3 weighted symbols per
pick for fast threshold accumulation.

## Self-Assessment (1-10 per goal)

| Goal | LL | PW | Notes |
|------|:--:|:--:|-------|
| 1. Simple | 9 | 8 | Both are one-sentence describable |
| 2. No actions | 10 | 10 | Verified: only card selection |
| 3. Not on rails | 3 | 7 | LL locks in permanently; PW adapts |
| 4. No forced decks | 7 | 7 | Large pool prevents forcing |
| 5. Flexible | 3 | 6 | LL cannot pivot; PW tokens shift |
| 6. Convergent | 8 | 4 | LL crosses 2.0; PW fails at 1.62 |
| 7. Splashable | 4 | 7 | LL: 0.64 off-arch; PW: 1.41 |
| 8. Open early | 7 | 8 | Both show 5+ archetypes early |
| 9. Signal reading | 3 | 5 | LL locks too fast to read signals |
