# Resonance V4 -- Algorithm Overview Catalog

This document catalogs all 25 algorithms proposed during V4 exploration (5 per domain), plus hybrid proposals from later rounds. For each: one-sentence description, how it works, domain, simulation status, and results or rejection reason.

---

## Domain 1: Rejection & Passing Mechanisms

The core idea: what you *don't* pick shapes future packs. The 3 cards passed per pick are the primary input signal.

### 1.1 Cooldown Exile
"Each card you pass on is removed from the draft pool for your next 5 picks, then returns." Passed cards enter a cooldown queue; packs draw from the active pool. **Not simulated.** Too weak -- only ~15 cards on cooldown (~4% of pool), insufficient density shift.

### 1.2 Rejection Cascade
"Each card you pass on is removed for 4 picks, and one additional random card sharing its primary resonance is also removed." Cascade creates 6 removals/pick but risks removing on-archetype cards. **Not simulated.** Collateral damage and added complexity.

### 1.3 Rejection Memory with Weighted Return
"Cards you pass are removed for 3 picks; matching cards return first, non-matching return last." Differential return times concentrate the pool. **Not simulated.** Two interacting systems fail the simplicity test.

---

### 1.4 Exile Pressure (CHAMPIONED)

**One sentence:** When you pass a card, add 2 to its primary resonance's exile counter and 1 per secondary/tertiary symbol; all counters decay by 1 each pick; each pack card is independently skipped with probability (its primary resonance's counter / 20), rerolling on a skip.

**How it works:** Maintain 4 exile counters. Passing cards increments counters; counters decay each pick. Pack generation uses rejection sampling -- each card is skipped with probability proportional to its resonance's exile counter. Creates continuous probabilistic bias without shrinking the pool.

**Domain:** Rejection & Passing. **Simulated:** Yes (1000 drafts). **Scorecard:**

| Metric | Target | Result | P/F |
|--------|--------|:-:|:-:|
| Late S/A | >= 2 | 1.57 | FAIL |
| Early S/A | <= 2 | 1.41 | PASS |
| Convergence pick | 5-8 | 7.4 | PASS |
| Deck concentration | 60-90% | 85% | PASS |
| S/A stddev | >= 0.8 | 1.00 | PASS |
| Off-archetype | >= 0.5 | 2.43 | PASS |
| Run overlap | < 40% | 8% | PASS |

7/9 pass. Critical failure: convergence capped at 1.57 by resonance-archetype dilution. Best archetype balance of all V4 algorithms (6.8-7.7 convergence spread). **Not recommended** -- structural ceiling unfixable via tuning.

---

### 1.5 Passed-Card Graveyard with Resonance Tides
"Every 5 picks, the resonance with the most passed cards has 10 cards permanently removed from the pool." Batch processing creates wave events. **Not simulated.** Fixed 5-pick intervals feel mechanical; permanent removal risks over-thinning; zero pressure picks 1-5.

---

## Domain 2: Soft Probabilistic Influence

The core idea: the player's draft history creates a soft bias in how cards are sampled for packs. All slots drawn from the same biased distribution.

### 2.1 Linear Resonance Weighting
"Each card is sampled with weight 1 + 0.3 times matching resonance symbols with your deck." Linear scaling with no cap. **Not simulated.** Unbounded late-draft bias risks "on rails" failure.

---

### 2.2 Square-Root Affinity Sampling (CHAMPIONED)

**One sentence:** Each card in the pool is drawn with weight 1.5 + min(sqrt(symbol overlap with your drafted deck), 4.5), so cards matching your deck appear more often but with diminishing returns.

**How it works:** Compute per-card raw affinity as sum of player's resonance counts for each card symbol. Weight = 1.5 + sqrt(affinity), capped at 6.0. Draw 4 cards independently from weighted distribution. Square root creates diminishing returns that prevent over-convergence.

**Domain:** Soft Probabilistic. **Simulated:** Yes (1000 drafts). **Scorecard:**

