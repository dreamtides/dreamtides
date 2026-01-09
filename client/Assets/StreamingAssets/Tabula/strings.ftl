# Energy icon
# ID: dbb75f1b-8c25-4c27-a598-7b300f5b5ca4
e = {$context ->
  [card-text] <color=#00838F>{"\ufa1f"}</color>
  *[interface] {"\ufa1f"}
}

# Fast icon
# ID: 7b21533d-7f49-451c-bfa1-949ab28ed258
f = {"\uf852"}

# Activated icon
# ID: adc9a41d-fd82-49b5-a099-a9f10a73afad
a = {"\uf90d"}

# Multi-activated icon
# ID: 121fc3af-6abd-4a90-9197-43c6e18eeca2
ma = {"\uf916"}

# Fast + Activated icon
# ID: e0eb0864-3150-4ac5-a903-258ada0f906a
fa = {f}<space="-0.25px">{a}

# Fast + Multi-Activated icon
# ID: 1921caf1-b581-4feb-b956-b27a553cbc43
fma = {f}<space="-0.25px">{ma}

# Points icon
# ID: 81fb9fe5-eff0-46ce-a10a-7aa4a45b0992
p = {"\ufb43"}

# Dev menu icon
# ID: eeb2bbe9-1952-43fa-9ae6-12512d005829
bug-icon = {"\uf88d"}

# Undo button icon
# ID: d439176c-7a40-410b-808d-cc2be53116d7
undo-icon = {"\ufd88"}

# Eye icon
# ID: ad291414-914c-4039-8f8f-8902b752bf74
eye-icon = {"\uf9f9"}

# Eye icon with slash through it
# ID: 2ca2c1dd-20f7-410b-9f5d-ce313a2d9175
eye-slash-icon = {"\uf9f8"}

# "Star" icon used to represent non-numeric costs
# ID: fd603588-7ab8-48c8-93b0-2bf6afc59325
asterisk-icon = {"\uf810"}

# Formatting that appears before keywords
# ID: cb899350-def7-4466-a1e2-2a59e105d9fb
keyword = <color=#AA00FF><b>

# Formatting that appears after keywords
# ID: bf435908-6161-4a6e-9407-81e0ccde2c4e
end-keyword = </b></color>

# Formatting for an amount of energy paid as a cost
# ID: cc5c5509-69c5-4c1f-8faa-b9948029dcb8
-energy-cost = <color=#00838F><b>{$e}{e}</b></color>
    .column-1 = <color=#00838F><b>{$e}‎</b></color>

# Formatting for an amount of energy gained as an effect
# ID: 50ef5a4a-86f6-4640-88f9-a249cc4cd5cf
-gained-energy = <color=#00838F><b>{$e}{e}</b></color>
    .column-1 = <color=#00838F><b>{$e}‎</b></color>

# Formatting for a number of cards drawn
# ID: 44535fb6-0190-4197-8bce-3acf063d4318
-drawn-cards = {$n ->
  [one] a card
  *[other] {$n} cards
}

# Formatting for a number of cards discarded
# ID: 0d6c79d7-af53-4477-977c-a248fb4fa0ba
-discarded-cards = {$n ->
  [one] a card
  *[other] {$n} cards
}

# Formatting for gaining an amount of spark
# ID: c49b155f-57a2-479a-bc45-254b2144d04b
-gained-spark = +{$n} spark

# Formatting for an arbitrary numerical value
# ID: 707467ad-9e99-4953-8e4d-cf5f47e31768
-count = {$n}

# Formatting to select a single mode of a modal card
# ID: 3057478f-1445-4b7d-8241-a90d97da7adf
choose-one = Choose one:

# Inserts a bullet character
# ID: abfc5cdc-e118-4e43-87a1-3493cc80df75
bullet = •

# Foresee keyword ability with quantity
# ID: 2e44ee4c-3218-45bf-a29e-0508f853c873
-foresee = {keyword}foresee{end-keyword} {$n}

# Foresee keyword ability with quantity, capitalized
# ID: b68a7c1c-5b2b-4885-863b-4ebf81279330
-Foresee = {keyword}Foresee{end-keyword} {$n}

# Reclaim keyword ability without energy cost
# ID: f19214ee-632b-4f0b-b3e4-d63e67b7ac03
reclaim = {keyword}reclaim{end-keyword}

# Reclaim keyword ability without energy cost, capitalized
# ID: 6640dfed-a5d6-4b68-88db-3b62a4400132
Reclaim = {keyword}Reclaim{end-keyword}

# Reclaim keyword ability with energy cost
# ID: fb895ce7-1f66-426d-8c29-b19fdeee7828
-reclaim-cost = {keyword}reclaim{end-keyword} <color=#00838F><b>{$e}{e}</b></color>
    .column-1 = {keyword}reclaim{end-keyword} <color=#00838F><b>{$e}‎</b></color>

