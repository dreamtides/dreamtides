use core_data::identifiers::{BaseCardId, DreamwellCardId};
use uuid::uuid;

/// {Dissolve} an enemy character.
pub const IMMOLATE: BaseCardId = BaseCardId(uuid!("d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a"));

/// {Prevent} a played enemy card.
pub const ABOLISH: BaseCardId = BaseCardId(uuid!("d07ac4fa-cc3b-4bb8-8018-de7dc1760f35"));

/// {Prevent} a played enemy event unless the enemy pays {-energy-cost(e:2)}.
pub const RIPPLE_OF_DEFIANCE: BaseCardId =
    BaseCardId(uuid!("d4207a7b-fc36-45e4-a1ee-f01b34485221"));

/// Pay one or more {e}: Draw {-drawn-cards(n:1)} for each {e} spent.
pub const DREAMSCATTER: BaseCardId = BaseCardId(uuid!("15a39336-2c71-44d6-b462-d9fd23a4d925"));

/// Whenever you play a card during the enemy's turn, this character gains
/// {-gained-spark(n:1)}.
pub const SUNDOWN_SURFER: BaseCardId = BaseCardId(uuid!("3380cf41-2bca-468c-95e0-57abafd29430"));

/// {fma} {-energy-cost(e:3)}: Draw {-drawn-cards(n:1)}.
pub const MINSTREL_OF_FALLING_LIGHT: BaseCardId =
    BaseCardId(uuid!("86c79455-f9ba-46e4-80a6-e018f330942b"));

/// Return one or two events from your void to your hand.
pub const ARCHIVE_OF_THE_FORGOTTEN: BaseCardId =
    BaseCardId(uuid!("07f737af-dbaf-471a-8edc-e1d987c23903"));

/// {choose-one} {bullet} {-energy-cost(e:2)}: Return an enemy character to
/// hand. {bullet} {-energy-cost(e:3)}: Draw {-drawn-cards(n:2)}.
pub const BREAK_THE_SEQUENCE: BaseCardId =
    BaseCardId(uuid!("33c0db9c-666d-4b4c-a596-b74106025be8"));

/// Give an allied character {anchored} until end of turn.
pub const TOGETHER_AGAINST_THE_TIDE: BaseCardId =
    BaseCardId(uuid!("9866955c-31af-4aad-8319-a52d2fd85d0f"));

/// {-Foresee(n:1)}. Draw {-drawn-cards(n:1)}.  {-Reclaim-Cost(e:3)}
pub const GUIDING_LIGHT: BaseCardId = BaseCardId(uuid!("5e70988b-ce14-45a0-8334-7cf4539ee2d8"));

/// {Prevent} a played enemy character.
pub const CRAGFALL: BaseCardId = BaseCardId(uuid!("1d41a21a-1beb-4e56-8d7f-be29c4a9d43d"));

pub const TEST_VANILLA_CHARACTER: BaseCardId =
    BaseCardId(uuid!("253ee0ca-f973-4d9f-ad37-abe548bc674f"));

/// {Dissolve} an enemy character.
pub const TEST_DISSOLVE: BaseCardId = BaseCardId(uuid!("d4854b6e-5274-4f6a-8a60-a1ea1c15e9a6"));

/// {Prevent} a played enemy card.
pub const TEST_COUNTERSPELL: BaseCardId = BaseCardId(uuid!("aad836b0-3ece-477c-b923-b099360f0115"));

/// {Prevent} a played enemy event unless the enemy pays {-energy-cost(e:2)}.
pub const TEST_COUNTERSPELL_UNLESS_PAYS: BaseCardId =
    BaseCardId(uuid!("76b6d00c-5a28-4ee3-9655-e4fea1d8a4d8"));

/// Pay one or more {e}: Draw {-drawn-cards(n:1)} for each {e} spent.
pub const TEST_VARIABLE_ENERGY_DRAW: BaseCardId =
    BaseCardId(uuid!("e06a8cfe-483f-42c0-aac8-9c12b21b3f99"));

