# Dreamtides Quest Redesign: Collection-Based Card Acquisition

## 1. Core Philosophy

**Draft is gone. The player builds a collection and constructs a deck from it.**

Card acquisition works like a single-player RPG card game (Shandalar,
Thronebreaker, Balatro) rather than a draft format:

- **Packs are loot.** You open them and get everything inside. The strategic
  choice is *where you go* to get specific packs, not *which cards to pick* from
  a pack.
- **Shops are the agency center.** You browse individual cards and buy what you
  need. A separate pack vendor sells themed packs. This is where essence matters
  most.
- **Deck editing is the strategic core.** The game-within-a-game is constructing
  and tuning the best possible deck from your growing collection. The deck
  editor is always accessible.
- **Ante creates stakes and acquisition.** Inspired by Marvel Snap's doubling
  cube and Shandalar's ante, both sides wager cards before battle. The winner
  takes the opponent's ante -- this IS card acquisition from battles, not a
  separate system. Mid-battle escalation at turn 6 forces a dramatic
  commitment-or-concession moment.
- **Dreamscape navigation is the macro decision.** You see pack themes, shop
  availability, and reward cards on the atlas before choosing where to go. This
  is where the "what do I pursue" decision lives.

The goal is a mode that feels like constructed Magic -- you know what's
available, you choose what to acquire, and you build around a plan. Variance
comes from what's offered at each dreamscape, shop inventories, battle
opponents, forge recipes, and dream journeys -- not from drafting.

`<angle bracket names>` denote configurable values with defaults and URL
parameter overrides for the quest simulator.

## 2. Tide System

Default to the **revised tide system** (docs/tides/tides_revised.md) where
archetypes live in the overlap between neighboring tides. The original tide
system is available via URL parameter.

With revised tides:

- A pack from a single tide gives you *tools* but not a *complete strategy*.
- Players naturally want cards from 2-3 adjacent tides.
- "A tide is not a deck" -- this makes collection-building more interesting.

With original tides:

- Each tide is more self-contained as a strategy.
- Single-tide packs feel more complete.

**URL:** `?revisedTides=true` (default: true)

The seven neighbor archetypes (revised tides):

| Alliance Name     | Tides         | Archetype             |
| ----------------- | ------------- | --------------------- |
| Verdant Ascension | Bloom + Arc   | Ramp / Go Tall        |
| Prismatic Echo    | Arc + Ignite  | Flicker               |
| Burning Legion    | Ignite + Pact | Warriors / Go Wide    |
| Shadow Bargain    | Pact + Umbra  | Sacrifice / Abandon   |
| Frozen Depths     | Umbra + Rime  | Self-Mill / Recursion |
| Shattered Verse   | Rime + Surge  | Discard Matters       |
| Dreamstorm        | Surge + Bloom | Storm                 |

The card pool with revised tides is at
rules_engine/tabula/rendered-cards.toml

The card pool with original tides is at rules_engine/tabula/rendered-cards-mono.toml

## 3. Starting a Quest

### 3.1 No Starting Choices

The quest begins immediately with no player decisions. A center tide and
starting dreamscape are selected at random, and the player receives starter
cards. This avoids the analysis paralysis of evaluating alliance options before
you've even seen a card.

The player's first meaningful choices happen *inside* the first dreamscape:
which sites to visit, which cards to put in their deck, and which dreamcaller to
select. These decisions are made with cards in hand rather than in the abstract.

### 3.2 Starting Tide Selection

A random center tide is chosen. The player receives cards from that tide plus
its two neighbors on the tide circle, giving `<starting_tides>` (default: **3**)
tides total.

For example, if Arc is selected: cards come from Bloom, Arc, and Ignite. This
covers two overlapping archetypes (Ramp and Flicker), giving the player two
viable directions to specialize into.

Whether the 3 starting tides are **sequential** (center + neighbors) or **fully
random** is configurable. Sequential gives coherent synergy from the start;
random creates more chaotic openings that require creative deckbuilding.

**URL:** `?startingTides=3&sequentialTides=true`

### 3.3 Receive Starter Cards

The player receives `<initial_cards>` (default: **25**) cards drawn equally from
all cards in their starting tides, plus `<starter_neutral>` (default: **5**)
Neutral cards for removal and utility. Total starting pool: 30 cards.

