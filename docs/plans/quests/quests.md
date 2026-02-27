# Dreamtides Quests: Master Design Document

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
information about the actual rules of the game.

## The Golden Rule: Configuration via TOML

The rest of this document goes into detail about specific game systems. To the
maximum extent possible, though, Dreamtides gameplay is intended to be
completely configurable via TOML file changes. If a section in the plan says
"shops contain 6 items", this is implied to be configured in TOML. Whenever
reasonable, we should even allow more complex algorithmic changes via data
(dreamscape generation, draft pool rules, battle rewards, etc). When
implementing any rules engine feature, we should ask the question "could we make
this configurable?"

This rule applies to user interface behavior as well as game design: things like
particle effects, sound effects, and animations are always configured in TOML
when possible.

## Overview

Quests revolve primarily around drafting a deck to bring into future battles.
Quests use a currency called "essence" which can be spent on shops and in
various other ways. Players start each quest with 250 essence and no cards in
their deck.

In addition to deck cards, users during a quest will select a "dreamcaller" to
lead their deck and may have some number of "dreamsigns":

- **Dreamcaller:** An animated 3D character who starts each battle already in
  play for both participants in a battle. Each dreamcaller has powerful ongoing
  static, triggered, or activated abilities.
- **Dreamsigns:** Cards with 2D illustrations of objects, which provide more
  minor ongoing effects. Dreamsign effects can apply during battles, on the
  quest map, or both. Generally we try to assign the splashy "build around"
  effects to dreamcallers and secondary effects to dreamsigns.

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

## Dreamscape Sites

