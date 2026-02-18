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

## Overview

Quests revolve primarily around drafting a deck to bring into future battles,
since by default users start with no cards. Quests use a currency called
"essence" which can be spent on shops and in various other ways. In addition to
default cards, users during a quest will select a "dreamcaller" to lead their
deck and one or more "dreamsigns":

- **Dreamcaller:** An animated 3D character who starts each battle already in
  play for both participants in a battle. Each dreamcaller has powerful ongoing
  static, triggered, or activated abilities.
- **Dreamsigns:** Cards with 2D illustrations of objects, which provide more
  minor ongoing effects. Generally we try to assign the splashy "build around"
  effects to dreamcallers and secondary effects to dreamsigns.

Quests display a 3D scene called a "dreamscape" from a top-down perspective. A
series of individual white icons with black circular backgounds are shown on the
scene called "sites". Each site icon corresponds to some specific quest effect,
and users can "visit" a site to activate the effect by clicking on the icon.
This causes the camera to zoom in on that site and then displays the site's
effect, often with a 3D animated NPC character introducing the site's concept.
Once all of the sites in a given dreamscape have been visited, the user may
navigate to the "battle" site to initiate a card battle. After completing a
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
and cannot be returned to. Dreamscapes always contain a battle site and will
contain around 3-6 other sites as described below in the
[Dreamscape Generation](#dreamscape-generation) section.

Many sites have an "enchanced" version with a stronger version of their ability
which can appear as described in the dreamscape generation rules.

### Battle

The Battle site is the core gameplay element of dreamtides, and it allows users
to play a match against an AI opponent. Each battle has an assigned opponent
dreamcaller with their own deck. Opponent decks are (for now) defined statically
in TOML. Before the battle begins, the opposing dreamcaller is displayed so the
user can understand any special abilities they have. Opposing dreamsigns are
also shown. When the battle completes, the [Victory or Defeat](#victory--defeat)
screen is shown along witih any associated battle rewards.

Site Icon: "Sword"

### Draft

The Draft site is the other core component of Dreamtides gameplay, allowing
users to add cards to their deck. A draft site will display sequences of cards
to select from, typically 4, and the user must pick a card to add to their deck.
A draft site will typically offer a sequence of draft picks, so for example a
user might end up drafting 5 cards to add to their deck over 5 draft picks, out
of a pool of 20 possible choices. There is no way to "skip" or "reroll" draft
picks by default, but of course all rules can be broken by specific dreamsigns.

Icon: "Rectangle Vertical"

### Dreamcaller Draft

The user activates the Dreamcaller Draft site to select their chosen dreamcaller
or to pick a new dreamcaller. This displays a selection of around 3
dreamcallers. Dreamcallers are animated 3D characters, and we'll typically play
character animations on this screen. The user can read the special abilities of
the offered dreamcallers and pick one to lead their deck. Dreamcallers affect
which cards are offered in future Draft sites, refer to the
[Resonance](resonance--draft-pick-generation) section below for more details.

There is a certain element of strategy to *when* the user visits this site, and
it's intended to not be obvious whether it's better to visit other sites before
selecting a dreamcaller.

Icon: "Crown"

### Shop

The shop is the primary site in which the user can spend their essence. Shops
offer individual cards and dreamsigns for purchase, and may rarely offer other
site options such as purchasing journeys, purging cards, transfiguring cards,
duplicating cards, etc. Shops do offer the ability to spend essence to "reroll"
(generate a new set of shop items to buy).

Shop base prices are static, defined in TOML. The shop implements a random
"discount" system where one or more items can be displayed as being on sale, for
between 30% and 90% cost reduction. Things like dramsigns or journey effects can
also modify shop prices.

Icon: "Store"

### Dreamsign

Text

Icon: "Sparkles"

### Dreamsign Draft

Text

Icon: "Sparkles Alt"

### Dream Journey

Text

Icon: "Moon + Star"

### Tempting Offer

Text

Icon: "Law"

### Purge

Text

Icon: "Hot"

### Essence

Text

Icon: "Diamond"

### Transfiguration

Text

Icon: "Science"

### Duplication

Text

Icon: "Copy"

### Reward

Text

Icon: "Treasure Chest"

### Discovery

Text

Icon: "Compass"

### Cleanse

Text

Icon: "Snowflake"

## Victory & Defeat

### Battle Rewards

## Resonance & Draft Pick Generation

## Dreamscape Generation

## Transfiguration
