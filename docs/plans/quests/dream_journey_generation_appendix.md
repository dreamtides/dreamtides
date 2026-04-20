# Dream Journey Generation Appendix

## Purpose

This appendix supplements
[dream_journey_generation.md](dream_journey_generation.md). It contains worked
generation examples, Journey-content classification guidance, testing
expectations, telemetry expectations, and operational notes that were split out
to keep the core spec focused.

## Related Information

- [Dream Journey Generation](dream_journey_generation.md) Core shape-first
  Journey generation design.
- [Quest Master Design](quests.md) High-level quest flow and site context that
  this appendix assumes.

## Worked Examples

The following examples are not canonical content templates. They are examples of
how the runtime should behave when applying the rules from the main document.

### Example 1: Early Curated Reward Trio

Run state:

- completion level 0
- deck still mostly starter cards
- no Banes
- no active hooks

Generation:

Early build pressure is high, so low-friction shapes such as
`curated_reward_trio`, `service_menu`, and `same_cost_different_rewards` score
well. The selected shape is `curated_reward_trio`. The filled options are a
visible card package, a visible dreamsign offer, and a modest essence-plus-
cleanup bundle.

Why it works:

The site helps the run build. It reads as one coherent discovery scene, not as
three unrelated mechanical offers.

### Example 2: Mid One Target, Many Operations

Run state:

- completion level 2
- deck has a clear carry character
- several starter cards remain
- one Bane is present

Generation:

Refinement pressure is rising, so `one_target_many_operations` scores highly.
The selected target is a visible carry character. The three operations are a
bounded cost reduction, a draw rider, and an offensive rewrite.

Why it works:

The site is recognizably a deck-surgery scene. The player is comparing different
ways to improve one real build piece, which is exactly the texture the midgame
should support.

### Example 3: Mid Risk Or Skip

Run state:

- completion level 3
- deck identity is established
- Bane load is still manageable
- essence is healthy

Generation:

`risk_or_skip` scores well because the run is stable enough to consider a sharp
temptation. The offered action grants a premium visible dreamsign and adds two
visible Banes. The player may refuse.

Why it works:

The scene has one clear emotional promise. The burden is visible. The player is
not being asked to parse a generic utility menu.

### Example 4: Mid Now Versus Later

Run state:

- completion level 3
- one future hook already exists
- economy is stable
- deck identity is established

Generation:

`now_vs_later` scores well, but more structural future-shaping scenes are
downweighted because the run already has one unresolved hook. The filled options
offer a modest visible reward now or a larger visible package after the next
victory.

Why it works:

The site points forward without overloading the run. The player understands both
the reward class and the trigger condition.

### Example 5: Late Alter Future Dreamscape

Run state:

- completion level 5
- few dreamscapes remain
- one structural persistence slot is still available

Generation:

`alter_future_dreamscape` scores highly because route shaping is still
meaningful and the persistence budget can still tolerate one structural hook.
The player chooses one of several future-site biases for the next dreamscape.

Why it works:

The site reads as a commitment point rather than a random utility menu. It
appears late enough to matter and early enough to still change the run.

### Example 6: Late Take Up To N

Run state:

- completion level 6
- deck is nearly final
- trivial broad rewards no longer matter

Generation:

`take_up_to_n` scores highly. Each step offers a strong reward and adds a
visible increasing burden. The player may stop after each acceptance.

Why it works:

This produces climax-tier tension appropriate for the late run without becoming
an unreadable random chain.

### Example 7: Sequential Offers

Run state:

- completion level 3
- several transformable cards remain
- the run can still benefit from deck surgery

Generation:

`sequential_offers` scores well. The scene presents three consecutive
transformation proposals of the same general type, one after another, with
accept-or-pass at each step.

Why it works:

The scene feels like browsing a sequence of proposals rather than reading a
fixed three-column comparison. It uses a Journey Shape that does not fit cleanly
into a simple static menu.