| Metric | Target | Result | P/F |
|--------|--------|:-:|:-:|
| Late S/A | >= 2 | 1.74 | FAIL |
| Convergence pick | 5-8 | 10.5 | FAIL |
| Deck concentration | 60-90% | 89% | PASS |
| S/A stddev | >= 0.8 | 1.00 | PASS |
| Off-archetype | >= 0.5 | 1.11 | PASS |

7/9 pass. Best splash (1.11 off-archetype) and flexibility among all algorithms. Convergence structurally insufficient at archetype level. Aggressive tuning (base=0.5, no cap) reaches 2.01 S/A but over-concentrates (94.2%). **Not recommended** -- fundamental tradeoff between convergence and the algorithm's signature splash advantage.

---

### 2.3 Deck Fingerprint Matching
"Each card is sampled with weight proportional to cosine similarity between its symbol vector and your resonance profile." 4D vector comparison. **Not simulated.** Adds conceptual complexity beyond sqrt without convergence benefit; small vectors produce noisy early results.

### 2.4 Momentum Weighting
"Each resonance has momentum that increases when drafted and decays 20% per pick; cards weighted by momentum." Exponential decay supports pivots but caps steady-state. **Not simulated.** Decay fights convergence; steady-state too weak for 2+ S/A.

### 2.5 Frequency-Inverse Weighting
"Most-drafted resonance gets +0.5 weight; least-drafted gets -0.5, on base 2.0." Rank-based fixed bonuses. **Not simulated.** Maximum 25% bias too gentle for archetype convergence.

---

## Domain 3: Economic & Resource Mechanisms

The core idea: the player accumulates resources and chooses when to spend them to influence packs. Player agency over timing creates natural variance.

### 3.1 Pack Widening (CHAMPIONED -- RECOMMENDED)

**One sentence:** Each symbol you draft earns 1 matching token (primary earns 2); you may spend 3 tokens of one resonance to add 1 extra card with that primary resonance to the pack.

**How it works:** Tokens accumulate from drafted symbols. Before pack generation, the player may spend 3 tokens of one resonance to add a bonus card of that resonance to a 4-card pack (making it 5 cards). Base 4 cards are always fully random.

**Domain:** Economic & Resource. **Simulated:** Yes (1000 drafts, as v2 with cost 2 / bonus 2). **Scorecard (v2):**

| Metric | Target | Result | P/F |
|--------|--------|:-:|:-:|
| Late S/A | >= 2 | 3.35 | PASS |
| Early S/A | <= 2 | 2.48 | FAIL |
| Convergence pick | 5-8 | 6.0 | PASS |
| Deck concentration | 60-90% | 98.6% | FAIL |
| S/A stddev | >= 0.8 | 0.94 | PASS |
| Off-archetype | >= 0.5 | 1.35 | PASS |

7/9 pass. Recommended as v3 (cost 3, bonus 1) to fix early openness and over-concentration. Parameter sweep confirms cost 3 / bonus 1 achieves 2.34 S/A. **RECOMMENDED ALGORITHM** -- the only V4 mechanism that crosses the 2.0 S/A threshold while offering natural variance and player agency.

---

### 3.2 Resonance Auction
"Start with 10 influence; spend 1-3 before a pack to replace that many cards with resonance-matched cards." Fixed budget, slow regeneration via generics. **Not simulated.** Two levers add complexity; replacement mechanic is near-banned slot assignment.

### 3.3 Surplus Investment
"Passed cards generate tokens; spend 4 to make next pack all one resonance." Tokens from rejections, mono-resonance spend-packs. **Not simulated.** Counterintuitive earning; mono-resonance packs are too mechanical.

### 3.4 Tempo Banking
"Earn 1 tempo/pick; spend 2 for 6-card pack or 4 for 8-card pack." Pure combinatorics, no resonance manipulation. **Not simulated.** Even 8 random cards yield only ~1.0 S/A; cannot hit 2.0 target.