/// Whenever you play a card during the enemy's turn, this character gains
/// {-gained-spark(n:1)}.
pub const TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN: BaseCardId =
    BaseCardId(uuid!("86ee5ad7-b60b-4596-af8c-7a495022ac61"));

/// {fma} {-energy-cost(e:3)}: Draw {-drawn-cards(n:1)}.
pub const TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER: BaseCardId =
    BaseCardId(uuid!("d8a8541f-5b00-4d91-9518-aa8ae70ea450"));

/// Return one or two events from your void to your hand.
pub const TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND: BaseCardId =
    BaseCardId(uuid!("559e62a0-7ead-4136-8bd4-6cc58db4bef7"));

/// {choose-one} {bullet} {-energy-cost(e:2)}: Return an enemy character to
/// hand. {bullet} {-energy-cost(e:3)}: Draw {-drawn-cards(n:2)}.
pub const TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO: BaseCardId =
    BaseCardId(uuid!("e8f937da-cca7-447d-a559-530d7c339325"));

/// Give an allied character {anchored} until end of turn.
pub const TEST_PREVENT_DISSOLVE_THIS_TURN: BaseCardId =
    BaseCardId(uuid!("0b783ac8-6aea-438e-a2d5-87bf68548eda"));

/// {-Foresee(n:1)}. Draw {-drawn-cards(n:1)}.  {-Reclaim-Cost(e:3)}
pub const TEST_FORESEE_ONE_DRAW_RECLAIM: BaseCardId =
    BaseCardId(uuid!("de21db6c-54b4-4bff-b1d6-5a4711ef5ed8"));

/// {Prevent} a played enemy character.
pub const TEST_COUNTERSPELL_CHARACTER: BaseCardId =
    BaseCardId(uuid!("86ed0d3f-320b-49d0-b022-cd94aa07edbd"));

/// {Dissolve} an enemy character.
pub const TEST_NAMED_DISSOLVE: BaseCardId =
    BaseCardId(uuid!("3c1dbdc7-702e-4748-af3c-4fd837bcb404"));

/// Draw {-drawn-cards(n:1)}.
pub const TEST_DRAW_ONE: BaseCardId = BaseCardId(uuid!("68f90d08-9b51-424e-90d1-d15ddd1ece93"));

/// Whenever you materialize another character, this character gains
/// {-gained-spark(n:1)}.
pub const TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER: BaseCardId =
    BaseCardId(uuid!("91c9ed93-5faf-4178-aec9-d631bbcf5d6a"));

/// Whenever you play a card during the enemy's turn, this character gains
/// {-gained-spark(n:2)}.
pub const TEST_TRIGGER_GAIN_TWO_SPARK_ON_PLAY_CARD_ENEMY_TURN: BaseCardId =
    BaseCardId(uuid!("82759c0b-5161-4f6f-91b3-d42c2b4e0f9f"));

/// {a} {-energy-cost(e:1)}: Draw {-drawn-cards(n:1)}.
pub const TEST_ACTIVATED_ABILITY_DRAW_CARD: BaseCardId =
    BaseCardId(uuid!("8dfeb2c1-2d72-411c-a8cc-7f84ca532c63"));

/// {ma} {-energy-cost(e:1)}: Draw {-drawn-cards(n:1)}.
pub const TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER: BaseCardId =
    BaseCardId(uuid!("56f944bb-333b-4e2c-9c8c-2068f41998c2"));

/// {fa} {-energy-cost(e:1)}: Draw {-drawn-cards(n:1)}.
pub const TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER: BaseCardId =
    BaseCardId(uuid!("44aa4a1a-e8c6-4969-94bc-5fdbe010395e"));

/// {a} {-energy-cost(e:2)}: {Dissolve} an enemy character.
pub const TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER: BaseCardId =
    BaseCardId(uuid!("785e0341-fdd8-4e05-acb4-cbceed70ea6c"));

/// {a} {-energy-cost(e:1)}: Draw {-drawn-cards(n:1)}.  {a} {-energy-cost(e:2)}:
/// Draw {-drawn-cards(n:2)}.
pub const TEST_DUAL_ACTIVATED_ABILITY_CHARACTER: BaseCardId =
    BaseCardId(uuid!("3af84464-874a-4fd2-89cb-1986dee59ae1"));

