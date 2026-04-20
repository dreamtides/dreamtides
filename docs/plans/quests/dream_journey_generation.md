# Dream Journey Generation

## Summary

This document defines the procedural generation system for Dreamtides Dream
Journeys. A Dream Journey is the game's equivalent of a roguelike deckbuilder
event: the player enters a Journey site, sees a small set of dream offers, and
chooses how to alter the run.

The design goal is not to maximize combinatorial variety. The design goal is to
produce outputs that feel like curated events in games such as Slay the Spire
while still achieving much higher replay variety through bounded procedural
instantiation. The system should make players think, "I know what kind of scene
this is, but I have not seen this exact version before."

The central decision is that the generator must be **family-first**. The primary
authored unit is not an atomic effect, a cost, a predicate, or a duration. The
primary authored unit is a **Journey Family**: a recognizable kind of scene with
a consistent dramatic promise. Each Journey site is generated as one coherent
family instance with one selected **scene pattern** and a bounded set of filled
payload slots. The generator is not allowed to build a site by sampling several
independent offers from a large global effect bucket.

The second central decision is that Dream Journey sites are **precommitted**.
When a dreamscape becomes available on the Dream Atlas, the Dream Journey site
for that dreamscape, if any, is fully generated and frozen. The site's family,
scene pattern, visible options, reveal policy, and relevant random outcomes are
determined then and do not change when the player later clicks the site. This is
necessary for player trust, preview stability, deterministic testing, and
content debugging.

The third central decision is that the generator should optimize for **run
texture by stage**, not just local choice quality. Early Journeys should help
the player build and stabilize. Mid-run Journeys should help the player shape
and refine. Late Journeys should help the player convert, commit, and fine-tune.
This is the Dream Journey equivalent of the way Slay the Spire's event texture
changes across acts.

This document intentionally focuses on:

- the conceptual model
- the top-level family taxonomy
- the runtime generation algorithm
- the content authoring contract
- validation and repair rules
- balancing and telemetry rules
- explicit V1 scope boundaries

This document does not attempt to exhaustively list every reward, cost, burden,
predicate, magnitude band, or payload template. A later companion document
should define those raw component catalogs.

## Related Information

- [Quest Master Design](quests.md) High-level quest vision, Dream Atlas rules,
  site generation, and the existing Dream Journey concept that this document
  refines.
- [Battle Rules](../../battle_rules/battle_rules.md) Core battle vocabulary,
  Dreamwell rules, battlefield structure, spark, and dreamsign context
  referenced by some Journey effects.
- [Dream Journey Generation Appendix](dream_journey_generation_appendix.md)
  Worked examples, testing strategy, telemetry, and operational notes that were
  split out to keep the core spec within the requested line budget.

## Problem And Context

Dream Journeys are the part of quest mode where Dreamtides can be most
surprising. They are where the game can:

- offer a dangerous premium reward
- introduce Banes through a dramatic bargain
- give the player a deck surgery choice
- store a future promise
- reshape the next dreamscape
- force a sharp late-run commitment
- deliver a memorable push-your-luck scene

Those ingredients are useful, but they are the wrong primary abstraction. A
naive generator built directly from them would tend to produce legal but weak
events: random offers, mismatched costs, incoherent option sets, repeated nouns
with no remembered identity, or technically interesting scenes that are hard to
compare at a glance.

Human-authored events in Slay the Spire work because each event is a coherent
scene. Before the player solves the mechanics, they understand the dramatic
shape:

- a shady bargain
- a dangerous temptation
- a card surgery workshop
- a cleansing refuge
- a risky excavation
- a future-facing prophecy

The choice structure itself carries meaning. The player is not merely reading
three unrelated mechanical offers. The scene feels authored as one thing.

Dreamtides wants that authored feel without collapsing to a fixed small event
library. The solution is to generate within bounded authored structures rather
than to procedurally assemble events from ever smaller atoms.

## Goals

- Generate Dream Journey sites that feel curated rather than improvised.
- Support strong replay variety through bounded recombination.
- Make Journey families recognizable across runs without making them stale.
- Adapt moderately to current run state without solving the run for the player.
- Guarantee that generated sites are legal, comprehensible, and strategically
  meaningful.