### 3.5 Resonance Futures
"Spend 2 tokens to permanently add one card of that resonance to every future pack." Permanent compounding investments. **Not simulated.** Creates "on rails" dynamic; early investment dominates; no pivot possible.

---

## Domain 4: Phantom Drafter / Competitive Scarcity

The core idea: invisible phantom drafters consume cards from the shared pool, creating natural scarcity. Pack generation is trivially simple (4 random from remaining pool).

### 4.1 Single Phantom, Fixed Resonance
"A phantom with a random resonance removes 2 cards/round matching its resonance." One phantom, 60 removals. **Not simulated.** Only discourages one resonance; too simple a landscape.

### 4.2 Rival Pack Draft
"8 cards drawn; phantom picks 2, you see remaining 6." Shared draft table. **Not simulated.** Changes pack size; insufficient pool-level impact from 2 removals per pack.

---

### 4.3 Multiple Phantoms, Ecosystem Competition (CHAMPIONED)

**One sentence:** Two phantom drafters, each assigned a random resonance (sometimes the same one), each remove the best-matching card from the pool each round; you draft from what remains.

**How it works:** 2 phantoms each remove 1 card/round (60 total over 30 picks, 17% of pool). Random resonance assignments create variable strategic landscapes per run. Packs are 4 random cards from depleted pool.

**Domain:** Phantom Drafter. **Simulated:** Yes (1000 drafts). **Scorecard:**

| Metric | Target | Result | P/F |
|--------|--------|:-:|:-:|
| Late S/A | >= 2 | 1.26 | FAIL |
| Convergence pick | 5-8 | 11.1 | FAIL |
| Deck concentration | 60-90% | 72% | PASS |
| S/A stddev | >= 0.8 | 0.92 | PASS |
| Off-archetype | >= 0.5 | 1.37 | PASS |
| Run overlap | < 40% | 4.9% | PASS |

7/9 pass. Worst convergence of all V4 algorithms (1.26 S/A). Pool suppression is fundamentally too weak -- removing 17% of the pool shifts resonance density from ~25% to ~30%, and half of those cards serve the wrong archetype. First-class signal reading (10/10) is unique among all algorithms. **Not recommended standalone**, but valuable as a signal-reading layer if combined with a convergence engine.

---

### 4.4 Mirrored Phantom
"Each card you draft causes a phantom to remove 2 same-resonance cards from the pool." Self-limiting feedback loop. **Not simulated.** Fights convergence by depleting the player's own preferred resonance.

### 4.5 Drifting Phantom Swarm
"Four phantoms shift toward your least-drafted resonance over time." Adaptive phantom migration. **Not simulated.** Fails simplicity; feels artificial as phantoms secretly serve player interests.

---

## Domain 5: Curated Randomness / Filtered Sampling

The core idea: generate a larger candidate set, then filter it using the player's history. Two-phase structure provides both variance (random generation) and convergence (selective filtering).

### 5.1 Oversample-and-Score
"Deal 8 random cards, score by symbol overlap with your deck, reveal the top 4." Rank-and-select. **Not simulated.** Deterministic selection kills variance; over-converges late.

### 5.2 Stochastic Sieve
"Draw cards one at a time; each passes with probability 50% + 10% per matching symbol; first 4 that pass become pack." Sequential probabilistic filtering. **Not simulated.** Deck Echo's fixed candidate pool gives better variance control.

### 5.3 Resonance Lens
"Draw 12 cards, pick 4 weighted by match to your top resonance." Single-resonance lens. **Not simulated.** Operates at resonance level, suffering full archetype dilution.

### 5.4 Affinity Threshold Filter
"Cards with matching resonance always enter the pack; non-matching must beat a coin flip." Binary filter. **Not simulated.** "Matching" too broad (6 of 8 archetypes pass); insufficient convergence.

---

### 5.5 Deck Echo Filter (CHAMPIONED)

**One sentence:** To make each pack, draw 12 random cards, then keep each independently with probability (2 + its weighted symbol overlap with your drafted deck) / 6, and fill remaining slots randomly from the rejects.

