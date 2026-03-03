# Agent 6: Escalating Influence

## Key Takeaways

- **Probabilistic approaches hit a structural ceiling around 1.7 S/A** (V4). Each resonance is shared by 4 archetypes, so even perfect resonance weighting delivers ~50% wrong-archetype cards. Escalation that stays purely probabilistic will hit the same wall no matter how steep the curve.
- **Additive or deterministic slot placement is required to cross 2.0 S/A** (V3/V4 structural finding). Lane Locking and Pack Widening proved that cards must be ADDED or PLACED, not merely weighted, to break through dilution. An escalating system must eventually transition from soft influence to hard placement.
- **Pair-based matching achieves ~80% S-tier precision but requires abundant dual-resonance cards** (V5). Under V6's 15% cap (~54 dual-type cards of 360), pair matching cannot be a primary strategy. Only ~25-30 cards exist per ordered pair, far too few for reliable slot filling.
- **Lane Locking's weakness is binary mechanical feel** (V3). The jump from 0 locked slots to 1 locked slot is abrupt. Escalation's core promise is replacing this binary transition with a smooth gradient -- the question is whether a smooth gradient can be strong enough.
- **Variance is a genuine design goal, not a consolation prize** (V4). Pack Widening's stddev of 0.94 and probabilistic approaches' stddev of ~1.0 both beat Lane Locking's 0.84. Escalation naturally produces variance because the influence probability creates good-pack and bad-pack runs. This is a structural advantage.
- **Single-resonance matching at high enough intensity becomes effective through volume**. If 3 of 4 slots are resonance-targeted by late draft, you see ~3 resonance-matched cards. Even at 50% archetype precision, that delivers ~1.5 S/A from targeted slots plus ~0.38 from the random slot, totaling ~1.88 -- still short of 2.0. This means pure escalating resonance weighting alone cannot cross 2.0; the escalation must add extra cards or use a second signal (like the rare dual-type cards) to push over the edge.
- **Convergence pick 5-8 requires the escalation curve to ramp aggressively between picks 3-6**. A linear ramp is too slow; the curve must be front-loaded or threshold-triggered to hit the convergence window.

## Five Algorithm Proposals

### 1. Graduated Slot Locking

