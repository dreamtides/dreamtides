# Core game symbols used throughout card text
energy-symbol = <color=#00838F>●</color>
points-symbol = <color=#F57F17>⍟</color>
fast-symbol = ↯

# Energy cost/value formatters with colored symbol (e.g., "2●")
e = <color=#00838F>{$e}●</color>
e1 = <color=#00838F>{$e1}●</color>
e2 = <color=#00838F>{$e2}●</color>
e3 = <color=#00838F>{$e3}●</color>
mode1-cost = <color=#00838F>{$mode1-cost}●</color>
mode2-cost = <color=#00838F>{$mode2-cost}●</color>

# Points value formatters with colored symbol (e.g., "3⍟")
points = <color=#F57F17>{$points}⍟</color>
points1 = <color=#F57F17>{$points1}⍟</color>
points2 = <color=#F57F17>{$points2}⍟</color>

# Used for cards that modify maximum energy pool
maximum-energy = {$max} maximum {energy-symbol}

# Trigger ability prefixes - displayed at start of triggered ability text
-trigger = ▸ <b>{$trigger}:</b>
Materialized = {-trigger(trigger: "Materialized")}
Judgment = {-trigger(trigger: "Judgment")}
Dissolved = {-trigger(trigger: "Dissolved")}
MaterializedJudgment = {-trigger(trigger: "Materialized, Judgment")}
MaterializedDissolved = {-trigger(trigger: "Materialized, Dissolved")}

# Phase name reference used in card text (e.g., "trigger an additional Judgment phase")
JudgmentPhaseName = <b>Judgment</b>

# Game keyword formatting - purple colored text for game mechanics
-keyword = <color=#AA00FF>{$k}</color>

dissolve = {-keyword(k: "dissolve")}
dissolved = {-keyword(k: "dissolved")}
Dissolve = {-keyword(k: "Dissolve")}
banish = {-keyword(k: "banish")}
Banish = {-keyword(k: "Banish")}
banished = {-keyword(k: "banished")}
discover = {-keyword(k: "discover")}
Discover = {-keyword(k: "Discover")}
reclaim = {-keyword(k: "reclaim")}
Reclaim = {-keyword(k: "Reclaim")}
materialize = {-keyword(k: "materialize")}
Materialize = {-keyword(k: "Materialize")}
prevent = {-keyword(k: "prevent")}
Prevent = {-keyword(k: "Prevent")}
kindle = {-keyword(k: "kindle")} {$k}
Kindle = {-keyword(k: "Kindle")} {$k}
foresee = {-keyword(k: "foresee")} {$foresee}
Foresee = {-keyword(k: "Foresee")} {$foresee}

# Fast keyword with lightning bolt symbol
fast = <b>↯fast</b>
Fast = <b>↯Fast</b>

# Reclaim ability with energy cost shown (e.g., "reclaim 2●")
reclaim-for-cost = {-keyword(k: "reclaim")} <color=#00838F>{$reclaim}●</color>
ReclaimForCost = {-keyword(k: "Reclaim")} <color=#00838F>{$reclaim}●</color>

# Modal card formatting
ChooseOne = <b>Choose One:</b>
bullet = •

# Card count with article (e.g., "a card" or "2 cards") - used for draw/discard effects
cards =
  {
    $cards ->
      [one] a card
      *[other] { $cards } cards
  }
cards1 =
  {
    $cards1 ->
      [one] a card
      *[other] { $cards1 } cards
  }
cards2 =
  {
    $cards2 ->
      [one] a card
      *[other] { $cards2 } cards
  }
cards3 =
  {
    $cards3 ->
      [one] a card
      *[other] { $cards3 } cards
  }

# Discard count with article - separate from 'cards' to allow different variable names
discards =
  {
    $discards ->
      [one] a card
      *[other] { $discards } cards
  }
discards1 =
  {
    $discards1 ->
      [one] a card
      *[other] { $discards1 } cards
  }
discards2 =
  {
    $discards2 ->
      [one] a card
      *[other] { $discards2 } cards
  }

# For effects that move cards from deck to void (e.g., "Put the top 3 cards of your deck into your void")
top-n-cards =
  {
    $to-void ->
      [one] top card
      *[other] top { $to-void } cards
  }

# Card count with numeral instead of article (e.g., "1 card" not "a card") - used for conditions like "When you play 2 cards"
cards-numeral =
  {
    $cards ->
      [one] { $cards } card
      *[other] { $cards } cards
  }

