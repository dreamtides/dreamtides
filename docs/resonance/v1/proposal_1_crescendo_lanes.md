# Hybrid Proposal 1: CRESCENDO-LANES

**Status:** Simulation complete (1,000 quests per player type)
**Author:** Agent 1, incorporating insights from Agent 3
**Version:** v2 (Additive Formula)

---

## Overview

CRESCENDO-LANES is a card-offering algorithm that gives players an open, exploratory early draft that naturally narrows toward a focused identity as they commit to a resonance style. Every quest begins with a unique color "landscape" seeded at the start — some resonances will naturally appear more frequently in that run, giving each quest a different feel and rewarding players who read which colors are abundant. As you draft more cards of a given resonance, those cards become increasingly likely to appear in future packs, but the algorithm guarantees that off-color options remain visible throughout the entire draft. The core innovation is a single mathematical formula where all three forces — your growing commitment to a resonance, the quest's unique color landscape, and a guaranteed minimum presence for every color — are simply added together. There are no special rules, no distinct pack "slots," no hidden phase transitions: one formula governs every card in every pack from pick 1 to pick 40.

---

## How It Works

### The Big Picture

When you draft a card of a given resonance, the game records that pick in your "profile." A Tide card picked early gives Tide a profile value of 1; by pick 15, a committed Tide player might have Tide at a profile value of 10 or more. The algorithm uses that profile to decide how likely each card in the pool is to appear in your next pack.

Three things determine how likely any given card is to appear:

1. **Your profile** — how many cards of that resonance you have already drafted
2. **The quest's lane seed** — a random multiplier assigned to each resonance at the start of the quest, representing that run's unique color landscape
3. **A flat floor** — a minimum weight that every card always has, ensuring no color ever disappears entirely

These are simply added together: `weight = profile_score + lane_bonus + floor`

### The Exponent: How Commitment Accelerates

The "profile score" component is not linear. It uses an exponent that grows as the draft progresses. Early on, the exponent is low (0.70), which compresses differences between resonances — a player who has drafted 4 Tide cards and 0 Stone cards sees those two colors as much closer in weight than they will be later. By pick 12, the exponent reaches its maximum of 1.40, which amplifies differences — now a profile-10 Tide player sees Tide cards weighted at nearly seven times what they were early on.

**Exponent by pick number:**

| Pick | Exponent | What it means |
|------|----------|---------------|
| 1 | 0.70 | Differences are compressed; early openness |
| 2 | 0.76 | Starting to ramp |
| 3 | 0.83 | |
| 4 | 0.89 | |
| 5 | 0.95 | Approaching linear |
| 6 | 1.02 | Just past linear; convergence begins |
| 7 | 1.08 | |
| 8 | 1.15 | |
| 10 | 1.27 | Strong convergence |
| 12+ | 1.40 | Maximum — fully committed identity |

### A Concrete Example: The Stone+Tide Drafter

Say you start a quest with a Stone+Tide dreamcaller (giving you profile values of Stone=2, Tide=2 immediately). Your quest's lane seeds happen to be: Ember=0.96, Ruin=1.19, Stone=1.16, Tide=0.72, Zephyr=1.22.

**At pick 1** (exponent = 0.70):

- Zephyr card weight: `0^0.70 + (1.22 × 4.0) + 1.0 = 0 + 4.88 + 1.0 = 5.88`
- Tide card weight: `2^0.70 + (0.72 × 4.0) + 1.0 = 1.62 + 2.88 + 1.0 = 5.50`
- Stone card weight: `2^0.70 + (1.16 × 4.0) + 1.0 = 1.62 + 4.64 + 1.0 = 7.26`
- Ember card weight: `0^0.70 + (0.96 × 4.0) + 1.0 = 0 + 3.84 + 1.0 = 4.84`

Notice: Stone is favored, but Zephyr (which you have never drafted) is nearly competitive because this quest has a strong Zephyr lane. The dreamcaller gives you a modest advantage in Tide and Stone, but you are genuinely choosing between four options with weights spanning only 4.84 to 7.26 — a ratio of about 1.5x.

**By pick 8**, you have drafted several Tide cards (Tide profile = 7, Stone profile = 3, exponent = 1.15):