### Example 8: Mid Single Offer

Run state:

- completion level 2
- deck still contains several weak starter cards
- essence is moderate

Generation:

`single_offer` scores well because the run can still benefit from one clean
service transaction without needing a full comparison menu. The site offers:
"Pay 75 essence to purge a chosen card." The player may accept or leave.

Why it works:

The tension is purely whether the exchange is worth the price. There is no
uncertainty and no hidden sting, so the scene should not be modeled as
`risk_or_skip`.

### Example 9: Mid Choose Your Loss

Run state:

- completion level 3
- deck identity is established
- several weak pieces still remain
- one sharp negative scene is acceptable

Generation:

`choose_your_loss` scores well because the run can survive one visible
sacrifice. The site presents three specific cards and asks the player which one
to lose.

Why it works:

The player is not choosing where to place a desired operation. The point of the
scene is to absorb a loss in the least harmful way.

### Example 10: Mid Single Wager

Run state:

- completion level 3
- essence is healthy
- the run can tolerate one swingy economy scene

Generation:

`single_wager` scores well. The site offers one visible gamble: "Stake 60
essence. 50%: gain 180 essence. 50%: gain nothing."

Why it works:

This is a one-shot bet with a bounded envelope. It should not be modeled as
`push_your_luck` because the scene does not ask the player whether to continue.

## Classifying Journey Content In The Shape-First Model

Journey content categories are not all the same kind of thing. The shape-first
model classifies them explicitly.

### Direct Journey Shape Inputs

The following Journey ideas map directly or nearly directly to Journey Shapes:

- intentionally eclectic option sets map to `random_allocation`
- "all cost + simple effect", "three options with the same cost and different
  effects", and many "shop-like" setups map to `same_cost_different_rewards`,
  `service_menu`, `shop_row`, or `single_offer` depending on option count and
  purchasing frame
- "three options with the same effect and different costs" and "gain a small
  reward or more with a cost" map to `same_reward_different_costs`
- "two effect choices on the same card" and "different effects where you choose
  the card" map to `one_target_many_operations`
- tightly parallel operation menus map to `mirrored_operations` when the overt
  symmetry is the point of the scene rather than a secondary fill rule
- "pick one of three cards to apply a transfiguration to" and several "same
  operation, different target" ideas map to `one_operation_many_targets`
- "choose one of these visible losses" maps to `choose_your_loss`
- "take this deterministic deal or leave it" maps to `single_offer`
- burdened temptations with an explicit refusal path map to `risk_or_skip`
- one-shot bets with a visible stake and bounded outcome envelope map to
  `single_wager`
- "gain a reward now or wait" maps to `now_vs_later`
- "gain a reward if you complete some condition" and other mini-quest promises
  map to `reward_after_trigger`
- shared-window menus such as "for your next battle, choose one of these
  effects" map to `timed_window_menu` when the common duration is the scene
  identity
- "gain a reward up to N times" maps to `take_up_to_n`
- "pay repeatedly to scale an effect" maps to `repeat_to_scale`
- escalating gamble scenes with stop-or-continue tension map to `push_your_luck`
- fixed small random series with bounded outcomes map to
  `resolved_random_series`
- "see sequential offers" maps to `sequential_offers`
- "reach deeper" and related digging scenes map to `escalating_search`
- "pair a major quest effect with a major reward" often maps to
  `commit_now_future_payoff`
- "add or remove future sites" and related route shaping map to
  `alter_future_dreamscape`

### Shape Modifiers Rather Than New Top-Level Shapes

Many Journey ideas are useful, but they should be fields on Journey Shapes
rather than additional top-level shapes unless that shared property is itself
the core promise of the scene:

- shared cost across all options
- shared reward class across all options
- shared target across all options
- shared timing across all options when the timing is not itself the scene
- shared burden across all options
- mechanically linked effect pool when the linkage is support rather than the
  main identity
