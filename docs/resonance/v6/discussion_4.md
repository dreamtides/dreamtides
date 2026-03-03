# Agent 4 Discussion Output: Pool Evolution

## Simplicity Ranking (Simplest to Most Complex)

1. **Ratcheting Slot Commitment (6)** -- three numbers, one mechanic
2. **Lane Locking (1)** -- two numbers, one mechanic
3. **Surge Packs (7)** -- one counter, one trigger, slot replacement
4. **Threshold Auto-Spend (2)** -- counter + spend + bonus generation
5. **Pool Sculpting (4)** -- dual tracking + replacement + reserve
6. **Soft Locks (3)** -- thresholds + probability + split targeting
7. **Cascading Enhancement (5)** -- per-card trigger + probability + dual-type
   pool + cap

## Scorecard (1-10)

| Goal               | LL (1) | AS (2) | SL (3) | PS (4) | CE (5) | RS (6) | SP (7) |
| ------------------ | :----: | :----: | :----: | :----: | :----: | :----: | :----: |
| 1. Simple          |   9    |   7    |   5    |   4    |   3    |   9    |   7    |
| 2. No actions      |   10   |   10   |   10   |   10   |   10   |   10   |   10   |
| 3. Not on rails    |   3    |   6    |   7    |   8    |   7    |   3    |   7    |
| 4. No forced decks |   6    |   7    |   7    |   8    |   7    |   5    |   7    |
| 5. Flexible        |   3    |   5    |   6    |   8    |   5    |   3    |   6    |
| 6. Convergent      |   7    |   6    |   5    |   3    |   5    |   8    |   4    |
| 7. Splashable      |   4    |   6    |   7    |   7    |   5    |   4    |   7    |
| 8. Open early      |   5    |   7    |   7    |   9    |   7    |   5    |   9    |
| 9. Signal reading  |   4    |   5    |   5    |   6    |   6    |   4    |   5    |

## Key Discussion Points

**Honest self-assessment:** My champion is the weakest on convergence. My own
math showed it capping at ~1.7-1.9 S/A even with aggressive 20-replacement
rates. Pool evolution cannot escape the probabilistic ceiling because packs are
still random draws from a modified pool.

**Best-of-breed:** Agent 2's unchampioned Proposal 4 (Momentum Auto-Spend) is
the strongest dismissed idea across all agents. The streak mechanic directly
addresses the "barely crossing 2.0" problem with escalating bonus cards.

**Simplicity audit:** My one-sentence hides a "complexity bomb" -- the reserve.
Where do replacement cards come from? How large is the reserve? What happens
when it is exhausted? No programmer could implement this from the sentence alone
without significant clarification.

**The S/A precision debate is devastating for pool evolution.** If deterministic
placement achieves 75-100% S/A precision from resonance pools, then pool
evolution's gradual probability shift is strictly inferior -- worse convergence
with more complexity. Pool evolution's only remaining advantage is ORGANIC
VARIANCE from random draws, but that alone does not justify a mechanism that
fails the primary convergence target.

**Why I am retaining this champion anyway:** The investigation benefits from a
pure probabilistic approach in simulation to quantify the actual ceiling under
V6 constraints. If simulation confirms ~1.7-1.9, this validates V4's structural
finding and proves pool evolution cannot stand alone.

## Final Champion

**Dual-Resonance Pool Sculpting** -- retained as a probabilistic ceiling test.

## Planned Modifications

1. **Increase replacement to 18/pick** (9 T1 + 9 T2) for aggressive variant.
2. **Recycle removed cards into reserve** to prevent exhaustion.
3. **Reserve enriched at 25% dual-type** to maximize archetype precision in
   replacements.
4. **Delayed start:** No replacements until pick 3 to preserve early randomness.

## Proposed Symbol Distribution

36 generic, 80 mono-1, 140 mono-2, 36 dual-2, 50 mono-3, 18 dual-3. Total dual:
54 (15%). Reserve: 800 cards at 25% dual-type.