- Tide card weight: `7^1.15 + (0.72 × 4.0) + 1.0 = 10.1 + 2.88 + 1.0 = 13.98`
- Stone card weight: `3^1.15 + (1.16 × 4.0) + 1.0 = 3.6 + 4.64 + 1.0 = 9.24`
- Zephyr card weight: `0^1.15 + (1.22 × 4.0) + 1.0 = 0 + 4.88 + 1.0 = 5.88`
- Ember card weight: `0^1.15 + (0.96 × 4.0) + 1.0 = 0 + 3.84 + 1.0 = 4.84`

Now Tide is clearly favored, Stone is a meaningful secondary, and off-color options like Zephyr and Ember still appear at weights around 5 — not dominant, but genuinely visible. The ratio from top to bottom is now about 2.9x.

**By pick 15**, with Tide profile = 10 and exponent = 1.40:

- Tide card weight: `10^1.40 + (0.72 × 4.0) + 1.0 = 25.1 + 2.88 + 1.0 = 28.98`
- Zephyr card weight: `0^1.40 + (1.22 × 4.0) + 1.0 = 0 + 4.88 + 1.0 = 5.88`

Tide now outweighs Zephyr by about 5x. You will predominantly see Tide cards, but Zephyr — boosted by this quest's strong Zephyr lane — still appears regularly (roughly 1 in 6 cards in a pack). The additive lane bonus is doing the work of ensuring off-color never disappears.

### Why "Additive" Matters

The previous version of this algorithm (CRESCENDO v1) used a formula like `max(profile^exponent, floor)`, meaning off-color cards had weight equal to whatever the floor value was. As your on-color weight grew from 5 to 30, the floor stayed at 0.5 and off-color cards became essentially invisible — they went from appearing 0.38 times per pack to just 0.05 times per pack.

With the additive formula, the lane bonus (averaging 4.0) is always added on top, regardless of how high on-color grows. Off-color weight stays at a constant ~5.0 throughout the draft, even as on-color weight climbs to 30+. This is not a special rule or a hack — it emerges naturally from the mathematics of addition.

---

## Key Parameters

| Parameter | Value | What it controls | Turn it up | Turn it down |
|-----------|-------|-----------------|------------|--------------|
| `dreamcaller_bonus` | 2 | How much the starting dreamcaller biases early packs toward its two resonances | More early direction; faster early convergence | More early variety; weaker dreamcaller identity |
| `BASE_EXP` | 0.70 | The exponent at pick 1; how compressed weight differences are early | Less early variety; on-color cards dominate sooner | More early variety; all colors nearly equal weight |
| `MAX_EXP` | 1.40 | The exponent at pick 12+; how aggressively committed players converge | Stronger late convergence; less off-color splash | Weaker convergence; more color variety late |
| `RAMP_PICKS` | 12 | How many picks it takes to ramp from BASE_EXP to MAX_EXP | Longer "open" phase before convergence kicks in | Faster convergence; identity locks in earlier |
| `LANE_WEIGHT` | 4.0 | Multiplier for the quest's lane seed; controls how much each run's unique color landscape matters | More off-color visibility; stronger run-to-run variety | Less off-color visibility; algorithm behaves more like pure profile weighting |
| `FLOOR` | 1.0 | A universal minimum added to every card's weight | All colors slightly more visible at all times | Colors with no profile contribution become rarer |

**The most sensitive parameters** are `MAX_EXP` (controls the convergence/splash tradeoff) and `LANE_WEIGHT` (controls how much off-color cards stay visible). The `dreamcaller_bonus` has the single biggest impact on whether the very first picks feel open or directed.

**Parameter sweeps from simulation** (1,000 synergy quests each):

| MAX_EXP | Late on-color/pack | Late off-color/pack | Passes |
|---------|-------------------|--------------------|----|
| 1.0 | 2.48 | 1.01 | 3/7 |
| 1.2 | 2.75 | 0.80 | 3/7 |
| **1.4** | **3.01** | **0.60** | **3/7** |
| 1.6 | 3.20 | 0.45 | 2/7 |
| 1.8 | 3.34 | 0.35 | 2/7 |