# Spark value passthrough - used in rules like "spark {s} or less"
s = { $s }

# Spark keyword for ability text
spark = spark

# Generic count passthrough for numeric conditions
count = { $count }

# Ally count with article (e.g., "an ally" or "2 allies") - used for targeting/conditions
count-allies =
  {
    $allies ->
      [one] an ally
      *[other] { $allies } allies
  }

# Allied character count with subtype (e.g., "an allied warrior" or "2 allied warriors")
count-allied-subtype =
  {
    $allies ->
      [one] an allied {subtype}
      *[other] { $allies } allied {plural-subtype}
  }

# Figment token formatting - gold colored, bold, underlined
-figment = <color=#F57F17><b><u>{$f} Figment</u></color></b>

-figments-plural = <color=#F57F17><b><u>{$f} Figments</u></color></b>

figments =
  {
    $figment ->
      [celestial] {-figments-plural(f: "Celestial")}
      [halcyon] {-figments-plural(f: "Halcyon")}
      [radiant] {-figments-plural(f: "Radiant")}
      *[other] Error: Unknown 'figment' for type: { $figment }
  }

a-figment =
  {
    $figment ->
      [celestial] a {-figment(f: "Celestial")}
      [halcyon] a {-figment(f: "Halcyon")}
      [radiant] a {-figment(f: "Radiant")}
      *[other] Error: Unknown 'a-figment' for type: { $figment }
  }

n-figments =
  {
    $number ->
      [one] {a-figment}
      *[other] { text-number } {figments}
  }

# Character subtype formatting - green colored, bold (e.g., "warrior", "ancient")
-type = <color=#2E7D32><b>{$value}</b></color>

# Subtype with lowercase article (e.g., "a warrior", "an ancient") - handles a/an correctly
a-subtype =
  {
    $subtype ->
      [ancient] an {-type(value: "ancient")}
      [child] a {-type(value: "child")}
      [detective] a {-type(value: "detective")}
      [enigma] an {-type(value: "enigma")}
      [explorer] an {-type(value: "explorer")}
      [hacker] a {-type(value: "hacker")}
      [mage] a {-type(value: "mage")}
      [monster] a {-type(value: "monster")}
      [musician] a {-type(value: "musician")}
      [outsider] an {-type(value: "outsider")}
      [renegade] a {-type(value: "renegade")}
      [spirit-animal] a {-type(value: "spirit animal")}
      [super] a {-type(value: "super")}
      [survivor] a {-type(value: "survivor")}
      [synth] a {-type(value: "synth")}
      [tinkerer] a {-type(value: "tinkerer")}
      [trooper] a {-type(value: "trooper")}
      [visionary] a {-type(value: "visionary")}
      [visitor] a {-type(value: "visitor")}
      [warrior] a {-type(value: "warrior")}
      *[other] Error: Unknown 'a-subtype' for type: { $subtype }
  }

# Subtype with capitalized article for sentence start (e.g., "A warrior", "An ancient")
ASubtype =
  {
    $subtype ->
      [ancient] An {-type(value: "ancient")}
      [child] An {-type(value: "child")}
      [detective] An {-type(value: "detective")}
      [enigma] An {-type(value: "enigma")}
      [explorer] An {-type(value: "explorer")}
      [hacker] An {-type(value: "hacker")}
      [mage] An {-type(value: "mage")}
      [monster] An {-type(value: "monster")}
      [musician] An {-type(value: "musician")}
      [outsider] An {-type(value: "outsider")}
      [renegade] An {-type(value: "renegade")}
      [spirit-animal] An {-type(value: "spirit animal")}
      [super] An {-type(value: "super")}
      [survivor] An {-type(value: "survivor")}
      [synth] An {-type(value: "synth")}
      [tinkerer] An {-type(value: "tinkerer")}
      [trooper] An {-type(value: "trooper")}
      [visionary] An {-type(value: "visionary")}
      [visitor] An {-type(value: "visitor")}
      [warrior] An {-type(value: "warrior")}
      *[other] Error: Unknown 'ASubtype' for type: { $subtype }
  }

