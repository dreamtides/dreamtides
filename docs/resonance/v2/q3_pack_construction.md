# Q3: The Pack Construction Problem

## Key Takeaways

- **Weighted random sampling cannot hit the convergence target on its own.** With realistic archetype card densities (~12-15% of pool at S/A tier for a given archetype), a purely random 4-card draw yields 2+ fitting cards only ~20-35% of the time. Some form of bias is required once the player commits.
- **Deterministic slot guarantees (e.g., "2 fitting + 1 splash + 1 random") are the strongest approach for convergence but create an "on rails" problem** where every pack looks structurally identical and the player stops making interesting decisions.
- **A hybrid approach -- weighted sampling with a soft floor guarantee -- occupies the most promising middle ground.** It biases toward the player's archetype without making pack structure predictable, preserving decision-making while meeting convergence targets.
- **Depletion-based systems are the only structural approach that naturally creates signal reading and run-to-run variety.** If picking from an archetype's pool shrinks it, the system organically rewards players who notice which pools are deep vs. shallow.
- **The pack construction mechanism cannot be designed independently of the fitness distribution.** A pool where 30% of cards are B+ in a given archetype needs a very different algorithm than one where only 12% qualify. The two questions are tightly coupled.
- **Transparency and opacity represent a fundamental design axis, not a binary.** Systems where the player can infer the construction rule (even approximately) produce signal-reading gameplay; fully opaque systems feel arbitrary and undermine learnability.
- **Early-draft and late-draft packs likely need different construction rules**, and the transition point between them is a critical parameter that shapes the entire draft experience.

---

## The Landscape of Approaches

### 1. Pure Weighted Random Sampling

The simplest approach: each card in the pool has a weight, and 4 cards are drawn without replacement proportional to those weights. The player's archetype commitment increases weights for fitting cards.

**Strengths:** Simple to implement and explain. Non-deterministic, so no two packs feel identical. Flexible -- the same mechanism works for all players at all stages.

**Weaknesses:** The math works against it. Suppose 50 of ~1000 pool entries are S/A-tier for the player's archetype (a generous estimate). Even with 3x weighting, the probability of drawing 2+ in a 4-card pack hovers around 30-40%. To hit "2+ most of the time" (>60%), weights need to be so aggressive (8x-10x+) that off-archetype cards nearly vanish, violating the splashability goal. The fundamental tension is that weighted random sampling treats convergence and variety as directly competing on the same axis: you turn one dial, and both goals respond inversely.

**Prediction:** Pure weighted random sampling with any single weight parameter will fail to simultaneously meet the convergence target (2+ fitting cards) and the splashability target (0.5+ strong off-archetype cards). Meeting one will violate the other.

### 2. Deterministic Slot Guarantees

Each pack is assembled from designated slots: for example, 2 cards drawn from the player's archetype pool, 1 card drawn from a "splash" pool (high fitness in a different archetype), and 1 fully random card.

**Strengths:** Guarantees convergence by construction. Easy to tune independently -- the splash slot ensures off-archetype options exist. Highly predictable for the designer.

**Weaknesses:** Fundamentally undermines "not on rails." If the player knows 2 of 4 cards will always fit their archetype, the decision space collapses. Worse, it makes the system transparent in a way that kills signal reading -- the player knows the system is curating for them, so the contents of a pack reveal nothing about the pool's composition. It also creates a sharp boundary problem: what happens at the transition from "uncommitted" to "committed"? A sudden shift from random packs to curated packs is jarring; a gradual shift is hard to explain. Finally, it interacts badly with flexible archetypes -- if a player is combining two archetypes, which one fills the guaranteed slots?

**Prediction:** Deterministic guarantees will score highest on convergence metrics but lowest on "not on rails" and "signal reading." Players will experience the draft as a series of forced choices rather than a landscape of options.

### 3. Sub-Pool Systems

Maintain separate card pools per archetype. When constructing a pack, draw from the player's archetype pool plus one or more other pools. Pools operate independently.

**Strengths:** Clean conceptual model. Each archetype's pool is self-contained, making balance easier. Natural support for the "splash" goal by explicitly drawing from a secondary pool. Can support flexible archetypes by blending two pools.

**Weaknesses:** Creates a bookkeeping problem -- 8 archetypes means 8 pools, and cards that are playable in multiple archetypes need to exist in multiple pools (either duplicated or shared, each causing different issues). If shared, picking a card from one pool depletes it from others, creating unintuitive knock-on effects. If duplicated, the total pool size inflates and becomes harder to reason about. Also creates the same "on rails" problem as deterministic slots if the player knows their pool is being drawn from preferentially. The biggest issue: it requires the system to have a firm read on the player's archetype to know which sub-pool to prioritize, and early in the draft that read is unreliable.

### 4. Cube-Style Pre-Constructed Packs

Before the draft begins, all 30 packs are pre-assembled as coherent units. Each pack is designed to contain cards from 2-3 archetypes with some intentional tension. The player sees pre-built packs in sequence.

**Strengths:** Maximum designer control over the draft experience. Every pack can be a meaningful decision point by construction. Eliminates the convergence problem entirely -- the designer ensures fitting cards appear at the right rate. Supports signal reading naturally: a pack heavy in one archetype signals that the archetype is well-represented in remaining packs.

**Weaknesses:** Extremely difficult to make non-deterministic. Even with randomized pack selection from a larger set, players will eventually recognize pack templates. Directly conflicts with "no forced decks" -- if packs are curated, experienced players can memorize which packs contain which key cards and force optimal paths. The biggest structural issue: pre-constructed packs assume you know the player's archetype when building them, but the whole point of the early draft is that the player hasn't committed yet. You could pre-build packs with archetype diversity rather than archetype focus, but then you lose the convergence guarantee.