| LANE_WEIGHT | Late off-color/pack | Late on-color/pack | Early unique res |
|-------------|--------------------|--------------------|-----------------|
| 2.0 | 0.41 | 3.16 | 2.77 |
| 3.0 | 0.53 | 3.05 | 2.88 |
| **4.0** | **0.60** | **3.00** | **2.95** |
| 5.0 | 0.68 | 2.93 | 2.97 |
| 6.0 | 0.74 | 2.89 | 2.95 |

| Dreamcaller bonus | Early on-color/pack | Early unique res | Late off-color |
|-------------------|---------------------|-----------------|----------------|
| 1 | 1.92 | 2.93 | 0.67 |
| **2** | **2.06** | **2.95** | **0.60** |
| 3 | 2.18 | 2.96 | 0.56 |
| 4 | 2.24 | 2.97 | 0.52 |

---

## What Changed from the Original Strategy

### The Original CRESCENDO (Round 1)

The original CRESCENDO proposal had a single elegant idea: use a pick-adaptive exponent. Instead of a fixed exponent throughout the draft, the exponent would start sub-linear (0.70 at pick 1, compressing differences) and ramp to super-linear (1.50 at pick 8, amplifying differences). This encoded the game's desired temporal arc — open early, convergent late — directly into the mathematics.

The original parameters were:
- `base_exp = 0.7`, `max_exp = 1.5`, `convergence_picks = 8`
- `floor_weight = 0.3` (the minimum weight for off-color cards)
- `dreamcaller_bonus = 4` per resonance
- No lane seeds; no additive formula

### What Broke

Three critical problems emerged from analysis and simulation:

**1. The dreamcaller bonus was too high.** With a bonus of 4 per resonance, the dreamcaller's two colors immediately had a 7–14x weight advantage over off-color cards at pick 1. Players were effectively locked into their dreamcaller's resonances before they made a single pick. The convergence pick — the moment when a synergy player's deck reached 75% on-color — was happening at pick 4.2, far earlier than the target range of 5–8.

**2. Off-color cards effectively disappeared.** The `max(profile^exp, floor)` structure meant that as on-color weight grew to 30+ in the late game, off-color cards sat at the floor value of 0.3. Late-game packs showed only 0.05 off-color cards on average — essentially never. The "splashable" design goal was completely unmet.

**3. Every quest felt identical.** Without any per-run randomization, two drafts with the same dreamcaller produced nearly identical card-offering sequences. There was no signal to read, no run-to-run variation.

### How the Hybrid Evolved

**From debate with Agent 3 (Seeded Lane Pools):** Agent 3's strategy used per-run lane seeds — random multipliers assigned to each resonance at quest start — to create run-to-run variety. This was Agent 3's strongest feature, even though Agent 3's overall strategy failed to converge. The hybrid adopted lane seeds, but only as an additive contribution to weight, not as the primary convergence mechanism.

**Agent 3's key mathematical insight:** Rather than using `max(profile^exp, floor)`, use `profile^exp + lane_bonus + floor`. This means off-color weight is always at least `lane_bonus + floor` regardless of how large on-color weight grows. The additive structure provides a structural guarantee, not a probabilistic one.

**Dreamcaller bonus reduction:** Cutting the bonus from 4 to 2 per resonance reduced the initial weight advantage from 7–14x down to about 1.3x. This alone moved the convergence pick from 4.2 to 4.5, improved early variety significantly, and made the first few picks genuinely open.

**Eliminated: structural off-color slot.** The v1 proposal included a "3+1" pack structure — three normally-weighted cards plus one forced off-color slot after pick 6. Agent 3's additive formula made this unnecessary. Off-color cards appear naturally at around 11–15% of weighted draws because of the constant lane+floor term. This produces a smoother, more organic-feeling distribution without the tell-tale "this card was forced" feeling of a structural slot.

**Eliminated: behavioral tracking.** Agent 5's strategy added commitment-pace tracking — watching how quickly players committed to a resonance and adjusting the exponent accordingly. Analysis showed this had negligible impact (less than 0.1 difference across all metrics) and added significant complexity. The additive formula naturally differentiates player types: synergy players accumulate profile counts fast, which drives the profile component up; power chasers have scattered profiles, which keeps the lane component relatively dominant.