**One-sentence:** Each pack slot has an independent probability of showing a resonance-matched card equal to min(your top resonance's weighted symbol count / 16, 0.75), checked per slot per pack.

**Technical description:** Track weighted symbol counts across all 4 resonances (primary=2, secondary/tertiary=1). Each pack slot independently rolls against P = min(top_count / 16, 0.75). On success, that slot draws from cards whose primary resonance matches the player's top resonance. On failure, draw from the full pool. By pick 6-7, a committed player has ~8-10 weighted symbols in their top resonance, giving P = 0.50-0.625 per slot, yielding ~2.0-2.5 resonance-matched cards. At 50% archetype precision, that is ~1.0-1.25 S/A from targeted slots plus ~0.25-0.38 from random slots, totaling ~1.25-1.63 S/A. This falls short of 2.0.

**Goal assessment:** Passes simplicity, no-actions, open-early, variance, and flexibility. Fails the 2.0 S/A convergence target due to single-resonance dilution. The escalation curve produces beautiful natural variance but cannot overcome the structural ceiling.

**Preferred distribution:** 10% dual-type (36 cards). Not enough dual cards to rely on.

### 2. Escalating Additive Injection

**One-sentence:** After each pick, gain resonance tokens (primary=2, others=1); before each pack, automatically spend 4 tokens of your highest resonance to inject 1 bonus card of that resonance into the pack, repeating until you cannot afford another spend.

**Technical description:** Tokens accumulate passively from every drafted card. Before generating each pack, the system checks the player's highest resonance token count. If it is >= 4, it deducts 4 and flags one bonus card injection. This repeats (checking again after deduction) until no resonance has >= 4 tokens. The pack then consists of 4 random base cards plus N bonus cards drawn from the top resonance's primary pool. A committed player earns ~3 tokens per pick in their top resonance, so after the first spend at pick ~2-3, they can sustain a spend roughly every 1.3 picks. By pick 6+, the player averages ~0.75 bonus cards per pack. At 50% archetype precision for bonus cards, that adds ~0.38 S/A. Combined with ~1.35 base S/A from random 4-card packs, total is ~1.73. Still short.

**Goal assessment:** Passes simplicity, no-actions, open-early. The auto-spend creates a natural rhythm. But at single-resonance precision, the bonus cards are too diluted. Would need cost 2 / bonus 1 to fire often enough, pushing total to ~2.0 but with volatile spending patterns.

**Preferred distribution:** 10% dual-type (36 cards).

### 3. Threshold-Gated Escalation (Soft-to-Hard Transition)

**One-sentence:** Each pack slot has an independent probability P of showing a card matching your top resonance, where P starts at 0 and increases by 5% per weighted symbol drafted in that resonance (capped at 90%), PLUS when your top resonance first reaches 6 weighted symbols, one designated slot is permanently locked to that resonance.

**Technical description:** This hybrid combines graduated probabilistic influence with a single hard lock triggered at a threshold. For the first 3-4 picks, all slots are probabilistic with low P (~15-25%), producing diverse packs with occasional resonance matches. At 6 weighted symbols (typically pick 3-4 for a committed player), one slot permanently locks to the top resonance, guaranteeing 1 resonance-matched card per pack. The remaining 3 slots continue escalating probabilistically. By pick 8+, P reaches 40-50% on the 3 free slots, so the player sees the locked card (~0.5 S/A at 50% archetype precision) plus ~1.5 probabilistic resonance cards (~0.75 S/A) plus ~1.5 random cards (~0.5 S/A) = ~1.75 S/A. Still borderline. The lock pushes it close but single-resonance dilution remains the bottleneck.

**Goal assessment:** Better convergence than pure escalation. The single lock at threshold 6 is less mechanical than Lane Locking's dual locks. But the hybrid nature adds complexity: the one-sentence description has a semicolon and two clauses. Marginal on simplicity.

**Preferred distribution:** 12% dual-type (43 cards).

### 4. Escalating Dual-Signal Injection

**One-sentence:** Gain resonance tokens from each pick (primary=2, others=1); before each pack, automatically add 1 bonus card for every 3 tokens in your top resonance (deducting 3 each time), where the bonus card is drawn from dual-type cards matching your top TWO resonances if available, otherwise from mono-type cards of your top resonance.

**Technical description:** This targets the rare dual-type cards preferentially. The auto-spend fires at cost 3, generating ~1 bonus card per pick once committed. When the player has a clear top-two resonance profile (e.g., Tide primary, Zephyr secondary), the bonus card pool is first searched for [Tide, Zephyr] dual-type cards. These have ~80% S-tier precision for the target archetype (per V5 findings). If the dual-type pool is exhausted or unavailable, fall back to mono-type Tide cards at ~50% precision. With ~54 dual-type cards total, ~7 per ordered pair, the dual pool is small but sufficient for occasional high-precision hits. Expected: ~40% of bonus draws hit the dual pool (yielding ~0.8 S/A per hit) and ~60% hit mono pool (~0.5 S/A per hit). Average bonus S/A: ~0.62 per card. At ~1 bonus per pack plus ~1.35 base S/A, total is ~1.97 -- nearly at the threshold. With cost 2 instead of 3, this crosses 2.0 but spends aggressively.

**Goal assessment:** The escalation is real: early picks have no bonuses, later picks have 1-2. The dual-signal preference is the key innovation -- it extracts maximum value from the 15% dual-type cards. Complexity concern: the fallback logic (dual preferred, mono fallback) adds a conditional that stretches the one-sentence rule. Passes no-actions, open-early, variance. Approaches but may not reliably cross 2.0 S/A.

**Preferred distribution:** Maximum 15% dual-type (54 cards), evenly distributed: ~7 per archetype. Maximizing dual-type cards is critical for this algorithm.

### 5. Ratcheting Slot Commitment (Champion)

**One-sentence:** Track weighted resonance symbols; when your top resonance reaches 3, 6, and 10, permanently lock one additional pack slot to show a random card of that resonance, with all unlocked slots remaining fully random.

**Technical description:** This is a three-threshold variant of Lane Locking with the escalation baked into the threshold spacing. The first lock at 3 symbols fires around pick 2-3, giving early feedback. The second lock at 6 fires around pick 4-5, creating momentum. The third lock at 10 fires around pick 6-7, delivering full convergence. Three locked slots drawing from a resonance shared by 4 archetypes gives ~3 resonance-matched cards at 50% archetype precision = ~1.5 S/A from locks. The 1 random slot contributes ~0.38 S/A. Total: ~1.88 S/A -- still below 2.0 with pure single-resonance matching. However, the key insight is that the locked resonance is the player's PRIMARY resonance, which is primary for exactly 2 archetypes and secondary for 2. Among cards with that primary resonance, roughly 50% belong to the player's archetype (S-tier) and ~25% to the adjacent archetype sharing that primary (A-tier). So the combined S/A rate from a locked resonance slot is approximately 75%, not 50%. This gives: 3 locked slots * 0.75 S/A + 1 random * 0.38 = 2.63 S/A. The 75% figure comes from V3's finding that locked resonance slots have "~75% chance of being S/A for the committed archetype" because each resonance is PRIMARY for 2 archetypes, and S-tier cards in the home archetype + A-tier cards in the adjacent archetype together constitute ~75% of the primary-resonance card pool.

With three thresholds instead of two, the escalation feels more gradual than Lane Locking. The player experiences three distinct "power-ups" rather than two, creating a smoother progression curve. The third lock at threshold 10 is the decisive one -- it pushes from ~1.8 S/A (2 locks) to ~2.6 S/A (3 locks), creating a satisfying late-draft payoff for commitment.

**Goal assessment:** Passes simplicity (three thresholds is only slightly more complex than Lane Locking's two). Passes no-actions, convergence (2.6+ S/A projected), open-early (first 2-3 picks are fully random), variance (1 random slot remains throughout, plus locked slots draw from a 75-card resonance pool creating natural variation). The main risk is the same as Lane Locking: mechanical feel from permanent locks. Mitigation: three gradual steps feel less binary than two.

**Preferred distribution:** 10% dual-type (36 cards). This algorithm does not rely on dual-type cards.

## Champion Selection: Ratcheting Slot Commitment

**Justification:** This is the only proposal that credibly crosses 2.0 S/A while remaining truly simple and requiring zero player decisions. The critical insight is re-examining the resonance-to-archetype math: locked resonance slots do not deliver 50% archetype precision -- they deliver ~75% S/A because both S-tier (home archetype) and A-tier (adjacent archetype sharing primary resonance) count as successes. V3 confirmed this with Lane Locking's 2.72 S/A from just 2 locked slots.

Proposals 1-3 fail the 2.0 threshold because they assume 50% precision from single-resonance matching. Proposal 4 gets close but requires complex dual-signal logic. Ratcheting Slot Commitment uses the same proven mechanism as Lane Locking (resonance slot locking) but spreads it across three thresholds instead of two, creating a more gradual escalation that better fits the "escalating influence" design brief while achieving strong convergence.

## Champion Deep-Dive: Ratcheting Slot Commitment

### Example Draft Sequences

**Committed Tide/Zephyr (Warriors) player:**

| Pick | Card Drafted | Symbols | Tide Count | Locks | Pack Composition |
|------|-------------|---------|-----------|-------|-----------------|
| 1 | Random | [Tide, Tide] | 3 | 0 -> 1 | 4 random. After: 1 Tide-locked slot triggers. |
| 2 | Tide character | [Tide] | 5 | 1 | 1 Tide + 3 random. Sees ~0.75 S/A from lock + ~1.1 from random. |
| 3 | Warriors card | [Tide, Zephyr] | 8 | 1 -> 2 at threshold 6 reached | 1 Tide + 3 random. After: second lock fires (count was 5, now 8, crossing 6). |
| 4 | Tide spell | [Tide, Tide] | 11 | 2 -> 3 at threshold 10 reached | 2 Tide + 2 random. After: third lock fires (crossed 10). |
| 5 | Tide card | [Tide] | 13 | 3 | 3 Tide + 1 random. ~2.25 S/A from locks + ~0.38 from random = ~2.63. |
| 6-10 | Committed picks | ~[Tide, X] | 15-25 | 3 | Steady state: 3 Tide-locked + 1 random. |

This is a fast-commitment scenario with high-symbol cards. The player locks all 3 slots by pick 4, achieving full convergence early.

**Gradual committer (explores first 4 picks):**

| Pick | Card Drafted | Symbols | Top Count | Locks |
|------|-------------|---------|----------|-------|
| 1 | Generic card | [] | 0 | 0 |
| 2 | Ember spell | [Ember] | Ember: 2 | 0 |
| 3 | Tide creature | [Tide, Tide] | Tide: 3 | 0 -> 1 (Tide) |
| 4 | Tide card | [Tide] | Tide: 5 | 1 |
| 5 | Tide/Zephyr | [Tide, Zephyr] | Tide: 7 | 1 -> 2 (crosses 6) |
| 6 | Tide card | [Tide, Tide] | Tide: 10 | 2 -> 3 (crosses 10) |
| 7+ | Committed | | 12+ | 3 |

Convergence at pick 6 -- within the target window. The first 3 picks show diverse packs.

### Failure Modes

1. **Split commitment.** A player who drafts evenly across two resonances (e.g., 5 Tide + 5 Ember by pick 5) locks 1 Tide slot at threshold 3, then locks 1 Ember slot when Ember hits 3. Now 2 of 4 slots are locked to different resonances, and neither resonance ever reaches the third threshold quickly. The player ends up with a scattered deck. This is working as intended -- it punishes indecision and rewards commitment -- but could feel frustrating.

2. **Premature lock.** A player who drafts a high-symbol card on pick 1 (e.g., [Ember, Ember, Ember] = 4 weighted Ember symbols) locks an Ember slot immediately, then realizes they want to draft Tide. One slot is permanently locked to the wrong resonance. With 3 remaining open slots and eventual Tide locks on 2 of them, the player gets 2 Tide + 1 Ember + 1 random -- still functional (~1.5 S/A from Tide + ~0.3 from Ember crossover + ~0.38 from random = ~2.18 S/A). Recoverable but suboptimal.

3. **Late generic/1-symbol streak.** If the player drafts several generic or 1-symbol cards in a row, resonance count growth stalls. The escalation pauses, and packs feel unresponsive. This is self-correcting (next high-symbol pick resumes progress) but could produce a dead stretch of 2-3 picks.

### Parameter Variants to Test

1. **Conservative thresholds (4, 8, 14):** Slower ramp. First lock at pick 3, second at pick 5-6, third at pick 8-9. More open-ended early game, later full convergence. Likely ~2.4 S/A at convergence (same endpoint, just delayed).

2. **Aggressive thresholds (2, 5, 9):** Faster ramp. First lock at pick 1-2, second at pick 3-4, third at pick 5-6. Very fast convergence but less early exploration. Risk: too-early commitment feels railroaded.

3. **Two-lock variant (3, 8) with 4th slot wild:** Drops the third lock entirely, keeping 2 locks + 2 random slots. This is closer to original Lane Locking but with a wider threshold gap (3 vs 8 instead of 3 and 8 in V3). Expected ~2.0 S/A -- borderline. Tests whether 2 locks with better variance can compete with 3 locks.

### Proposed Symbol Distribution

| Symbol Count | % of Non-Generic | Cards | Rationale |
|---|---|---|---|
| 0 (generic) | -- | 36 | Standard 10% generic pool. |
| 1 symbol | 25% | 81 | Enough 1-symbol cards to create variance in accumulation rate. |
| 2 symbols | 55% | 178 | Core of the pool. 2-symbol cards give 3 weighted symbols (2+1) for mono-type or 2+1 for dual-type. |
| 3 symbols | 20% | 65 | High-symbol cards that can trigger thresholds quickly. |

**Dual-type allocation:** 10% of total pool = 36 dual-type cards. Distributed as ~4-5 per archetype. These are not critical to the algorithm (it uses single-resonance locking) but provide archetype-identifying signals for players who notice ordered pairs. The 10% rate (below the 15% cap) leaves headroom for future tuning.

**Symbol weighting per pick:** With this distribution, the average non-generic card provides ~2.5 weighted symbols in the primary resonance. A committed player drafting on-resonance every pick reaches threshold 3 by pick 2, threshold 6 by pick 3-4, and threshold 10 by pick 5-6. This aligns with the convergence pick 5-8 target.

**Why not maximize dual-type at 15%?** Ratcheting Slot Commitment does not use pair matching, so dual-type cards provide no mechanical advantage. Keeping dual-type at 10% preserves design space -- the dual-type cards serve as flavor/identity signals rather than algorithmic inputs. If simulation reveals that more archetype-identifying cards improve player experience without being mechanistically required, the cap can be raised to 15%.