- Produce distinct early-, mid-, and late-run event texture.
- Keep the system data-driven enough that most new content can be authored
  without engine changes.
- Keep the runtime deterministic enough for exact replay, golden-seed testing,
  telemetry, and debugging.

## Non-Goals

- Exhaustively list all raw reward and cost payloads.
- Treat costs, predicates, durations, or triggers as top-level event families.
- Create a general-purpose scripting language for Journey behavior.
- Support every extreme quest-rule rewrite in V1.
- Solve final balancing numerics in this document.

## Design Principles

### Families First

The primary authored unit is a family, not an effect atom. A player should feel
that they encountered "a temptation scene" or "a reforging scene," not "a
cost-reward tuple with a delayed rider."

### Interaction Form Beats Reward Type

The top-level taxonomy should be based on how the scene works, not on what noun
it gives. A dreamsign reward can appear in a Bargain, a Windfall, a Temptation,
a Vision, or a Threshold. Those scenes are different because the interaction
form is different.

### Precommitment Builds Trust

Once a dreamscape appears on the atlas, its Dream Journey site already exists.
The player is choosing among real committed options, not causing the game to
roll a new scene on click.

### Stage Texture Is First-Class

The generator should not merely scale numbers by completion level. It should
change the types of scenes that dominate the run at different stages.

### Bounded Variety Beats Infinite Soup

A smaller number of strong recurring structures with internal variance is better
than a giant freeform combination space that produces noisy near-duplicates.

### One Site, One Dramatic Promise

A site may contain multiple mechanical elements, but it should still read as one
coherent scene. If the site feels like two event ideas stitched together, the
generator has failed.

### No Hidden Punishment

The game may vary how much it reveals, but the player must understand the class
of consequence they are opting into. Hidden randomness is acceptable only inside
clearly signaled envelopes.

## Conceptual Model

The generator uses five core concepts.

### Journey Family

A Journey Family is a broad player-facing scene category. It owns:

- the dramatic promise
- the stage role
- the allowed scene patterns
- the allowed gameplay scopes
- the anti-patterns

### Scene Pattern

A Scene Pattern is a specific interaction structure inside a family. It defines:

- how many options appear
- how those options relate to each other
- what slot payloads are required
- what information is revealed
- what counts as a valid or invalid instantiation

### Slot Catalog

A Slot Catalog is a bounded menu of fillable payloads such as:

- reward packages
- burden packages
- target predicates
- timing packages
- follow-up conditions
- preview styles

### Run Context Snapshot

A Run Context Snapshot is the generator input when a dreamscape becomes
available. It contains the quest state features the generator is allowed to
reason about.

### Site Manifest

A Site Manifest is the committed output for one Journey site. It contains:

- family ID
- scene pattern ID
- filled slot payloads
- preview text
- reveal policy
- precommitted random outcomes
- generation metadata for replay and debugging

## Final Umbrella Taxonomy

The V1 family set should be:

- Bargains
- Temptations
- Windfalls
- Reforging
- Sanctuaries
- Visions
- Ordeals
- Thresholds

Every Slay the Spire event in the validation inventory can map to one family,
Monster Train events also fit cleanly enough to the same set, the families are
player-facing and learnable, and Dreamtides-specific route shaping fits
naturally inside Thresholds instead of needing its own top-level family.

## Family Definitions

### Bargains

An explicit exchange, service, toll, or trade. The player gives up something
knowable to get something knowable. The price is supposed to feel legible and
transactional rather than corrupting.

Common scene patterns:

- same cost, different rewards
- same reward, different costs
- service menu
- sacrifice one resource for another

### Temptations

A seductive upside with a sting. The scene is about greed, corruption, danger,
or burdened convenience. The player is being offered more than a normal Bargain,
but the attached poison is the point of the scene.

Common scene patterns:

- premium reward with attached burden
- cursed boon menu
- risky gain versus safe refusal
- unstable gift that may cost more later

### Windfalls

A low-friction gain scene. Windfalls add positive matter to the run without
first asking the player to repair an existing problem.

Common scene patterns:

- choose one of several gifts
- curated reward cache
- boon menu
- patronage package

### Reforging

A direct rewrite of the build. The scene modifies what the run already has
rather than mainly adding new matter.