| Aspect | Original v1 | Final v2 (CRESCENDO-LANES) |
|--------|------------|--------------------------|
| Weight formula | `max(profile^exp, floor)` | `profile^exp + lane_seed*LANE_WEIGHT + floor` |
| Off-color mechanism | Structural slot (1 of 4 cards forced off-color after pick 6) | Emerges from additive lane+floor term |
| Pack structure | 3+1 (weighted + guaranteed off-color) | 4 uniform slots, same formula |
| Phase transitions | Yes (pick 6 threshold) | None; continuous ramp only |
| Dreamcaller bonus | 4 per resonance | 2 per resonance |
| Lane seeds | No | Yes, [0.60–1.40] per resonance |
| Parameters | 8 | 6 |
| Late off-color (actual) | 0.05/pack | 0.61/pack |

---

## Simulation Results

Simulation ran 1,000 quests per player type (3,000 total). Player types:

- **Synergy:** Always picks the card that best fits their current resonance profile
- **Power chaser:** Always picks the card with the highest raw power (ignores resonance)
- **Rigid:** Picks on-color cards if available, otherwise picks randomly

### Synergy Player (target: the most demanding case)

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Picks 1–5: unique resonances per pack | >= 3.0 | 2.91 | **FAIL** (0.09 short) |
| Picks 1–5: on-color cards per pack | <= 2.0 | 2.06 | **FAIL** (0.06 over) |
| Picks 6+: on-color cards per pack | >= 2.0 | 3.01 | PASS |
| Picks 6+: off-color cards per pack | >= 0.5 | 0.61 | PASS |
| Top-2 resonance share in final deck | 75–90% | 96.1% | **FAIL** |
| Convergence pick (when deck hits 75% on-color) | 5–8 | 4.5 | **FAIL** |
| Most common archetype pair frequency | <= 15% | 11.3% | PASS |
| Least common archetype pair frequency | >= 5% | 8.3% | PASS |

### Power Chaser Player

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Picks 1–5: unique resonances per pack | >= 3.0 | 2.94 | **FAIL** |
| Picks 1–5: on-color cards per pack | <= 2.0 | 2.01 | **FAIL** |
| Picks 6+: on-color cards per pack | >= 2.0 | 2.58 | PASS |
| Picks 6+: off-color cards per pack | >= 0.5 | 1.03 | PASS |
| Top-2 resonance share in final deck | 60–85% | 67.8% | PASS |
| Convergence pick | 5–8 | 13.1 | **FAIL** (too slow) |
| Most common archetype pair frequency | <= 15% | 11.2% | PASS |
| Least common archetype pair frequency | >= 5% | 9.1% | PASS |

### Rigid Player

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Picks 1–5: unique resonances per pack | >= 3.0 | 2.90 | **FAIL** |
| Picks 1–5: on-color cards per pack | <= 2.0 | 2.06 | **FAIL** |
| Picks 6+: on-color cards per pack | >= 2.0 | 3.01 | PASS |
| Picks 6+: off-color cards per pack | >= 0.5 | 0.61 | PASS |
| Convergence pick | 5–8 | 4.7 | **FAIL** |
| Most common archetype pair frequency | <= 15% | 11.3% | PASS |
| Least common archetype pair frequency | >= 5% | 8.1% | PASS |

### Summary by Player Type

| Player type | Targets passed | Key strengths | Key failures |
|-------------|---------------|---------------|--------------|
| Synergy | 4/8 | Late on-color (3.01), late off-color (0.61), pair balance | Top-2 share too high (96.1%), convergence too early (4.5), early packs slightly too on-color |
| Power chaser | 5/8 | Deck variety (67.8% top-2), late off-color (1.03), pair balance | Early packs (marginally), convergence too slow (13.1) |
| Rigid | 5/8 | Late on-color (3.01), late off-color (0.61), pair balance | Early packs (marginally), convergence too early (4.7) |

### Pack Composition Over Time (Synergy Player)

