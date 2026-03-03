# Design Agent 6: Smooth Delivery Architectures

## Key Takeaways

- **Surge+Floor's bimodal distribution is the core experience problem.** Surge
  packs (~2.5 S/A) and floor packs (~1.2 S/A) create alternation where loss
  aversion makes valleys feel ~2x worse than peaks feel good.
- **Smoothness and high M3 are not opposed.** The bimodal pattern is an artifact
  of the surge/non-surge binary. Continuous targeting can deliver comparable
  averages with unimodal distributions.
- **Pair-matching is essential for smooth delivery.** R1 filtering at 75%
  precision needs 3.5 targeted slots for M3=2.0 (impossible in 4-card packs).
  Pair-matching at 85% precision needs only 2.07 slots, enabling every pack to
  have 2 targeted slots without surging.
- **The 40% dual-resonance pool unlocks smooth algorithms.** With ~36
  pair-matched cards per archetype, algorithms sustain 2-3 pair draws per pack
  across 25+ packs without exhaustion.
- **Monotonic escalation creates optimal feel.** Gradually increasing targeted
  slots from 0 to 3 produces the "building momentum" feel Research Agent C
  identified as optimal.
- **M10 compliance requires structural guarantees.** Every post-commitment pack
  must provide >= 1 pair-matched slot to ensure >= 1.5 S/A with >= 95%
  reliability.

______________________________________________________________________

## Five Algorithm Proposals

### 1. Continuous Ramp (CR)

Each drafted card's pair data gradually increases pair-matched slots from 0 to
3\. Pair counters for 8 archetype pairs accumulate; slots = floor(top_pair / K),
capped at 3, remaining random.

| Fitness               |  M3  | M10  | Distribution |
| --------------------- | :--: | :--: | :----------: |
| Optimistic            | ~2.5 | Pass |   Unimodal   |
| Grad. Realistic (36%) | ~1.9 | Pass |   Unimodal   |
| Pessimistic (21%)     | ~1.6 | Pass |   Unimodal   |

### 2. Sliding Window Targeting (SWT)

Targeted slots = floor(count of last 5 picks matching leading pair / 2).
Responsive to recent behavior but creates instability when players draft
off-pair cards.

| Fitness         |  M3  |   M10    | Distribution |
| --------------- | :--: | :------: | :----------: |
| Optimistic      | ~2.4 |   Pass   |   Unimodal   |
| Grad. Realistic | ~1.8 |   Pass   |   Unimodal   |
| Pessimistic     | ~1.5 | Marginal |   Unimodal   |

### 3. Probabilistic Pair Weighting (PPW)

Every slot draws from a pool weighted toward the leading pair, weight increasing
with pair affinity score (pair_count / total_picks). No discrete triggers. Hits
the ~2.0 probabilistic ceiling from V6.

| Fitness         |  M3  | M10  | Distribution |
| --------------- | :--: | :--: | :----------: |
| Optimistic      | ~2.2 | Pass |  Very tight  |
| Grad. Realistic | ~1.7 | Pass |    Tight     |
| Pessimistic     | ~1.4 | Pass |    Tight     |

### 4. Guaranteed Floor + Pair Escalation (GF+PE)

Every post-commitment pack gets >= 1 pair-matched slot; additional slots earned
at thresholds T1=4, T2=8 (permanent). Escalation is monotonic -- quality never
drops.

| Fitness         |  M3  | M10  |   Distribution   |
| --------------- | :--: | :--: | :--------------: |
| Optimistic      | ~2.6 | Pass | Unimodal, narrow |
| Grad. Realistic | ~2.0 | Pass |     Unimodal     |
| Pessimistic     | ~1.7 | Pass |     Unimodal     |

### 5. Adaptive Pair Slots with Jitter (APS-J)

Base slots from pair count + random jitter (-1/0/+1 at 20%/50%/30%). Positive
skew creates pleasant surprises more often than disappointments. Risk: jitter
can create occasional 0-slot packs at low base counts.

| Fitness         |  M3  | M10  |  Distribution  |
| --------------- | :--: | :--: | :------------: |
| Optimistic      | ~2.5 | Pass | Wider unimodal |
| Grad. Realistic | ~1.9 | Pass |    Organic     |
| Pessimistic     | ~1.6 | Pass |    Unimodal    |

______________________________________________________________________

## Champion: Guaranteed Floor + Pair Escalation (GF+PE)

GF+PE uniquely combines three properties: structural dead-pack elimination
(guaranteed floor), projected M3=2.0 under Graduated Realistic fitness, and
monotonically increasing quality. CR is a strong runner-up but non-permanent --
off-pair picks create micro-valleys. PPW hits the probabilistic ceiling. APS-J's
jitter risks dead packs at low base counts.

GF+PE combines Research Agent A's 40% dual-resonance threshold with V5's
pair-matching insight into a smooth delivery mechanism.

______________________________________________________________________

## Champion Deep-Dive

**Player-facing:** "As you draft cards matching your archetype's resonance pair,
packs gradually improve -- first one matched card per pack, then two, then
three. Packs never get worse."

**Specification:** Maintain pair counters for 8 ordered pairs. Each pick adds +2
if primary symbol matches pair's R1, +1 for R2 match. Before each pack (pick
3+): identify leading pair P; compute L = 1 + (1 if counter >= T1) + (1 if
counter >= T2); fill L slots from (R1,R2) pair-filtered pool, remainder random.
Picks 1-2 fully random.