# Reclaim keyword ability with energy cost, capitalized
# ID: 5674a8dd-b4eb-4630-85c2-79d6b95cdaae
-Reclaim-Cost = {keyword}Reclaim{end-keyword} <color=#00838F><b>{$e}{e}</b></color>
    .column-1 = {keyword}Reclaim{end-keyword} <color=#00838F><b>{$e}‎</b></color>

# Kindle keyword ability with quantity
# ID: bf189fe0-75e2-4a87-9dca-67dd5f755766
-kindle = {keyword}kindle{end-keyword} {$s}

# Kindle keyword ability with quantity, capitalized
# ID: d1c8ff05-9dd8-4ed6-9436-746a9caf5a63
-Kindle = {keyword}Kindle{end-keyword} {$s}

# Formatting for gaining an amount of points
# ID: 70374512-2121-4569-a028-e871209fad38
-gained-points = {$n}{p}
    .column-1 = {$n}‎ﱪ

# Dissolve keyword ability
# ID: 5771085d-3c89-423f-be05-c728035e7cd0
dissolve = {keyword}dissolve{end-keyword}

# Dissolve keyword ability, capitalized
# ID: 8b9ef6b9-a6e1-4abe-be4d-cf7fa3ce41b8
Dissolve = {keyword}Dissolve{end-keyword}

# Prevent keyword ability
# ID: 1a9e01e9-e7aa-4692-9b1b-5dd9e0b4a7f9
prevent = {keyword}prevent{end-keyword}

# Prevent keyword ability, capitalized
# ID: fb38409b-cbaf-41a5-96fc-af622ce6f333
Prevent = {keyword}Prevent{end-keyword}

# Anchored keyword ability
# ID: 901b84a1-c886-447b-bd0d-005e240a23de
anchored = {keyword}anchored{end-keyword}

# Anchored keyword ability, capitalized
# ID: 1bb69fb4-344a-48f2-a7a2-95a30ac4a2f2
Anchored = {keyword}Anchored{end-keyword}

# Prompt message to target a character
# ID: 66edfc8f-483b-4ad0-b37b-c66aa7c4347b
prompt-choose-character = Choose a character

# Prompt message to pick a card on the stack
# ID: 3f16a9f0-c905-4040-b4e3-58eafa3144f2
prompt-select-stack-card = Select a card

# Prompt message to pick a card from your void
# ID: 95c481aa-46e6-4e03-ab9f-ae162207cd39
prompt-select-from-void = Select from your void

# Prompt message to pick a card from your hand
# ID: 68ddc76a-67ee-4d95-a0a1-ff688752b5f8
prompt-select-from-hand = Select from your hand

# Prompt message to pick a choice among several options
# ID: 02b21277-492f-4a76-9d9e-b71ea88e72ad
prompt-select-option = Select an option

# Prompt message to pick an amount of energy
# ID: 165d5300-2f77-4c23-9b36-217902fab895
prompt-choose-energy-amount = Choose energy amount

# Prompt message to pick card ordering within the deck
# ID: 4b770aa5-47e8-4dab-b879-c2d3aaa40ba2
prompt-select-card-order = Select card position

# Prompt message to pick a mode of a modal card to play
# ID: df13df4b-c876-48ec-aa81-5e25b0c6c9cb
prompt-pick-mode = Choose a mode

# Dev menu button label
# ID: e0dd5336-cb19-49f5-ad50-aa3a61823405
dev-menu-button = {bug-icon} Dev

# Decline to take the action associated with a prompt
# ID: 6095730f-d43c-49cd-a5dc-2781882389ed
decline-prompt-button = Decline

# Choose to pay energy to take a prompt action
# ID: 211e9d51-07ed-4261-88ce-fbfeb3390449
pay-energy-prompt-button = Spend {$energy}{e}
    .column-1 = Spend {$energy}‎

# Button to confirm the amount of energy to pay as an additional cost
# ID: 3d41f282-892b-47e6-9e8c-b82ca534ca20
pay-energy-addtional-cost-button = Spend {$energy}{e}
    .column-1 = Spend {$energy}‎

# Button to confirm selection of target cards in the void
# ID: a637975a-fb02-40b1-8dbe-95f278867ef8
primary-button-submit-void-card-targets = Submit

# Button to confirm selection of target cards in the hand
# ID: be3fe1f9-17ed-4324-b1d9-f1d5e6ccf2c3
primary-button-submit-hand-card-targets = Submit

# Button to confirm selection of ordering of cards in deck
# ID: f03ee2d3-e278-40bc-a449-25571d36fa56
primary-button-submit-deck-card-order = Submit

