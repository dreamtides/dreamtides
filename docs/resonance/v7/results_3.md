# V7 Agent 3 Results: Aspiration Packs + Pair Preference

## Algorithm Descriptions

**Aspiration Packs + Pair Preference (main champion):** "After each pick,
compute top resonance pair (R1, R2); if R2 >= 3 tokens AND R2 >= 50% of R1,
one slot shows an R1 card (preferring those with R2 symbols), one slot shows an
R2 card, two random; otherwise all four random."

**Pure Aspiration (comparison variant):** Same algorithm but the R1 slot draws
any R1 card without preferring those carrying R2 symbols.

## Scorecard

| Metric | Target | Asp+PP [A] | Asp+PP [B] | Asp+PP [C] | Pure [A] | Pure [B] | Pure [C] | Surge [A] | Surge [B] | Surge [C] |
|--------|--------|-----------|-----------|-----------|---------|---------|---------|----------|----------|----------|
| M1 EarlyUnique | >=3 | 5.19 P | 4.22 P | 3.79 P | 5.19 P | 4.25 P | 3.77 P | 4.69 P | 3.93 P | 3.53 P |
| M2 EarlySA | <=2 | 0.99 P | 0.73 P | 0.63 P | 0.97 P | 0.73 P | 0.65 P | 1.42 P | 1.00 P | 0.79 P |
| M3 LateSA | >=2 | 1.02 F | 0.84 F | 0.78 F | 0.93 F | 0.72 F | 0.63 F | 1.79 F | 1.24 F | 1.03 F |
| M4 OffArch | >=0.5 | 2.98 P | 3.16 P | 3.22 P | 3.07 P | 3.28 P | 3.37 P | 2.21 P | 2.76 P | 2.97 P |
| M5 Conv | 5-8 | 20.3 F | 26.0 F | 27.6 F | 21.9 F | 27.1 F | 28.4 F | 5.8 P | 10.2 F | 14.8 F |
| M6 DeckConc | 60-90% | 71% P | 64% P | 60% P | 66% P | 56% F | 51% F | 74% P | 64% P | 58% F |
| M7 Overlap | <40% | 8% P | 8% P | 7% P | 8% P | 9% P | 8% P | 8% P | 9% P | 8% P |
| M9 StdDev | >=0.8 | 0.81 P | 0.75 F | 0.72 F | 0.83 P | 0.75 F | 0.71 F | 1.38 P | 1.17 P | 1.07 P |
| **Pass** | | **6/8** | **5/8** | **5/8** | **6/8** | **4/8** | **4/8** | **7/8** | **6/8** | **5/8** |

**No algorithm reaches the M3 >= 2.0 S/A target under any fitness model.**
Surge V6 at 1.79 under Optimistic is the closest. Aspiration Packs reaches only
1.02 under Optimistic -- a catastrophic shortfall.

## R2 Slot S/A Breakdown

| Algorithm | Model | S% | A% | B% | C% | F% | S/A% |
|-----------|-------|----|----|----|----|----|------|
| Asp+PairPref | A | 1.6 | 2.2 | 49.1 | 31.1 | 16.0 | 3.8 |
| Asp+PairPref | B | 2.1 | 1.3 | 49.0 | 28.8 | 18.8 | 3.4 |
| Asp+PairPref | C | 2.5 | 0.7 | 48.5 | 29.3 | 18.9 | 3.2 |
| Pure Aspiration | A | 5.3 | 5.5 | 45.0 | 18.6 | 25.6 | 10.8 |
| Pure Aspiration | B | 5.7 | 2.8 | 47.3 | 17.9 | 26.2 | 8.6 |
| Pure Aspiration | C | 5.4 | 1.8 | 46.3 | 22.3 | 24.1 | 7.2 |

The R2 slot delivers S/A cards only 3-11% of the time. The R2 resonance is the
player's *second-strongest* resonance, which means it is likely the secondary
resonance of the player's archetype. Cards with that primary resonance belong to
*different* archetypes. Under the fitness model, secondary-resonance cards are
rated B-tier for the player's archetype, which is exactly what we observe: ~47-49%
of R2 slot cards are B-tier. The R2 slot is structurally incapable of delivering
S/A cards because the secondary resonance maps to adjacent archetypes, not the
player's home archetype.

## Fitness Degradation Curve

| Algorithm | A->B Drop | B->C Drop | Total A->C Drop |
|-----------|-----------|-----------|-----------------|
| Asp+PairPref M3 | 1.02->0.84 (-0.18) | 0.84->0.78 (-0.06) | -0.24 (23.5%) |
| Pure Aspiration M3 | 0.93->0.72 (-0.21) | 0.72->0.63 (-0.09) | -0.30 (32.3%) |
| Surge V6 M3 | 1.79->1.24 (-0.55) | 1.24->1.03 (-0.21) | -0.76 (42.5%) |

Aspiration Packs degrades more slowly in *percentage* terms (23.5% vs 42.5%),
but this is misleading. It starts so low that there is little to lose. Surge V6
loses 0.76 absolute S/A from A to C but still ends at 1.03, matching
Aspiration's best-case performance. Graceful degradation from a weak baseline
is not a useful property.

## Per-Archetype Convergence (Asp+PairPref, Model B)

| Archetype | Convergence Pick |
|-----------|-----------------|
| Flash | 24.8 |
| Blink | 24.9 |
| Storm | 25.7 |
| Self-Discard | 25.5 |
| Self-Mill | 26.8 |
| Sacrifice | 26.9 |
| Warriors | 26.4 |
| Ramp | 26.4 |

