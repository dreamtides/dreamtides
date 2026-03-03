# V7 Agent 1: Baseline Algorithms Under Realistic Fitness

## 1. Three Baseline Algorithm Specifications

### Surge Packs (V6 Winner) -- T=4, S=3

Maintain 4 resonance token counters starting at 0. After each pick, add +2 for the drafted card's primary symbol, +1 for each secondary/tertiary symbol. Before generating each pack, if the highest counter >= 4, subtract 4 and fill 3 of 4 pack slots with cards of that resonance (4th slot random). Otherwise, all 4 slots random.

V6 results (optimistic): 2.05 S/A, convergence pick 5.9, stddev 1.42, concentration 76.5%. 9/9 metrics.

### Lane Locking (V3 Winner) -- Thresholds 3/8

Pack has 4 slots. Track weighted resonance counters. When the top resonance first reaches 3, lock one open slot to that resonance permanently. When any resonance reaches 8, lock a second slot. Locked slots always show a card with that primary resonance.

V6 results (optimistic): 2.22 S/A, convergence pick 3.3, stddev 0.50, concentration 96.1%. 6/9 metrics (fails M5, M6, M9).

### Double Enhancement (V6 Runner-Up) -- T=1, B=2

Draw 4 random cards. If the player's top resonance counter has reached 4+ weighted symbols AND at least 1 of the 4 base cards shares primary resonance with that top resonance, add 2 bonus cards of that resonance to the pack. Player picks 1 from up to 6 cards.

V6 results (optimistic): 2.13 S/A, convergence pick 7.4, stddev 1.71, concentration 63.4%. 9/9 metrics.

---

## 2. Fitness Models

### Model A: Optimistic (V6 Baseline)

Cross-archetype fitness assignment:
- Home archetype: S-tier (100%)
- Adjacent archetype sharing primary resonance: A-tier (100%)
- Archetypes sharing secondary resonance: B-tier
- Distant archetypes: C-tier

S/A precision for resonance-matched slots: **~100%**. Every Tide card drawn is either S (Warriors-home) or A (Sacrifice-home) for a Warriors player.

### Model B: Moderate

Cross-archetype fitness assignment for adjacent-primary cards:
- Home archetype: S-tier (100%)
- Adjacent archetype sharing primary resonance: **50% A, 30% B, 20% C** (rolled per card at pool creation)
- Archetypes sharing secondary resonance: B-tier
- Distant archetypes: C-tier

S/A precision for resonance-matched slots: **~75%**. Of the ~50% of resonance-matched cards that come from the sibling archetype, only half are A-tier. So each resonance-matched slot has: 50% chance S (home) + 25% chance A (sibling rolled A) = 75% S/A. This represents a game where card designers achieve moderate cross-archetype playability -- roughly half the cards in each archetype have enough "good stuff" quality to be drafted by the sibling.

### Model C: Pessimistic

Cross-archetype fitness for adjacent-primary cards:
- Home archetype: S-tier (100%)
- Adjacent archetype sharing primary resonance: **25% A, 40% B, 35% C** (rolled per card)
- Archetypes sharing secondary resonance: B-tier
- Distant archetypes: C-tier

S/A precision for resonance-matched slots: **~62.5%**. Each resonance-matched slot: 50% S (home) + 12.5% A (sibling rolled A) = 62.5% S/A. This represents highly specialized archetypes where most cards are mechanically narrow -- a Warriors tribal buff is useless in Sacrifice.

### Model D: Severe

Cross-archetype fitness for adjacent-primary cards:
- Home archetype: S-tier (100%)
- Adjacent archetype sharing primary resonance: **10% A, 30% B, 60% C** (rolled per card)
- Archetypes sharing secondary resonance: B-tier
- Distant archetypes: C-tier

S/A precision for resonance-matched slots: **~55%**. Each resonance-matched slot: 50% S + 5% A = 55%. This is the worst plausible case -- archetypes are almost entirely mechanically disjoint, and only a handful of "good stuff" cards cross over.

---

## 3. Predicted Degradation Analysis

### How Fitness Models Affect Each Algorithm

The key insight: all three baselines use resonance-level targeting. When they fill a slot with "a card of resonance X," the S/A rate of that slot depends entirely on the fitness model. Under optimistic, every resonance-matched card is S or A. Under realistic models, some fraction is B or C.

**Resonance-matched slot S/A rates by model:**

| Model | S (home) | A (sibling) | Total S/A | B/C (wasted) |
|-------|----------|-------------|-----------|--------------|
| A: Optimistic | 50% | 50% | 100% | 0% |
| B: Moderate | 50% | 25% | 75% | 25% |
| C: Pessimistic | 50% | 12.5% | 62.5% | 37.5% |
| D: Severe | 50% | 5% | 55% | 45% |

Random (unmatched) slots have a base S/A rate of ~25% (1 of 8 archetypes is S, 1 is A under optimistic, but under realistic models the A-rate drops proportionally). Estimated random-slot S/A rates: Optimistic ~25%, Moderate ~21.9%, Pessimistic ~20.3%, Severe ~19.4%.

### Surge Packs Degradation

Surge packs fill 3/4 slots with resonance-matched cards. In a surge pack, expected S/A = 3 * (resonance S/A rate) + 1 * (random S/A rate). Normal packs have S/A = 4 * (random S/A rate). Committed players get roughly 55% surge packs late.

