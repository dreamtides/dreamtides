# Agent 7 Discussion Output: Open Exploration

## Simplicity Ranking (Simplest to Most Complex)

1. **Lane Locking (1)** -- gold standard for simplicity
2. **Ratcheting Slot Commitment (6)** -- one more threshold than Lane Locking
3. **Surge Packs (7)** -- single counter, single trigger, slot filling
4. **Double Enhancement (5, revised)** -- pack inspection + conditional bonus
5. **Threshold Auto-Spend (2)** -- token accumulation + auto-spend
6. **Soft Locks (3, revised)** -- three thresholds + hard locks + split resonance
7. **Pool Sculpting (4)** -- invisible pool manipulation, reserve management

## Scorecard (1-10)

| Goal | LL (1) | AS (2) | SL (3) | PS (4) | DE (5) | RS (6) | SP (7) |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 9 | 7 | 6 | 4 | 7 | 9 | 8 |
| 2. No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 3 | 6 | 6 | 8 | 6 | 3 | 8 |
| 4. No forced decks | 6 | 7 | 7 | 7 | 7 | 5 | 7 |
| 5. Flexible | 3 | 5 | 5 | 7 | 5 | 3 | 7 |
| 6. Convergent | 7 | 6 | 6 | 3 | 6 | 9 | 5 |
| 7. Splashable | 4 | 6 | 7 | 7 | 6 | 5 | 8 |
| 8. Open early | 5 | 7 | 7 | 9 | 7 | 5 | 9 |
| 9. Signal reading | 4 | 5 | 5 | 4 | 5 | 4 | 5 |

## Key Discussion Points

**S/A precision changes everything for Surge Packs.** My Round 1 math assumed 50% S/A precision on resonance-matched slots. Agent 1's analysis suggests 75-100%. At 75% precision, aggressive Surge Packs (3 surge slots) delivers: surge packs = 3 * 0.75 + 1 * 0.25 = 2.5 S/A; non-surge packs = 1.0 S/A. With surges firing ~75% of the time post-commitment: 0.75 * 2.5 + 0.25 * 1.0 = 2.13 S/A. This crosses 2.0.

**Surge Packs' unique feature: the alternating rhythm.** Non-surge packs are fully random; surge packs are enhanced. This creates a pulse players can feel. Lane Locking and Ratcheting create permanent state changes; Surge creates a recurring heartbeat with genuine non-surge valleys.

**Convergence hierarchy:** Deterministic placement (Lane Locking, Ratcheting) > additive injection (Auto-Spend) > slot-filling surges (Surge Packs) > conditional bonuses (Double Enhancement) > probabilistic biasing (Pool Sculpting). Surge sits between injection and placement -- it fills existing slots rather than adding new ones, which is weaker in raw S/A but cleaner in UX (constant pack size of 4).

**My unchampioned Proposal 5 (Draft Imprint) had non-permanent tracking** -- slots update to the current top resonance rather than being permanently locked. This allows pivoting. I should test whether Surge Packs naturally provides this (since surges track the current highest counter, not a locked resonance).

## Final Champion

**Surge Packs** -- retained with aggressive parameters.

## Planned Modifications

1. **3 surge slots** (up from 2) -- decisive change for crossing 2.0.
2. **Lower threshold to 4-5** (from 6) -- faster surge cadence.
3. **Non-permanent:** Surges track current highest counter, so pivoting naturally shifts surge resonance.
4. **Revised one-sentence:** "Each drafted symbol adds 1 token to that resonance (primary adds 2); when any resonance reaches 4 tokens, spend 4 and fill 3 of the next pack's 4 slots with random cards of that resonance, the fourth slot always random."
5. Test thresholds 4, 5, and 6 in simulation to find optimal surge frequency.

## Proposed Symbol Distribution

36 generic, 81 mono-1 (22.5%), 130 mono-2 (36%), 54 dual-2 (15%), 59 mono-3 (16.4%). Total dual: 54 (15%). High mono-1 count creates variance in surge timing; some picks generate only 2 tokens, creating unpredictable surge rhythm.