### Example Drafts

**Committed Warriors (Graduated Realistic):** Picks 1-2 random (~1.0 S/A). Pick
3: counter ~3, L=1, S/A ~1.2. Pick 5: counter ~5, L=2, S/A ~1.9. Pick 8: counter
~9, L=3, S/A ~2.7. Picks 8-30: L=3 sustained, minimum S/A ~1.5.

**Explorer committing pick 7:** L=1 through pick 6, counter reaches T1 at pick
7, L=2 (S/A ~1.8). Counter reaches T2 at pick 10, L=3 (S/A ~2.4). Smooth ramp
spanning picks 3-10.

**Flash (worst-case pair, 10% sibling A-tier):** Pair precision = 0.80 +
0.20*0.10 = 82%. At L=3: 3*0.82 + 1\*0.125 = 2.59 S/A. Pair-matching
concentrates home-archetype draws, making even worst-case pairs strong.

### Failure Modes

1. **Pool exhaustion:** 36-card pair pool sustains 66 draws (3/pack x 22 packs)
   at ~1.8 repetitions each -- within acceptable range. Mitigate with
   within-pack no-replacement.
2. **Slow escalation for explorers:** Diverse drafting delays L=3 until pick
   12-13. Intentional -- rewards commitment.
3. **M9 variance risk:** Permanent slots may reduce stddev below 0.8. Random
   slot(s) plus 15-20% B/C-tier cards in the pair pool provide natural
   variation.

### Parameter Variants

| Variant  | T1  | T2  | L=3 by pick | Risk        |
| -------- | :-: | :-: | :---------: | ----------- |
| Fast     |  3  |  6  |     6-7     | M5 too fast |
| Standard |  4  |  8  |     8-9     | Balanced    |
| Slow     |  5  | 10  |    10-11    | Lower M3    |

______________________________________________________________________

## Set Design Specification

### 1. Pool Breakdown by Archetype

| Archetype            |  Total  | Home-Only | Cross-Archetype | Generic |
| -------------------- | :-----: | :-------: | :-------------: | :-----: |
| Each of 8 archetypes |   40    |    22     |       18        |   --    |
| Generic              |   40    |    --     |       --        |   40    |
| **Total**            | **360** |  **176**  |     **144**     | **40**  |

18/40 = 45% cross-archetype rate per archetype.

### 2. Symbol Distribution

|    Symbol Count     | Cards |  %  | Example               |
| :-----------------: | :---: | :-: | --------------------- |
|     0 (generic)     |  40   | 11% | No symbols            |
|      1 symbol       |  32   | 9%  | (Tide)                |
| 2 different symbols |  144  | 40% | (Tide, Zephyr)        |
|      3 symbols      |  144  | 40% | (Tide, Zephyr, Ember) |

### 3. Dual-Resonance Breakdown

| Type             | Cards |  %  | Filtering                             |
| ---------------- | :---: | :-: | ------------------------------------- |
| Generic          |  40   | 11% | Not filtered                          |
| Single-resonance |  32   | 9%  | Matches 2 archetypes on R1            |
| Dual-resonance   |  144  | 40% | ~80% precision on (R1,R2) pair filter |
| Tri-resonance    |  144  | 40% | Same pair precision; R3 adds data     |

Total 2+ resonance: 288 cards (80%), well above the 40% threshold.

### 4. Per-Resonance Pool Sizes

Each resonance: 80 cards as R1, ~180 in any position. Each archetype's
pair-filtered pool: ~36 cards (~20 home S-tier + ~16 sibling/adjacent). Sustains
3 draws/pack over 22 packs at 1.8 reps/card.

### 5. Cross-Archetype Requirements

| Pair                         |  Cross-Arch Cards  | Design Guidance                                               |
| ---------------------------- | :----------------: | ------------------------------------------------------------- |
| Warriors/Sacrifice (Tide)    | 18/archetype (45%) | Materialized triggers, Kindle, combat-relevant death triggers |
| Self-Disc./Self-Mill (Stone) |      18 (45%)      | Void-entry triggers, Reclaim enablers                         |
| Blink/Storm (Ember)          |      18 (45%)      | Characters with cast triggers, Foresee + Materialized         |
| Flash/Ramp (Zephyr)          |      18 (45%)      | Energy-scaling effects, Fast with high-energy upside          |

### 6. What the Card Designer Must Do Differently

1. **Increase dual-resonance cards from 54 to 288.** Every non-generic card
   needs 2+ resonance symbols -- the designer must assign a meaningful secondary
   resonance to each card.
2. **Maintain 45% cross-archetype A-tier rate.** 18 of 40 cards per archetype
   must work in the sibling. Natural for high-overlap pairs
   (Warriors/Sacrifice); requires bridge mechanics for low-overlap pairs
   (Flash/Ramp).
3. **Reduce single-resonance cards from 270 to 32.** The designer should ask:
   "What archetype pair does this card belong to?" and assign symbols
   accordingly.
4. **Design bridge mechanics for difficult pairs.** Flash/Ramp: energy-scaling
   cards ("Fast. Pay extra energy to draw that many cards"). Blink/Storm:
   "Materialized: Copy the next Event you play this turn."