/// {-Foresee(n:1)}.
pub const TEST_FORESEE_ONE: BaseCardId = BaseCardId(uuid!("8217b59b-6573-484b-9f3b-203e86e1d841"));

/// {-Foresee(n:2)}.
pub const TEST_FORESEE_TWO: BaseCardId = BaseCardId(uuid!("89e34264-a69c-48a4-867e-add7b811394b"));

/// {-Foresee(n:1)}. Draw {-drawn-cards(n:1)}.
pub const TEST_FORESEE_ONE_DRAW_A_CARD: BaseCardId =
    BaseCardId(uuid!("820faab3-37c1-46fa-a314-5f023ec739a1"));

/// Draw {-drawn-cards(n:1)}.  {-Reclaim-Cost(e:1)}
pub const TEST_DRAW_ONE_RECLAIM: BaseCardId =
    BaseCardId(uuid!("0cba1386-d1b6-4f57-8ccc-d92f8be01d7c"));

/// {-Foresee(n:1)}.  {-Reclaim-Cost(e:3)}
pub const TEST_FORESEE_ONE_RECLAIM: BaseCardId =
    BaseCardId(uuid!("86ffc58b-96db-4106-a892-8ae2a70719e6"));

/// Return a card from your void to your hand.
pub const TEST_RETURN_VOID_CARD_TO_HAND: BaseCardId =
    BaseCardId(uuid!("46e20fe4-36ca-438d-91a6-fac880ee9495"));

/// {choose-one} {bullet} {-energy-cost(e:1)}: Draw {-drawn-cards(n:1)}.
/// {bullet} {-energy-cost(e:3)}: Draw {-drawn-cards(n:2)}.
pub const TEST_MODAL_DRAW_ONE_OR_DRAW_TWO: BaseCardId =
    BaseCardId(uuid!("029889e9-25bc-438f-a492-8813febd65d8"));

/// {choose-one} {bullet} {-energy-cost(e:1)}: Draw {-drawn-cards(n:1)}.
/// {bullet} {-energy-cost(e:2)}: {Dissolve} an enemy character.
pub const TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY: BaseCardId =
    BaseCardId(uuid!("9847b3fc-1e7f-44e5-90af-1240ae12aaee"));

/// Return an enemy character to hand.
pub const TEST_RETURN_TO_HAND: BaseCardId =
    BaseCardId(uuid!("cf2f292b-f02c-4130-aff7-3f48fd147633"));

/// Gain {-gained-points(n: 2)}.
pub const TEST_GAIN_POINTS: BaseCardId = BaseCardId(uuid!("995b52a1-b368-4c83-ae05-8ab3800ca618"));

/// Gain {-gained-energy(e: 1)}.
pub const TEST_GAIN_ENERGY: BaseCardId = BaseCardId(uuid!("db191470-7a5b-4133-b731-8a81767d46e1"));

/// Put the top {-count(n: 3)} cards of your deck into your void.
pub const TEST_DECK_TO_VOID: BaseCardId = BaseCardId(uuid!("a7340f26-b759-458c-b8cb-b6fcc36fe412"));

/// Discard {-discarded-cards(n: 1)}.
pub const TEST_DISCARD: BaseCardId = BaseCardId(uuid!("6e76f193-dcf0-4faf-b1f7-50af2e0dc8a2"));

/// Discard {-discarded-cards(n: 2)}.
pub const TEST_DISCARD_TWO: BaseCardId = BaseCardId(uuid!("ef6d55f9-49ba-4637-af50-91068cb3a2b2"));

pub const DREAMWELL_PRODUCE_0: DreamwellCardId =
    DreamwellCardId(uuid!("146ae27e-a8ac-4f3c-aef2-cf2211e4bcfe"));

pub const DREAMWELL_PRODUCE_1: DreamwellCardId =
    DreamwellCardId(uuid!("ee7b0367-f7c3-46c3-94db-b29cfd8dc2d2"));

