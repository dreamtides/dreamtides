# Dreamtides Quests: Master Design Document

This is the master design document for the Dreamtides "quests" system. Quests
are the meta layer in which the user navigates various encounters on a map
screen in order to improve their deck, while battles are individual card
matches. Quests are similar to "runs" in other roguelike deckbuilding games,
while battles are similar to "fights". Quests will be at least as complicated to
implement as battles, and almost every existing line of code for supporting
battles will require an equivalent for quests.

This document is the high level "vision" for quests, other documents in this
directory providem more detailed gameplay & technical breakdowns of the feature.

## Overview

Quests revolve primarily around drafting a deck to bring into future battles,
since by default users start with no cards. Quests use a currency called
"essence" which can be spent on shops and in various other ways. In addition to
default cards, users during a quest will select a "dreamcaller" to lead their
deck and one or more "dreamsigns":

- Dreamcaller: An animated 3D character who starts each battle already in play
  for both participants in a battle. Each dreamcaller has powerful ongoing
  static, triggered, or activated abilities.
- Dreamsigns: Cards with 2D illustrations of objects, which provide more minor
  ongoing effects. Generally we try to assign the splashy "build around" effects
  to dreamcallers and secondary effects to dreamsigns.

Quests display a 3D scene called a "dreamscape" from a top-down perspective. A
series of individual white icons with black circular backgounds are shown on the
scene called "sites". Each site icon corresponds to some specific quest effect,
and users can "visit" a site to activate the effect by clicking on the icon.
This causes the camera to zoom in on that site and then displays the site's
effect, often with a 3D animated NPC character introducing the site's concept.
Once all of the sites in a given dreamscape have been visited, the user may
navigate to the "battle" site to initiate a card battle. After completing a
battle, the player is able to select another dreamscape to navigate to, and the
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