All cards are drawn with equal probability regardless of rarity. The only
guarantee is a playable energy curve: at least `<starter_low_cost>` (default:
**5**) cards costing 0-2, at least `<starter_mid_cost>` (default: **4**) costing
3-4, and at least `<starter_high_cost>` (default: **2**) costing 5+.

All cards go into the **card pool**. The player's deck starts as a copy of the
full pool.

**URL:**
`?initialCards=10&starterNeutral=5&starterLowCost=4&starterMidCost=3&starterHighCost=1`

### 3.4 Starting Resources

- `<starting_essence>` (default: **250**) essence.
- No dreamcaller (selected during first dreamscape).
- No dreamsigns.

**URL:** `?startingEssence=250`

### 3.5 First Dreamscape

The first dreamscape is **automatically entered** -- the player does not choose
from the atlas. It always contains:

- Dreamcaller Draft (1)
- Loot Pack (2, from the player's starting tides)
- Card Shop (1)
- Battle (1)

This ensures the player's first experience is: open packs, browse a shop, pick a
dreamcaller, edit their deck, fight. No atlas navigation decisions until after
the first battle.

After completing the first dreamscape, the Dream Atlas opens and the player
begins making navigation choices with a real collection to reason about.

## 4. Card Pool and Deck

### 4.1 The Card Pool

The **card pool** is every card the player has acquired this quest. It grows
throughout the quest and has no size limit. Cards enter the pool from: starter
cards, loot packs, shop purchases, battle trophies, forge output, dream
journeys, reward sites, and duplication.

### 4.2 The Deck

The **deck** is a subset of the pool that goes into battle.

- Minimum size: `<minimum_deck_size>` (default: **30**)
- Maximum size: `<maximum_deck_size>` (default: **30**)
- Max copies of any card: `<max_copies>` (default: **2**)

**URL:** `?minimumDeckSize=25&maximumDeckSize=50&maxCopies=2`

### 4.3 Deck Editor

Accessible anytime: from the Dream Atlas, before battles, or by clicking the
deck icon during a dreamscape. Features:

- **Split view**: Pool on one side, deck on the other. Click to move cards
  between them.
- **Filters**: By tide, energy cost, card type, keyword.
- **Deck stats**: Tide distribution, energy curve histogram, card type
  breakdown.
- **Deck size indicator**: Current size and min/max boundaries.

### 4.4 Pre-Battle Sideboarding

Before each battle, after all other sites are visited, the player sees:

- The opponent's **dreamcaller** and `<opponent_preview_cards>` (default: **3**)
  representative cards from the opponent's deck. Enough to read their strategy
  without information overload.
- Their current deck with access to the full pool for final adjustments.

**URL:** `?opponentPreviewCards=3`

## 5. Card Acquisition Paths

### 5.1 Loot Packs (Primary volume, no card-level choice)

**What they are:** Themed packs tied to dreamscape sites. When you visit a Loot
Pack site, you open the pack and **all cards go to your pool**. No selection.

**Pack contents:** `<loot_pack_size>` (default: **4**) cards drawn with equal
probability from all cards in the pack's tide(s). No rarity weighting.

**Duplicate protection:** Scales with how many copies the player already owns.
First copy: no penalty. Second copy: weight reduced by `<dupe_penalty_2>`
(default: **50%**). Third+ copy: weight reduced by `<dupe_penalty_3>` (default:
**90%**). This means you'll sometimes get useful duplicates (a second copy of a
card you want to run 2-of) but rarely get a third copy of something you already
have plenty of.

**URL:** `?lootPackSize=4&dupePenalty2=50&dupePenalty3=90`

**How packs are themed:** Each loot pack site on a dreamscape has a tide
associated with it. Dreamscapes can contain packs from **different tides** -- a
single dreamscape might offer a Bloom pack and an Ignite pack. This is
especially relevant at early completion levels where dreamscapes have 2-3 pack
sites.

**Where the agency lives:** Pack themes are visible on the Dream Atlas before
choosing a dreamscape. "That dreamscape has a Bloom pack and a Pact pack; this
one has two Ignite packs and a Forge." The strategic choice is navigation, not
card picking.

**Pack theme generation:** Loot pack themes are generated to be relevant to the
player's pool:

- `<pack_on_theme_weight>` (default: **60%**): A tide matching the player's
  most-represented pool tides.
- `<pack_adjacent_weight>` (default: **25%**): A tide adjacent to the player's
  dominant tides on the circle.