# Subtype without article (e.g., "warrior", "ancient") - for use after "allied" or other modifiers
subtype =
  {
    $subtype ->
      [ancient] {-type(value: "ancient")}
      [child] {-type(value: "child")}
      [detective] {-type(value: "detective")}
      [enigma] {-type(value: "enigma")}
      [explorer] {-type(value: "explorer")}
      [hacker] {-type(value: "hacker")}
      [mage] {-type(value: "mage")}
      [monster] {-type(value: "monster")}
      [musician] {-type(value: "musician")}
      [outsider] {-type(value: "outsider")}
      [renegade] {-type(value: "renegade")}
      [spirit-animal] {-type(value: "spirit animal")}
      [super] {-type(value: "super")}
      [survivor] {-type(value: "survivor")}
      [synth] {-type(value: "synth")}
      [tinkerer] {-type(value: "tinkerer")}
      [trooper] {-type(value: "trooper")}
      [visionary] {-type(value: "visionary")}
      [visitor] {-type(value: "visitor")}
      [warrior] {-type(value: "warrior")}
      *[other] Error: Unknown 'type' for type: { $subtype }
  }

# Plural subtype (e.g., "warriors", "children") - note irregular plurals like child→children
plural-subtype =
  {
    $subtype ->
      [ancient] {-type(value: "ancients")}
      [child] {-type(value: "children")}
      [detective] {-type(value: "detectives")}
      [enigma] {-type(value: "enigmas")}
      [explorer] {-type(value: "explorers")}
      [hacker] {-type(value: "hackers")}
      [mage] {-type(value: "mages")}
      [monster] {-type(value: "monsters")}
      [musician] {-type(value: "musicians")}
      [outsider] {-type(value: "outsiders")}
      [renegade] {-type(value: "renegades")}
      [spirit-animal] {-type(value: "spirit animals")}
      [super] {-type(value: "supers")}
      [survivor] {-type(value: "survivors")}
      [synth] {-type(value: "synths")}
      [tinker] {-type(value: "tinkerers")}
      [trooper] {-type(value: "troopers")}
      [visionary] {-type(value: "visionaries")}
      [visitor] {-type(value: "visitors")}
      [warrior] {-type(value: "warriors")}
      *[other] Error: Unknown 'plural-type' for type: { $subtype }
  }

# Convert number to word (1→"one", 2→"two", etc.) - falls back to numeral for 10+
text-number =
  {
    $number ->
      [1] one
      [2] two
      [3] three
      [4] four
      [5] five
      [6] six
      [7] seven
      [8] eight
      [9] nine
      *[other] { $number }
  }

# Turn duration with repetition (e.g., "this turn", "this turn twice", "this turn three times")
this-turn-times =
  {
    $number ->
      [1] this turn
      [2] this turn twice
      *[other] this turn {text-number} times
  }

# Multiplier effect (e.g., "Double", "Triple") - falls back to "Multiply by X" for 6+
MultiplyBy =
  {
    $number ->
      [2] Double
      [3] Triple
      [4] Quadruple
      [5] Quintuple
      *[other] Multiply by { $number }
  }

# Copy count with article (e.g., "a copy", "two copies")
copies =
  {
    $number ->
      [one] a copy
      *[other] { text-number } copies
  }

# Random character targeting (e.g., "a random character", "two random characters")
n-random-characters =
  {
    $number ->
      [1] a random character
      *[other] { text-number } random characters
  }

# Optional event targeting (e.g., "an event", "one or two events", "up to 3 events")
up-to-n-events  =
  {
    $number ->
      [1] an event
      [2] one or two events
      *[other] up to { $number } events
  }

# Optional ally targeting (e.g., "an ally", "one or two allies", "up to 3 allies")
up-to-n-allies  =
  {
    $number ->
      [1] an ally
      [2] one or two allies
      *[other] up to { $number } allies
  }

# Pronoun agreement for variable counts (e.g., "banish it" vs "banish them")
it-or-them =
  {
    $number ->
      [1] it
      *[other] them
  }

# Dev menu icon
bug-icon = {"\uf88d"}

# Undo button icon
undo-icon = {"\ufd88"}

# Eye icon
eye-icon = {"\uf9f9"}

# Eye icon with slash through it
eye-slash-icon = {"\uf9f8"}

# "Star" icon used to represent non-numeric costs
asterisk-icon = {"\uf810"}

# Prompt message to target a character
prompt-choose-character = Choose a character

# Prompt message to pick a card on the stack
prompt-select-stack-card = Select a card

# Prompt message to pick a card from your void
prompt-select-from-void = Select from your void

# Prompt message to pick a card from your hand
prompt-select-from-hand = Select from your hand

# Prompt message to pick a choice among several options
prompt-select-option = Select an option

# Prompt message to pick an amount of energy
prompt-choose-energy-amount = Choose energy amount