pub const DREAMWELL_PRODUCE_2_STARTER: DreamwellCardId =
    DreamwellCardId(uuid!("308fd4c0-ca98-4bfa-a9be-c29b36a145fd"));

/// {-Foresee(n:1)}.
pub const DREAMWELL_FORESEE: DreamwellCardId =
    DreamwellCardId(uuid!("40c77ea8-a021-4bc6-8970-0853c03f3fe0"));

/// Gain {-gained-points(n: 2)}.
pub const DREAMWELL_GAIN_POINTS: DreamwellCardId =
    DreamwellCardId(uuid!("d386663c-9e9f-4b8e-b410-f3467e39801b"));

/// Gain {-gained-energy(e: 1)}.
pub const DREAMWELL_GAIN_ENERGY: DreamwellCardId =
    DreamwellCardId(uuid!("107c3b3f-6131-4ff8-afcb-f0ce4188848f"));

/// Draw {-drawn-cards(n: 1)}. Discard {-discarded-cards(n: 1)}.
pub const DREAMWELL_DRAW_DISCARD: DreamwellCardId =
    DreamwellCardId(uuid!("40e4381f-12f7-46b9-ae50-67b3195781b1"));

/// Put the top {-count(n: 3)} cards of your deck into your void.
pub const DREAMWELL_MILL_3: DreamwellCardId =
    DreamwellCardId(uuid!("a2cdf115-8e1a-455e-a118-123f6f36c7ba"));

pub const ALL_TEST_CARD_IDS: &[BaseCardId] = &[
    IMMOLATE,
    ABOLISH,
    RIPPLE_OF_DEFIANCE,
    DREAMSCATTER,
    SUNDOWN_SURFER,
    MINSTREL_OF_FALLING_LIGHT,
    ARCHIVE_OF_THE_FORGOTTEN,
    BREAK_THE_SEQUENCE,
    TOGETHER_AGAINST_THE_TIDE,
    GUIDING_LIGHT,
    CRAGFALL,
    TEST_VANILLA_CHARACTER,
    TEST_DISSOLVE,
    TEST_COUNTERSPELL,
    TEST_COUNTERSPELL_UNLESS_PAYS,
    TEST_VARIABLE_ENERGY_DRAW,
    TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN,
    TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER,
    TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND,
    TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO,
    TEST_PREVENT_DISSOLVE_THIS_TURN,
    TEST_FORESEE_ONE_DRAW_RECLAIM,
    TEST_COUNTERSPELL_CHARACTER,
    TEST_NAMED_DISSOLVE,
    TEST_DRAW_ONE,
    TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER,
    TEST_TRIGGER_GAIN_TWO_SPARK_ON_PLAY_CARD_ENEMY_TURN,
    TEST_ACTIVATED_ABILITY_DRAW_CARD,
    TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER,
    TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER,
    TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER,
    TEST_DUAL_ACTIVATED_ABILITY_CHARACTER,
    TEST_FORESEE_ONE,
    TEST_FORESEE_TWO,
    TEST_FORESEE_ONE_DRAW_A_CARD,
    TEST_DRAW_ONE_RECLAIM,
    TEST_FORESEE_ONE_RECLAIM,
    TEST_RETURN_VOID_CARD_TO_HAND,
    TEST_MODAL_DRAW_ONE_OR_DRAW_TWO,
    TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY,
    TEST_RETURN_TO_HAND,
    TEST_GAIN_POINTS,
    TEST_GAIN_ENERGY,
    TEST_DECK_TO_VOID,
    TEST_DISCARD,
    TEST_DISCARD_TWO,
];

pub const ALL_TEST_DREAMWELL_CARD_IDS: &[DreamwellCardId] = &[
    DREAMWELL_PRODUCE_0,
    DREAMWELL_PRODUCE_1,
    DREAMWELL_PRODUCE_2_STARTER,
    DREAMWELL_FORESEE,
    DREAMWELL_GAIN_POINTS,
    DREAMWELL_GAIN_ENERGY,
    DREAMWELL_DRAW_DISCARD,
    DREAMWELL_MILL_3,
];