- `<pack_explore_weight>` (default: **15%**): A random tide the player has few
  cards from (forge fuel, splash opportunities).

When a dreamscape has multiple pack sites, the algorithm ensures they offer
**different** tides.

**URL:** `?packOnTheme=60&packAdjacent=25&packExplore=15`

**All dreamscapes contain at least 1 loot pack site.** Pack sites per completion
level:

| Completion Level | Loot Pack Sites |
| ---------------- | --------------- |
| 0, 1             | 3               |
| 2, 3             | 2               |
| 4, 5             | 1               |
| 6                | 1               |

### 5.2 Card Shop (Targeted individual card acquisition)

The Card Shop sells `<card_shop_size>` (default: **4**) individual cards,
browsable face-up with full rules text. Weighted toward the player's dominant
pool tides but including some adjacent-tide and Neutral options.

**Pricing:** Each card has a randomized price drawn from a range:
`<card_price_min>` (default: **50**) to `<card_price_max>` (default: **100**).
Random discounts: 1-2 cards may be on sale (30-70% off). Prices are in
increments of 5.

**Reroll:** Refreshes the card selection. Costs `<reroll_base>` (default:
**40**) + `<reroll_increment>` (default: **20**) per previous reroll this visit.

**URL:**
`?cardShopSize=4&cardPriceMin=50&cardPriceMax=100&rerollBase=40&rerollIncrement=20`

Icon: "Store"

### 5.3 Pack Shop (Bulk themed acquisition)

The Pack Shop sells `<pack_shop_size>` (default: **3**) themed packs for
purchase with essence. Unlike free loot packs, these cost essence but let you
target specific themes you might not find on the atlas.

**Pack types:** The Pack Shop **primarily offers tide packs** -- single-tide
packs are the bread and butter. Occasionally, alliance packs or mechanical packs
appear as special offerings:

| Pack Type     | Contents                                    | Price |
| ------------- | ------------------------------------------- | ----- |
| Tide Pack     | 4 cards from a single tide                  | 100   |
| Alliance Pack | 4 cards from an alliance pair               | 125   |
| Removal Pack  | 4 removal/interaction cards, any tides      | 125   |
| Aggro Pack    | 4 low-cost aggressive characters, any tides | 100   |
| Events Pack   | 4 event cards, any tides                    | 100   |

The frequency of non-tide packs: `<special_pack_chance>` (default: **20%**). The
remaining 80% of pack shop slots are tide packs weighted to the player's tides.

Packs do **not** refresh on reroll. They are set when the Pack Shop is
generated.

**URL:** `?packShopSize=3&specialPackChance=20`

Icon: "Layers"

### 5.4 Ante (Card acquisition through battle)

The ante system is the primary way battles produce card rewards. Before each
battle, both sides wager cards. The winner takes the opponent's anted cards. See
Section 6 for the full ante system design.

Ante serves as both acquisition and tension:

- **Splash cards**: Opponents wager cards from their tides, giving access to
  off-tide cards the player wouldn't normally encounter.
- **Stakes**: Every battle has something meaningful at risk beyond just quest
  progress.
- **Concession strategy**: The mid-battle escalation at turn 6 creates a
  dramatic commitment-or-fold moment.

### 5.5 Forge (Mid-late game, transmutation)

The Forge lets the player sacrifice cards from their pool to create a specific
new card. The design is deliberately simple:

**How it works:** The Forge shows `<forge_recipes>` (default: **3**) offers.
Each offer is:

> "Sacrifice `<forge_cost>` (default: **4**) [Tide] cards → Gain \[specific card
> from a different tide\]"

The output card is always from a **different tide** than the sacrifice. This is
the Forge's key purpose: converting cards you don't need into cards from a tide
you're building toward.

**Constraints:**

- The Forge **only shows offers the player can currently fulfill** with their
  pool. No teasing recipes you can't afford.
- The sacrifice tide is chosen from whichever tides the player has the most
  excess cards in (cards in pool but not in deck).
- The output card is weighted toward the player's deck tides.

Example: A player heavy in Bloom cards with an Arc-leaning deck might see:
"Sacrifice 4 Bloom cards → Gain [specific Arc card]."

The player may decline all offers.

**URL:** `?forgeRecipes=3&forgeCost=4`

Icon: "Anvil"

### 5.6 Draft Site (Occasional, small draft element)

