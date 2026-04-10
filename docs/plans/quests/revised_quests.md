# Dreamtides Quests: Revised Design Document

This is the master design document for the Dreamtides "quests" system. Quests
are the meta layer in which the user navigates various encounters on a map
screen in order to improve their deck, while battles are individual card
matches. Quests are similar to "runs" in other roguelike deckbuilding games,
while battles are similar to "fights". Quests will be at least as complicated to
implement as battles, and almost every existing line of code for supporting
battles will require an equivalent for quests.

This document is the high level "vision" for quests, other documents in this
directory provide more detailed gameplay & technical breakdowns of the feature.
The document at [battle_rules](../../battle_rules/battle_rules.md) provides more
information about the actual rules of the game. See
[Boss Dreamcallers](bosses.md) for boss details and
[Meta Progression](meta_progression.md) for unlock systems.

## The Golden Rule: Configuration via TOML

To the maximum extent possible, Dreamtides gameplay is intended to be completely
configurable via TOML file changes. If a section in this plan says "shops
contain 4 items", this is implied to be configured in TOML. Whenever reasonable,
we should even allow more complex algorithmic changes via data (dreamscape
generation, pack contents, battle rewards, etc). When implementing any rules
engine feature, we should ask the question "could we make this configurable?"

This rule applies to user interface behavior as well as game design: things like
particle effects, sound effects, and animations are always configured in TOML
when possible.