**Prediction:** Cube-style packs will produce the best individual draft experiences but the worst replayability. After 5-10 runs, the draft becomes solvable.

### 5. Depletion-Based Systems

The pool starts with a fixed composition. Cards are permanently removed when drafted or when they appear in packs and are not selected. Over time, the pool's composition changes, and certain archetypes become more or less available.

**Strengths:** This is the only approach that creates *emergent* signal reading. If the Reanimator pool is running low (because the system has been offering lots of Reanimator cards), an observant player notices fewer Reanimator cards appearing and pivots. It also creates natural run-to-run variety without any explicit randomization of the starting pool -- small differences in early picks cascade into very different late-draft pools. It rewards the signal-reading player strategy by giving information about what's been drafted vs. what remains.

**Weaknesses:** Convergence becomes progressively harder, not easier. As a player drafts their archetype's cards, those cards leave the pool, making future packs *less* likely to contain fitting cards. This is the opposite of what the convergence goal requires. You can counteract this with weighting that increases as the archetype pool shrinks, but this fights against the natural dynamics of the system. Depletion also makes the system harder to explain -- "each card exists in limited copies" is simple, but "the system weights cards based on remaining pool composition and your inferred archetype" is not.

**Prediction:** Pure depletion without compensating weights will cause convergence to *decrease* over the course of a draft, producing the opposite of the desired curve (less archetype focus as you commit more). However, depletion is the strongest mechanism for signal reading and variety.

### 6. Adaptive Ramp Systems

A hybrid not listed in the brief: the system starts fully random and gradually increases archetype bias as the player commits. The bias function is continuous, not binary. Early picks (1-5) have zero or near-zero bias. By pick 10, the system applies moderate weighting. By pick 20, strong weighting. The ramp curve is a design parameter.

**Strengths:** Directly addresses the "open-ended early, convergent late" tension. No sharp transition point between uncommitted and committed drafting. The player experiences a natural narrowing that mirrors their own decision-making. Compatible with weighted random sampling as the underlying mechanism -- the ramp just controls the weight strength over time.

**Weaknesses:** Requires the system to continuously estimate the player's archetype commitment strength, which is a fuzzy inference problem. If the player is exploring broadly, the system might incorrectly identify an archetype and start biasing toward it. Also, if the ramp is too gradual, convergence arrives too late; if too aggressive, early picks feel constrained. The ramp curve interacts with the fitness distribution in complex ways.

---

## Cross-Cutting Tensions

**Convergence vs. Surprise.** Every mechanism that guarantees archetype cards reduces the chance of seeing something unexpected. The best pack construction approaches find a way to guarantee a *floor* of fitting cards while leaving room for variance above that floor.

**Transparency vs. Manipulation.** If the player understands the pack construction rule, they can game it. If they don't understand it, they can't read signals. The sweet spot is a system that is *approximately* legible -- the player can intuit "I'm seeing more of my archetype's cards" without knowing the exact formula.

**Early vs. Late Draft Needs.** These are structurally different problems. Early packs need archetype diversity (many archetypes represented). Late packs need archetype concentration (fitting cards available). Any single-mechanism approach handles one well and the other poorly. This strongly suggests the system needs at least two phases or a continuous parameter shift.

---

## Surprising Insights

1. **Depletion and convergence are natural enemies.** The most intuitive "resource management" model (cards get used up) directly undermines the convergence goal. This means any depletion-based system needs an explicit counter-mechanism, and the interaction between depletion and compensation is where the interesting design lives.

2. **The "committed archetype" detection problem is as hard as the pack construction problem itself.** Most approaches assume the system knows the player's archetype. But archetype detection from draft picks is noisy -- a player who drafts 2 Reanimator cards and 2 Tokens cards after 4 picks has not committed. Systems that require high-confidence archetype detection to function will fail in the early-to-mid draft transition zone (picks 4-8), which is exactly where convergence needs to kick in.

3. **Guaranteed slots and weighted sampling are not a spectrum -- they are structurally different.** Weighted sampling can approximate guaranteed slots with extreme weights, but the player experience is fundamentally different. With guaranteed slots, the player knows a fitting card exists; with heavy weighting, they believe one probably exists. This difference in player certainty affects decision-making more than the actual card distributions.

---

## Parameters for Round 2 Simulation

- **Weight multiplier for archetype-fitting cards** (range: 1x to 15x) and its interaction with archetype pool density
- **Ramp curve shape** (linear, exponential, step-function) controlling how bias increases from pick 1 to pick 30
- **Commitment detection threshold** -- how many S/A picks trigger the system's archetype identification?
- **Floor guarantee probability** -- if using a hybrid, what percentage of packs should guarantee at least 1 fitting card? At least 2?
- **Depletion rate** -- whether undrafted pack cards return to pool, are removed, or are partially removed
- **Phase transition point** -- if using a two-phase system, when does the switch from "diversity mode" to "convergence mode" occur?

---

## Concrete Predictions

- **I predict that a weight multiplier of 4x-6x combined with an adaptive ramp will be the minimum viable configuration** for hitting 2+ fitting cards per pack at a >60% rate after pick 6, assuming ~12% of the pool is S/A-tier for any given archetype.
- **I predict that any system with deterministic slot guarantees will produce measurably lower player engagement** (proxied by decision variance -- how often does the player's optimal pick differ from their actual pick in simulation).
- **I predict that depletion without compensation will cause convergence rates to drop by 20-40% between picks 10 and 25**, making the late draft frustrating for committed players.
- **I predict that the commitment detection problem will be the dominant source of failure cases** -- mislabeling a player's archetype and biasing toward the wrong cards will produce worse outcomes than no biasing at all.