A minor site type providing a small draft-like selection. The player sees
`<draft_site_total>` (default: **4**) cards and keeps `<draft_site_keep>`
(default: **1**). The rest are discarded.

This site appears in the site pool starting at completion level 2. It provides a
targeted acquisition moment that feels different from shops (curated set, no
essence cost) and different from loot packs (card-level agency). It's one site
among many, not the core loop.

Cards shown are weighted to the player's tides.

**URL:** `?draftSiteTotal=4&draftSiteKeep=1`

Icon: "Rectangle Vertical"

### 5.7 Provisioner (Buy sites for the current dreamscape)

The Provisioner is a new site type where the player can spend essence to **add a
site to the current dreamscape**. The Provisioner shows `<provisioner_options>`
(default: **3**) potential sites with their costs:

| Purchasable Site   | Cost |
| ------------------ | ---- |
| Forge              | 100  |
| Transfiguration    | 75   |
| Duplication        | 75   |
| Draft Site         | 100  |
| Dreamsign Offering | 125  |
| Extra Loot Pack    | 75   |
| Essence (200)      | 50   |

The purchased site appears immediately in the dreamscape and can be visited
before the battle. The player can buy multiple sites if they can afford them.

This creates interesting economic tension: spend essence on a direct card
purchase at the shop, or invest in a site that provides a different kind of
value?

**URL:** `?provisionerOptions=3`

Icon: "Compass"

### 5.8 Reward Sites (Navigation agency, fully known in advance)

Reward sites show a specific card or cards on the Dream Atlas preview. The
player knows exactly what they'll get before navigating to that dreamscape.
Full-information acquisition with no randomness.