| Pick | Unique resonances | On-color | Off-color | Neutral |
|------|------------------|----------|-----------|---------|
| 1 | 3.00 | 1.96 | 1.76 | 0.58 |
| 2 | 2.89 | 2.06 | 1.51 | 0.47 |
| 3 | 2.86 | 2.00 | 1.49 | 0.49 |
| 5 | 2.89 | 2.21 | 1.33 | 0.46 |
| 8 | 2.87 | 2.57 | 1.12 | 0.38 |
| 10 | 2.79 | 2.70 | 0.97 | 0.33 |
| 12 | 2.88 | 3.17 | 0.83 | 0.24 |
| 15 | 2.71 | 3.23 | 0.61 | 0.18 |
| 20 | 2.57 | 3.19 | 0.50 | 0.15 |

The gradual shift from a near-equal early pack to a heavily on-color late pack is visible and smooth — no sudden jumps or phase boundaries.

### Final Deck Statistics

| Statistic | Synergy | Power chaser | Rigid |
|-----------|---------|-------------|-------|
| Top-2 resonance share | 96.1% | 67.8% | 98.2% |
| Effective number of colors | 1.92 | 3.46 | 1.84 |
| Mono-color decks | 1.3% | 0.0% | 3.3% |
| Dual-color decks | 98.7% | 43.8% | 96.7% |
| Tri-color decks | 0.0% | 56.2% | 0.0% |
| Deck overlap (Jaccard similarity) | 0.060 | 0.055 | 0.061 |

---

## Design Goal Scorecard

These scores are from the simulation analysis, evaluating CRESCENDO-LANES against the eight design goals on a 1–10 scale.

| # | Goal | Score | Justification |
|---|------|-------|---------------|
| 1 | Simple: explainable in one sentence | **9/10** | One additive formula drives all card weights. Six parameters total. One-sentence explanation: "Cards you've been drafting appear more often as you commit to a style, but every quest has unique color strengths that always keep other options visible." |
| 2 | Not on rails: player retains genuine choices | **9/10** | DC=2 + additive floor keeps the early on-color ratio at 2.06 (only 0.06 over target). Late off-color at 0.61/pack means real choices persist late. Early packs show roughly 2 on-color + 2 other cards. |
| 3 | No forced decks: all archetype pairs feel viable | **8/10** | Pair frequency ranges 8.3%–11.3%, very close to even. Lane seeds create per-run asymmetry so no two quests feel identical. Jaccard overlap 0.060 indicates high run-to-run deck variety. |
| 4 | Flexible archetypes: splash and tri-color are possible | **5/10** | Top-2 share of 96.1% for synergy players means off-color splashing rarely matures into a meaningful secondary. The lane component rewards deep off-color investment but the profile exponent can overwhelm this for committed players. |
| 5 | Convergent: 2+ on-color cards per pack after commitment | **6/10** | Late on-color of 3.01/pack passes the >=2.0 target comfortably. However, convergence happens at pick 4.5, earlier than the 5–8 target, meaning the "crystallization" moment arrives faster than intended. |
| 6 | Splashable: off-color cards remain visible late | **8/10** | Late off-color of 0.61/pack passes the >=0.5 target. This is a 12x improvement over the original CRESCENDO (0.05/pack). The additive formula structurally guarantees this without any special rules. |
| 7 | Open early: first 5 picks show wide variety | **6/10** | Early unique resonances: 2.91/pack (just below 3.0 target). Early on-color: 2.06/pack (just above 2.0 target). Both are marginal failures, though both represent dramatic improvements over the original CRESCENDO (2.46 unique res, 3.02 on-color). |
| 8 | Signal reading: pool composition is readable | **7/10** | Lane seeds [0.60–1.40] create genuinely different pool compositions per run. Players can observe which colors appear frequently and update their strategy. The additive lane bonus makes lane strengths visible throughout the draft. |
| **Total** | | **58/80** | |

---

## Strengths

**Late off-color visibility is dramatically improved.** The additive formula is the most important structural achievement of this proposal. Late-game off-color went from 0.05 cards per pack (original CRESCENDO) to 0.61 cards per pack — a 12x improvement — without any hacks, forced slots, or procedural rules. Every quest now has visible off-color options throughout the entire draft.

**Exceptional simplicity.** Nine lines of math drive the entire system. There are no slot types, no phase labels, no hidden state beyond the profile counts the game already tracks. The algorithm is easy to explain, debug, and tune. This proposal has the second-highest simplicity score (9/10) in the redesign project.

