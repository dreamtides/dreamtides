# Q4: The Variety and Signaling Problem

## Key Takeaways

- **Variety lives on three axes -- pool composition, decision topology, and outcome space -- and a system must vary at least two to prevent staleness.** Changing which cards appear (pool) is necessary but not sufficient; the *shape* of decisions (whether you face agonizing 2-vs-2 splits or obvious picks) matters more for perceived variety than the specific cards involved.
- **Signaling and variety are complementary, not opposed.** A system that creates per-run asymmetries automatically creates something for observant players to detect. The design question is not whether to have signals but how *legible* to make them.
- **The most dangerous form of sameness is strategic sameness -- drafting the same archetype with roughly the same card pool every run.** Card-level variety (seeing different commons) matters far less than archetype-level variety (being pulled toward Reanimator one run and Tokens the next) for perceived replayability.
- **Pool restriction is the single most powerful variety lever.** Making a subset of archetypes unavailable or under-represented per run forces players into new territory far more effectively than weighted random sampling, which converges toward offering everything every time.
- **Explicit signals are better than implicit ones for learnability, but implicit signals are better for long-term depth.** The ideal system layers both: an obvious surface signal (e.g., a visible starting condition) plus subtle signals for experts to decode (e.g., frequency of certain cards appearing, indicating deeper pool composition).
- **Player agency over starting conditions dramatically increases perceived variety at minimal fairness cost.** Letting the player make one pre-draft choice (even a constrained one) makes every run feel "chosen" rather than "assigned," which psychological research on autonomy consistently shows improves satisfaction.
- **Depletion-based variety (the pool gets thinner as you draft) creates emergent signaling for free.** When picking cards from your archetype reduces their future frequency, later packs naturally signal "this archetype is being drafted" vs. "this archetype is wide open" -- without any explicit mechanism.

---

## What Makes Runs Feel Same-y vs. Varied?

There are at least four distinct dimensions where sameness can occur, and they contribute unequally to the feeling of staleness:

**1. Card-level repetition.** Seeing the same specific cards across runs. This matters least -- players in roguelikes accept seeing familiar items. What matters is whether the *combination* feels novel. With 360 unique cards and 30 picks from packs of 4, any individual run samples roughly 120 cards (seen) and picks 30. Two runs with zero overlap in cards seen would require a pool of 240+ unique viewings, which is already close to impossible from a 360-card pool. Some card repetition is inevitable and acceptable.

**2. Archetype repetition.** Drafting the same archetype every run. This is the most damaging form of sameness. If Reanimator is consistently the strongest or most available archetype, experienced players will default to it. The system must make different archetypes the "best available" on different runs.

**3. Decision topology repetition.** Facing the same *kind* of decision every pack. If every pack contains 2 good archetype cards and 2 bad off-archetype cards, the decision becomes "pick the better of the 2 good ones" every time. Varied decision topology means some packs present agonizing splits between archetypes, some present a clear best pick with interesting second-best options, and some present no good options (forcing a "best of bad" choice). The mix of decision types matters enormously for engagement.

**4. Deck structure repetition.** Ending up with structurally similar decks. Even if the specific cards differ, if every Reanimator deck has the same curve, the same key pieces, and the same filler, runs blur together. Internal archetype variety -- multiple viable builds within an archetype -- is a powerful antidote.

The hierarchy of importance is roughly: archetype repetition > decision topology > deck structure > card-level repetition.

## Mechanisms for Per-Run Asymmetry

### Pool Restriction (High Impact, Moderate Complexity)

The most powerful approach: each run uses a *subset* of the full card pool. This can work several ways:

- **Archetype restriction:** 2-3 of N archetypes are suppressed per run (fewer S/A cards available). Forces players into the remaining archetypes.
- **Card-level restriction:** A random 20-30% of cards are removed from the pool each run. Creates organic asymmetry without targeting archetypes explicitly.
- **Copy count variance:** V1's approach -- each archetype's cards have a random multiplier on copy counts (0.75x-1.25x). Subtler than full removal.

Pool restriction creates a *real* asymmetry (some strategies are genuinely better than others this run) which is both what makes runs feel different and what creates readable signals. The risk is unfairness: if the "best" archetype varies too much in strength, some runs feel like freebies and others feel impossible.

### Starting Conditions (High Impact, Low Complexity)

A visible initial condition that nudges the run's identity:

- **Starting card/artifact:** You begin with one card that has high fitness in 1-2 archetypes.
- **Patron/quest giver:** A narrative element that offers a bonus for specific archetypes.
- **Pool preview:** You see 8-10 cards before drafting begins, giving information about pool composition.

Starting conditions are excellent for signaling because they are *explicit* -- the player knows something is different. They also create a natural narrative arc: "This is the run where I started with the necromancer's tome."

### Depletion Dynamics (Medium Impact, Emerges Free)

If the pool is finite and cards are removed when picked, later packs are naturally shaped by earlier picks. This matters most in a single-player roguelike (no other drafters), so the only depletion comes from the player's own choices. With ~120 cards viewed and 30 picked out of ~1000 pool entries, depletion is gentle but detectable.

More aggressive depletion (smaller pool, or removing unpicked cards too) amplifies this effect. The tradeoff: aggressive depletion creates stronger emergent signals but reduces the player's ability to pivot mid-draft.

## The Signaling Design Space

Signals exist on a spectrum from fully explicit to deeply hidden:

**Fully explicit:** "This run features Reanimator and Tokens as bonus archetypes." This is the Slay the Spire approach -- you know your character's strengths before you start. Maximum learnability, minimum discovery depth.

**Semi-explicit:** You see a starting card or initial pack that *implies* what's strong. A player who recognizes that "Bone Harvest" is an S-tier Reanimator card knows the signal; a new player just sees a card. This layers nicely: accessible surface, deep reading for experts.

**Implicit (frequency-based):** Over picks 1-5, an observant player notices they're seeing more Reanimator-adjacent cards than usual. This is the Magic: The Gathering draft signal -- "read what's open." It requires the player to have baseline frequency expectations, which means it only works after multiple runs. High skill ceiling but poor onboarding.

**Hidden (statistical):** The pool has asymmetries, but they're so subtle that only data-tracking players could detect them. This is effectively random from the player's perspective. It creates variety but not readable signals.

The ideal system almost certainly layers multiple signal types. A semi-explicit starting condition gives every player *something* to work with, while implicit frequency signals reward experienced players who pay attention.

### Surprising Insight 1: Signals are more valuable when they're *wrong* sometimes.

If a signal perfectly predicts the best archetype, the draft becomes "decode the signal, then execute." If the signal is 70-80% reliable, it becomes a Bayesian reasoning challenge: "The starting card suggests Reanimator, but I'm seeing a lot of Token support -- maybe Tokens is actually more open." This creates the most engaging decision space. Simulations should test signal reliability levels.

### Surprising Insight 2: Variety and fairness are not in tension -- they're in tension with *learnability*.

A system where every run has the same archetype power levels is "fair" but boring. A system where power levels vary is varied but still fair *if the player can adapt*. The real tension is that highly varied systems require more expertise to navigate. A beginner who doesn't read signals will stumble into the weak archetype and have a bad time. The solution is semi-explicit signals that protect beginners while rewarding experts.

### Surprising Insight 3: The player's *memory of options not taken* is a major variety driver that most analysis ignores.

When a player passes on an exciting card (because it doesn't fit their archetype), they remember that card. If the system ensures different exciting-but-passed cards across runs, players perceive higher variety even if their *picked* cards overlap significantly. This suggests that off-archetype card quality matters for variety -- showing mediocre off-archetype cards is a missed opportunity.

## Player Agency and Its Interaction with Variety

Should the player influence which archetypes are available? Three positions:

**No agency (system-driven):** Pool composition is entirely random. The player's only choice is how to respond. Pro: simplest, most replayable in theory. Con: players feel at the mercy of RNG, and experienced players may feel they lack control.

**Constrained agency:** The player makes one choice that influences (but doesn't determine) the run's archetype landscape. Examples: choosing a starting card from 3 options, selecting a "patron" that boosts an archetype, or picking a quest path that determines which card pools are available. Pro: players feel ownership of the run's direction. Con: optimal choices may emerge, reducing effective variety.

**Full agency:** The player selects their archetype before drafting. Pro: never forces an unwanted archetype. Con: eliminates the discovery and adaptation that makes drafting fun. Effectively converts the draft into a deckbuilder where you already know your deck type.

Constrained agency is almost certainly correct. It provides the psychological benefit of choice (autonomy) while preserving the adaptability and discovery that make drafting engaging. The constraint should be tight enough that the player can't force one archetype every time (addressing design goal 3) but meaningful enough to feel like a real decision.

## Predictions for Simulation Testing

- **I predict that removing 2 of 8 archetypes from the pool per run will produce more perceived variety than copy-count variance alone**, while keeping fairness acceptable (remaining archetypes should be similarly viable).
- **I predict that a semi-explicit starting signal (e.g., a visible S-tier starting card) combined with copy-count variance will hit the sweet spot** for signaling: readable for all players, with additional depth for experts who track card frequencies.
- **I predict that archetype overlap (how many B+ cards are shared between archetypes) is the hidden variable controlling variety.** High overlap means pivoting is easy and archetypes blur together (less variety). Low overlap means pivoting is hard and runs feel locked in (less agency). The sweet spot is probably 15-25% of cards being A-tier or better in 2+ archetypes.
- **I predict that decision topology variety matters more than card variety for perceived replayability**, but is harder to measure. Simulations should track a "decision difficulty" metric: how close in expected value are the top 2 cards in each pack? High variance in this metric across picks and runs indicates good decision topology variety.
- **I predict that showing 1 strong off-archetype card per pack (the "splashable" target) is more important for variety perception than for deck quality**, because it gives the player a memorable "road not taken" experience.

## Dimensions for Round 2 Simulation Testing

1. **Pool restriction strength:** None / copy variance only / remove 1 archetype / remove 2-3 archetypes. Measure archetype frequency distribution and run-to-run card overlap.
2. **Signal explicitness:** No signal / starting card / starting card + pool preview / full archetype announcement. Measure signal-reader strategy advantage.
3. **Starting condition agency:** No choice / choose from 2-3 starting cards / choose archetype directly. Measure archetype frequency skew and deck quality variance.
4. **Decision difficulty variance:** Track EV gap between top-2 cards in each pack. Compare across different pool restriction and fitness distribution configurations.
5. **Depletion aggressiveness:** Standard (remove picked only) / moderate (remove picked + 1 random per pack) / aggressive (remove all unpicked). Measure how pivot-ability and convergence change.