Reward site contents are generated to be relevant (weighted to player's tides).

Icon: "Treasure Chest"

### 5.9 Acquisition Summary

| Source     | Volume      | Agency      | How it feels               |
| ---------- | ----------- | ----------- | -------------------------- |
| Loot Packs | ~44 cards   | Navigation  | "Exploring and finding"    |
| Card Shop  | ~8 cards    | Full (buy)  | "Shopping for what I need" |
| Pack Shop  | ~8 cards    | Buy + theme | "Investing in a theme"     |
| Ante wins  | ~7-14 cards | Risk/reward | "Claiming a trophy"        |
| Forge      | ~3 cards    | Creative    | "Transmuting"              |
| Draft Site | ~2 cards    | Pick 1 of 4 | "Curated browsing"         |
| Other      | ~5 cards    | Variable    | Journeys, rewards, etc.    |

**Estimated total over a quest:** ~15 (starter) + ~77-84 (acquired) = ~92-99
cards in pool. Ante wins add variance -- a player who accepts and wins every
ante gets more cards; a cautious player who declines or concedes gets fewer but
loses fewer too.

## 6. Ante System

### 6.1 Overview

The ante system unifies battle stakes and card acquisition into a single
mechanism. Both sides wager cards before battle. The winner takes the opponent's
anted cards as trophies. At turn 6, escalation forces a dramatic
commitment-or-fold moment.

### 6.2 Pre-Battle Ante

Before each battle, the opponent **antes a card from their deck**, shown
face-up. The opponent's ante is always desirable -- weighted to be useful for
the player's tides or a strong off-tide splash card.

The player must also ante a card from their pool. They choose which card to
risk. Then the battle begins.

**Outcome:**

- **Win**: The player gains the opponent's anted card (added to pool) and keeps
  their own. This is the primary way battles produce card rewards.
- **Lose**: The player loses their anted card permanently from their pool.

The player may **decline the ante** -- the battle proceeds normally with no
cards at stake and no card reward. This is the safe option for players who can't
afford to lose anything.

### 6.3 Turn 6 Escalation

At the start of **each player's turn 6**, the stakes escalate. Each side antes
an additional card:

- The opponent reveals a second card from their deck as an additional wager.
- The player must choose: **match** (ante another card from their pool) or
  **concede** (forfeit immediately, losing only the original ante).

If both sides match, there are now 2 cards at stake per side. The eventual
winner takes both of the opponent's anted cards.

**Why turn 6:** By turn 6, the board state is developed enough that both sides
have meaningful information about who's winning. The escalation forces a
dramatic decision at a moment when the player can actually evaluate their
position. It creates a natural "second act" -- the early game is about
establishing position, and turn 6 is when you commit or fold.

### 6.4 Concession as Strategy

Conceding at turn 6 is a legitimate strategic choice, not a failure state. If
you're losing badly, conceding costs you 1 card (the original ante) instead of 2
(if you match and lose). Over a full quest, the player who concedes wisely will
have a stronger pool than the player who stubbornly fights every escalation.
This is directly inspired by Marvel Snap where knowing when to retreat is a core
skill.

### 6.5 AI Escalation Behavior

The AI opponent's escalation decision at turn 6 is based on its estimated win
probability. An AI that's ahead will always escalate. An AI that's behind will
sometimes bluff-escalate (configurable probability), creating uncertainty for
the player.

### 6.6 Configurable Parameters

- `<ante_enabled>` (default: **true**): Whether the ante system is active. When
  disabled, battles produce no card rewards.
- `<escalation_turn>` (default: **6**): The turn at which escalation happens.
- `<max_ante_cards>` (default: **2**): Maximum cards each side can ante.

**URL:** `?anteEnabled=true&escalationTurn=6&maxAnteCards=2`

## 7. Dreamscape Sites (Full Roster)

### 7.1 Core Sites

**Battle** -- Play a match against an AI opponent. Unchanged. Icon: "Sword"

**Loot Pack** -- Open a themed pack, all cards to pool. See Section 5.1. Icon:
"Layers"

**Card Shop** -- Buy individual cards. See Section 5.2. Icon: "Store"

**Pack Shop** -- Buy themed packs. See Section 5.3. Icon: "Gift" (or similar,
distinct from Card Shop)

**Dreamcaller Draft** -- Select from `<dreamcaller_choices>` (default: **3**)
dreamcallers. Appears in first dreamscape only. See Section 9.1. Icon: "Crown"

### 7.2 Acquisition Sites

**Draft Site** -- See 4 cards, keep 1. See Section 5.6. Icon: "Rectangle
Vertical"

**Forge** -- Sacrifice cards of one tide to gain a card of another. See Section
5.5. Icon: "Anvil"

**Provisioner** -- Buy additional sites for the current dreamscape. See Section
5.7. Icon: "Compass"

**Reward** -- Known card(s) visible on atlas preview. Unchanged. Icon: "Treasure
Chest"

### 7.3 Dreamsign & Journey Sites

**Dreamsign Offering** -- Gain a single dreamsign (weighted to tides).
Unchanged. Icon: "Sparkles"

**Dreamsign Draft** -- Choose from ~3 dreamsigns. Unchanged. Icon: "Sparkles
Alt"

**Dream Journey** -- Random event with two options. Unchanged. Icon: "Moon +
Star"

**Tempting Offer** -- Benefit + cost paired choices. Unchanged. Icon: "Law"

### 7.4 Refinement Sites

**Transfiguration** -- Modify a card's rules text. Operates on cards in the
deck. Unchanged mechanics. Icon: "Science"

**Duplication** -- Shows 3 random cards from the deck, choose one to duplicate.
Copies go to the pool. Icon: "Copy"

**Essence** -- Gain a fixed amount of essence. Unchanged. Icon: "Diamond"

**Cleanse** -- Remove banes. Unchanged. Icon: "Snowflake"

### 7.5 Removed Sites

**Draft (old)** -- Removed. Replaced by Loot Packs, Card/Pack Shops, and other
paths.

**Purge** -- Removed. Free deck editing replaces this.

**Specialty Emporium** -- Removed. The Card Shop and Pack Shop split covers this
functionality.

### 7.6 Enhanced Sites (Biome Affinities)

| Biome                | Enhanced Effect                                                                        |
| -------------------- | -------------------------------------------------------------------------------------- |
| Verdant Hollow       | **Card Shop**: Free reroll                                                             |
| Starfall Glade       | **Dreamsign Offering**: Becomes dreamsign draft                                        |
| Wanderer's Threshold | **Dream Journey**: 3rd option                                                          |
| The Gilded Maw       | **Tempting Offer**: 3 options instead of 2                                             |
| Ashfall Basin        | **Loot Pack**: Pack contains double cards                                              |
| Crystal Spire        | **Essence**: Amount doubled                                                            |
| Shadowforge          | **Forge**: Player picks the output card from any card in the game matching their tides |
| Hall of Echoes       | **Duplication**: Player picks which card to duplicate                                  |
| The Obsidian Bazaar  | **Pack Shop**: All packs are free                                                      |

## 8. Economy

### 8.1 Essence Sources

| Source                  | Amount                                                        |
| ----------------------- | ------------------------------------------------------------- |
| Starting                | `<starting_essence>` (default: **250**)                       |
| Dreamcaller bonus       | 50-150 (varies by dreamcaller)                                |
| Battle reward (base)    | `<battle_essence>` (default: **150**)                         |
| Battle reward (scaling) | +`<essence_per_level>` (default: **50**) per completion level |
| Essence site            | `<essence_site_amount>` (default: **200**)                    |
| Dream journey effects   | Variable                                                      |
| Ante wins               | Cards, not essence (see Section 6)                            |

**URL:** `?battleEssence=150&essencePerLevel=50&essenceSiteAmount=200`

### 8.2 Essence Sinks

| Sink                       | Cost             |
| -------------------------- | ---------------- |
| Card Shop: card            | 50-100 (random)  |
| Pack Shop: Tide Pack       | 100              |
| Pack Shop: Alliance Pack   | 125              |
| Pack Shop: Mechanical Pack | 125              |
| Card Shop: Reroll          | 40 + 20 per prev |
| Dreamsign                  | 150              |
| Tide Crystal               | 200              |
| Provisioner sites          | 50-125 (varies)  |

### 8.3 Economy Analysis

Over a 7-battle quest, estimated total essence:

- Starting: 250
- Dreamcaller bonus: ~100
- Battle rewards: 150+200+250+300+350+400+450 = 2,100
- Essence sites (~2): 400
- **Total: ~2,850**

Expected spending:

- Card Shop purchases (~8): ~600
- Pack Shop purchases (~3): ~350
- Dreamsigns (~2): ~300
- Tide crystals (~1): ~200
- Rerolls: ~150
- Provisioner: ~200
- Remaining for situational: ~1,050

## 9. Dream Atlas and Dreamscape Generation

### 9.1 Atlas

The Dream Atlas works the same: nexus center, connected dreamscape nodes,
completed/available/unavailable states. Atlas is additive, 2-4 nodes generated
per completion. Player visits 7 dreamscapes.

**Key difference:** The first dreamscape is entered automatically (no atlas
navigation). The atlas opens after the first battle.

Each dreamscape node on the atlas shows:

- **Pack tide icons** for each loot pack site (e.g., two tide-colored dots if
  the dreamscape has two packs from different tides).
- **Site icons** for non-pack, non-battle sites.
- **Reward card previews** if it has a reward site.

### 9.2 Dreamscape Generation (Revised Site Pool)

**All dreamscapes contain at least 1 loot pack site.**

**Completion Level 0 (First dreamscape, automatic):** Fixed: Dreamcaller Draft
(1), Loot Pack (3), Card Shop (1), Battle (1)

**Completion Level 1:** Fixed: Loot Pack (3), Battle (1) From pool: 1-2 from
{Card Shop, Pack Shop, Essence, Dreamsign Offering}

**Completion Level 2:** Fixed: Loot Pack (2), Battle (1) From pool: 2-3 from
{Card Shop, Pack Shop, Draft Site, Essence, Dreamsign Draft, Dream Journey,
Reward}

**Completion Level 3 (Miniboss):** Fixed: Loot Pack (2), Battle (1) From pool:
2-3 from {Card Shop, Pack Shop, Forge, Draft Site, Dream Journey, Tempting
Offer, Essence}

**Completion Level 4:** Fixed: Loot Pack (1), Battle (1) From pool: 3-4 from
{Card Shop, Pack Shop, Forge, Provisioner, Transfiguration, Duplication, Draft
Site, Dream Journey, Tempting Offer, Dreamsign Draft}

**Completion Level 5:** Fixed: Loot Pack (1), Battle (1) From pool: 3-4 from
{Card Shop, Pack Shop, Forge, Provisioner, Transfiguration, Duplication, Dream
Journey, Tempting Offer}

**Completion Level 6 (Final Boss):** Fixed: Loot Pack (1), Battle (1) From pool:
2-3 from {Card Shop, Pack Shop, Forge, Transfiguration, Essence}

### 9.3 Dreamscape Pack Theme Generation

Loot packs within a dreamscape can be from **different tides**. A dreamscape
with 3 pack sites might offer Bloom, Bloom, and Ignite packs, or Arc, Pact, and
Surge packs.

When a dreamscape becomes available, each pack site's theme is rolled
independently using the weights from Section 5.1. The algorithm ensures that
when multiple dreamscapes are available simultaneously, they collectively offer
**variety** in pack themes so navigation is a meaningful choice.

## 10. Dreamcaller and Dreamsign Curation

### 10.1 Dreamcaller Draft

At the Dreamcaller Draft site (first dreamscape), the player sees
`<dreamcaller_choices>` (default: **3**) dreamcallers.

Since the player has no chosen tide (start is random), and they've likely opened
2-3 loot packs before visiting this site, the algorithm counts tides in the
player's current pool and:

- Offers dreamcallers weighted toward the player's pool tides.
- Ensures at least 2 different tides are represented among the choices.
- The dreamcaller choice is the player's **first deliberate strategic
  commitment** -- it grants a permanent tide crystal, which is a strong signal
  of what tide they want to invest in.

**URL:** `?dreamcallerChoices=3`

### 10.2 Dreamsigns

Weighted to the player's dominant pool tides. Unchanged algorithm.

## 11. Banes

Bane cards must be included in the active deck and cannot be removed via the
deck editor. They can only be removed via Cleanse sites or specific dream
journey effects. This preserves the punitive nature of Tempting Offer costs.

## 12. Victory, Defeat, and Limits

**Victory/Defeat:** Unchanged. Win 7 battles to win the quest. 4th is miniboss,
7th is final boss.

**Limits:**

| Limit                | Value                                   |
| -------------------- | --------------------------------------- |
| Minimum deck size    | `<minimum_deck_size>` (default: **25**) |
| Maximum deck size    | `<maximum_deck_size>` (default: **50**) |
| Max copies per card  | `<max_copies>` (default: **2**)         |
| Maximum dreamsigns   | 12                                      |
| Maximum dreamcallers | 1                                       |
| Tide crystal cap     | 3 per tide (during battle)              |
| Card pool size       | No limit                                |

## 13. Full Parameter Table

| Parameter                | Default | URL Parameter          | Description                        |
| ------------------------ | ------- | ---------------------- | ---------------------------------- |
| `revised_tides`          | true    | `revisedTides`         | Use revised tide system            |
| `starting_tides`         | 3       | `startingTides`        | Tides in starting pool             |
| `sequential_tides`       | true    | `sequentialTides`      | Sequential (true) or random tides  |
| `initial_cards`          | 10      | `initialCards`         | Non-neutral cards in starting pool |
| `starter_neutral`        | 5       | `starterNeutral`       | Neutral cards in starter           |
| `starter_low_cost`       | 4       | `starterLowCost`       | Min 0-2 cost cards in starter      |
| `starter_mid_cost`       | 3       | `starterMidCost`       | Min 3-4 cost cards in starter      |
| `starter_high_cost`      | 1       | `starterHighCost`      | Min 5+ cost cards in starter       |
| `starting_essence`       | 250     | `startingEssence`      | Essence at quest start             |
| `loot_pack_size`         | 4       | `lootPackSize`         | Cards per loot pack                |
| `dupe_penalty_2`         | 50      | `dupePenalty2`         | Weight reduction % for 2nd copy    |
| `dupe_penalty_3`         | 90      | `dupePenalty3`         | Weight reduction % for 3rd+ copy   |
| `pack_on_theme_weight`   | 60      | `packOnTheme`          | % weight for on-theme pack tides   |
| `pack_adjacent_weight`   | 25      | `packAdjacent`         | % weight for adjacent pack tides   |
| `pack_explore_weight`    | 15      | `packExplore`          | % weight for off-theme pack tides  |
| `minimum_deck_size`      | 25      | `minimumDeckSize`      | Min cards in deck for battle       |
| `maximum_deck_size`      | 50      | `maximumDeckSize`      | Max cards in deck                  |
| `max_copies`             | 2       | `maxCopies`            | Max copies of one card in deck     |
| `card_shop_size`         | 4       | `cardShopSize`         | Individual cards in Card Shop      |
| `card_price_min`         | 50      | `cardPriceMin`         | Min card price in Card Shop        |
| `card_price_max`         | 100     | `cardPriceMax`         | Max card price in Card Shop        |
| `reroll_base`            | 40      | `rerollBase`           | Base reroll cost                   |
| `reroll_increment`       | 20      | `rerollIncrement`      | Added cost per previous reroll     |
| `pack_shop_size`         | 3       | `packShopSize`         | Packs for sale in Pack Shop        |
| `special_pack_chance`    | 20      | `specialPackChance`    | % of non-tide packs in Pack Shop   |
| `ante_enabled`           | true    | `anteEnabled`          | Whether ante system is active      |
| `escalation_turn`        | 6       | `escalationTurn`       | Turn at which escalation happens   |
| `max_ante_cards`         | 2       | `maxAnteCards`         | Max cards each side can ante       |
| `forge_recipes`          | 3       | `forgeRecipes`         | Offers shown at forge              |
| `forge_cost`             | 4       | `forgeCost`            | Cards sacrificed per forge         |
| `draft_site_total`       | 4       | `draftSiteTotal`       | Cards shown at draft site          |
| `draft_site_keep`        | 1       | `draftSiteKeep`        | Cards kept from draft site         |
| `provisioner_options`    | 3       | `provisionerOptions`   | Site options at provisioner        |
| `dreamcaller_choices`    | 3       | `dreamcallerChoices`   | Dreamcallers at draft              |
| `opponent_preview_cards` | 3       | `opponentPreviewCards` | Opponent cards shown before battle |
| `battle_essence`         | 150     | `battleEssence`        | Base essence per battle win        |
| `essence_per_level`      | 50      | `essencePerLevel`      | Extra essence per completion level |
| `essence_site_amount`    | 200     | `essenceSiteAmount`    | Essence from essence sites         |

## 14. Run Variance Analysis

Five quests with identical starting conditions should diverge because:

01. **Random starting tides**: Different center tide each run.
02. **Dreamscape atlas topology** (random generation): Different paths.
03. **Loot pack contents** (random within theme): Different cards each time.
04. **Loot pack themes** (weighted random per dreamscape): Different tides
    available at each dreamscape.
05. **Dreamscape navigation** (player choice): Different dreamscapes visited.
06. **Card Shop inventories** (random, tide-weighted): Different cards for sale.
07. **Pack Shop themes** (random): Different bulk options.
08. **Battle opponents** (from completion-level pool): Different ante cards.
09. **Dreamcaller offering** (weighted random): Different abilities/crystals.
10. **Forge offers** (pool-dependent): Different transmutation options.
11. **Dream journey outcomes**: Can dramatically reshape decks.
12. **Ante outcomes**: Won/lost cards change the pool.
13. **Deck construction** (player choice): Different players build different
    decks from the same pool.

## Appendix A: Alternative Designs for Playtesting

### A.1 Starting Tide Count Variants

- `startingTides=2`: Narrower start, single alliance. Less initial choice.
- `startingTides=4`: Four adjacent tides, three archetypes. Very broad.
- `startingTides=7`: All tides. Maximum variance, least direction.

**URL:** `?startingTides=2` through `?startingTides=7`

### A.2 Card Trader Site

An NPC offers 3 card-for-card trades: "Give [your card] -> Get [their card]"
from a different tide. Power level roughly matched. May be redundant with Forge.

**URL:** `?traderEnabled=true`

### A.3 Pack-Only Economy

No Card Shop. Card acquisition comes only from loot packs, Pack Shop, trophies,
and forge. Tests whether targeted individual card buying is necessary.

**URL:** `?cardShopSize=0`

### A.4 Smaller Deck Experiment

`minimumDeckSize=15, maximumDeckSize=30`. Every card slot critical.

**URL:** `?minimumDeckSize=15&maximumDeckSize=30`

### A.5 Larger Starting Pool

`initialCards=25, starterNeutral=5`. More cards to choose from immediately.

**URL:** `?initialCards=25`

### A.6 Collection Discovery Pricing

Cards you've never owned this quest cost 50% more in card shops. Rewards
exploration and pack diversity.

**URL:** `?discoveryPricing=true`

### A.7 Archetype Bonus

If your deck contains 15+ cards from a single alliance pair, gain a passive
battle bonus. Rewards focused deckbuilding.

**URL:** `?archetypeBonus=true&archetypeThreshold=15`

### A.8 No Ante (Baseline Comparison)

Disable ante entirely to compare engagement and concession behavior. Battles
produce only essence rewards, no card acquisition.

**URL:** `?anteEnabled=false`

### A.9 Earlier/Later Escalation

Test escalation at different turns to find the sweet spot. Earlier (turn 4)
means less information and more bluffing. Later (turn 8) means more certainty
and less drama.

**URL:** `?escalationTurn=4` or `?escalationTurn=8`