| Model | Surge Pack S/A | Normal Pack S/A | Blended Late S/A | Delta from V6 |
|-------|---------------|-----------------|-------------------|---------------|
| A: Optimistic | 3.25 | 1.00 | ~2.05 | -- |
| B: Moderate | 2.47 | 0.88 | ~1.60 | -0.45 |
| C: Pessimistic | 2.08 | 0.81 | ~1.38 | -0.67 |
| D: Severe | 1.84 | 0.78 | ~1.25 | -0.80 |

**Surge Packs drops below 2.0 S/A at the Moderate model.** This is the central finding: even a modest departure from optimistic assumptions pushes Surge Packs below its passing threshold. The mechanism delivers resonance-matched cards reliably, but resonance matching alone is insufficient when cross-archetype fitness is imperfect.

### Lane Locking Degradation

Lane Locking has 2 permanently locked slots late in the draft. Expected S/A = 2 * (resonance S/A rate) + 2 * (random S/A rate).

| Model | Locked Slot S/A | Open Slot S/A | Blended Late S/A | Delta |
|-------|----------------|---------------|-------------------|-------|
| A: Optimistic | 2.00 | 0.50 | ~2.22 | -- |
| B: Moderate | 1.50 | 0.44 | ~1.72 | -0.50 |
| C: Pessimistic | 1.25 | 0.41 | ~1.47 | -0.75 |
| D: Severe | 1.10 | 0.39 | ~1.32 | -0.90 |

Lane Locking degrades **faster** than Surge Packs in absolute terms but starts higher. It crosses below 2.0 at the same point (Moderate). The permanent-lock structure means Lane Locking cannot adapt if the locked resonance turns out to have poor cross-archetype fitness in a particular draft -- Surge Packs' non-permanent tracking offers slightly more resilience.

However, Lane Locking already fails 3 metrics under optimistic (M5, M6, M9). Under realistic fitness, it will additionally fail M3, making it 5/9. Lane Locking is not a viable algorithm under any fitness model.

### Double Enhancement Degradation

Double Enhancement adds 2 bonus resonance-matched cards when triggered (~63% of packs late). Enhanced pack: 4 random + 2 resonance-matched. Expected S/A when triggered = 4 * (random) + 2 * (resonance S/A rate). When not triggered: 4 * (random).

| Model | Enhanced S/A | Non-Enhanced S/A | Blended Late S/A | Delta |
|-------|-------------|-----------------|-------------------|-------|
| A: Optimistic | 3.00 | 1.00 | ~2.13 | -- |
| B: Moderate | 2.38 | 0.88 | ~1.65 | -0.48 |
| C: Pessimistic | 2.06 | 0.81 | ~1.43 | -0.70 |
| D: Severe | 1.88 | 0.78 | ~1.30 | -0.83 |

Double Enhancement degrades at nearly the same rate as Surge Packs. It drops below 2.0 at Moderate. Its advantage under optimistic (higher raw S/A at 2.13) does not translate to advantage under realistic fitness because the degradation slopes are almost identical.

### Crossover Analysis: Does Double Enhancement Ever Beat Surge Packs?

Under optimistic, Double Enhancement (2.13) already beats Surge Packs (2.05). Under all realistic models, both degrade at similar rates, so Double Enhancement maintains a slim ~0.05 advantage in raw S/A. However, this is within noise. The real question is which algorithm's *other* metrics hold up better:

- Double Enhancement's convergence (7.4) is later than Surge's (5.9), getting worse under realistic fitness since fewer enhanced packs reach 2+ S/A, pushing convergence even later.
- Double Enhancement's concentration (63.4%) is already low and will drop further.
- Double Enhancement's variance (1.71) will remain higher than Surge's (1.42).

Neither algorithm dominates the other under realistic fitness. They both fail M3 at Moderate. Double Enhancement never cleanly beats Surge Packs because its other metrics are weaker.

---

## 4. Key Conclusions for V7

**The realistic S/A range to target is 1.3--1.7.** Under the Moderate model (which represents achievable card design), all three baselines land between 1.6 and 1.7 S/A. Under Pessimistic, they land between 1.3 and 1.5. The V6 target of 2.0 S/A is only achievable under optimistic assumptions or with mechanisms that go beyond pure resonance-level targeting.

**Implications for V7 agents:**

1. **Agents exploring archetype disambiguation (Agent 3) address the root cause.** If the algorithm can distinguish Warriors from Sacrifice within Tide cards, it can maintain high S/A even under pessimistic fitness. This is the highest-leverage investigation.

2. **Layered mechanisms (Agent 4) may partially compensate.** Adding a secondary mechanism on top of Surge Packs could recover 0.2--0.4 S/A, potentially keeping the algorithm above 1.8 under Moderate.

3. **Pack structure innovations (Agent 5) could change the math.** If packs can be structured to preferentially surface home-archetype cards rather than just resonance-matched cards, the fitness model matters less.

4. **The 2.0 S/A target may need to be lowered to 1.7 or 1.8** for V7's realistic assessment. Alternatively, the card designer's brief should specify the minimum cross-archetype A-tier rate needed to keep the algorithm above 2.0. For Surge Packs, that minimum is approximately 80% of sibling-archetype cards rated A-tier -- achievable only with deliberate "good stuff" card design within each resonance.

5. **All three baselines degrade at similar rates** (~0.45 S/A per step from Optimistic to Moderate). The degradation is structural to resonance-level targeting, not algorithm-specific. V7 must find algorithms that are structurally less dependent on cross-archetype fitness.