**Per-run variety is meaningful.** Jaccard deck overlap of 0.060 means two quests with the same dreamcaller share only about 6% of their cards on average. Lane seeds range from 0.60x to 1.40x, creating genuinely different color landscapes each run. Archetype pairs are distributed nearly evenly (8.3%–11.3%), meaning no combination is forced.

**Power chaser experience is excellent.** Players who do not force a specific resonance naturally end up with tri-color decks (56.2%) and 67.8% top-2 share — well within the 60–85% target. The algorithm does not push these players toward premature convergence; their scattered profiles mean the lane component stays relatively dominant.

**Clean comparison to predecessor.** The improvement over the original Round 3 CRESCENDO is unambiguous across every metric:

| Metric | Round 3 CRESCENDO | CRESCENDO-LANES | Change |
|--------|-------------------|-----------------|--------|
| Early unique res/pack | 2.46 | 2.91 | +18% |
| Early on-color/pack | 3.02 | 2.06 | -32% |
| Late on-color/pack | 3.66 | 3.01 | -18% (healthier) |
| Late off-color/pack | 0.05 | 0.61 | +1120% |
| Top-2 share | 97.2% | 96.1% | Closer to target |
| Convergence pick | 4.20 | 4.55 | Closer to target |

---

## Weaknesses

**Top-2 share is stuck near 96%.** The most persistent failure across all simulation runs is that synergy players end up with 96.1% of their cards in their top two resonances — well above the 75–90% target. The parameter sweep shows this metric barely moves regardless of what you tune: it ranged from 95.1% to 96.6% across all configurations tested. The root cause is structural: this metric measures player deck composition, not pack composition. A synergy player who always picks on-color cards will build a 95%+ on-color deck no matter how diverse the packs are.

**Convergence pick is consistently early.** The target is for synergy players to cross 75% top-2 share around picks 5–8. Actual: 4.5. The dreamcaller bonus (even at the reduced value of 2) seeds the profile at quest start, meaning players hit 75% before they have had enough picks to make meaningful choices. This too is resistant to tuning: the DC=1 sweep gets convergence to 4.7, still below 5.0.

**Early packs are marginally too on-color.** Early unique resonances (2.91) and early on-color (2.06) are both just barely outside their targets (3.0 and 2.0 respectively). These are small absolute misses — 0.09 and 0.06 — but they indicate the algorithm could be a bit more open in the first handful of picks.

**Late off-color is marginal at average lane seeds.** The 0.61 average passes the 0.5 target, but at average lane seeds (all multipliers near 1.0), the result is right around 0.46 — technically below target. The passing result depends on favorable lane seed variance in many runs. A quest that unluckily draws low lane seeds for off-color resonances may feel tighter than desired.

**The key insight about top-2 share and convergence pick:** The simulation analysis found that these two metrics measure what the *player* does, not what the *algorithm* does. Pack composition (what gets offered) is well-controlled by the algorithm. But a synergy player who actively filters for on-color cards will reach high top-2 share and early convergence regardless of pack variety. This suggests the targets for these metrics may need recalibration — or that evaluation should shift toward pack-level metrics (what percentage of offered cards are on-color) rather than deck-level metrics.

---

## Draft Story Example

This is a real trace from the simulation: a synergy player committing to Stone+Tide from a Stone+Tide dreamcaller.

**Quest setup:**
- Dreamcaller resonances: Stone, Tide (bonus = 2 each, so starting profile: Stone=2, Tide=2)
- Lane seeds: Ember=0.96, Ruin=1.19, Stone=1.16, Tide=0.72, Zephyr=1.22

