# Dream Journey Generation Appendix

## Purpose

This appendix supplements
[dream_journey_generation.md](dream_journey_generation.md). It contains worked
generation examples, testing and telemetry expectations, operational notes, and
the rejected alternatives that informed the final design. These sections were
split out of the main document to keep the core spec inside the requested line
budget while still preserving the algorithmic detail needed to implement and
evaluate the system.

## Worked Examples

The following examples are not canonical content templates. They are examples of
how the runtime should behave when applying the rules from the main document.

### Example 1: Early Windfall

Run state:

- completion level 0
- deck still mostly starter cards
- no Banes
- no active hooks

Generation:

Windfalls score highly because the run still needs raw matter and direction. The
selected pattern is a simple boon menu. The filled options are a visible card
offer, a visible dreamsign offer, and a modest essence package.

Why it works:

The site helps the run build. It reads as one coherent discovery scene, not as
three unrelated mechanics.

### Example 2: Mid Reforging

Run state:

- completion level 2
- deck has a clear carry character
- several starters remain
- one Bane is present

Generation:

Reforging scores highly because the run now has enough structure for targeted
surgery to matter. The selected pattern is one chosen target with three
operations. The three operations are bounded cost reduction, a draw rider, and
an offensive rewrite.

Why it works:

The scene is recognizably a workshop. The player is comparing different kinds of
refinement on one build piece, which is exactly the texture the midgame should
support.

### Example 3: Mid Temptation

Run state:

- completion level 3
- deck identity is established
- Bane load is still manageable

Generation:

Temptations score well. The selected pattern is premium object with attached
burden. The options are a premium dreamsign plus Banes, a high-band rewrite plus
essence loss, and a refusal option.

Why it works:

The scene has a strong emotional identity. The burden is visible. The player is
not being asked to parse a generic trade menu.

### Example 4: Mid Omen

Run state:

- completion level 3
- one future hook already exists
- economy is stable
- deck identity is established

Generation:

Omens score well, but Thresholds are slightly downweighted because the run
already has one future hook. The selected pattern is smaller now versus larger
later. The filled options offer a modest visible reward now or a larger visible
package after the next victory.

Why it works:

The site points forward without overloading the run. The player understands both
the reward class and the trigger condition.

### Example 5: Late Threshold

Run state:

- completion level 5
- few dreamscapes remain
- one persistence slot is still available

Generation:

Thresholds score highly because route and follow-up commitments are now more
valuable than broad acquisition. The selected pattern alters the next dreamscape
by inserting one of several future site biases.

Why it works:

Route shaping appears late enough to be meaningful and early enough to still
matter. The site reads as a commitment point, not as a random utility menu.

### Example 6: Late Ordeal

Run state:

- completion level 6
- deck is nearly final
- trivial rewards no longer matter

Generation:

Ordeals score highly. The selected pattern is take up to N. Each step offers a
strong reward and adds a visible increasing burden. The player may stop after
each step.

Why it works:

This produces climax-tier tension appropriate for the late run without becoming
an unreadable random chain.

## Testing Strategy

The implementation should support three test layers.

### Integration Tests

For fixed seeds and run states, assert:

- chosen family
- chosen pattern
- visible options
- hidden-policy metadata
- committed outcomes

### Property Tests

Assert invariants such as:

- no impossible costs
- no empty target pools
- no untracked delayed hooks
- determinism under identical state and content version
- no mutation of already generated dreamscapes by unrelated later RNG

### Golden-Seed Reviews

Maintain curated seed sets for:

- early runs
- mid runs
- late runs
- high-Bane runs
- cleanup-heavy runs
- hook-saturated runs
- essence-rich runs
- essence-poor runs

Golden-seed review is not just a correctness check. It is the main authored-feel
check.

## Telemetry

The runtime should log enough to answer:

- which families appear
- which patterns appear
- which options players choose
- how long they take
- which patterns are skipped
- whether delayed hooks are realized
- which families correlate with later busted or flat runs

Telemetry is not a nice-to-have. The family-first system still needs empirical
feedback to know which patterns are producing fake choices, overpowered
temptations, or late-run clutter.

## Operational Requirements

Because site manifests are precommitted, save data should persist the committed
manifests, not just the seed and a promise to regenerate them later. Each
manifest should record a content version or hash. Repair steps and fallback use
should be visible in logs rather than silently swallowed.

## Implementation Clarifications

The main document intentionally avoids turning into a file-by-file contract, but
the following clarifications should be treated as the default implementation
interpretation for V1 unless later superseded by a dedicated schema document.

### Scoring Band

When the main spec says "choose within the top scoring band," the default
meaning should be:

- compute all legal candidate scores
- discard any candidate below 85% of the highest remaining score
- sample proportionally to the remaining scores
- break exact ties by stable content ID

This gives the generator controlled variety without letting clearly weaker
candidates through.

### Persistent Hook Definitions

A **major persistent hook** is any unresolved state that survives site
resolution and can change future Journey generation, future dreamscape content,
or future battle setup.

A **persistence footprint** should use a simple V1 scale:

- `0`: immediate-only effect
- `1`: delayed local effect or single follow-up payoff
- `2`: structural hook such as route shaping or a follow-up scene seed

The main spec's rule of "at most two unresolved long-tail hooks, and at most one
structural Threshold hook" should be implemented using this scale.

### RNG Branch Derivation

When the main spec refers to a versioned RNG tree, the default derivation should
be a stable hash of:

- run seed
- dreamscape ID
- branch label
- content version hash

This is sufficient to prevent unrelated later rolls from perturbing already
generated Journey manifests.

## Alternatives Rejected

### Freeform Atomic Composer

Rejected because it would maximize raw variance at the cost of scene coherence,
balancing tractability, and player trust.

### Giant Static Event Catalog

Rejected as the primary model because it would undershoot Dreamtides' replay
variety goals, though the final design still borrows the static-catalog lesson
that every generated site should feel like one authored unit.

### Hybrid Fallback Composer

Rejected for V1 because it would likely create an obvious quality cliff between
carefully authored family scenes and lower-quality filler scenes.
