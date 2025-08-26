use tabula_data::localized_strings::StringId;
use uuid::uuid;

/// Energy icon
pub const E: StringId = StringId(uuid!("dbb75f1b-8c25-4c27-a598-7b300f5b5ca4"));

/// Fast icon
pub const F: StringId = StringId(uuid!("7b21533d-7f49-451c-bfa1-949ab28ed258"));

/// Activated icon
pub const A: StringId = StringId(uuid!("adc9a41d-fd82-49b5-a099-a9f10a73afad"));

/// Multi-activated icon
pub const MA: StringId = StringId(uuid!("121fc3af-6abd-4a90-9197-43c6e18eeca2"));

/// Fast + Activated icon
pub const FA: StringId = StringId(uuid!("e0eb0864-3150-4ac5-a903-258ada0f906a"));

/// Fast + Multi-Activated icon
pub const FMA: StringId = StringId(uuid!("1921caf1-b581-4feb-b956-b27a553cbc43"));

/// Dev menu icon
pub const BUG_ICON: StringId = StringId(uuid!("eeb2bbe9-1952-43fa-9ae6-12512d005829"));

/// Undo button icon
pub const UNDO_ICON: StringId = StringId(uuid!("d439176c-7a40-410b-808d-cc2be53116d7"));

/// Eye icon
pub const EYE_ICON: StringId = StringId(uuid!("ad291414-914c-4039-8f8f-8902b752bf74"));

/// Eye icon with slash through it
pub const EYE_SLASH_ICON: StringId = StringId(uuid!("2ca2c1dd-20f7-410b-9f5d-ce313a2d9175"));

/// "Star of life" icon used to represent non-numeric costs
pub const ASTERISK_ICON: StringId = StringId(uuid!("fd603588-7ab8-48c8-93b0-2bf6afc59325"));

/// Formatting that appears before keywords
pub const KEYWORD: StringId = StringId(uuid!("cb899350-def7-4466-a1e2-2a59e105d9fb"));

/// Formatting that appears after keywords
pub const END_KEYWORD: StringId = StringId(uuid!("bf435908-6161-4a6e-9407-81e0ccde2c4e"));

/// Formatting for an amount of energy paid as a cost
pub const ENERGY_COST: StringId = StringId(uuid!("cc5c5509-69c5-4c1f-8faa-b9948029dcb8"));

/// Formatting for a number of cards
pub const CARDS: StringId = StringId(uuid!("87fcba1e-5935-4de4-8048-81a343f072a5"));

/// Formatting for gaining an amount of spark
pub const GAINED_SPARK: StringId = StringId(uuid!("c49b155f-57a2-479a-bc45-254b2144d04b"));

/// Formatting to select a single mode of a modal card
pub const CHOOSE_ONE: StringId = StringId(uuid!("3057478f-1445-4b7d-8241-a90d97da7adf"));

/// Formatting that appears before an ability of a card with mutiple abilities
pub const ABILITY: StringId = StringId(uuid!("a8108996-e881-4b2e-80dd-3bfe5bf8dcd4"));

/// Formatting that appears after an ability of a card with multiple abilities
pub const END_ABILITY: StringId = StringId(uuid!("12487a98-0f31-4f71-bd51-9a85bfcd281c"));

/// Inserts a linebreak in card rules text
pub const BR: StringId = StringId(uuid!("e189f8a2-0901-41b9-9023-78dd196f60d5"));

/// Formatting that appears before a modal ability
pub const MODE: StringId = StringId(uuid!("3cc400d3-0594-4663-9187-e8c46613a21f"));

/// Formatting that appears after a modal ability
pub const END_MODE: StringId = StringId(uuid!("4a6ed656-0057-44c7-977d-8c0cedf1f725"));

/// Foresee keyword ability with quantity
pub const FORESEE: StringId = StringId(uuid!("2e44ee4c-3218-45bf-a29e-0508f853c873"));

/// Reclaim keyword ability without energy cost
pub const RECLAIM: StringId = StringId(uuid!("f19214ee-632b-4f0b-b3e4-d63e67b7ac03"));

/// Reclaim keyword ability with energy cost
pub const RECLAIM_COST: StringId = StringId(uuid!("fb895ce7-1f66-426d-8c29-b19fdeee7828"));

/// Kindle keyword ability with quantity
pub const KINDLE: StringId = StringId(uuid!("bf189fe0-75e2-4a87-9dca-67dd5f755766"));

/// Dissolve keyword ability
pub const DISSOLVE: StringId = StringId(uuid!("5771085d-3c89-423f-be05-c728035e7cd0"));

/// Prevent keyword ability
pub const PREVENT: StringId = StringId(uuid!("1a9e01e9-e7aa-4692-9b1b-5dd9e0b4a7f9"));

/// Anchored keyword ability
pub const ANCHORED: StringId = StringId(uuid!("901b84a1-c886-447b-bd0d-005e240a23de"));