- thematically related objects across options
- explicit refusal option
- explicit stop-or-continue after each acceptance
- bounded random envelope when the fixed random cadence is not itself the scene

### Effect-List Content

Several content categories are effect-list material rather than shape material:

- `Simple Effects` become reusable reward-effect entries
- `Costs` become cost or burden entries
- many `Compound Effects` become special reward or burden entries rather than
  top-level Journey Shapes

### Fill Parameters

Several content categories are parameters that Journey Shapes can use while
filling:

- `Conditions/Triggers`
- `Durations`
- `Predicates`

### Payload Surfaces With Separate V1 Scope Decisions

Several content categories are payload surfaces that may plug into Journey
Shapes later if they survive the V1 scope filter:

- `Custom Cards, Dreamsigns, and Transfigurations`
- `Battlefield Slot Modifications`
- `Statuses`

The important design point is that these are payload surfaces, not additional
top-level scene abstractions.

## Testing Strategy

The implementation should support three test layers.

### Integration Tests

For fixed seeds and run states, assert:

- chosen Journey Shape
- chosen site tag profile
- visible options
- hidden-policy metadata
- committed outcomes
- repair or fallback usage if it occurs

### Property Tests

Assert invariants such as:

- no impossible costs
- no empty target pools
- no untracked delayed hooks
- determinism under identical state and content version
- no mutation of already generated dreamscapes by unrelated later RNG
- no invalid repetition-loop structure in repeated-accept Journey Shapes

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
- Journey Shape coverage reviews
- site-tag coverage reviews

Golden-seed review is not just a correctness check. It is the main authored-feel
check.

## Telemetry

The runtime should log enough to answer:

- which Journey Shapes appear
- which site tags appear
- which options players choose
- how long they take
- which Journey Shapes are skipped
- whether delayed hooks are realized
- which Journey Shapes correlate with later busted or flat runs
- where repair and fallback are happening

Telemetry is not a nice-to-have. The shape-first system still needs empirical
feedback to know which Journey Shapes are producing fake choices, overpowered
temptations, opaque repeated loops, or late-run clutter.

## Operational Requirements

Because site manifests are precommitted, save data should persist the committed
manifests, not just the seed and a promise to regenerate them later. Each
manifest should record a content version or hash, the chosen Journey Shape, and
the final site tag profile. Repair steps and fallback use should be visible in
logs rather than silently swallowed.

Players should be able to inspect active delayed hooks in quest UI. The system
should not depend on the player remembering unresolved future promises.

## Implementation Clarifications

The main document intentionally avoids turning into a file-by-file contract, but
the following clarifications should be treated as the default implementation
interpretation for V1 unless later superseded by a dedicated schema document.

### Scoring Band

When the main spec says "choose within the top scoring band," the default
meaning should be:

- compute all legal candidate scores
- discard any candidate below 85 percent of the highest remaining score
- sample proportionally to the remaining scores
- break exact ties by stable content ID

This gives the generator controlled variety without letting clearly weaker
candidates through.

### Persistence Footprint

A persistence footprint should use a simple V1 scale:

- `0`: immediate-only effect
- `1`: delayed local effect or single follow-up payoff
- `2`: structural hook such as route shaping or future dreamscape bias

The main spec's rule of "at most two unresolved long-tail hooks, and at most one
of those route-altering" should be implemented using this scale.

### RNG Branch Derivation

When the main spec refers to a versioned RNG tree, the default derivation should
be a stable hash of:

- run seed
- dreamscape ID
- branch label
- content version hash

This is sufficient to prevent unrelated later rolls from perturbing already
generated Journey manifests.

### Safe Fallback Shapes

The system should define a small safe fallback set for each stage. Fallback
should be rare and visible in logs.

Suggested defaults:

- early: `curated_reward_trio`, `service_menu`
- mid: simple `same_cost_different_rewards`, `one_target_many_operations`
- late: precise `one_target_many_operations`, `same_reward_different_costs`