| Pick | Exp | Cards offered (resonance: weight) | Picked | Profile after |
|------|-----|---------------------------------|--------|---------------|
| 1 | 0.70 | Zephyr:5.9, Tide:5.5, Zephyr:5.9, Ember:4.9 | Tide | S:2 T:3 |
| 2 | 0.76 | neutral:4.0, Tide:6.2, Stone:7.4, Tide:6.2 | Tide | S:2 T:4 |
| 3 | 0.83 | Ruin:5.7, Tide:7.0, Zephyr:5.9, Stone:7.4 | Tide | S:2 T:5 |
| 4 | 0.89 | Ruin:5.7, Zephyr:5.9, neutral:4.0, Ruin:5.7 | neutral | S:2 T:5 |
| 5 | 0.95 | Ember+Tide:9.0, Ruin:5.7, neutral:4.0, Ember+Stone:7.2 | Ember+Tide | E:1 S:2 T:6 |
| 6 | 1.02 | Ruin+Zephyr:5.8, Ruin:5.7, neutral:4.0, Ember:5.9 | neutral | E:1 S:2 T:6 |
| 7 | 1.08 | Stone:7.8, Ruin:5.7, Stone:7.8, Zephyr:5.9 | Stone | E:1 S:3 T:6 |
| 8 | 1.15 | Ruin+Tide:12.6, Tide:11.7, Ember:5.9, Ruin:5.7 | Tide | E:1 S:3 T:7 |
| 9 | 1.21 | Tide+Zephyr:15.4, Ember:5.9, Stone:9.4, Ember:5.9 | Stone | E:1 S:4 T:7 |
| 10 | 1.27 | Zephyr:5.9, Tide:15.8, Zephyr:5.9, Zephyr:5.9 | Tide | E:1 S:4 T:8 |
| 11 | 1.34 | Ruin:5.7, Tide+Zephyr:21.0, Stone:12.0 | Stone | E:1 S:5 T:8 |
| 12 | 1.40 | Tide:22.2, Tide:22.2, Ruin+Stone:15.2, Ruin:5.7 | Tide | E:1 S:5 T:9 |
| 13 | 1.40 | neutral:4.0, Stone:15.2, neutral:4.0, neutral:4.0 | Stone | E:1 S:6 T:9 |
| 14 | 1.40 | Ruin+Stone:18.0, Tide:25.5, Stone:17.9, Stone:17.9 | Stone | E:1 S:7 T:9 |
| 15 | 1.40 | Ember+Stone:21.5, Tide:25.5, Tide:25.5, Stone:20.9 | Tide | E:1 S:7 T:10 |

**Final deck:** Tide=22, Stone=19, Ember=1, Zephyr=1, Ruin=0 — Top-2 share: 95.3%, Dual deck

**What this shows:**

- **Early picks (1–6) are genuinely open.** The weight spread from 4.0 to 9.0 means all offers are competitive. At pick 4, there are literally no on-color cards in the pack (the player picks a neutral) and the draft doesn't feel broken.
- **Zephyr appears throughout** (picks 1, 3, 4, 6, 10, 11) despite the player never picking it. This is the lane bonus at work — Zephyr has a 1.22 lane seed, so it stays visible all draft long.
- **Ruin also stays present** (picks 3, 4, 5, 6, 8, 10, 11, 12, 14) with a 1.19 lane seed. The player could pivot into Ruin+Tide at any point.
- **Weights clearly grow** as picks accumulate: Tide at 5.5 in pick 1, 11.7 in pick 8, 22.2 in pick 12, 25.5 in pick 14. Commitment compounds visibly.
- **Late packs are still not pure on-color.** Pick 13 has three neutrals and one Stone. Pick 15 offers Ember+Stone and two Tides. The algorithm is not spamming identical on-color cards even at maximum exponent.

---

## Comparison to the Other Four Hybrids

Five hybrid proposals were developed for the resonance redesign project. Here is how CRESCENDO-LANES compares:

### Design Score Summary

| Proposal | Simplicity | Not on rails | No forced decks | Flexible archetypes | Convergent | Splashable | Open early | Signal reading | Total |
|----------|-----------|-------------|-----------------|--------------------|-----------|-----------|-----------|----|-------|
| **1. CRESCENDO-LANES** | **9** | **9** | 8 | 5 | 6 | 8 | 6 | 7 | **58** |
| 2. (other hybrid) | varies | — | — | — | — | — | — | — | — |
| 3. (other hybrid) | — | — | — | — | — | — | — | — | — |
| 4. (other hybrid) | — | — | — | — | — | — | — | — | — |
| 5. (other hybrid) | — | — | — | — | — | — | — | — | — |