/// Prompt message to target a character
pub const PROMPT_CHOOSE_CHARACTER: StringId =
    StringId(uuid!("66edfc8f-483b-4ad0-b37b-c66aa7c4347b"));

/// Prompt message to pick a card on the stack
pub const PROMPT_SELECT_STACK_CARD: StringId =
    StringId(uuid!("3f16a9f0-c905-4040-b4e3-58eafa3144f2"));

/// Prompt message to pick a card from your void
pub const PROMPT_SELECT_FROM_VOID: StringId =
    StringId(uuid!("95c481aa-46e6-4e03-ab9f-ae162207cd39"));

/// Prompt message to pick a choice among several options
pub const PROMPT_SELECT_OPTION: StringId = StringId(uuid!("02b21277-492f-4a76-9d9e-b71ea88e72ad"));

/// Prompt message to pick an amount of energy
pub const PROMPT_CHOOSE_ENERGY_AMOUNT: StringId =
    StringId(uuid!("165d5300-2f77-4c23-9b36-217902fab895"));

/// Prompt message to pick card ordering within the deck
pub const PROMPT_SELECT_CARD_ORDER: StringId =
    StringId(uuid!("4b770aa5-47e8-4dab-b879-c2d3aaa40ba2"));

/// Prompt message to pick a mode of a modal card to play
pub const PROMPT_PICK_MODE: StringId = StringId(uuid!("df13df4b-c876-48ec-aa81-5e25b0c6c9cb"));

/// Dev menu button label
pub const DEV_MENU_BUTTON: StringId = StringId(uuid!("e0dd5336-cb19-49f5-ad50-aa3a61823405"));

/// Decline to take the action associated with a prompt
pub const DECLINE_PROMPT_BUTTON: StringId = StringId(uuid!("6095730f-d43c-49cd-a5dc-2781882389ed"));

/// Choose to pay energy to take a prompt action
pub const PAY_ENERGY_PROMPT_BUTTON: StringId =
    StringId(uuid!("211e9d51-07ed-4261-88ce-fbfeb3390449"));

/// Button to confirm the amount of energy to pay as an additional cost
pub const PAY_ENERGY_ADDTIONAL_COST_BUTTON: StringId =
    StringId(uuid!("3d41f282-892b-47e6-9e8c-b82ca534ca20"));

/// Button to confirm selection of target cards in the void
pub const PRIMARY_BUTTON_SUBMIT_VOID_CARD_TARGETS: StringId =
    StringId(uuid!("a637975a-fb02-40b1-8dbe-95f278867ef8"));

/// Button to confirm selection of ordering of cards in deck
pub const PRIMARY_BUTTON_SUBMIT_DECK_CARD_ORDER: StringId =
    StringId(uuid!("f03ee2d3-e278-40bc-a449-25571d36fa56"));

/// Button to resolve the top card of the stack
pub const PRIMARY_BUTTON_RESOLVE_STACK: StringId =
    StringId(uuid!("5c8a0769-2507-4082-84c8-5d7fd69855d8"));

/// Button to end your turn
pub const PRIMARY_BUTTON_END_TURN: StringId =
    StringId(uuid!("08d4e22b-5eab-4354-9cb5-7ff68c3b5196"));

/// Button to end the opponent's turn and begin your turn
pub const PRIMARY_BUTTON_START_NEXT_TURN: StringId =
    StringId(uuid!("bb1acabc-e74c-44d2-a586-54689f5de23b"));

/// Button to increment the energy amount in a prompt to pick an energy value
pub const INCREMENT_ENERGY_PROMPT_BUTTON: StringId =
    StringId(uuid!("010f152b-115a-4a52-bae0-3f9961b0990f"));

/// Button to decrement the energy amount in a prompt to pick an energy value
pub const DECREMENT_ENERGY_PROMPT_BUTTON: StringId =
    StringId(uuid!("5632f153-07d7-41de-bc85-5864f5578d73"));

/// Button to hide the stack and view the battlefield
pub const HIDE_STACK_BUTTON: StringId = StringId(uuid!("bec6ea4b-55b9-4eb9-8173-9dd4f03eaf05"));

/// Button to show the stack after hiding it
pub const SHOW_STACK_BUTTON: StringId = StringId(uuid!("e2cbff72-868a-4be2-a13a-37815ce0a5f2"));

/// Addition to card rules text showing how much energy was spent on a card with
/// a variable energy cost
pub const CARD_RULES_TEXT_ENERGY_PAID: StringId =
    StringId(uuid!("1231c8b5-de17-4cf3-b45d-f42d62143916"));

/// Addition to card rules text showing that a card was played with the
/// "reclaim" ability
pub const CARD_RULES_TEXT_RECLAIMED: StringId =
    StringId(uuid!("dd5982c1-cfba-4608-a0ce-abf4257fcd5a"));

/// Addition to card rules text showing that a card has been "anchored"
pub const CARD_RULES_TEXT_ANCHORED: StringId =
    StringId(uuid!("6d3f9774-49a1-4263-b354-177dd069c329"));