Common scene patterns:

- one chosen target, several operations
- one operation, several possible targets
- prune and replace
- transform, transfigure, or merge
- duplicate with downside

### Sanctuaries

A stabilizing or cleansing scene. Sanctuaries remove liabilities already inside
the run or restore a depleted quest resource such as essence.

Common scene patterns:

- cleanse versus restore essence
- remove burden versus stabilize
- refuge menu
- Bane cleansing

### Visions

A scene about visions, prophecy, memory, future-facing value, or delayed fate.
The identity is not the reward noun. It is the fact that the scene points
forward. A Vision may promise or delay value, but it does not structurally
reroute the future atlas the way a Threshold does.

Common scene patterns:

- smaller now versus larger later
- reward after a visible trigger
- stored package
- visible countdown reward

### Ordeals

A trial, wager, push-your-luck sequence, or escalating risk ladder. The scene is
about pressing deeper or betting on uncertainty.

Common scene patterns:

- reach deeper
- wager on uncertain outcome
- take up to N offers
- escalating search

### Thresholds

A commitment point that changes the later arc of the run. Thresholds include
follow-up chains, route shaping, and delayed structural consequences.

Common scene patterns:

- commit now for later payoff
- seed a follow-up encounter
- alter future dreamscape composition
- store something and reclaim it later

## Family Boundary Rules

The family layer only works if the lines are hard enough that content authors
and runtime validators can classify a scene the same way every time.

- `Bargains` versus `Temptations`: A Bargain is a legible trade with a
  fair-price texture. A Temptation offers an outsized upside with attached
  corruption, fragility, or poison. If the scene is meant to make the player
  feel greedy or compromised, it is a Temptation.
- `Windfalls` versus `Sanctuaries`: A Windfall grows the run by adding positive
  matter. A Sanctuary repairs the run by removing liabilities or replenishing a
  depleted resource. If the scene is reactive and stabilizing, it is a
  Sanctuary.
- `Sanctuaries` versus `Reforging`: A Sanctuary reduces friction. Reforging
  changes build structure. If the scene asks the player to choose how to rewrite
  a target, it is Reforging even if the result also improves stability.
- `Visions` versus `Thresholds`: A Vision points forward through information,
  timing, or delayed payoff. A Threshold changes the later shape of the run
  itself through route changes, follow-up scenes, or structural commitments.

## Scene Pattern Layer

The scene-pattern layer is where most authored craft belongs.

Each family should ship with several scene patterns. A good V1 target is:

- 4 to 6 patterns per family, with 5 as the default target
- 1 to 3 slot schemas per pattern
- multiple payload catalogs per slot schema

That is enough breadth to give each family internal variance without exploding
the top-level taxonomy.

### Pattern Responsibilities

A pattern definition should specify:

- option count
- symmetry type
- slot schema
- legal slot catalogs
- stage availability
- magnitude band
- reveal style range
- persistence footprint
- invalid-state rules
- repair preferences

## Slot Catalog Layer

Slot catalogs should be curated menus, not open-ended logic.

V1 should support at least:

- reward packages
- burden packages
- cost packages
- target predicates
- timing packages
- follow-up conditions
- reveal styles

### Reward Packages

Reward packages may include things such as:

- visible card offers
- visible dreamsign offers
- essence bundles
- bounded purge or cleanup
- bounded transfiguration
- duplication under conditions
- delayed reward packages
- future-site hooks

### Burden And Cost Packages

Burden packages may include:

- essence loss
- omen loss
- visible Bane gain
- visible future fragility
- temporary sacrifice
- sealed or delayed cost

These packages should be authored with magnitude bands and exclusion tags so the
runtime can tell when they fit a pattern and a run stage.

### Predicates And Timings

Predicates and timings are parameters, not top-level scene identities.

Predicates should target bounded domains such as:

- starter cards
- Banes
- characters
- events
- dreamsigns
- cards above or below a visible threshold
- curated pools

Timings should prefer memorable structures such as:

- immediate
- after next battle
- after next victory
- after two victories
- at the next dreamscape

## Run Context Snapshot

When a dreamscape becomes available, the generator should build a run context
snapshot. The snapshot should contain only the state features needed for
moderate adaptation.