Configurable parameters use `<angle_bracket_names>` with defaults throughout
this document. All parameters are collected in the
[Full Parameter Table](#full-parameter-table) at the end.

## Overview

Quests revolve primarily around building a **collection** of cards and
**constructing a deck** from that collection to bring into future battles. Card
acquisition works like a single-player RPG card game (Shandalar, Thronebreaker,
Balatro) rather than a draft format:

- **Packs are loot.** You open them and get everything inside. The strategic
  choice is *where you go* to get specific packs, not *which cards to pick* from
  a pack.
- **Shops are the agency center.** You browse individual cards and buy what you
  need. A separate pack vendor sells themed packs. This is where essence matters
  most.
- **Deck editing is the strategic core.** The game-within-a-game is constructing
  and tuning the best possible deck from your growing collection. The deck
  editor is always accessible.
- **Ante creates stakes and acquisition.** Both sides wager cards before battle.
  The winner takes the opponent's ante -- this IS card acquisition from battles,
  not a separate system. Mid-battle escalation at turn 6 forces a dramatic
  commitment-or-concession moment.
- **Dreamscape navigation is the macro decision.** You see pack themes, shop
  availability, and reward cards on the atlas before choosing where to go. This
  is where the "what do I pursue" decision lives.

The goal is a mode that feels like constructed Magic -- you know what's
available, you choose what to acquire, and you build around a plan. Variance
comes from what's offered at each dreamscape, shop inventories, battle
opponents, forge recipes, and dream journeys -- not from drafting.

Quests use a currency called "essence" which can be spent on shops and in
various other ways. Players start each quest with `<starting_essence>` (default:
**250**) essence.

In addition to deck cards, users during a quest will select a "dreamcaller" to
lead their deck and may have some number of "dreamsigns":

- **Dreamcaller:** An animated 3D character who starts each battle already in
  play for both participants in a battle. Each dreamcaller has powerful ongoing
  static, triggered, or activated abilities. Each dreamcaller is associated with
  a tide and typically grants 1 permanent tide crystal (see
  [Tide Crystals](#tide-crystals)).
- **Dreamsigns:** Cards with 2D illustrations of objects, which provide more
  minor ongoing effects. Dreamsign effects can apply during battles, on the
  quest map, or both. Generally we try to assign the splashy "build around"
  effects to dreamcallers and secondary effects to dreamsigns. Dreamsigns are
  associated with tides.

Quests display a top-level 3D screen called the [Dream Atlas](#dream-atlas) with
a series of "dreamscapes" the user can navigate to. Each dreamscape is
associated with "sites", specific rewards available in that dreamscape.

Dreamscapes show a group of individual white icons with black circular
backgrounds for their sites. Each site icon corresponds to some specific quest
effect, and users can "visit" a site to activate the effect by clicking on the
icon. This causes the camera to zoom in on that site and then displays the
site's effect, often with a 3D animated NPC character introducing the site's
concept. Once all of the sites in a given dreamscape have been visited, the user
must navigate to the "battle" site to initiate a card battle. After completing a
battle, the user is able to select another dreamscape to navigate to, and the
process repeats.

## Current Quest Prototype

A prototype of client UI patterns for the quest system is available in
client/Assets/Dreamtides/Prototype. This should serve as a starting point for
the implementation, but is by no means definitive and many aspects of its design
are already outdated. This document supersedes all quest prototype decisions.

The prototype demonstrates the basics of the `UpdateQuestCommand` command and
the `QuestView` type. Quests use the same general mechanisms as battles and run
in the same Unity scene. The
[DreamscapeLayout](client/Assets/Dreamtides/Layout/DreamscapeLayout.cs) class is
the entrypoint to many quest-specific Unity components, while
[DreamscapeService](client/Assets/Dreamtides/Services/DreamscapeService.cs) owns
top-level quest functionality for a single dreamscape. Both of these classes
should still be treated as prototype quality despite existing outside of the
`Prototype/` directory.

The [current_prototype.md](current_prototype.md) document is a technical
reference for the current quest prototype implementation, covering Rust types,
client layout and site system, prototype interaction flows, and implementation
gaps. Read when implementing quest features or migrating prototype logic to the
rules engine.

## Tides

Every card, dreamsign, and dreamcaller in Dreamtides is associated with a
**tide**, which represents a deck archetype and philosophical identity. See
[Tides](../../tides/tides.md) for the full tide design including archetype
descriptions and alliances.

The seven core tides are: **Bloom**, **Arc**, **Ignite**, **Pact**, **Umbra**,
**Rime**, and **Surge**. **Wild** is a neutral tide that sits at the center,
compatible with all strategies.

### Revised Tide System

The default tide system for quests is the **revised tide system**
(docs/tides/tides_revised.md) where archetypes live in the overlap between
neighboring tides rather than within single tides. This makes
collection-building more interesting because:

- A pack from a single tide gives you *tools* but not a *complete strategy*.
- Players naturally want cards from 2-3 adjacent tides.
- "A tide is not a deck" -- acquiring from multiple tides is the norm.

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
`rules_engine/tabula/rendered-cards.toml`. The card pool with original tides is
at `rules_engine/tabula/rendered-cards-mono.toml`.

Tides have direct mechanical impact on gameplay through **tide crystals** -- a
resource required to play cards during battles. See
[Tide Crystals](#tide-crystals) for details.

## Starting a Quest

### No Starting Choices

The quest begins immediately with no player decisions. A center tide and
starting dreamscape are selected at random, and the player receives starter
cards. This avoids the analysis paralysis of evaluating alliance options before
you've even seen a card.

The player's first meaningful choices happen *inside* the first dreamscape:
which sites to visit, which cards to put in their deck, and which dreamcaller to
select. These decisions are made with cards in hand rather than in the abstract.

### Starting Tide Selection

A random center tide is chosen. The player receives cards from that tide plus
its two neighbors on the tide circle, giving `<starting_tides>` (default: **3**)
tides total.

For example, if Arc is selected: cards come from Bloom, Arc, and Ignite. This
covers two overlapping archetypes (Ramp and Flicker), giving the player two
viable directions to specialize into.

Whether the 3 starting tides are **sequential** (center + neighbors) or **fully
random** is configurable. Sequential gives coherent synergy from the start;
random creates more chaotic openings that require creative deckbuilding.

### Receive Starter Cards

The player receives `<initial_cards>` (default: **10**) cards drawn equally from
all cards in their starting tides, plus `<starter_neutral>` (default: **5**)
Neutral cards for removal and utility. Total starting pool: 15 cards.

All cards are drawn with equal probability regardless of rarity. The only
guarantee is a playable energy curve: at least `<starter_low_cost>` (default:
**4**) cards costing 0-2, at least `<starter_mid_cost>` (default: **3**) costing
3-4, and at least `<starter_high_cost>` (default: **1**) costing 5+.

All cards go into the **card pool**. The player's deck starts as a copy of the
full pool.

### Starting Resources

- `<starting_essence>` (default: **250**) essence.
- No dreamcaller (selected during first dreamscape).
- No dreamsigns.

### First Dreamscape

The first dreamscape is **automatically entered** -- the player does not choose
from the atlas. It always contains:

- Dreamcaller Draft (1)
- Loot Pack (3, from the player's starting tides)
- Card Shop (1)
- Battle (1)

This ensures the player's first experience is: open packs, browse a shop, pick a
dreamcaller, edit their deck, fight. No atlas navigation decisions until after
the first battle.

After completing the first dreamscape, the Dream Atlas opens and the player
begins making navigation choices with a real collection to reason about.

## Card Pool and Deck

### The Card Pool

The **card pool** is every card the player has acquired this quest. It grows
throughout the quest and has no size limit. Cards enter the pool from: starter
cards, loot packs, shop purchases, battle trophies (ante), forge output, dream
journeys, reward sites, and duplication.

### The Deck

The **deck** is a subset of the pool that goes into battle.

- Minimum size: `<minimum_deck_size>` (default: **25**)
- Maximum size: `<maximum_deck_size>` (default: **50**)
- Max copies of any card: `<max_copies>` (default: **2**)

### Deck Editor

Accessible anytime: from the Dream Atlas, before battles, or by clicking the
deck icon during a dreamscape. Features:

- **Split view**: Pool on one side, deck on the other. Click to move cards
  between them.
- **Filters**: By tide, energy cost, card type, keyword.
- **Deck stats**: Tide distribution, energy curve histogram, card type
  breakdown.
- **Deck size indicator**: Current size and min/max boundaries.

### Pre-Battle Sideboarding

Before each battle, after all other sites are visited, the player sees:

- The opponent's **dreamcaller** and `<opponent_preview_cards>` (default: **3**)
  representative cards from the opponent's deck. Enough to read their strategy
  without information overload.
- Their current deck with access to the full pool for final adjustments.

## Card Acquisition Paths

### Loot Packs (Primary volume, no card-level choice)

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

**All dreamscapes contain at least 1 loot pack site.** Pack sites per completion
level:

| Completion Level | Loot Pack Sites |
| ---------------- | --------------- |
| 0, 1             | 3               |
| 2, 3             | 2               |
| 4, 5             | 1               |
| 6                | 1               |

**UI:** The cards available in the pack are shown in the 3D scene. The pack of
cards animates open and the cards fan out to be displayed. All cards then
animate to the quest deck/pool area. Cards are shown with an orange outline.

Icon: "Layers"

### Card Shop (Targeted individual card acquisition)

The Card Shop sells `<card_shop_size>` (default: **4**) individual cards,
browsable face-up with full rules text. Weighted toward the player's dominant
pool tides but including some adjacent-tide and Neutral options.

**Pricing:** Each card has a randomized price drawn from a range:
`<card_price_min>` (default: **50**) to `<card_price_max>` (default: **100**).
Random discounts: 1-2 cards may be on sale (30-70% off). Prices are in
increments of 5.

**Reroll:** Refreshes the card selection. Costs `<reroll_base>` (default:
**40**) + `<reroll_increment>` (default: **20**) per previous reroll this visit.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. Items are displayed
beside the NPC in landscape mode and below the NPC in portrait mode. Each item
has a purple button under it showing the essence cost to purchase. Clicking the
button for a card animates it to the quest deck or pool display. The other items
do not move on purchase, leaving a gap. One of the items shown may be a "reroll"
option. When this is selected, the items do a staggered scale-down animation,
then the new options perform a scale-up animation in-place. Clicking the close
button completes the site visit and pulls the camera back to the map screen.

Icon: "Store"

### Pack Shop (Bulk themed acquisition)

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

**UI:** Similar to the Card Shop, but items are displayed as pack icons with
tide indicators. An NPC is present. Clicking a pack purchase button opens the
pack and animates its contents to the player's pool.

Icon: "Gift"

### Ante (Card acquisition through battle)

The ante system is the primary way battles produce card rewards. Before each
battle, both sides wager cards. The winner takes the opponent's anted cards.

Ante serves as both acquisition and tension:

- **Splash cards**: Opponents wager cards from their tides, giving access to
  off-tide cards the player wouldn't normally encounter.
- **Stakes**: Every battle has something meaningful at risk beyond just quest
  progress.
- **Concession strategy**: The mid-battle escalation at turn 6 creates a
  dramatic commitment-or-fold moment.

See [Ante System](#ante-system) for the full design.

### Forge (Mid-late game, transmutation)

The Forge lets the player sacrifice cards from their pool to create a specific
new card. The design is deliberately simple:

**How it works:** The Forge shows `<forge_recipes>` (default: **3**) offers.
Each offer is:

> "Sacrifice `<forge_cost>` (default: **4**) [Tide] cards -> Gain \[specific
> card from a different tide\]"

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
"Sacrifice 4 Bloom cards -> Gain [specific Arc card]."

The player may decline all offers.

**UI:** An NPC is shown. The forge offers are displayed as recipe cards showing
the sacrifice cost on the left and the output card on the right. Clicking an
offer animates the sacrifice cards out of the pool and plays a
forge/transmutation effect, then the output card animates into the pool.

Icon: "Anvil"

### Draft Site (Occasional, small draft element)

A minor site type providing a small draft-like selection. The player sees
`<draft_site_total>` (default: **4**) cards and keeps `<draft_site_keep>`
(default: **1**). The rest are discarded.

This site appears in the site pool starting at completion level 2. It provides a
targeted acquisition moment that feels different from shops (curated set, no
essence cost) and different from loot packs (card-level agency). It's one site
among many, not the core loop.

Cards shown are weighted to the player's tides.

**UI:** The cards available for the current pick are shown in multiple rows. The
available cards animate in to be selected. Clicking a card animates it to the
quest deck, and the remaining cards animate away. Cards are shown with an orange
outline.

Icon: "Rectangle Vertical"

### Provisioner (Buy sites for the current dreamscape)

The Provisioner is a site where the player can spend essence to **add a site to
the current dreamscape**. The Provisioner shows `<provisioner_options>`
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

**UI:** An NPC is shown. Available site options are displayed as icons with
purple purchase buttons showing the cost. Purchasing a site causes it to animate
onto the dreamscape map as a new visitable icon.

Icon: "Compass"

### Reward Sites (Navigation agency, fully known in advance)

Reward sites show a specific card or cards on the Dream Atlas preview. The
player knows exactly what they'll get before navigating to that dreamscape.
Full-information acquisition with no randomness.

Reward site contents are generated to be relevant (weighted to player's tides).

**UI:** The camera pulls in on a scene showing the reward items in question,
with a purple "accept" button and a gray "decline" button. Accepting the reward
plays the standard animation for that item type, for example animating to the
quest deck, and then the camera pulls back to the map screen.

Icon: "Treasure Chest"

### Acquisition Summary

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

## Ante System

### Overview

The ante system unifies battle stakes and card acquisition into a single
mechanism. Both sides wager cards before battle. The winner takes the opponent's
anted cards as trophies. At turn 6, escalation forces a dramatic
commitment-or-fold moment.

### Pre-Battle Ante

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

### Turn 6 Escalation

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

### Concession as Strategy

Conceding at turn 6 is a legitimate strategic choice, not a failure state. If
you're losing badly, conceding costs you 1 card (the original ante) instead of 2
(if you match and lose). Over a full quest, the player who concedes wisely will
have a stronger pool than the player who stubbornly fights every escalation.
This is directly inspired by Marvel Snap where knowing when to retreat is a core
skill.

### AI Escalation Behavior

The AI opponent's escalation decision at turn 6 is based on its estimated win
probability. An AI that's ahead will always escalate. An AI that's behind will
sometimes bluff-escalate (configurable probability), creating uncertainty for
the player.

### Ante Parameters

- `<ante_enabled>` (default: **true**): Whether the ante system is active. When
  disabled, battles produce no card rewards.
- `<escalation_turn>` (default: **6**): The turn at which escalation happens.
- `<max_ante_cards>` (default: **2**): Maximum cards each side can ante.

## Dreamscape Sites (Full Roster)

### Core Sites

**Battle** -- The Battle site is the core gameplay element of Dreamtides, and it
allows users to play a match against an AI opponent. Each battle has an assigned
opponent dreamcaller with their own deck. Opponent decks are (for now) defined
statically in TOML. Before the battle begins, the opposing dreamcaller is
displayed so the user can understand any special abilities they have. Opposing
dreamsigns are also shown. When the battle completes, the
[Victory or Defeat](#victory--defeat) screen is shown along with any associated
battle rewards.

**UI:** The camera pans in to the battle scene. The "full body" card
representation of the enemy dreamcaller animates in from a small size at the
center of the battle area. The enemy's deck is present in the center of the
scene. The dreamcaller character within the card performs a humanoid animation.
The rules text on the enemy dreamcaller is displayed, along with any enemy
dreamsigns. A "start battle" button is shown. Clicking the start battle button
causes the enemy dreamcaller to animate to their battle position in the small
dreamcaller card format (head only, no text). The user dreamcaller and user
quest deck animate to their starting positions. The enemy quest deck animates to
its starting position. An opening hand of cards is dealt to both players.

Icon: "Sword"

**Loot Pack** -- Open a themed pack, all cards to pool. See
[Loot Packs](#loot-packs-primary-volume-no-card-level-choice). Icon: "Layers"

**Card Shop** -- Buy individual cards. See
[Card Shop](#card-shop-targeted-individual-card-acquisition). Icon: "Store"

**Pack Shop** -- Buy themed packs. See
[Pack Shop](#pack-shop-bulk-themed-acquisition). Icon: "Gift"

**Dreamcaller Draft** -- At the Dreamcaller Draft site (first dreamscape), the
player sees `<dreamcaller_choices>` (default: **3**) dreamcallers. Since the
player has no chosen tide (start is random), and they've likely opened 2-3 loot
packs before visiting this site, the algorithm counts tides in the player's
current pool and:

- Offers dreamcallers weighted toward the player's pool tides.
- Ensures at least 2 different tides are represented among the choices.
- The dreamcaller choice is the player's **first deliberate strategic
  commitment** -- it grants a permanent tide crystal, which is a strong signal
  of what tide they want to invest in.

Each dreamcaller is associated with a **tide** and grants **1 permanent tide
crystal** of that tide, which the player starts each battle with.

Each dreamcaller comes with a different **essence bonus** gained for selecting
that option, which serves as a lever for balancing more powerful dreamcallers.
Bonus amounts are configured in TOML.

**UI:** Dreamcallers are shown in their full-body "card" representation, with
ability text displayed alongside their 3D models and essence bonuses. The
dreamcaller cards animate in from a small size in the center of the screen. Each
dreamcaller does a different humanoid animation within its card frame. A primary
action button appears below each dreamcaller allowing them to be selected. The
selected dreamcaller animates to the bottom left of the screen to appear in a
"square" frame (head only). The other cards animate back to a small size.

Icon: "Crown"

### Acquisition Sites

**Draft Site** -- See 4 cards, keep 1. See
[Draft Site](#draft-site-occasional-small-draft-element). Icon: "Rectangle
Vertical"

**Forge** -- Sacrifice cards of one tide to gain a card of another. See
[Forge](#forge-mid-late-game-transmutation). Icon: "Anvil"

**Provisioner** -- Buy additional sites for the current dreamscape. See
[Provisioner](#provisioner-buy-sites-for-the-current-dreamscape). Icon:
"Compass"

**Reward** -- Known card(s) visible on atlas preview. See
[Reward Sites](#reward-sites-navigation-agency-fully-known-in-advance). Icon:
"Treasure Chest"

### Dreamsign & Journey Sites

**Dreamsign Offering** -- At a dreamsign offering site, the user is presented
with a single dreamsign to gain. The offering may be rejected, but there is no
reward for doing so. Dreamsigns are associated with tides. The offered dreamsign
is **weighted to match the player's tides** -- dreamsigns matching the player's
dominant deck tides are more likely to appear.

**UI:** The dreamsign animates to be displayed from screen center at a small
scale. A purple accept button and a gray reject button are displayed. The
dreamsign animates to the bottom right dreamsign display if accepted and
animates back to a small scale if rejected.

Icon: "Sparkles"

**Dreamsign Draft** -- At a dreamsign draft site, the user is presented with
around three dreamsigns and is able to select one to gain. It is again possible
to select no dreamsign. The presented dreamsigns are **weighted to match the
player's tides**.

**UI:** The three dreamsigns animate in at full size from the bottom of the
screen in a staggered animation, positioning themselves in a single row. Purple
accept buttons are shown below each one. A red close button is shown top left.
Accepting a dreamsign animates it to the user's dreamsign display area in the
bottom right of the screen.

Icon: "Sparkles Alt"

**Dream Journey** -- A dream journey functions in a manner similar to a random
event in other roguelike deckbuilding games. The user is offered a selection
between two circular cards with unique art. Each card has a description,
although the amount of information revealed about the effects is variable, and
some dream journeys have highly random effects which are not disclosed in
advance. This is where we put the biggest random effects which can structurally
change a quest or modify the user's entire deck. A close button is displayed
allowing the user to reject the dream journey options.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. The journey cards animate
from the center of the NPC's chest at a small size and are shown side-by-side
(next to the NPC landscape, below in portrait). A purple button is displayed
under each journey card to accept it. Clicking this button causes the
not-selected journey card to animate down to a small size and vanish. The
accepted journey card animates up to appear in screen center, then plays a
dissolve animation. The effects of the journey are shown via a custom animation.
Once the effect animation completes, the camera pulls back to the map screen. A
dream journey is a circular card image which displays its rules text on
hover/long press.

Icon: "Moon + Star"

**Tempting Offer** -- A tempting offer is a site where the user is faced with a
pair of dream journey options with positive effects. This time, however, each
dream journey is also associated with a 'cost' card with its own card and
description, showing some price to be paid to unlock the journey effect. The
user may select an option to pay its cost and receive the benefit. A close
button is displayed allowing the user to reject the options.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. The journey/cost card
pairs animate out from the center of the NPC's chest at a small scale in a
staggered animation. The cards are displayed in two rows, with the journey card
on the left side of the row and the cost card on the right side of the row, and
with a purple button displayed under each pair to select that option. Picking an
option performs the same resolution animation as above, with the journey card
first animating to a large size in the center of the screen, dissolving, and
playing a custom effect animation, then the cost card animating to screen center
and playing its custom animation. Journey and Cost cards will often have
associated sound effects and particle effects for their abilities.

Icon: "Law"

### Refinement Sites

**Transfiguration** -- A transfiguration site shows the user 3 random cards from
their deck, and they may select one to apply a transfiguration to, modifying
that card's rules text. Each card can only receive a single transfiguration;
cards that have already been transfigured are not eligible. If multiple
transfigurations are applicable to a card, a random one is selected to suggest.

Transfigurations are named after colors, and cause the card name and any
modified rules text to display in a different color to indicate the
transfiguration. Possible transfigurations include:

- Viridian Transfiguration: Reduces the energy cost of the card by 50%, rounded
  to the nearest whole number (4->2, 3->2, 2->1, 1->0, etc). Not available for
  cards which cost 0.
- Golden Transfiguration: Improves the effect of the card by increasing or
  decreasing a number in its rules text by 1. Only available for cards with
  numbers in their text. The golden variant of each card is defined in TOML.
- Scarlet Transfiguration: Doubles the base spark of a character, or sets it to
  1 for characters with 0 spark. Only available for characters.
- Magenta Transfiguration: Increases the frequency of named card triggers,
  changing:
  - A "materialized" trigger to also happen when the card dissolves
  - A "judgment" trigger to also happen when the card is materialized
  - A "once per turn" trigger to happen any number of times per turn
- Azure Transfiguration: Appends "draw a card" to the text of an event card.
  Only available for events.
- Bronze Transfiguration: Adds "reclaim" to the text of an event card. Only
  available for events.
- Rose Transfiguration: Reduces the cost of an activated ability by 1. Only
  available for cards with activated abilities that cost energy.
- Prismatic Transfiguration: Adds all of the above transfigurations to a card
  which are available. Only available for cards which are eligible for 2 or more
  transfigurations.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. 3 cards from the quest
deck animate to appear in a row via a staggered move animation (they flip to be
face-up). As with other sites, they appear beside the NPC in landscape and below
the NPC in portrait. Each card is augmented to show the transfigured version
being offered, with the card name and card text tinted to the new color. Each
card gets a purple "Transfigure" button to accept that transfiguration. When
clicked the other cards fall away, and then the selected card animates to screen
center and displays a visual effect specific to the transfiguration being
applied, then flips over and returns to the quest deck in the bottom right of
the screen. A close button is displayed to allow the user to decline a
transfiguration.

Icon: "Science"

**Duplication** -- A duplication site shows the user 3 random cards from their
deck along with a proposed random number of copies to create for each card
between 1 and 4. The user may pick one of the proposed options to add that many
duplicates of that card to their pool.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. 3 cards from the quest
deck animate to appear in a row via a staggered move animation. A purple button
like "Duplicate x3" appears under each one. Clicking this button causes the
other cards to fall away, and then a particle effect plays and additional copies
of the card emerge from the selected card. All copies then animate to the quest
deck, and the camera pulls back to the map screen. A close button is displayed
to allow the user to decline duplication.

Icon: "Copy"

**Essence** -- An essence site grants the user a fixed amount of essence, often
around `<essence_site_amount>` (default: **200**).

**UI:** Unlike with other sites, the camera does not zoom in to essence sites.
Instead the button simply vanishes on click and a purple particle effect
appears, animating in a winding path to the user's essence total and then plays
a 'hit' particle effect when it reaches the bottom left essence total and
updates the quantity of essence shown.

Icon: "Diamond"

**Cleanse** -- A Cleanse site allows the user to remove up to 3 random
[Banes](#banes) from their deck or dreamsigns.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. The randomly selected
cards or dreamsigns to cleanse emerge from the quest deck or dreamsign display.
A purple "cleanse" button is displayed, along with a gray "decline" button.
Selecting "cleanse" causes the bane cards to play a dissolve animation, and then
the camera pulls back to the map screen.

Icon: "Snowflake"

### Removed Sites (vs. Original Design)

**Cube Draft** -- Removed. The original cube draft system (10-person simulated
draft table with AI bot drafters) has been replaced by loot packs, card/pack
shops, and other acquisition paths.

**Purge** -- Removed. Free deck editing via the always-accessible deck editor
replaces the need for a dedicated purge site.

**Specialty Shop** -- Removed. The Card Shop and Pack Shop split covers this
functionality.

### Enhanced Sites (Biome Affinities)

Each dreamscape is associated with a specific "biome" which dictates the 3D
environment assets used in generation. Biomes are purely visual aside from their
enhanced site affinity. There is one biome per enhanced site type, and biome
configuration and assignment are defined in TOML. Each dreamscape biome has an
affinity for a specific site, and produces an "enhanced site" of that type when
visited:

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

## Victory & Defeat

Initially, a Quest ends in defeat if the user loses a battle. As described in
the [Meta Progression](meta_progression.md) document, the user eventually
unlocks the ability to continue in a quest after a first loss.

**UI:** When a battle ends, a particle effect plays alongside a sound effect,
and the word "Victory" or "Defeat" is displayed at screen center. The text then
animates upward to reveal a summary panel showing battle rewards earned, quest
statistics, and a button to continue to the Dream Atlas (on victory) or to end
the quest (on defeat).

A Quest ends in victory if the user wins 7 battles. The 4th battle they face is
against a miniboss, and the 7th battle is against the final boss of Dreamtides.
Bosses are dreamcallers that have their own unique abilities, dreamsigns, or
custom cards in their decks. See [Boss Dreamcallers](bosses.md) for details.

### Battle Rewards

Completing a battle always grants an essence reward. Base reward is
`<battle_essence>` (default: **150**) plus `<essence_per_level>` (default:
**50**) per completion level, so rewards increase as the user completes more
dreamscapes.

The primary card reward from battles comes through the
[Ante System](#ante-system) -- the winner takes the opponent's anted cards as
trophies.

## Limits

| Limit                | Value                                   |
| -------------------- | --------------------------------------- |
| Minimum deck size    | `<minimum_deck_size>` (default: **25**) |
| Maximum deck size    | `<maximum_deck_size>` (default: **50**) |
| Max copies per card  | `<max_copies>` (default: **2**)         |
| Maximum dreamsigns   | 12                                      |
| Maximum dreamcallers | 1                                       |
| Tide crystal cap     | 3 per tide (during battle)              |
| Card pool size       | No limit                                |

If the deck size limits are violated before battle, the player must adjust their
deck via the deck editor before proceeding.

Users can have a maximum of 12 dreamsigns at any time. If they would receive
another dreamsign, an overlay is shown and they must immediately purge a
dreamsign.

## Banes

Certain cards and dreamsigns, called "banes", can be given to the user during a
quest, typically as a result of a [Tempting Offer](#tempting-offer) choice. Bane
cards generally have negative effects when drawn, while bane dreamsigns provide
ongoing negative effects on the quest.

Bane cards must be included in the active deck and cannot be removed via the
deck editor. They can only be removed via [Cleanse](#refinement-sites) sites or
specific dream journey effects. This preserves the punitive nature of Tempting
Offer costs.

## Dream Atlas

The Dream Atlas is the screen players see after completing the first dreamscape.
It shows a 3D map of dreamscapes represented as circular miniature "worlds,"
connected by dotted lines. The player can hover over or long-press a dreamscape
to preview its biome and available sites, then click it again to zoom the camera
in to that dreamscape.

Each dreamscape can be in one of three states:

- **Completed**: The player has already visited this dreamscape and finished its
  battle.
- **Available**: The player can choose this dreamscape as their next
  destination.
- **Unavailable**: The player cannot choose this dreamscape yet.

The player begins at the center of the Dream Atlas, called the **Nexus**. At the
start, the first dreamscape is entered automatically -- no atlas navigation.
After the first battle, any dreamscapes connected to the Nexus are
**Available**.

After the player visits a dreamscape and completes its battle, that dreamscape
becomes **Completed**. Any dreamscapes directly connected to it then also become
**Available**. The number of dreamscapes the user has completed is called the
'Completion Level' for that quest. In other words, a dreamscape is **Available**
only if it is connected to the Nexus or to at least one **Completed**
dreamscape.

Each dreamscape node on the atlas shows:

- **Pack tide icons** for each loot pack site (e.g., two tide-colored dots if
  the dreamscape has two packs from different tides).
- **Site icons** for non-pack, non-battle sites.
- **Reward card previews** if it has a reward site.

This allows the user to make an informed decision about which dreamscape to
visit next. Winning the 7th battle causes the player to win the quest.

### Dream Atlas Generation

The dream atlas is generated dynamically throughout the quest, with new
dreamscapes being added as dreamscapes are completed. The new dreamscapes are
added as 'unavailable' nodes adjacent to the newly 'available' nodes. Around 2-4
nodes are randomly generated and placed in this manner each time a dreamscape is
completed, creating a web of interconnected nodes. The atlas is purely additive
and is never pruned; the player will visit 7 dreamscapes in a typical quest (or
8 with the battle-skip meta progression unlock). Initial atlas topology is
configured in TOML.

## Dreamscape Generation

Dreamscapes are generated by drawing sites from a pool. Sites are selected when
the dreamscape becomes available. The pool for site generation changes over
time, with new options being shuffled in after each dreamscape is completed.
Each completed dreamscape shuffles in a new set of sites as defined in TOML for
that completion level. All sites can appear a maximum of 1 time in a dreamscape,
with the exception that there can be up to 3 Loot Pack sites and up to 2 Essence
sites.

**All dreamscapes contain at least 1 loot pack site.** Loot pack sites are
handled with deterministic counts based on completion level (see
[Loot Packs](#loot-packs-primary-volume-no-card-level-choice)).

Battle sites are also distinct: Dreamscapes have one Battle site, or zero if
this has been modified by [meta progression](meta_progression.md). The opponent
dreamcaller, dreamsigns, and deck for the battle is selected from a pool of
opponents defined in TOML for a given completion level. Difficulty scaling is
configured in TOML.

The Dreamcaller Draft site is distinct and always appears in the first
dreamscape visited, and only in that dreamscape.

### Dreamscape Composition by Completion Level

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

### Dreamscape Pack Theme Generation

Loot packs within a dreamscape can be from **different tides**. A dreamscape
with 3 pack sites might offer Bloom, Bloom, and Ignite packs, or Arc, Pact, and
Surge packs.

When a dreamscape becomes available, each pack site's theme is rolled
independently using the pack theme generation weights. The algorithm ensures
that when multiple dreamscapes are available simultaneously, they collectively
offer **variety** in pack themes so navigation is a meaningful choice.

## Tide Crystals

Tide crystals are the resource system that governs which cards a player can play
during a battle. Each card has a **tide cost** indicating how many crystals of
its tide are required to play it:

- Most cards cost **1** tide crystal of their tide.
- Cards that heavily commit to a specific archetype may cost **2 or 3** crystals
  of their tide.
- **Wild** cards cost **0** tide crystals and can always be played regardless of
  crystal state.
- No cards currently require crystals from multiple different tides.

### Crystal Pool Generation

Before each battle begins, a **crystal pool** is assembled based on the
composition of the player's deck. The algorithm functions like a skilled Magic:
the Gathering deck builder designing a mana base -- it asks "what distribution
of tide crystals would be most likely to let this player play their cards on
curve?" The crystal pool is a fixed list of approximately 30 crystals.

The design goal is that **mono-tide decks can be played without any thought
about crystals**, while **splashing additional tides carries added cost** and
requires deliberate investment. Players who want to play cards from multiple
tides must plan accordingly.

### Crystal Accumulation During Battle

During each **Dreamwell phase** of a battle, the player receives 1 random
crystal drawn from their crystal pool. Crystals accumulate over the course of
the battle, with a **cap of 3 crystals per tide**. If the crystal pool is
somehow exhausted, a randomized second copy is generated.

### Acquiring Additional Crystals

Players have several ways to improve their tide crystal situation beyond the
automatic Dreamwell phase allocation:

- **Dreamcallers** grant 1 permanent tide crystal of their associated tide. The
  player starts each battle already having this crystal in play.
- **Card Shops** sell the ability to gain a tide crystal in exchange for
  essence, allowing the player to start battles with a pre-purchased crystal in
  play. This is a key tool for enabling multi-tide decks.
- **Cards** -- certain cards will be designed that generate tide crystals as
  part of their effects, allowing players to fix their tide pool and play cards
  from multiple tides.

## Economy

### Essence Sources

| Source                  | Amount                                                        |
| ----------------------- | ------------------------------------------------------------- |
| Starting                | `<starting_essence>` (default: **250**)                       |
| Dreamcaller bonus       | 50-150 (varies by dreamcaller)                                |
| Battle reward (base)    | `<battle_essence>` (default: **150**)                         |
| Battle reward (scaling) | +`<essence_per_level>` (default: **50**) per completion level |
| Essence site            | `<essence_site_amount>` (default: **200**)                    |
| Dream journey effects   | Variable                                                      |
| Ante wins               | Cards, not essence (see [Ante System](#ante-system))          |

### Essence Sinks

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

### Economy Analysis

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

## NPC Sites

Many sites are associated with an NPC, a 3D humanoid character that can play
character animations and show a speech bubble. This NPC is always the same for a
given site (e.g. all Card Shops have the same NPC), and their behavior and
dialog are configured via TOML. For sites with an NPC, portrait mode frames the
NPC at the top of the screen with content below, while landscape mode places the
NPC to one side with content beside them.

## Implementation Strategy and QA

The overall implementation strategy for the Quests game mode is to rely heavily
on both *integration testing* and *manual QA*. The integration testing
philosophy should follow what we use for the battle game mode, writing tests
that operate against the real QuestView/Commands interface. Philosophically,
Dreamtides does not employ unit testing.

The manual QA strategy here is based on validating all changes against a running
instance of the Unity editor using the [abu](../../abu/abu.md) tool. *Every*
change to the Quest game mode should interact with Unity, perform the required
user interactions, and take screenshots of the new UI to check for display
issues. Testing *must* be at minimum performed once on a landscape/desktop
display resolution and once on a mobile/portrait display resolution. The device
can be configured before entering play mode via the `abu set-device` command:
`abu set-device landscape-16x10` or `abu set-device iphone-se`. We should be
interactively building a high-quality `DreamtidesSceneWalker.Quest.cs` scene
`abu` representation during development.

## Run Variance Analysis

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

## Full Parameter Table

| Parameter                | Default | Description                        |
| ------------------------ | ------- | ---------------------------------- |
| `revised_tides`          | true    | Use revised tide system            |
| `starting_tides`         | 3       | Tides in starting pool             |
| `sequential_tides`       | true    | Sequential (true) or random tides  |
| `initial_cards`          | 10      | Non-neutral cards in starting pool |
| `starter_neutral`        | 5       | Neutral cards in starter           |
| `starter_low_cost`       | 4       | Min 0-2 cost cards in starter      |
| `starter_mid_cost`       | 3       | Min 3-4 cost cards in starter      |
| `starter_high_cost`      | 1       | Min 5+ cost cards in starter       |
| `starting_essence`       | 250     | Essence at quest start             |
| `loot_pack_size`         | 4       | Cards per loot pack                |
| `dupe_penalty_2`         | 50      | Weight reduction % for 2nd copy    |
| `dupe_penalty_3`         | 90      | Weight reduction % for 3rd+ copy   |
| `pack_on_theme_weight`   | 60      | % weight for on-theme pack tides   |
| `pack_adjacent_weight`   | 25      | % weight for adjacent pack tides   |
| `pack_explore_weight`    | 15      | % weight for off-theme pack tides  |
| `minimum_deck_size`      | 25      | Min cards in deck for battle       |
| `maximum_deck_size`      | 50      | Max cards in deck                  |
| `max_copies`             | 2       | Max copies of one card in deck     |
| `card_shop_size`         | 4       | Individual cards in Card Shop      |
| `card_price_min`         | 50      | Min card price in Card Shop        |
| `card_price_max`         | 100     | Max card price in Card Shop        |
| `reroll_base`            | 40      | Base reroll cost                   |
| `reroll_increment`       | 20      | Added cost per previous reroll     |
| `pack_shop_size`         | 3       | Packs for sale in Pack Shop        |
| `special_pack_chance`    | 20      | % of non-tide packs in Pack Shop   |
| `ante_enabled`           | true    | Whether ante system is active      |
| `escalation_turn`        | 6       | Turn at which escalation happens   |
| `max_ante_cards`         | 2       | Max cards each side can ante       |
| `forge_recipes`          | 3       | Offers shown at forge              |
| `forge_cost`             | 4       | Cards sacrificed per forge         |
| `draft_site_total`       | 4       | Cards shown at draft site          |
| `draft_site_keep`        | 1       | Cards kept from draft site         |
| `provisioner_options`    | 3       | Site options at provisioner        |
| `dreamcaller_choices`    | 3       | Dreamcallers at draft              |
| `opponent_preview_cards` | 3       | Opponent cards shown before battle |
| `battle_essence`         | 150     | Base essence per battle win        |
| `essence_per_level`      | 50      | Extra essence per completion level |
| `essence_site_amount`    | 200     | Essence from essence sites         |

## Appendix A: Alternative Designs for Playtesting

### A.1 Starting Tide Count Variants

- `startingTides=2`: Narrower start, single alliance. Less initial choice.
- `startingTides=4`: Four adjacent tides, three archetypes. Very broad.
- `startingTides=7`: All tides. Maximum variance, least direction.

### A.2 Card Trader Site

An NPC offers 3 card-for-card trades: "Give [your card] -> Get [their card]"
from a different tide. Power level roughly matched. May be redundant with Forge.

### A.3 Pack-Only Economy

No Card Shop. Card acquisition comes only from loot packs, Pack Shop, trophies,
and forge. Tests whether targeted individual card buying is necessary.

### A.4 Smaller Deck Experiment

`minimumDeckSize=15, maximumDeckSize=30`. Every card slot critical.

### A.5 Larger Starting Pool

`initialCards=25, starterNeutral=5`. More cards to choose from immediately.

### A.6 Collection Discovery Pricing

Cards you've never owned this quest cost 50% more in card shops. Rewards
exploration and pack diversity.

### A.7 Archetype Bonus

If your deck contains 15+ cards from a single alliance pair, gain a passive
battle bonus. Rewards focused deckbuilding.

### A.8 No Ante (Baseline Comparison)

Disable ante entirely to compare engagement and concession behavior. Battles
produce only essence rewards, no card acquisition.

### A.9 Earlier/Later Escalation

Test escalation at different turns to find the sweet spot. Earlier (turn 4)
means less information and more bluffing. Later (turn 8) means more certainty
and less drama.
