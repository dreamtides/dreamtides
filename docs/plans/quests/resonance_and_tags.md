# Resonance and Card Tags

This companion document to [quests.md](quests.md) details the resonance system
and card tagging algorithm for Dreamtides. Resonance defines the game's "color
pie" and drives draft picks, shop offerings, and all other card selection during
a quest. Tags are a separate, internal-only organizational system used
exclusively by [Discovery Draft](quests.md#discovery-draft) and
[Specialty Shop](quests.md#specialty-shop) sites to generate thematic card
groupings. Tags never influence normal drafting or shop selection.

## Design Goals

1. **Emergent identity.** A player's deck should develop a clear strategic
   identity over 5-10 draft picks without explicitly declaring one.
2. **Meaningful choice tension.** The best draft pick should frequently *not* be
   the highest-resonance card.
3. **Replayability.** Two quests with the same dreamcaller should diverge based
   on early picks and pool variance.
4. **Convergence to two.** The system naturally converges toward two dominant
   resonances. Mono and three-resonance decks are possible but require
   deliberate effort.
5. **Data-driven tuning.** Every constant, curve, and threshold is in TOML.

## Resonance

### The Five Resonances

Each card carries 0-2 resonance symbols from: **Tide**, **Ember**, **Zephyr**,
**Stone**, **Ruin**. Each resonance has a mechanical identity:

| Resonance | Theme                         | Primary Keywords                                | Playstyle                     |
| --------- | ----------------------------- | ----------------------------------------------- | ----------------------------- |
| Tide      | Flow, cycles, inevitability   | Foresee, Discover, draw                         | Patient, information-oriented |
| Ember     | Speed, directness, momentum   | Kindle, low costs, point gain                   | Aggressive, high-velocity     |
| Zephyr    | Surprise, adaptability        | Fast, Prevent, Copy, banish-until               | Reactive, timing-dependent    |
| Stone     | Permanence, growth, weight    | High spark, energy production, static abilities | Board-centric, midrange       |
| Ruin      | Entropy, recursion, sacrifice | Dissolve, Abandon, Reclaim, discard             | Attrition, recursive          |

### Resonance Pairs

The ten two-resonance combinations define natural deck archetypes:

| Pair           | Name     | Strategy                                                              |
| -------------- | -------- | --------------------------------------------------------------------- |
| Tide + Ember   | Tempest  | Spellslinger -- chain cheap spells, draw into more                    |
| Tide + Zephyr  | Mirage   | Draw-go control -- react with fast spells, win through card selection |
| Tide + Stone   | Depths   | Ramp and value -- draw into expensive threats                         |
| Tide + Ruin    | Undertow | Recursion -- fill the void, reclaim the best cards                    |
| Ember + Zephyr | Gale     | Tempo -- cheap threats with fast disruption                           |
| Ember + Stone  | Crucible | Go wide -- flood the board, tribal pump effects                       |
| Ember + Ruin   | Cinder   | Aristocrats -- sacrifice cheap characters for value                   |
| Zephyr + Stone | Basalt   | Blink -- re-trigger materialized abilities                            |
| Zephyr + Ruin  | Eclipse  | Discard control -- deny resources, profit from the void               |
| Stone + Ruin   | Bedrock  | Reanimator -- get big things in void, bring them back                 |

### Resonance on Cards

Each card definition in TOML includes a `resonance` field:

```toml
[[card]]
name = "Titan of Forgotten Echoes"
resonance = ["Stone", "Ruin"]
```

**Distribution guidelines:**

- ~70% of cards carry exactly 1 resonance symbol
- ~10% carry exactly 2 (more common at uncommon/rare)
- ~20% carry 0 (neutral)
- Each resonance appears on roughly equal numbers of cards
- Dreamcallers always carry 1-2 resonance symbols (never neutral)
- Dreamsigns always carry exactly 1 resonance symbol

### Player Resonance Profile

A player's **resonance profile** is a vector of five values:

```
resonance_profile[r] = (
    sum(1 for each card in deck with resonance r)
  + sum(1 for each dreamsign with resonance r)
  + dreamcaller_resonance_bonus[r]
)
```

The **dreamcaller resonance bonus** is defined per dreamcaller in TOML,
typically 3-5 per resonance symbol. This ensures the dreamcaller meaningfully
shapes early offerings before any cards have been drafted.

Example: Dreamcaller with bonus (Tide: 4, Ruin: 4), player has drafted 3 Tide, 2
Ruin, 1 Stone:

```
Tide: 7, Ember: 0, Zephyr: 0, Stone: 1, Ruin: 6
```

## The Draft Pool

### Pool Generation

At quest start, a draft pool is generated from the full card collection. Each
card can appear multiple times, with copies determined by rarity:

| Rarity    | Base Copies               |
| --------- | ------------------------- |
| Common    | 4                         |
| Uncommon  | 3                         |
| Rare      | 2                         |
| Legendary | 1                         |
| Special   | 0 (never in random pools) |

**Starting variance:** Each resonance receives a random multiplier from
`[0.75, 1.25]`. Cards with that resonance have their copy count scaled by this
multiplier (rounded half-up, minimum 1). Dual-resonance cards use the average of
their two multipliers. Neutral cards are unaffected.

### Drawing from the Pool

Cards are drawn from the pool **without replacement**. When a draft site offers
4 cards per pick for 5 picks, the 5 selected cards are removed permanently. The
15 unselected cards return to the pool with a **staleness penalty**.

**Staleness:** Each card has a staleness counter (initially 0). Offered but not
selected in any context (draft, shop, battle reward, themed site): +1 staleness.
After each dreamscape completes: -1 staleness (minimum 0). Staleness reduces a
card's effective weight via:

```
final_weight(c) = resonance_weight(c) / (1 + staleness(c) * staleness_factor)
```

Where `staleness_factor` is a TOML constant (default 0.3).

## Resonance Weighting Algorithm

When selecting cards from the pool, each card receives a **selection weight**
determining its probability of being chosen.

### Weight Calculation

For card `c` with resonance set `S_c`:

```
if S_c is empty (neutral card):
    resonance_weight(c) = neutral_base

if S_c is non-empty:
    resonance_weight(c) = floor_weight + sum(
        player_resonance[r] ^ affinity_exponent  for r in S_c
    )
```

Where:

- `neutral_base`: Weight for neutral cards (TOML, default 3.0)
- `floor_weight`: Minimum weight for resonance cards (TOML, default 0.5)
- `affinity_exponent`: Curve shape (TOML, default 1.4)

The exponent is the key tuning lever. At 1.0 (linear) the system converges
slowly. At 2.0 (quadratic) it locks in fast. The default 1.4 produces dominant
resonance weights 15-30x higher than the off-color floor after 5-10 picks.

Note that off-color resonance cards (where the player has 0 in that resonance)
receive only `floor_weight` (0.5), making them less likely than neutral cards
(`neutral_base` 3.0). This is intentional: neutral cards are generic utility,
while off-color cards would actively dilute the player's resonance identity.

### Worked Example

Player resonance: `{Tide: 7, Ember: 0, Zephyr: 0, Stone: 1, Ruin: 6}`.
Constants: exponent 1.4, floor 0.5, neutral base 3.0.

| Card              | Resonance    | Weight                   |
| ----------------- | ------------ | ------------------------ |
| Whirlpool Seer    | Tide         | 0.5 + 7^1.4 ≈ 15.7       |
| Void Harvester    | Ruin         | 0.5 + 6^1.4 ≈ 12.8       |
| Abyssal Reclaimer | Tide + Ruin  | 0.5 + 15.2 + 12.3 ≈ 28.0 |
| Boulder Sentinel  | Stone        | 0.5 + 1^1.4 = 1.5        |
| Flame Imp         | Ember        | 0.5 + 0 = 0.5            |
| Field Medic       | (neutral)    | 3.0                      |
| Echoing Monolith  | Stone + Ruin | 0.5 + 1.0 + 12.3 ≈ 13.8  |

The dual Tide+Ruin card scores highest. Echoing Monolith (Stone+Ruin) still
scores well via the Ruin component -- a "splash" opportunity.

### Selection Probability

```
probability(c) = final_weight(c) / sum(final_weight(x) for x in pool)
```

Cards are drawn sequentially via weighted random sampling without replacement
within a single draw operation.

### Selection Procedure

When selecting N cards to present (for drafts, shops, etc.):

1. Compute `final_weight(c)` for every card in the eligible pool
2. Draw N cards via weighted random sampling (without replacement)
3. **Resonance diversity check:** If all N cards share a single resonance and N
   \>= 4, replace the lowest-weight card with the highest-weight card from a
   different resonance (neutral cards count as "different" for this purpose)
4. **Rarity guarantee:** For N >= 4, at least 1 card must be uncommon+. If all
   commons, replace the lowest-weight common with the highest-weight uncommon+
5. Mark drawn cards with +1 staleness. Selected cards are removed from the pool

For steps 3-4, if the pool has no qualifying replacement card, refill the pool
with fresh copies of all cards at base rarity copy counts (staleness reset to 0,
ignoring starting variance) and retry. If the refilled pool still has no
qualifying card, skip the guarantee and proceed as drawn.

### Rarity Override for Battle Rewards

The rare draft pick after winning a battle filters the pool to Rare and
Legendary cards only. If this subset is empty, refill Rare and Legendary cards
at base copy counts (staleness reset to 0). Resonance weighting applies normally
within this subset.

### Dreamcaller Influence

Dreamcallers influence selection through resonance seeding only -- contributing
a large resonance bonus (3-5 per symbol) that biases early offerings.

```toml
[[dreamcaller]]
name = "Vesper, Twilight Arbiter"
resonance = ["Tide", "Ruin"]
resonance_bonus = { Tide = 4, Ruin = 4 }
tags = ["mechanic:reclaim", "mechanic:dissolve"]
essence_bonus = 50  # starting essence; see quests.md
```

### Dreamsign Influence

Dreamsigns are not part of the draft pool -- they are offered through dedicated
[Dreamsign Offering](quests.md#dreamsign-offering) and
[Dreamsign Draft](quests.md#dreamsign-draft) sites. However, dreamsigns
contribute their resonance symbols to the player's resonance profile, just like
deck cards. Tags on dreamsigns only matter for Discovery Draft and Specialty
Shop theme selection.

## Card Tags

Tags define a card's mechanical role within specific archetypes. They are
finer-grained than resonance and cut across resonance boundaries -- a
`tribal:spirit-animal` tag might appear on Tide, Stone, or Ruin cards.

### Tag Categories

**Tribal tags** indicate a card is a member of or cares about a subtype:

```
tribal:warrior, tribal:spirit-animal, tribal:ancient, tribal:survivor,
tribal:visitor, tribal:explorer, tribal:synth, tribal:outsider,
tribal:musician, tribal:mage
```

Only subtypes with ~5+ support cards get tribal tags.

**Mechanic tags** identify participation in a mechanical theme:

```
mechanic:reclaim, mechanic:dissolve, mechanic:sacrifice, mechanic:kindle,
mechanic:foresee, mechanic:figments, mechanic:draw, mechanic:discard,
mechanic:fast, mechanic:ramp, mechanic:copy, mechanic:banish, mechanic:bounce
```

**Role tags** describe strategic purpose (at most 1 per card):

```
role:removal, role:finisher, role:engine, role:utility, role:payoff,
role:enabler
```

### Tag Assignment

Tags are defined in TOML alongside resonance:

```toml
[[card]]
name = "Titan of Forgotten Echoes"
resonance = ["Stone", "Ruin"]
tags = ["tribal:ancient", "mechanic:reclaim", "role:finisher"]
```

Guidelines:

- Most cards have 1-3 tags
- Characters with supported subtypes get the corresponding tribal tag
- Events referencing a subtype also get the tribal tag
- Mechanic tags are assigned liberally
- Tags need not align with resonance

### Tag Profile

The player's **tag profile** tracks tag counts across their deck and dreamsigns.
It has no effect on normal drafting -- it is used exclusively for Discovery
Draft and Specialty Shop [theme selection](#theme-selection).

```
tag_count[t] = (
    number of cards in deck with tag t
  + number of dreamsigns with tag t
  + dreamcaller_tag_bonus[t]
)
```

The **dreamcaller tag bonus** works like the resonance bonus: each dreamcaller
defines 1-3 tags in TOML with a bonus value (typically 1-2). This seeds the tag
profile so that early Discovery Drafts and Specialty Shops reflect the
dreamcaller's mechanical identity rather than falling back to uniform random.

## Discovery Draft and Specialty Shop Generation

[Discovery Draft](quests.md#discovery-draft) and
[Specialty Shop](quests.md#specialty-shop) sites present cards with a unifying
mechanical theme, selected using the tag profile.

### Theme Selection

1. **Candidate filtering.** Consider each tag `t` with at least
   `min_theme_cards` cards in the pool (TOML, default 6).

2. **Tag affinity scoring.** For each candidate tag, compute affinity using a
   diminishing returns curve:

   ```
   tag_affinity(t) = ln(1 + tag_count[t]) * tag_scale
   ```

   Where `tag_scale` is TOML (default 1.5). The logarithm means the first few
   cards with a tag significantly increase affinity, while additional cards have
   diminishing impact.

3. **Theme score.** Blend tag affinity with pool depth:

   ```
   theme_score(t) = tag_affinity(t) * relevance_boost + pool_depth(t) * depth_factor
   ```

   - `relevance_boost`: TOML multiplier for tag affinity (default 2.0)
   - `pool_depth(t)`: cards with tag `t` remaining in pool
   - `depth_factor`: TOML weight for pool depth (default 0.1)

4. **Weighted random selection.** Select a theme with probability proportional
   to its score. Tags the player has drafted into are preferred, but zero-
   affinity tags can still appear if they have deep pool coverage.

5. **Cold start.** If no tag has any affinity (early quest, no dreamcaller tag
   bonus), fall back to uniform random from eligible tags.

### Themed Card Selection

Once a theme is selected, cards are drawn using the standard resonance-based
algorithm, restricted to cards with the selected tag. Resonance weighting,
staleness, the rarity guarantee, and the diversity check all still apply. If the
tagged subset of the pool has fewer cards than needed, refill it by adding fresh
copies of all cards with that tag (at their base rarity copy counts, ignoring
starting variance) before drawing. Staleness on refilled cards resets to 0. The
player experiences themed sites as curated collections that fit their deck
without needing to understand any tag mechanic.

## Convergence Dynamics

**Early quest (picks 1-5).** The dreamcaller's resonance bonus dominates.
Offerings are biased toward the dreamcaller's resonances but with significant
variance. Neutral cards (weight 3.0) appear frequently since on-color cards only
reach weights of ~5-9 (profile 4-5, exponent 1.4).

**Mid quest (picks 6-15).** Drafted resonances snowball. On-color mono weights
reach ~13-16 (profile 7-8) while off-color stays at 0.5. Dual-resonance cards
bridging the two strongest resonances become common, reaching weights of ~25-30.
The tag profile accumulates enough data that Discovery Drafts and Specialty
Shops offer relevant themes. The deck's identity crystallizes.

**Late quest (picks 16+).** Dominant resonances produce very high weights
(profile 12+, weights 30+). The player mostly sees their two main resonances.
Themed sites reliably offer archetype staples. The player refines rather than
defines.

**Mono-resonance** decks emerge when the dreamcaller has only one resonance and
the player commits. Viable but narrow -- sacrifices dual-resonance synergy for
deeper single-resonance options.

**Three-resonance** decks require deliberate splitting. The exponent >1 means no
single resonance hits the high-weight zone, producing varied but unfocused
offerings. Dreamsigns help provide direction.

## TOML Configuration Reference

```toml
[draft_pool]
copies_common = 4
copies_uncommon = 3
copies_rare = 2
copies_legendary = 1
variance_min = 0.75
variance_max = 1.25

[resonance]
neutral_base = 3.0
floor_weight = 0.5
affinity_exponent = 1.4

[tags]
tag_scale = 1.5
min_theme_cards = 6
relevance_boost = 2.0
depth_factor = 0.1

[staleness]
staleness_factor = 0.3

[selection]
enforce_resonance_diversity = true
enforce_rarity_guarantee = true
```

## Example: Vesper, Twilight Arbiter Quest

The player selects Vesper (Tide + Ruin, resonance bonus Tide: 4, Ruin: 4, tag
bonus `mechanic:reclaim`: 2, `mechanic:dissolve`: 1). Initial resonance profile:
`{Tide: 4, Ruin: 4}`. Tag profile: `{reclaim: 2, dissolve: 1}`.

**First draft (dreamscape 0).** Offerings favor Tide and Ruin. The player picks
Void Harvester (Ruin, tags: `mechanic:reclaim`). Profile: `{Tide: 4, Ruin: 5}`.

Over the next few picks the player drafts 2 more Ruin cards and 1 Tide card.
Profile: `{Tide: 5, Ruin: 7}`. Tag profile:
`{reclaim: 4, dissolve: 2, draw: 1}`.

**Discovery Draft at dreamscape 2.** The theme selector computes tag affinity:
`mechanic:reclaim` has count 4, giving `ln(5) * 1.5 ≈ 2.41`. Other tags score
lower. The system selects reclaim as the theme and offers 4 reclaim-tagged cards
from the pool, weighted by resonance toward Tide and Ruin. The player picks a
Tide+Ruin dual reclaim card, reinforcing the Undertow archetype.