# Prompt message to pick card ordering within the deck
prompt-select-card-order = Select card position

# Prompt message to pick a mode of a modal card to play
prompt-pick-mode = Choose a mode

# Dev menu button label
dev-menu-button = {bug-icon} Dev

# Decline to take the action associated with a prompt
decline-prompt-button = Decline

# Choose to pay energy to take a prompt action
pay-energy-prompt-button = Spend {e}

# Button to confirm the amount of energy to pay as an additional cost
pay-energy-addtional-cost-button = Spend {e}

# Button to confirm selection of target cards in the void
primary-button-submit-void-card-targets = Submit

# Button to confirm selection of target cards in the hand
primary-button-submit-hand-card-targets = Submit

# Button to confirm selection of ordering of cards in deck
primary-button-submit-deck-card-order = Submit

# Button to resolve the top card of the stack
primary-button-resolve-stack = Resolve

# Button to end your turn
primary-button-end-turn = End Turn

# Button to end the opponent's turn and begin your turn
primary-button-start-next-turn = Next Turn

# Button to increment the energy amount in a prompt to pick an energy value
increment-energy-prompt-button = +1{energy-symbol}

# Button to decrement the energy amount in a prompt to pick an energy value
decrement-energy-prompt-button = -1{energy-symbol}

# Button to hide the stack and view the battlefield
hide-stack-button = {eye-icon}

# Button to show the stack after hiding it
show-stack-button = {eye-slash-icon}

# Button to show the battlefield (synonym for hide-stack)
show-battlefield-button = {eye-icon}

# Button to hide the battlefield (synonym for show-stack)
hide-battlefield-button = {eye-slash-icon}

# Addition to card rules text showing how much energy was spent on a card with a variable energy cost
card-rules-text-energy-paid = ({e} paid)

# Addition to card rules text showing that a card was played with the "reclaim" ability
card-rules-text-reclaimed = (Reclaimed)

# Addition to card rules text showing that a card has been "anchored"
card-rules-text-anchored = (Anchored)

# Card name for a card representing a numbered modal effect choice
modal-effect-choice-card-name = Choice {$number}

# Card name for a card representing an ability of a character
character-ability-card-name = {$character-name} Ability

# Message describing the effects of exceeding the hand size limit
hand-size-limit-exceeded-warning-message = Note: Cards drawn in excess of 10 become {energy-symbol} instead.

# Message describing the effects of exceeding the character limit
character-limit-exceeded-warning-message = Character limit exceeded: A character will be abandoned, with its spark permanently added to your total.

# Message describing the effects of exceeding both the character limit and the hand size limit
combined-limit-warning-message = Character limit exceeded: A character will be abandoned. Cards drawn in excess of 10 become {e} instead.

# Title for a panel displaying an error message
error-message-panel-title = Error

# Card type for character cards
card-type-character = Character

# Card type for event cards
card-type-event = Event

# Card type for dreamsign cards
card-type-dreamsign = Dreamsign

# Card type for dreamcaller cards
card-type-dreamcaller = Dreamcaller

# Card type for dreamwell cards
card-type-dreamwell = Dreamwell

# Info zoom help text, displayed on a tooltip to describe card abilities
help-text-dissolve = {Dissolve}: Send a character to the void

# Info zoom help text, displayed on a tooltip to describe card abilities
help-text-prevent = {Prevent}: Send a card to the void in response to it being played

# Info zoom help text, displayed on a tooltip to describe card abilities
help-text-foresee-1 = {Foresee}: Look at the top card of your deck. You may put it into your void.

# Info zoom help text, displayed on a tooltip to describe card abilities
help-text-foresee-n = {Foresee}: Look at the top {$foresee} cards of your deck. You may put them into your void or put them back in any order.

# Info zoom help text, displayed on a tooltip to describe card abilities
help-text-anchored = <color=#AA00FF><b>Anchored</b></color>: Cannot be dissolved.

# Info zoom help text, displayed on a tooltip to describe card abilities
help-text-reclaim-without-cost = {Reclaim}: You may play a card from your void, then banish it when it leaves play.

# Info zoom help text, displayed on a tooltip to describe card abilities
help-text-reclaim-with-cost = {Reclaim} {e}: You may play this card from your void for {e}, then banish it.

# Card type for an activated ability card
token-type-activated-ability = Activated Ability

# Card type for a triggered ability card
token-type-triggered-ability = Triggered Ability

# Card type for a reclaim ability card
token-type-reclaim-ability = Reclaim Ability