The following dreamscape sites are planned for eventual implementation in
Dreamtides. Sites can generally be visited in any order, with the exception that
the "Battle" site must be visited last. Each site must be visited exactly once
and cannot be returned to. Dreamscapes contain between 3 and 6 total sites
(including battle and draft sites, configured in TOML) as described below in the
[Dreamscape Generation](#dreamscape-generation) section.

Many sites have an "enhanced" version with a stronger version of their ability
which can appear as described in [Enhanced Sites](#enhanced-sites) below.

Many sites are associated with an NPC, a 3D humanoid character that can play
character animations and show a speech bubble. This NPC is always the same for a
given site (e.g. all shops are the same NPC), and their behavior and dialog are
configured via TOML. For sites with an NPC, portrait mode frames the NPC at the
top of the screen with content below, while landscape mode places the NPC to one
side with content beside them.

### Battle

The Battle site is the core gameplay element of Dreamtides, and it allows users
to play a match against an AI opponent. Each battle has an assigned opponent
dreamcaller with their own deck. Opponent decks are (for now) defined statically
in TOML. Before the battle begins, the opposing dreamcaller is displayed so the
user can understand any special abilities they have. Opposing dreamsigns are
also shown. When the battle completes, the [Victory or Defeat](#victory--defeat)
screen is shown along with any associated battle rewards.

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

### Draft

The Draft site is the other core component of Dreamtides gameplay, allowing
users to add cards to their deck. A draft site will display groups of cards to
select from, typically 4, and the user must pick a card to add to their deck. A
draft site will typically offer a repeated sequence of draft picks, so for
example a user might end up drafting 5 cards to add to their deck over 5 draft
picks, out of a pool of 20 possible choices. There is no way to "skip" or
"reroll" draft picks by default, but of course all rules can be broken by
specific dreamsigns.

**UI:** 4 cards are shown in a row (landscape mode) or in two rows (portrait
mode). The cards to draft from are shown in a pile in the 3D scene, then 4 of
them animate in to be selected. Clicking a card animates it to the quest deck,
and the others animate away, then 4 more cards from the pile animate in. After
all drafts are completed, the camera automatically pulls back to the map view.
Cards are shown with an orange outline.

Icon: "Rectangle Vertical"

### Dreamcaller Draft

The user activates the Dreamcaller Draft site to select their chosen
dreamcaller. This displays a selection of around 3 dreamcallers. Dreamcallers
are animated 3D characters, and we'll typically play character animations on
this screen. The user can read the special abilities of the offered dreamcallers
and pick one to lead their deck. Dreamcallers affect which cards are offered in
future Draft sites, refer to the [Resonance](#resonance) section below for more
details.

Each dreamcaller comes with a different **essence bonus** gained for selecting
that option, which serves as a lever for balancing more powerful dreamcallers.
Bonus amounts are configured in TOML.

Since all non-battle sites must be visited before entering battle, the
dreamcaller is always selected before the battle begins. There is a certain
element of strategy to *when* the user visits this site relative to other sites
like shops and drafts, and it's intended to not be obvious whether it's better
to visit other sites before selecting a dreamcaller.

**UI:** Dreamcallers are shown in their full-body "card" representation, with
ability text displayed alongside their 3D models and essence bonuses. The
dreamcaller cards animate in from a small size in the center of the screen. Each
dreamcaller does a different humanoid animation within its card frame. A primary
action button appears below each dreamcaller allowing them to be selected. The
selected dreamcaller animates to the bottom left of the screen to appear in a
"square" frame (head only). The other cards animate back to a small size.

Icon: "Crown"

### Discovery Draft

Shows four cards which have some unifying mechanical theme, for example showing
warrior cards, removal spells, cards that involve discarding a card, rare cards,
only dreamsigns, only triggered abilities, etc. The user selects a single card.

**UI:** This site follows the same interface behavior as the "Draft" site.

Icon: "Compass"

### Shop

The shop is the primary site in which the user can spend their essence. Shops
offer individual cards and dreamsigns for purchase, and may rarely offer other
site options such as purchasing journeys, purging cards, transfiguring cards,
duplicating cards, etc. Shops do offer the ability to spend essence to "reroll"
(generate a new set of shop items to buy).

Shop base prices and the overall essence economy are defined in TOML. The shop
implements a random "discount" system where one or more items can be displayed
as being on sale, for between 30% and 90% cost reduction. Things like dreamsigns
or journey effects can also modify shop prices.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. Two rows of three items
each are displayed, along with a close button. The items are beside the NPC in
landscape mode and below the NPC in portrait mode. Each item has a purple button
under it showing the essence cost to purchase that item. Clicking the button for
a card or dreamsign animates it to the quest deck or dreamsign display in the
bottom right corner of the screen. The other items do not move on purchase,
leaving a gap. One of the items shown may be a "reroll" option. When this is
selected, the items do a staggered scale-down animation, then the 6 new options
perform a scale-up animation in-place. Clicking the close button completes the
site visit and pulls the camera back to the map screen. The items remain in
place visually rather than animating away, but the site cannot be revisited.

Icon: "Store"

### Specialty Shop

A specialty shop functions in the same manner as a regular shop, but it shows
items with a unifying mechanical theme. Specialty shops use the same generation
algorithm and tagging system as [Discovery Draft](#discovery-draft) sites.

**UI:** Identical UI to the regular shop site except that it features a
different NPC.

Icon: "Store Alt 2"

### Dreamsign Offering

At a dreamsign offering site, the user is presented with a single dreamsign to
gain. The offering may be rejected, but there is no reward for doing so.

**UI:** The dreamsign animates to be displayed from screen center at a small
scale. A purple accept button and a gray reject button are displayed. The
dreamsign animates to the bottom right dreamsign display if accepted and
animates back to a small scale if rejected.

Icon: "Sparkles"

### Dreamsign Draft

At a dreamsign draft site, the user is presented with around three dreamsigns
and is able to select one to gain. It is again possible to select no dreamsign.

**UI:** The three dreamsigns animate in at full size from the bottom of the
screen in a staggered animation, positioning themselves in a single row. Purple
accept buttons are shown below each one. A red close button is shown top left,
functioning in a similar way to the Shop close button. Accepting a dreamsign
animates it to the user's dreamsign display area in the bottom right of the
screen.

Icon: "Sparkles Alt"

### Dream Journey

A dream journey functions in a manner similar to a random event in other
roguelike deckbuilding games. The user is offered a selection between two
circular cards with unique art. Each card has a description, although the amount
of information revealed about the effects is variable, and some dream journeys
have highly random effects which are not disclosed in advance. This is where we
put the biggest random effects which can structurally change a quest or modify
the user's entire deck. A close button is displayed in a similar manner to the
shop screen allowing the user to reject the dream journey options.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. The journey cards animate
from the center of the NPC's chest at a small size and are shown side-by-side in
a similar layout to the shop screen (next to the NPC landscape, below in
portrait). A purple button is displayed under each journey card to accept it.
Clicking this button causes the not-selected journey card to animate down to a
small size and vanish. The accepted journey card animates up to appear in screen
center, then plays a dissolve animation. The effects of the journey are shown
via a custom animation (e.g. cards might fade in and then be animated to the
user's quest deck if the journey effect is "add 3 cards to your deck"). Once the
effect animation completes, the camera pulls back to the map screen. A dream
journey is a circular card image which displays its rules text on hover/long
press.

Icon: "Moon + Star"

### Tempting Offer

A tempting offer is a site where the user is faced with a pair of dream journey
options with positive effects, in a similar manner to the dream journey site.
This time, however, each dream journey is also associated with a 'cost' card
with its own card and description, showing some price to be paid to unlock the
journey effect. The user may select an option to pay its cost and receive the
benefit. A close button is displayed in a similar manner to the shop screen
allowing the user to reject the dream journey options.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. The journey/cost card
pairs animate out from the center of the NPC's chest at a small scale in a
staggered animation (identical to a Dream Journey). The cards are displayed in
two rows, with the journey card on the left side of the row and the cost card on
the right side of the row, and with a purple button displayed under each pair to
select that option. Picking an option performs the same resolution animation as
above, with the journey card first animating to a large size in the center of
the screen, dissolving, and playing a custom effect animation, then the cost
card animating to screen center and playing its custom animation. Journey and
Cost cards will often have associated sound effects and particle effects for
their abilities.

Icon: "Law"

### Purge

A purge site allows the user to remove up to 3 cards from their deck, allowing
them to remove cards that don't fit with their overall gameplan.

**UI:** The camera pulls in to see an NPC at the site, who performs a character
animation and displays a speech bubble. After a pause, the user's quest deck
opens its browser view, showing cards, and a message instructs the user to
select cards to purge (0/3). Selected cards get a red outline. The quest deck
browser can also be opened outside of sites by clicking the quest deck in the
bottom right of the screen. A red X close button is displayed as in the normal
deck browser view. A red button with e.g. "purge 3 cards" appears at the bottom
of the screen when cards are selected. Clicking this button closes the quest
deck browser but causes the selected cards to animate to screen center. They
then play a dissolve animation and fade away. Once this animation completes, the
camera pulls back to the map screen.

Icon: "Hot"

### Essence

An essence site grants the user a fixed amount of essence, often around 200-300.

**UI:** Unlike with other sites, the camera does not zoom in to essence sites.
Instead the button simply vanishes on click and a purple particle effect
appears, animating in a winding path to the user's essence total and then plays
a 'hit' particle effect when it reaches the bottom left essence total and
updates the quantity of essence shown.

Icon: "Diamond"

### Transfiguration

A transfiguration site shows the user 3 random cards from their deck, and they
may select one to apply a transfiguration to, modifying that card's rules text.
Each card can only receive a single transfiguration; cards that have already
been transfigured are not eligible. If multiple transfigurations are applicable
to a card, a random one is selected to suggest.

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

### Duplication

A duplication site shows the user 3 random cards from their deck along with a
proposed random number of copies to create for each card between 1 and 4. The
user may pick one of the proposed options to add that many duplicates of that
card to their deck.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. 3 cards from the quest
deck animate to appear in a row via a staggered move animation. A purple button
like "Duplicate x3" appears under each one. Clicking this button causes the
other cards to fall away, and then a particle effect plays and additional copies
of the card emerge from the selected card. All copies then animate to the quest
deck, and the camera pulls back to the map screen. A close button is displayed
to allow the user to decline duplication.

Icon: "Copy"

### Reward Site

A reward site is a special site for granting the user a fixed reward (a specific
card or cards, dreamsign, group of dreamsigns, etc). The distinguishing factor
of reward sites is that these rewards are *known in advance* before selecting a
dreamscape to activate on the [Dream Atlas](#dream-atlas).

**UI:** The camera pulls in on a scene showing the reward items in question,
with a purple "accept" button and a gray "decline" button. Accepting the reward
plays the standard animation for that item type, for example animating to the
quest deck, and then the camera pulls back to the map screen.

Icon: "Treasure Chest"

### Cleanse

A Cleanse site allows the user to remove up to 3 random [Banes](#banes) from
their deck or dreamsigns.

**UI:** An NPC is shown who performs an animation and displays a speech bubble
with some dialog when the camera arrives at this site. The randomly selected
cards or dreamsigns to cleanse emerge from the quest deck or dreamsign display.
A purple "cleanse" button is displayed, along with a gray "decline" button.
Selecting "cleanse" causes the bane cards to play a dissolve animation, and then
the camera pulls back to the map screen.

Icon: "Snowflake"

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
custom cards in their decks.

### Battle Rewards

Completing a battle always grants an essence reward, which increases as the user
completes more dreamscapes. The user also gets a "rare draft" pick, functioning
like a single pick from a normal [Draft](#draft) site but drawing only from rare
cards in the pool. As with all draft picks, card selection is influenced by
resonance and tags. This draft pick cannot be skipped.

## Limits

Quest decks can contain a maximum of 50 cards during battles. If this limit is
exceeded, before the battle starts the user gains the ability to purge cards of
their choice to get back down under 50 cards.

Quest decks must contain a minimum of 25 cards. If the user has not completed
enough drafts to reach this threshold, additional copies of their deck are added
during a battle until they exceed 25 (for example, a player with 9 cards in
their deck will end up with 27 cards during a battle).

Users can have a maximum of 12 dreamsigns at any time. If they would receive
another dreamsign, an overlay is shown and they must immediately purge a
dreamsign.

Users may have only 1 dreamcaller.

## Banes

Certain cards and dreamsigns, called "banes", can be given to the user during a
quest, typically as a result of a [Tempting Offer](#tempting-offer) choice. Bane
cards generally have negative effects when drawn, while bane dreamsigns provide
ongoing negative effects on the quest. Bane cards can be [purged](#purge) as
normal. Bane cards and bane dreamsigns can be removed via the
[cleanse](#cleanse) site.

## Resonance

A critical component of the drafting system in Dreamtides is "card resonance",
which performs a function similar to the color pie in Magic: the Gathering. Each
card, dreamsign, and dreamcaller has zero or more resonance symbols associated
with it, drawn from:

- Tide
- Ember
- Zephyr
- Stone
- Ruin

When generating draft picks, shop offerings, or dreamsign offerings, the user's
*current* deck and dreamcaller are evaluated for a combined resonance score, and
the selection of draft cards is weighted towards that score, i.e. a deck that
contains a lot of Tide and Stone cards will generally see more Tide and Stone
cards. As more cards with a given resonance are added, the chance of seeing
other resonances diminishes. Generally the system converges towards decks having
2 main resonances after 5-10 draft picks.

Draft picks are drawn from a "pool" of cards generated at the start of a quest.
When presenting draft options, cards are selected from this pool with
probability proportional to how well their resonance matches the user's current
resonance score, so the pool contents are fixed but the likelihood of being
offered any given card shifts as the deck evolves. Cards are drawn without
replacement, meaning the odds of seeing cards more than once diminish over time.

When starting a new quest, the draft pool is weighted based on card rarity, with
more copies of common cards and fewer copies of rare/legendary cards. There is
also a slight random starting bias in the pool to make the play experience
variable; there might for example be 20% more tide cards, 10% more ember cards,
10% less stone cards, and 20% less ruin cards in the pool.

The exact configuration of the draft pool, including weighting algorithms, is
all data-driven and managed by TOML files.

### Card Tagging

In addition to resonance, cards can have zero or more **tags** defined on them.
Tags can cover any sort of mechanical theme and generally correspond to possible
deck archetypes, such as cards that care about discard, cards that support a
specific tribe like spirit animals, cards that care about reclaim, etc. Tags
influence card selection in a similar manner to resonance: as the user drafts
more cards sharing a tag, subsequent offerings are more likely to include cards
with that same tag. [Discovery Draft](#discovery-draft) and
[Specialty Shop](#specialty-shop) sites use tags to generate their thematic
groupings, selecting a tag and then offering cards that share it. Tag behavior
is configured in TOML.

## Dream Atlas

The Dream Atlas is the screen players see at the start of a quest. It shows a 3D
map of dreamscapes represented as circular miniature "worlds," connected by
dotted lines. The player can hover over or long-press a dreamscape to preview
its biome and available sites, then click it again to zoom the camera in to that
dreamscape.

Each dreamscape can be in one of three states:

- **Completed**: The player has already visited this dreamscape and finished its
  battle.
- **Available**: The player can choose this dreamscape as their next
  destination.
- **Unavailable**: The player cannot choose this dreamscape yet.

The player begins at the center of the Dream Atlas, called the **Nexus**. At the
start, any dreamscapes connected to the Nexus are **Available**.

After the player visits a dreamscape and completes its battle, that dreamscape
becomes **Completed**. Any dreamscapes directly connected to it then also become
**Available**. The number of dreamscapes the user has completed is called the
'Completion Level' for that quest.

In other words, a dreamscape is **Available** only if it is connected to the
Nexus or to at least one **Completed** dreamscape.

Each dreamscape displays a preview of what sites are available in that location.
This shows 2-3 site icons, not including "draft" or "battle" sites, allowing the
user to make an informed decision about which dreamscape to visit next. This is
also where [Reward Site](#reward-site) rewards are shown. Winning the 7th battle
causes the player to win the quest.

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

Dreamscapes are generated by drawing sites from a pool, in a similar manner to
how draft picks are generated. Sites are selected when the dreamscape becomes
available. The pool for site generation changes over time, with new options
being shuffled in after each dreamscape is completed. Each completed dreamscape
shuffles in a new set of sites as defined in TOML for that completion level.
Transfiguration, Purge, and Duplication sites are more common later in the
Quest, for example.

All sites can appear a maximum of 1 time in a dreamscape, with the exception
that there can be up to 2 Draft sites (as noted below) and up to 2 Essence
sites.

Draft sites are handled differently. Dreamscapes have a deterministic number of
draft sites based on completion level.

| Completion Level | Draft Sites |
| ---------------- | ----------- |
| 0, 1             | 2           |
| 2, 3             | 1           |
| 4+               | 0           |

Battle sites are also distinct: Dreamscapes have one Battle site, or zero if
this has been modified by [meta progression](meta_progression.md). The opponent
dreamcaller, dreamsigns, and deck for the battle is selected from a pool of
opponents defined in TOML for a given completion level. Difficulty scaling is
configured in TOML.

The Dreamcaller Draft site is distinct and always appears in the first
dreamscape visited, and only in that dreamscape.

### Enhanced Sites

Each dreamscape is associated with a specific "biome" which dictates the 3D
environment assets used in generation. Biomes are purely visual aside from their
enhanced site affinity. There is one biome per enhanced site type, and biome
configuration and assignment are defined in TOML. Each dreamscape biome has an
affinity for a specific site, and produces an "enhanced site" of that type when
visited. The available enhanced sites are:

- **Shop**: The reroll option is free
- **Dreamsign Offering/Dreamsign Draft**: A dreamsign draft is offered instead,
  or a draft is offered with an additional option
- **Dream Journey**: A 3rd dream journey option is provided
- **Tempting Offer**: 3 tempting offer options are displayed
- **Purge**: Up to 6 cards can be removed from the deck
- **Essence**: The essence amount given is doubled
- **Transfiguration**: The player may select which card in their deck receives
  transfiguration
- **Duplication**: The player may select which card in their deck is duplicated
- **Discovery**: The player may select any number of the offered cards to add to
  their deck.

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
