# Dream Journey TypeScript Shape Contracts

## Purpose

This appendix provides the shape-local implementation details that the main
TypeScript Dream Journey document intentionally keeps abstract.

The worked examples in
[dream_journey_generation_appendix.md](dream_journey_generation_appendix.md)
remain illustrative unless this appendix gives them a concrete dossier. When a
shape dossier exists here, it is the default V1 implementation contract for the
CLI.

## Related Information

- [Dream Journey Generation TypeScript Prototype]
  (dream_journey_generation_typescript.md): Main CLI design and schema contract.
- [Dream Journey Generation Appendix](dream_journey_generation_appendix.md):
  Illustrative worked examples and classification notes.

## Dossier: `curated_reward_trio`

### Player-Facing Promise

`curated_reward_trio` is a transparent three-offer discovery scene. The player
is choosing between three intentionally different immediate build rewards that
still read as one coherent stop rather than as a generic utility menu.

### Topology

- root node kind: `choose_one`
- allowed follow-up node kinds: none
- root option count: exactly `3`
- maximum node depth: `1`
- maximum total nodes: `1`
- maximum future-hook budget consumed by the shape: `0`
- default reveal policy: `fully_revealed`

This shape is single-step on purpose. If the scene needs a refusal branch, a
price menu, or a second step, it is not `curated_reward_trio`.

### Required Shared Axes

Every filled `curated_reward_trio` must satisfy all of the following:

- shared scene motif: all three options must fit one authored discovery premise
- shared timing: all three options are immediate visible gains
- shared pressure band: all three options must plausibly address the same broad
  run need
- shared transparency: no hidden downsides, no hidden targets, no delayed hook

The generator must treat the scene motif as a real fill input, not as flavor
text added after the fact.

### Fill Slots

The V1 fill slots are:

1. `scene_motif`
2. `need_profile`
3. `family_a_reward`
4. `family_b_reward`
5. `family_c_reward`
6. `option_summary_text`

No cost slot, burden slot, trigger slot, or future-hook slot is legal for this
shape in V1.

### Distinct Reward-Family Rule

The three reward slots must come from three distinct reward families. For V1,
the allowed families are:

- `card_package`
- `dreamsign_offer`
- `essence_cleanup_bundle`
- `economy_package`
- `card_refinement`

The same family may not appear twice in one trio. If the generator cannot fill
three distinct families while preserving coherence, it must repair or fall back
to a different shape.

### Early-Stage Default

For completion levels `0` and `1`, the default desired tags for this shape are:

- `build`
- `reward`
- `immediate`
- `discovery`

The early-stage default family mix for Example 1 is:

- one `card_package`
- one `dreamsign_offer`
- one `essence_cleanup_bundle`

This is not the only legal early trio, but it is the default authored pattern
the CLI should use for the appendix example.

### Shape-Local Selection Rules

When the stage is `early`, `starterCardCount >= 6`, `baneCount == 0`, and
`activePersistentHooks` is empty:

- boost `curated_reward_trio` for matching `build` and `immediate` tags
- reject any fill that introduces a persistence footprint above `0`
- reject any fill that uses sacrifice, gamble, or route-change surfaces
- prefer card and dreamsign nouns that improve identity rather than cleanup-only
  nouns
- cap the cleanup bundle at a modest secondary role rather than the scene's main
  emotional promise

This is the concrete interpretation of Example 1's prose statement that the run
has "early build pressure."

### Shape-Local Validation Rules

In addition to the global validator, `curated_reward_trio` must reject any
manifest where:

- fewer than three distinct reward families are visible
- any option contains a meaningful downside or refusal frame
- any option creates a future hook
- all three options point at unrelated site tags or unrelated run needs
- one option is strictly a larger version of another with no family difference
- the cleanup bundle is the only option that interacts with weak starter cards

The last rule matters because the shape should read as "three ways to improve
the run now," not "two rewards plus one chore."

## Example 1 Contract

### Run Context

Example 1 from the appendix should be interpreted as the following concrete
context snapshot:

```toml
[context]
completion_level = 0
stage = "early"
deck_size = 12
starter_card_count = 8
bane_count = 0
transfigured_card_count = 0
dreamsign_count = 0
current_essence = 55
current_omens = 0
active_persistent_hooks = []
recent_shapes = []
recent_site_tags = []
remaining_dreamscapes_estimate = 6

[context.need_profile]
primary = "build"
secondary = "cleanup"
```

### Stage Profile Fragment

The early stage profile must be explicit enough to make the example
reproducible. A minimal V1 fragment is:

```toml
[[stage_profiles]]
id = "early"
completion_levels = [0, 1]
desired_site_tags = ["build", "reward", "immediate", "discovery"]
persistence_tolerance = 0

[stage_profiles.shape_weight_multipliers]
curated_reward_trio = 1.35
service_menu = 1.15
same_cost_different_rewards = 1.1
risk_or_skip = 0.55
paired_return = 0.2
alter_future_dreamscape = 0.1
```

The exact numbers may change during tuning, but the stage profile must expose an
explicit authored preference of this form. The implementation should not infer
it from prose.

### Shape Definition Fragment

The shape entry for Example 1 should look like this:

```toml
[[shapes]]
id = "curated_reward_trio"
comparison_promise = "Choose one of three different immediate build rewards."
root_node_kind = "choose_one"
normal_option_count = 3
required_shared_axes = ["scene_motif", "need_profile", "timing:immediate"]
permitted_node_kinds = ["choose_one"]
fill_slots = [
  "scene_motif",
  "need_profile",
  "family_a_reward",
  "family_b_reward",
  "family_c_reward",
]
tag_affinities = ["build", "reward", "immediate", "discovery"]
forbidden_tags = ["gamble", "sacrifice", "delayed", "route_change"]
selection_bounds = "choose_one"
maximum_branch_width = 3
maximum_branch_depth = 1
repair_order = [
  "swap_conflicting_reward_family",
  "reroll_scene_motif",
  "fallback_shape",
]

[shapes.family_rules]
require_distinct_reward_families = true
allowed_reward_families = [
  "card_package",
  "dreamsign_offer",
  "essence_cleanup_bundle",
  "economy_package",
  "card_refinement",
]
```

### Content Entry Fragments

Example 1's three filled options correspond to three separate reward entries:

```toml
[[content]]
id = "early_card_package_growth"
role = "reward"
surface = "cards"
reward_family = "card_package"
preview_templates = [
  "Add one of these cards to your deck: Tidebound Sentry, Patient Cartographer, or Lantern Sprite.",
]
contributed_site_tags = ["build", "reward", "immediate", "discovery"]
stage_weights = { early = 1.0, mid = 0.45, late = 0.1 }
base_selection_weight = 1.0
compatibility_tags = ["motif:discovery_cache", "need:build"]
exclusion_tags = ["cost", "burden", "delayed"]
context_requirements = ["starter_card_count >= 4"]
persistence_footprint = 0
reveal_contract = "fully_revealed"
runtime_readiness = "preview_only"

[[content]]
id = "early_dreamsign_offer_kindle"
role = "reward"
surface = "dreamsigns"
reward_family = "dreamsign_offer"
preview_templates = [
  "Gain Emberwake Sigil, a Dreamsign that helps your first character come online faster.",
]
contributed_site_tags = ["build", "reward", "immediate", "discovery"]
stage_weights = { early = 0.95, mid = 0.5, late = 0.15 }
base_selection_weight = 0.9
compatibility_tags = ["motif:discovery_cache", "need:build"]
exclusion_tags = ["cost", "burden", "delayed"]
context_requirements = ["dreamsign_count <= 1"]
persistence_footprint = 0
reveal_contract = "fully_revealed"
runtime_readiness = "preview_only"

[[content]]
id = "early_essence_cleanup_bundle"
role = "reward"
surface = "compound"
reward_family = "essence_cleanup_bundle"
preview_templates = [
  "Gain 60 essence and purge a starter card from your deck.",
]
contributed_site_tags = ["build", "cleanup", "reward", "immediate", "discovery"]
stage_weights = { early = 0.85, mid = 0.65, late = 0.2 }
base_selection_weight = 0.75
compatibility_tags = ["motif:discovery_cache", "need:build", "need:cleanup"]
exclusion_tags = ["burden", "delayed", "future_hook"]
context_requirements = ["starter_card_count >= 1"]
persistence_footprint = 0
reveal_contract = "fully_revealed"
runtime_readiness = "preview_only"
```

These are three reward entries, not one shape with hardcoded strings. The shape
chooses families; content entries supply the visible offers.

### Scene Motif Asset

Example 1 also requires an authored motif to make the trio coherent:

```toml
[[journey_motifs]]
id = "discovery_cache"
summary = "An intact cache of dreamcraft fragments offers three ways to improve a young run."
allowed_shapes = ["curated_reward_trio"]
required_tags = ["build", "reward", "immediate", "discovery"]
```

The motif is what makes the card package, dreamsign offer, and cleanup bundle
read as one discovery scene instead of three unrelated offers.

## Example 1 Fill Walkthrough

The CLI implementation should treat Example 1 as the following deterministic
fill sequence:

1. derive stage `early` and need profile `build` plus secondary `cleanup`
2. enumerate legal shapes and score `curated_reward_trio`, `service_menu`, and
   `same_cost_different_rewards`
3. select `curated_reward_trio`
4. choose scene motif `discovery_cache`
5. fill one `card_package` reward compatible with `motif:discovery_cache`
6. fill one `dreamsign_offer` reward compatible with `motif:discovery_cache`
7. fill one `essence_cleanup_bundle` reward compatible with
   `motif:discovery_cache`
8. verify all three families are distinct and all persistence footprints are `0`
9. render one `choose_one` node with three visible summaries

The implementation must not skip step 4. Coherence is enforced through the motif
and compatibility tags, not by hoping the rendered text happens to sound
related.

## Example 1 Expected Manifest Shape

A compliant manifest can be rendered as:

```text
Dream Journey: discovery_cache

Choose one:
1. Add one of these cards to your deck: Tidebound Sentry, Patient Cartographer, or Lantern Sprite.
2. Gain Emberwake Sigil, a Dreamsign that helps your first character come online faster.
3. Gain 60 essence and purge a starter card from your deck.
```

This output is illustrative text, but the topology and family split are
normative for the example.