/// Card name for a card representing a numbered modal effect choice
pub const MODAL_EFFECT_CHOICE_CARD_NAME: StringId =
    StringId(uuid!("76e30436-5446-4b71-8189-439c843184ad"));

/// Card name for a card representing an ability of a character
pub const CHARACTER_ABILITY_CARD_NAME: StringId =
    StringId(uuid!("740ba650-6ee0-4fa2-9f6a-7c163454a191"));

/// Message describing the effects of exceeding the hand size limit
pub const HAND_SIZE_LIMIT_EXCEEDED_WARNING_MESSAGE: StringId =
    StringId(uuid!("f144ac6d-bd7a-460c-9990-06812c084191"));

/// Message describing the effects of exceeding the character limit
pub const CHARACTER_LIMIT_EXCEEDED_WARNING_MESSAGE: StringId =
    StringId(uuid!("9cffdaea-7a9f-4ca0-80af-6c414cb5c4f0"));

/// Message describing the effects of exceeding both the character limit and the
/// hand size limit
pub const COMBINED_LIMIT_WARNING_MESSAGE: StringId =
    StringId(uuid!("59bc7390-ee3f-426c-8874-48d56d62d7ea"));

/// Title for a panel displaying an error message
pub const ERROR_MESSAGE_PANEL_TITLE: StringId =
    StringId(uuid!("a36c2696-4c39-49d5-9c63-3e5b7149cefb"));

/// Card type for character cards
pub const CARD_TYPE_CHARACTER: StringId = StringId(uuid!("b7b16e85-8cbf-4433-8ee1-805858a10493"));

/// Card type for event cards
pub const CARD_TYPE_EVENT: StringId = StringId(uuid!("11b07242-b514-42cb-aed9-a64992ac2e2b"));

/// Card type for dreamsign cards
pub const CARD_TYPE_DREAMSIGN: StringId = StringId(uuid!("3876f34a-f8c3-49f3-bec5-5ae740e2978f"));

/// Card type for dreamcaller cards
pub const CARD_TYPE_DREAMCALLER: StringId = StringId(uuid!("a71f9d3b-ffc9-4edd-9be1-e998bdc5310a"));

/// Card type for dreamwell cards
pub const CARD_TYPE_DREAMWELL: StringId = StringId(uuid!("a2bcbaa7-2167-47fc-9db7-5267a91b17c4"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_ANCIENT: StringId = StringId(uuid!("bee9a795-eb5a-46a1-aac5-692c7a070c64"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_CHILD: StringId = StringId(uuid!("602ea4ae-0f44-4704-9cf4-c1ef70248642"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_DETECTIVE: StringId =
    StringId(uuid!("94f37651-7ffa-4bb1-97b9-40b5640db8ab"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_ENIGMA: StringId = StringId(uuid!("9aa89193-4555-4906-9830-8c4584d4e882"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_EXPLORER: StringId = StringId(uuid!("45d5bee1-12b7-4150-983c-4c4ce721d091"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_HACKER: StringId = StringId(uuid!("411b42ec-fa60-44a0-8282-35b9f1e6dcd8"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_MAGE: StringId = StringId(uuid!("0c84be77-7bac-4820-a5c2-7b648522db47"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_MONSTER: StringId = StringId(uuid!("7947e5c2-bf87-4840-90be-4bbff0639b5f"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_MUSICIAN: StringId = StringId(uuid!("bcc76e70-cb45-4667-8e89-7b2dacb81ebc"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_OUTSIDER: StringId = StringId(uuid!("f786771e-b0fa-483a-b773-72fc9e9b56a5"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_RENEGADE: StringId = StringId(uuid!("f8032816-7252-4c33-8ce4-ffd3ba279a2a"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_SPIRIT_ANIMAL: StringId =
    StringId(uuid!("60921d72-fde8-4958-b425-9bdc0964b0a3"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_SUPER: StringId = StringId(uuid!("1a6221e0-a5bd-4a48-8629-51a2796ad166"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_SURVIVOR: StringId = StringId(uuid!("649edeb4-6c1d-48f4-bb39-8e01caba3d2a"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_SYNTH: StringId = StringId(uuid!("82f771ba-f7d6-4718-9e26-a9ff3dcac12f"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_TINKERER: StringId = StringId(uuid!("40e3fefd-b41e-4dfc-a467-0c59766116c8"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_TROOPER: StringId = StringId(uuid!("f29c168c-6f67-4682-b948-dddd535fc467"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_VISIONARY: StringId =
    StringId(uuid!("42abc8f3-b741-4d49-97b1-d96a29563f18"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_VISITOR: StringId = StringId(uuid!("727a0d85-e67d-49c6-920c-95619b488402"));

/// Card subtype, displayed on character cards
pub const CARD_SUBTYPE_WARRIOR: StringId = StringId(uuid!("68338f97-ffd9-4082-8c5a-41091323e8cb"));