# Button to resolve the top card of the stack
# ID: 5c8a0769-2507-4082-84c8-5d7fd69855d8
primary-button-resolve-stack = Resolve

# Button to end your turn
# ID: 08d4e22b-5eab-4354-9cb5-7ff68c3b5196
primary-button-end-turn = End Turn

# Button to end the opponent's turn and begin your turn
# ID: bb1acabc-e74c-44d2-a586-54689f5de23b
primary-button-start-next-turn = Next Turn

# Button to increment the energy amount in a prompt to pick an energy value
# ID: 010f152b-115a-4a52-bae0-3f9961b0990f
increment-energy-prompt-button = +1{e}
    .column-1 = +1‎

# Button to decrement the energy amount in a prompt to pick an energy value
# ID: 5632f153-07d7-41de-bc85-5864f5578d73
decrement-energy-prompt-button = -1{e}
    .column-1 = -1‎

# Button to hide the stack and view the battlefield
# ID: bec6ea4b-55b9-4eb9-8173-9dd4f03eaf05
hide-stack-button = {eye-icon}

# Button to show the stack after hiding it
# ID: e2cbff72-868a-4be2-a13a-37815ce0a5f2
show-stack-button = {eye-slash-icon}

# Addition to card rules text showing how much energy was spent on a card with a variable energy cost
# ID: 1231c8b5-de17-4cf3-b45d-f42d62143916
card-rules-text-energy-paid = ({$energy}{e} paid)
    .column-1 = ({$energy}‎ paid)

# Addition to card rules text showing that a card was played with the "reclaim" ability
# ID: dd5982c1-cfba-4608-a0ce-abf4257fcd5a
card-rules-text-reclaimed = (Reclaimed)

# Addition to card rules text showing that a card has been "anchored"
# ID: 6d3f9774-49a1-4263-b354-177dd069c329
card-rules-text-anchored = (Anchored)

# Card name for a card representing a numbered modal effect choice
# ID: 76e30436-5446-4b71-8189-439c843184ad
modal-effect-choice-card-name = Choice {$number}

# Card name for a card representing an ability of a character
# ID: 740ba650-6ee0-4fa2-9f6a-7c163454a191
character-ability-card-name = {$character-name} Ability

# Message describing the effects of exceeding the hand size limit
# ID: f144ac6d-bd7a-460c-9990-06812c084191
hand-size-limit-exceeded-warning-message = Note: Cards drawn in excess of 10 become {e} instead.
    .column-1 = Note: Cards drawn in excess of 10 become ‎ instead.

# Message describing the effects of exceeding the character limit
# ID: 9cffdaea-7a9f-4ca0-80af-6c414cb5c4f0
character-limit-exceeded-warning-message = Character limit exceeded: A character will be abandoned, with its spark permanently added to your total.

# Message describing the effects of exceeding both the character limit and the hand size limit
# ID: 59bc7390-ee3f-426c-8874-48d56d62d7ea
combined-limit-warning-message = Character limit exceeded: A character will be abandoned. Cards drawn in excess of 10 become {e} instead.
    .column-1 = Character limit exceeded: A character will be abandoned. Cards drawn in excess of 10 become ‎ instead.

# Title for a panel displaying an error message
# ID: a36c2696-4c39-49d5-9c63-3e5b7149cefb
error-message-panel-title = Error

# Card type for character cards
# ID: b7b16e85-8cbf-4433-8ee1-805858a10493
card-type-character = Character

# Card type for event cards
# ID: 11b07242-b514-42cb-aed9-a64992ac2e2b
card-type-event = Event

# Card type for dreamsign cards
# ID: 3876f34a-f8c3-49f3-bec5-5ae740e2978f
card-type-dreamsign = Dreamsign

# Card type for dreamcaller cards
# ID: a71f9d3b-ffc9-4edd-9be1-e998bdc5310a
card-type-dreamcaller = Dreamcaller

# Card type for dreamwell cards
# ID: a2bcbaa7-2167-47fc-9db7-5267a91b17c4
card-type-dreamwell = Dreamwell

# Card subtype, displayed on character cards
# ID: bee9a795-eb5a-46a1-aac5-692c7a070c64
card-subtype-ancient = Ancient

# Card subtype, displayed on character cards
# ID: 602ea4ae-0f44-4704-9cf4-c1ef70248642
card-subtype-child = Child

# Card subtype, displayed on character cards
# ID: 94f37651-7ffa-4bb1-97b9-40b5640db8ab
card-subtype-detective = Detective

# Card subtype, displayed on character cards
# ID: 9aa89193-4555-4906-9830-8c4584d4e882
card-subtype-enigma = Enigma

# Card subtype, displayed on character cards
# ID: 45d5bee1-12b7-4150-983c-4c4ce721d091
card-subtype-explorer = Explorer