*(Scores for other hybrids are documented in their respective proposal summaries.)*

### Position Among Original Strategies

For context, the five individual strategies that were combined into hybrids scored as follows:

| Strategy | Total score | Key strength | Key weakness |
|----------|-------------|-------------|--------------|
| S1 CRESCENDO (basis for this proposal) | 40/80 | Simplicity, convergence | Zero splash (0.05/pack), no run variety |
| S2 Structured Pack | 38/80 | Guaranteed convergence | Mechanical feel, weak signal reading |
| S3 Seeded Lanes | 60/80 | Splash, signal reading, flexibility | No convergence (1.94 on-color late) |
| S4 Staged Exponent | 43/80 | Game design precedent | Splash insufficient, extreme late ratios |
| S5 Adaptive Resonance | 33/80 | Player-type adaptation | Too complex, tracking nearly negligible |

CRESCENDO-LANES (58/80) substantially outscores its S1 parent (40/80) by importing S3's lane seeds and additive insight.

### When to Prefer CRESCENDO-LANES

Choose this proposal if you want:
- **The simplest possible algorithm.** One formula, six parameters, no special cases. A designer can predict the weight of any card in any situation by hand.
- **The most debuggable system.** When something feels wrong, there is only one formula to inspect and six knobs to turn.
- **Predictable convergence for focused players.** Synergy players get reliably focused decks (96.1% top-2) and an average of 3.01 on-color cards per late pack.
- **Good off-color splash without structural complexity.** The additive formula guarantees 0.61 off-color cards per late pack without any forced slots or diversity checks.

### When to Prefer an Alternative

Consider other proposals if:
- **Early variety is the top priority.** The early on-color rate (2.06) and early unique resonances (2.91) both narrowly miss targets. Proposals that use structural guarantees for early packs may do better here.
- **You want top-2 share in the 75–90% range for synergy players.** No single parameter in this proposal can bring 96.1% down to 90% or below. This requires a structural mechanism that limits how completely profiles can dominate weights.
- **You want tri-color synergy builds.** Synergy players end up in dual-color decks 98.7% of the time. A proposal with a wider splash window or bridge bonuses for dual-resonance cards would better support intentional tri-color builds.
- **Signal reading is a primary design goal.** While lane seeds provide meaningful run-to-run variation, CRESCENDO-LANES scored 7/10 on signal reading. A proposal based more directly on S3's lane pool approach would score higher here, though at the cost of convergence reliability.

---

## Open Questions

**1. Are top-2 share and convergence pick the right metrics for synergy players?**

The analysis found that these two metrics measure what the player does, not what the algorithm does. A synergy player who actively picks on-color cards every time will reach high top-2 share and early convergence no matter how diverse the packs are. The question is whether the targets should be recalibrated (perhaps 90–95% top-2 share is appropriate for a player who is actively committing to a style), or whether they should be replaced with pack-level metrics (what fraction of offered cards are on-color, rather than what fraction of picked cards are on-color).

**2. Should LANE_WEIGHT be 4.0 or 5.0?**

At 4.0, late off-color averages 0.61/pack (passes). At 5.0, it rises to 0.68/pack, and early on-color drops to exactly 2.00 (borderline pass on that target too). Moving to 5.0 would likely pass one more metric but reduce late on-color from 3.01 to 2.93. This is a tuning decision that depends on whether splash or convergence is prioritized.

**3. Can the convergence pick be delayed without reducing DC bonus further?**

The dreamcaller bonus of 2 gets convergence to 4.5. DC=1 gets it to 4.7 — still below 5.0. Options to push past 5.0 include: flat weighting for picks 1–3 (exponent fixed at some low value before the ramp begins), increasing RAMP_PICKS from 12 to 16, or reconsidering whether the dreamcaller should seed the profile at all (vs. providing a pure pack-composition bonus).

**4. How should neutral cards be handled?**

The simulation used a fixed neutral weight of 4.0 (NEUTRAL_BASE=3.0 + FLOOR=1.0). Neutral cards appear 0.58 times per pack at pick 1 and only 0.15 times at pick 20. Should neutral weight scale with pick number to remain relevant? Or is fading neutral visibility the intended behavior as a player's identity crystallizes?