Moderate adaptation means the generator may react to summary features of the
run, but it should not attempt to solve the run card by card or hand the player
an obviously perfect event every time.

At minimum it should include:

- run seed
- generator version
- content version hash
- Dreamcaller and selected package identity
- completion level
- biome
- current deck size
- count of starter cards
- count of Banes
- count of transfigured cards
- current essence
- current omens
- dreamsign count
- active follow-up hooks
- recent Journey family history
- recent scene-pattern history
- remaining atlas outlook known to the run

The generator may derive a small number of synthetic signals such as stage,
build-versus-refine pressure, economy pressure, cleanup need, hook saturation,
and recent family fatigue. These are inputs to scoring, not player-facing data.

For clarity:

- deck-need affinity means a bonus based on broad run needs such as raw matter,
  cleanup, or fine-tuning, not a full deck solver
- cleanup affinity means a bonus when the run still contains obvious friction
  such as starters, Banes, or low-synergy clutter
- biome affinity means a small authored flavor-and-pattern bonus, not a hard
  gate unless a family explicitly says so
- remaining atlas outlook means already generated adjacent dreamscapes, the
  visible site previews attached to them, and the number of dreamscapes left in
  the run
- author special-case biases means small manual score nudges for specific
  authored pairings and should never override hard legality or stage rules

## Deterministic RNG

The generator should use a versioned RNG tree rather than one flat run stream.

At minimum there should be separate deterministic branches for:

- atlas expansion
- dreamscape site roster selection
- family selection
- scene-pattern selection
- slot filling
- reveal-policy commitment
- repair and fallback

This prevents unrelated later rolls from changing already generated sites and
makes replay practical.

## Generation Hook

Dream Journey generation happens when a dreamscape becomes available.

That hook should:

1. determine the dreamscape's ordinary site roster and biome
2. determine whether the dreamscape contains a Dream Journey site
3. if so, generate the Journey site manifest
4. persist the committed manifest in quest state

The Journey site therefore already exists by the time the player sees the
dreamscape on the Dream Atlas.

## Runtime Generation Algorithm

The runtime algorithm should use the following stages.

### 1. Build The Snapshot

Read quest state and produce the run context snapshot for this dreamscape. Also
derive the synthetic signals such as stage, cleanup need, build-versus-refine
pressure, and hook saturation.

### 2. Compute Legal Family Candidates

Enumerate only families that are legal for this stage and this state.

Examples of hard exclusions:

- a Sanctuary pattern that removes Banes when the run has none
- a Reforging pattern that requires a valid chosen target when no such target
  exists
- a Bargain with a cost the player cannot pay
- a Vision or Threshold that would exceed the persistence budget

### 3. Score Families

Score each legal family using a deterministic weighted sum of:

- stage weight
- biome affinity
- deck-need affinity
- economy-state affinity
- cleanup affinity
- recent family penalty
- recent pattern-shape penalty
- novelty bonus
- authored special-case biases

Family selection should never be uniform.

### 4. Select A Family

Choose within the top scoring band using deterministic weighted sampling rather
than pure argmax or pure uniform sampling.

This produces replay variety without letting obviously bad candidates through.

### 5. Score And Select A Scene Pattern

Within the chosen family, score patterns by:

- stage fit
- target availability
- run-state resonance
- repetition penalty
- reveal-policy fit
- persistence footprint

Then choose the best-scoring pattern using the same bounded weighted sampling
principle.

### 6. Fill Slots

Instantiate the selected pattern's slots from its allowed catalogs.

Recommended fill order:

1. dominant structural payload
2. target domain if any
3. cost or burden frame if any
4. secondary payloads
5. reveal policy
6. preview text objects

This order helps prevent incoherent scenes.

### 7. Validate

Run legality, coherence, choice-quality, information-contract, and persistence
checks.

### 8. Repair

If validation fails, repair in this order:

1. swap payloads within the same slot schema
2. switch to another schema in the same pattern
3. switch to another pattern in the same family
4. switch to the next best family candidate

Repair should be deterministic and logged.

### 9. Freeze

Once the site validates, persist the final site manifest. Entering the site
later only resolves the already committed choices. It does not generate new
content.

## Information Contract