**How it works:** 12 random candidates, independent per-card acceptance filter based on symbol overlap (primary symbols weighted 1.5x). Survivors become the pack; if fewer than 4 survive, fill from rejects. Two-phase structure: random generation + stochastic filtering.

**Domain:** Filtered Sampling. **Simulated:** Yes (1000 drafts). **Scorecard:**

| Metric | Target | Result | P/F |
|--------|--------|:-:|:-:|
| Late S/A | >= 2 | 1.55 | FAIL |
| Convergence pick | 5-8 | 3.5-6.1 | PARTIAL |
| Deck concentration | 60-90% | 87.2% | PASS |
| S/A stddev | >= 0.8 | 0.97 | PASS |
| Off-archetype | >= 0.5 | 1.27 | PASS |
| Run overlap | < 40% | 11% | PASS |

7/9 pass. Most natural-feeling variance (14% zero-S/A packs, 14% three+ S/A packs). Convergence capped at 1.55 because 12 candidates contain only ~1.3 expected S/A cards for any archetype -- the filter can amplify tendencies but cannot create cards that are not in the candidate pool. **Not recommended** -- same structural ceiling as Exile Pressure.

---

## Hybrid Proposals (from Rounds 2 and 4)

### H1: Phantoms + Pack Widening

**One sentence:** Two phantom drafters deplete the pool (for signal reading), and the player may spend resonance tokens to widen packs (for convergence).

**Proposed by:** Agents 2 and 5 in Round 4. **Status:** Rejected 3-2 by the team. Reason: two independent mechanisms violate the one-sentence simplicity test (Goal #1, highest priority). Signal reading is Goal #8 (lowest priority).

### H2: Modified Pack Widening v3 (cost 3 / bonus 1)

**One sentence:** Each symbol you draft earns 1 matching token (primary earns 2); you may spend 3 tokens of one resonance to add 1 extra card with that primary resonance to the pack.

**Proposed by:** All 5 agents converged on this in Round 4. **Status:** RECOMMENDED ALGORITHM. Cost 3 creates genuine save/spend decisions. Bonus 1 reduces over-concentration from 98.6% to projected 80-88%. Parameter sweep data (cost 3 / bonus 1 = 2.34 S/A) confirms viability.

### H3: Gated Pack Widening (cost 3 / bonus 2 / pick 5 gate)

**One sentence:** Each symbol you draft earns 1 matching token (primary earns 2); starting at pick 5, you may spend 3 tokens of one resonance to add 2 extra cards with that primary resonance to the pack.

**Proposed by:** Agent 1 in Round 4. **Status:** Considered but not recommended. Bonus 2 likely over-concentrates. The pick 5 gate adds an exception clause to the one-sentence description. Cost 3 alone is sufficiently self-limiting.

---

## Summary: Final Rankings

| Rank | Algorithm | Domain | Status |
|------|-----------|--------|--------|
| **1** | **Pack Widening v3** | **Economic** | **RECOMMENDED** |
| 2 | Lane Locking (V3) | Threshold | Baseline -- still strong |
| 3 | Exile Pressure | Rejection | Best balance, insufficient convergence |
| 4 | Deck Echo Filter | Filtered | Best variance, insufficient convergence |
| 5 | Sqrt Affinity | Soft Prob | Best splash, too slow |
| 6 | Multiple Phantoms | Phantom | Best signals, worst convergence |

The remaining 19 unchampioned proposals were correctly filtered during Round 1. None would have outperformed their domain's champion based on the structural analysis.

**The key V4 finding:** Probabilistic resonance-based mechanisms (filtering, weighting, scarcity) cannot overcome the ~50% archetype dilution inherent in the 4-resonance/8-archetype mapping. Only mechanisms that ADD targeted cards to packs (Pack Widening, Lane Locking) generate enough archetype-specific density to cross the 2.0 S/A threshold. This is a mathematical limit, not a tuning problem.