Convergence picks of 24-27 mean the algorithm essentially never converges within
a 30-pick draft. This confirms the M5 failure: with only 2 guaranteed slots
(often only 1 S/A-capable), the algorithm cannot deliver the sustained 2+ S/A
per pack needed for convergence.

## Parameter Sensitivity (Model B)

| Config | Late S/A | StdDev | Conv Pick | Deck Conc | Gate% |
|--------|----------|--------|-----------|-----------|-------|
| R2>=2, >=40% | 0.93 | 0.74 | 25.1 | 70% | 52.6% |
| R2>=3, >=50% | 0.86 | 0.75 | 26.0 | 64% | 36.5% |
| R2>=4, >=60% | 0.76 | 0.76 | 26.1 | 57% | 18.6% |

Loosening the gate (R2>=2, 40%) raises S/A from 0.86 to 0.93 by opening the
gate more often (52.6% vs 36.5%), but this is still far below 2.0. The
fundamental problem is structural, not parametric: 1 R1 slot + 1 R2 slot cannot
match 3 resonance-matched slots.

## Draft Traces (Model B, Asp+PairPref)

**Trace 1 (Committed Warriors):** Gate opens at pick 6 after accumulating
Zephyr (6) and Tide (4). From pick 7 onward, every pack has 1 Tide-primary
card and 1 Zephyr-primary card. However, the Zephyr-primary cards are
Flash/Ramp-home (B-tier for Warriors), not S/A. The committed player picks
the single Warriors card from the Tide slot or falls back to B-tier options.
SA per pack hovers at 1, occasionally 2.

**Trace 2 (Power Chaser):** Gate opens early at pick 4 because the power
chaser's diverse picks spread resonance tokens roughly evenly, making R2 easily
reach 50% of R1. Gate is open 27 of 30 packs. The algorithm responds strongly
to diffuse drafting, but the R1+R2 cards have no archetype focus.

**Trace 3 (Signal Reader):** The signal reader heavily concentrates on Stone
(Self-Mill), reaching Stone=32 by pick 15 while Tide stays at 4. The R2/R1
ratio (4/32 = 12.5%) never reaches the 50% gate threshold. Gate opens 0 out
of 30 packs. The algorithm completely fails for strongly committed players
whose secondary resonance lags far behind their primary.

## Multi-Strategy Check (Model B)

| Strategy | Asp+PP | Pure Asp | Surge V6 |
|----------|--------|----------|----------|
| Committed | 0.65 | 0.55 | 0.64 |
| Power | 0.17 | 0.18 | 0.18 |
| Signal | 0.64 | 0.56 | 0.66 |

All three algorithms perform similarly at the deck level. The power chaser
strategy yields ~0.17 late S/A across all algorithms, confirming that power
chasing provides no archetype convergence regardless of pack structure.

## Self-Assessment

**Aspiration Packs + Pair Preference fails comprehensively.** The core
mechanism -- 1 R1 slot + 1 R2 slot + 2 random -- is structurally too weak to
deliver meaningful archetype convergence. The key failures:

1. **Insufficient matched slots.** One R1 slot provides ~50% chance of an S/A
   card (under optimistic fitness). One R2 slot provides ~3-11% chance. This
   gives an expected S/A per aspiration pack of ~0.5-0.6 for the matched slots
   plus ~0.5 from random = ~1.0-1.1. Surge Packs provides 3 matched slots at
   ~50% each = ~1.5 plus ~0.25 random = ~1.75. The math was never close.

2. **R2 slot is structurally useless for S/A.** The R2 resonance targets the
   player's secondary resonance, which maps to *different archetypes*. These
   cards are B-tier by definition. The pair-preference modification on the R1
   slot (preferring cards with R2 symbols) actually makes things worse by
   selecting dual-resonance cards, which are more likely to be from the
   player's home archetype but represent only 15% of the pool, reducing the
   effective candidate pool.

3. **Gate condition creates a paradox.** Committed players who strongly draft
   one resonance push R2 far below R1, preventing the gate from opening. The
   gate opens most reliably for power-chasers who draft broadly -- exactly the
   players who least benefit from resonance targeting.

4. **Pair preference has negative marginal value.** Asp+PairPref outperforms
   Pure Aspiration by +0.09-0.15 S/A, but this comes from the R1 slot
   filtering toward dual-resonance cards, not from the R2 slot. The "pair
   preference" concept adds complexity without meaningful benefit.

**Compared to Surge V6:** Surge dominates on M3 (1.79 vs 1.02 under A; 1.24
vs 0.84 under B), M5 (5.8 vs 20.3), and M9 (1.38 vs 0.81). Aspiration wins
on M1, M2, and M4, but these "wins" simply reflect the algorithm's failure to
provide convergence -- an algorithm that does nothing also scores well on early
exploration metrics.

**Recommendation:** Aspiration Packs should not be considered as a V7
candidate. The dual-resonance disambiguation hypothesis -- that targeting R1+R2
simultaneously improves archetype precision -- is correct in principle but
requires more than 2 slots to be effective. The mechanism delivers B-tier cards
from the secondary resonance, not S/A-tier cards from the home archetype. Any
viable disambiguation mechanism must find a way to filter *within* the primary
resonance pool (as Surge does with 3 matched slots) rather than splitting
resources across two resonances.