Dream Journeys may vary how much they reveal, but the player must understand the
class of consequence they are choosing.

Good examples:

- gain one of these visible cards
- gain this premium dreamsign and two visible Banes
- receive this visible reward after your next victory
- reach deeper up to three times; each reach may add a visible burden

Bad examples:

- accept an unspecified hidden punishment
- choose an option whose real outcome is rolled after commitment
- offer a huge invisible reward range with no clue about the stakes

### Reveal Policies

V1 should support a small set of reveal policies:

- fully revealed
- bounded hidden target with visible reward class
- visible delayed package with visible trigger
- bounded random within an explicit envelope

V1 should not support deep hidden outcome stacks.

### Precommitted Randomness

Any randomness that will matter later should already be committed when the site
is generated. This includes delayed rewards, hidden but bounded outcomes,
follow-up variants, and similar structures.

## Run Texture By Stage

This is a core invariant, not a tuning note.

### Early Stage

Early stage means completion levels 0 and 1.

The design intent is:

- build
- stabilize
- remove obvious friction
- give direction

Family emphasis should be roughly:

- Windfalls high
- Sanctuaries high
- Bargains medium-high
- Reforging medium with simple patterns
- Visions low to medium
- Temptations low
- Ordeals low to medium
- Thresholds low

Early patterns should mostly add matter, remove friction, and offer clean
medium-sized choices. They should avoid heavy commitments and too many delayed
hooks.

### Mid Stage

Mid stage means completion levels 2 and 3.

The design intent is:

- shape
- refine
- sharpen identity
- begin asking for real tradeoffs

Family emphasis should be roughly:

- Reforging high
- Bargains medium-high
- Visions medium-high
- Temptations medium
- Sanctuaries medium
- Windfalls medium-low
- Ordeals medium
- Thresholds medium

Mid-stage patterns should increasingly reward refinement, burden management, and
visible future planning.

### Late Stage

Late stage means completion levels 4, 5, and 6.

The design intent is:

- convert
- commit
- optimize
- gamble sharply

Family emphasis should be roughly:

- Reforging high with precise patterns
- Temptations medium-high
- Thresholds high
- Ordeals medium-high
- Visions medium
- Bargains medium
- Sanctuaries low to medium
- Windfalls low

Late-stage patterns should mostly avoid low-impact broad acquisition. They
should focus on cleanup, conversion, commitment, precise surgery, and meaningful
late risk.

Summarized simply: early Journeys build, mid Journeys shape, and late Journeys
convert.

## Pacing Ledger

The generator should maintain a run-level pacing ledger tracking:

- how often each family has appeared
- how often each scene pattern has appeared
- recent reward categories
- recent burden categories
- active follow-up hooks
- active persistent mutations
- recent safe-versus-risky distribution

The ledger should feed both family scoring and pattern scoring.

Its job is to prevent repeated family spam, repeated pattern-shape spam, too
many delayed hooks at once, too much early refinement, too much late raw
acquisition, and flat risk texture.

## Choice Quality Rules

Legality is necessary but insufficient. The system must also guard against bad
choice structure.

### Reject Or Penalize Sites Where

- one option clearly dominates the others
- two options are strategically identical
- the visible framing suggests symmetry but the outcome does not
- the scene mixes too many unrelated gameplay layers
- the player lacks enough information to compare the options fairly
- the site reads like several small events jammed together

Three-option scenes should usually have visible internal order:

- same cost, different rewards
- same reward, different costs
- one target, different operations

Truly heterogeneous menus should be rare and deliberate.

## Persistence Budget

Persistent hooks are exciting, but they are also the easiest way to make the run
hard to reason about.

The generator should budget for:

- follow-up chains
- delayed rewards
- ongoing burdens
- route changes
- temporary battle-affecting states
- sealed packages

V1 should follow conservative rules:

- a normal site should create at most one major persistent hook
- a site may combine one persistent hook with one local immediate effect
- if the run already holds several unresolved hooks, new Visions and Thresholds
  should be downweighted
- at most two unresolved long-tail hooks should exist at once, and at most one
  of those may be a structural Threshold hook

## Thresholds And Follow-Up Chains

Thresholds need special handling because they alter later generation.

### Follow-Up Chain Model

A follow-up chain should consist of:

- a seed created now
- explicit visible trigger conditions
- a precommitted future payload
- a maximum lifetime

The player should be able to inspect active hooks in quest UI. The system should
not depend on the player remembering hidden promises.

### V1 Limits

V1 should prefer:

- one-step returns
- simple two-step escalations only in rare content
- visible triggers such as next victory, next battle, or next dreamscape

V1 should avoid:

- long nested chains
- ambiguous triggers
- many concurrent chain variants

## Authoring Contract

The content system should have exactly three authored strata.

### 1. Family Definitions

A family definition should specify:

- name
- dramatic promise
- stage weights
- cooldown rules
- biome affinities
- reveal-style allowances
- anti-patterns
- allowed scene patterns

### 2. Scene Pattern Definitions

A pattern definition should specify:

- name
- family membership
- option count
- symmetry type
- slot schema
- stage availability
- magnitude band
- persistence footprint
- invalid-state rules
- repair preferences
- preview framing rules

### 3. Slot Catalog Entries

A slot entry should specify:

- payload identity
- payload description
- magnitude band
- compatibility tags
- exclusion tags
- target requirements
- stage availability
- persistence footprint
- preview support

### What Authors Should Not Need

Authors should not need to:

- write arbitrary conditional logic
- manually nest effect trees
- solve legality by hand
- understand RNG flow
- encode whole scenes as mini programs

If a desired scene cannot be expressed through family, pattern, and bounded
slots, the answer is to add a new pattern or catalog entry, not to create a DSL.

## Validation Contract

Validation must be a first-class part of the generation model.

### Legality Checks

Reject sites where:

- the player cannot pay the cost
- no legal targets exist
- a delayed hook cannot be tracked
- the stage rules are violated
- the content payload is unavailable

### Coherence Checks

Reject or penalize sites where:

- options visibly belong to different families
- the site touches too many unrelated gameplay layers
- the framing and the stakes disagree
- the pattern's promised shape is broken

### Choice Checks

Reject or penalize sites where:

- one option obviously dominates
- two options are redundant
- the trade relation is too opaque

### Persistence Checks

Reject or penalize sites where:

- too many long-tail effects are stacked
- a normal-band pattern tries to inject multiple major persistent changes

## Repair Strategy

Repair should be conservative and deterministic.

Repair order should be:

1. swap payloads inside the same schema
2. switch schema inside the same pattern
3. switch pattern inside the same family
4. switch family

The system should define a small safe fallback set for each stage, probably
centered on Windfalls or Sanctuaries. Fallback should be rare and visible in
logs.

Worked examples, test strategy, telemetry, operational requirements, and the
rejected alternative designs are moved to the appendix. They are not omitted;
they were separated only to keep this core spec within the requested line
budget.

## V1 Scope Boundaries

V1 should deliberately defer or tightly constrain:

- broad procedural custom-card generation
- broad procedural custom-dreamsign generation
- battlefield slot modifications as a major pillar
- deep battle-rule rewrites
- hidden punishments resolved after commitment
- long recursive follow-up chains
- arbitrary random-effect chains
- player-facing tide instruction or tide picking inside Journeys
- too many simultaneous persistent run mutations

These ideas may be revisited later, but they should not define the first
generation system.

## Acceptance Criteria

The Dream Journey generation system should be considered successful when:

- generated sites are consistently recognizable as one coherent family scene
- early, mid, and late runs show distinct Journey texture
- players can learn the family set across runs without the system becoming stale
- every generated site is deterministic, replayable, and inspectable
- incoherent sites are prevented by validation rather than manual cleanup
- the content layer can extend the system without turning into a scripting
  language
- playtesting indicates that the outputs feel closer to curated event design
  than to uniform random combination

## Final Recommendation

Implement Dream Journey generation as a deterministic, family-first,
stage-textured site generator. Use the eight umbrella families in this document
as the stable top-level taxonomy. Put the real authored craft into the
scene-pattern layer, not into a giant effect soup. Generate and freeze each
Journey site when its dreamscape becomes available. Make early Journeys build,
mid Journeys shape, and late Journeys convert. Keep V1 bounded, coherent, and
inspectable, then expand the effect surface only after this structure has proven
that it can reliably create authored-feel scenes.