# Card subtype, displayed on character cards
# ID: 411b42ec-fa60-44a0-8282-35b9f1e6dcd8
card-subtype-hacker = Hacker

# Card subtype, displayed on character cards
# ID: 0c84be77-7bac-4820-a5c2-7b648522db47
card-subtype--mage = Mage

# Card subtype, displayed on character cards
# ID: 7947e5c2-bf87-4840-90be-4bbff0639b5f
card-subtype-monster = Monster

# Card subtype, displayed on character cards
# ID: bcc76e70-cb45-4667-8e89-7b2dacb81ebc
card-subtype-musician = Musician

# Card subtype, displayed on character cards
# ID: f786771e-b0fa-483a-b773-72fc9e9b56a5
card-subtype-outsider = Outsider

# Card subtype, displayed on character cards
# ID: f8032816-7252-4c33-8ce4-ffd3ba279a2a
card-subtype-renegade = Renegade

# Card subtype, displayed on character cards
# ID: 60921d72-fde8-4958-b425-9bdc0964b0a3
card-subtype-spirit-animal = Spirit Animal

# Card subtype, displayed on character cards
# ID: 1a6221e0-a5bd-4a48-8629-51a2796ad166
card-subtype-super = Super

# Card subtype, displayed on character cards
# ID: 649edeb4-6c1d-48f4-bb39-8e01caba3d2a
card-subtype-survivor = Survivor

# Card subtype, displayed on character cards
# ID: 82f771ba-f7d6-4718-9e26-a9ff3dcac12f
card-subtype-synth = Synth

# Card subtype, displayed on character cards
# ID: 40e3fefd-b41e-4dfc-a467-0c59766116c8
card-subtype-tinkerer = Tinkerer

# Card subtype, displayed on character cards
# ID: f29c168c-6f67-4682-b948-dddd535fc467
card-subtype-trooper = Trooper

# Card subtype, displayed on character cards
# ID: 42abc8f3-b741-4d49-97b1-d96a29563f18
card-subtype-visionary = Visionary

# Card subtype, displayed on character cards
# ID: 727a0d85-e67d-49c6-920c-95619b488402
card-subtype-visitor = Visitor

# Card subtype, displayed on character cards
# ID: 68338f97-ffd9-4082-8c5a-41091323e8cb
card-subtype-warrior = Warrior

# Info zoom help text, displayed on a tooltip to describe card abilities
# ID: a4e6687d-4c0e-4de9-b3aa-1e49e09b4668
help-text-dissolve = {Dissolve}: Send a character to the void

# Info zoom help text, displayed on a tooltip to describe card abilities
# ID: c58bd162-3392-444f-868e-488c023e9e91
help-text-prevent = {Prevent}: Send a card to the void in response to it being played

# Info zoom help text, displayed on a tooltip to describe card abilities
# ID: 893f14c2-d7d6-4adf-a97a-66847febe6ed
help-text-foresee-1 = {keyword}Foresee{end-keyword} 1: Look at the top card of your deck. You may put it into your void.

# Info zoom help text, displayed on a tooltip to describe card abilities
# ID: 283a1526-f6e7-4885-a000-1b0cd54a75ed
help-text-foresee-n = {keyword}Foresee{end-keyword} {$n}: Look at the top {$n} cards of your deck. You may put them into your void or put them back in any order.

# Info zoom help text, displayed on a tooltip to describe card abilities
# ID: 85f5f06f-417d-4bb7-bf99-51eef3d4a5ac
help-text-anchored = {Anchored}: Cannot be dissolved.

# Info zoom help text, displayed on a tooltip to describe card abilities
# ID: 3e724a9f-4eed-4044-85ee-12187f1aef65
help-text-reclaim-without-cost = {Reclaim}: You may play a card from your void, then banish it when it leaves play.

# Info zoom help text, displayed on a tooltip to describe card abilities
# ID: 7b13a745-c94b-437c-bc2d-73c64b479147
help-text-reclaim-with-cost = {keyword}Reclaim{end-keyword} {$e}{e}: You may play this card from your void for {$e}{e}, then banish it.
    .column-1 = {keyword}Reclaim{end-keyword} {$e}‎: You may play this card from your void for {$e}‎, then banish it.

# Card type for an activated ability card
# ID: cf017f6f-7cc5-4b40-8819-25ec7fb5acd2
token-type-activated-ability = Activated Ability

# Card type for a triggered ability card
# ID: f4fe91cb-3bb9-4b58-9949-6652a388c0c8
token-type-triggered-ability = Triggered Ability

# Card type for a reclaim ability card
# ID: 1dd37348-a684-4f5f-a8e3-87ecb7354c85
token-type-reclaim-ability = Reclaim Ability
