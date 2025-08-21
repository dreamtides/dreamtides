use core_data::identifiers::BaseCardId;
use uuid::uuid;

pub const TEST_VANILLA_CHARACTER: BaseCardId =
    BaseCardId(uuid!("253ee0ca-f973-4d9f-ad37-abe548bc674f"));

/// {Dissolve} an enemy character.
pub const TEST_DISSOLVE: BaseCardId = BaseCardId(uuid!("d4854b6e-5274-4f6a-8a60-a1ea1c15e9a6"));

/// {Prevent} a played enemy card.
pub const TEST_COUNTERSPELL: BaseCardId = BaseCardId(uuid!("aad836b0-3ece-477c-b923-b099360f0115"));

/// {Prevent} a played enemy event unless the enemy pays {-energy-cost(e:2)}.
pub const TEST_COUNTERSPELL_UNLESS_PAYS: BaseCardId =
    BaseCardId(uuid!("76b6d00c-5a28-4ee3-9655-e4fea1d8a4d8"));

/// Pay one or more {e}: Draw {-cards(n:1)} for each {e} spent.
pub const TEST_VARIABLE_ENERGY_DRAW: BaseCardId =
    BaseCardId(uuid!("e06a8cfe-483f-42c0-aac8-9c12b21b3f99"));

/// Whenever you play a card during the enemy's turn, this character gains
/// {-gained-spark(n:1)}.
pub const TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN: BaseCardId =
    BaseCardId(uuid!("86ee5ad7-b60b-4596-af8c-7a495022ac61"));

/// {fma} {-energy-cost(e:3)}: Draw {-cards(n:1)}.
pub const TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER: BaseCardId =
    BaseCardId(uuid!("d8a8541f-5b00-4d91-9518-aa8ae70ea450"));

/// Return one or two events from your void to your hand.
pub const TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND: BaseCardId =
    BaseCardId(uuid!("559e62a0-7ead-4136-8bd4-6cc58db4bef7"));

/// {choose-one} {mode}{-energy-cost(e:2)}: Return an enemy character to
/// hand.{end-mode} {mode}{-energy-cost(e:3)}: Draw {-cards(n:2)}.{end-mode}
pub const TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO: BaseCardId =
    BaseCardId(uuid!("e8f937da-cca7-447d-a559-530d7c339325"));

/// Give an allied character {anchored} until end of turn.
pub const TEST_PREVENT_DISSOLVE_THIS_TURN: BaseCardId =
    BaseCardId(uuid!("0b783ac8-6aea-438e-a2d5-87bf68548eda"));

/// {ability}{-Foresee(n:1)}. Draw {-cards(n:1)}.{end-ability}
/// {ability}{-Reclaim-Cost(e:3)}{end-ability}
pub const TEST_FORESEE_ONE_DRAW_RECLAIM: BaseCardId =
    BaseCardId(uuid!("de21db6c-54b4-4bff-b1d6-5a4711ef5ed8"));

/// {Prevent} a played enemy character.
pub const TEST_COUNTERSPELL_CHARACTER: BaseCardId =
    BaseCardId(uuid!("86ed0d3f-320b-49d0-b022-cd94aa07edbd"));

/// {Dissolve} an enemy character.
pub const TEST_NAMED_DISSOLVE: BaseCardId =
    BaseCardId(uuid!("3c1dbdc7-702e-4748-af3c-4fd837bcb404"));

/// Draw {-cards(n:1)}.
pub const TEST_DRAW_ONE: BaseCardId = BaseCardId(uuid!("68f90d08-9b51-424e-90d1-d15ddd1ece93"));

/// Whenever you materialize another character, this character gains
/// {-gained-spark(n:1)}.
pub const TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER: BaseCardId =
    BaseCardId(uuid!("91c9ed93-5faf-4178-aec9-d631bbcf5d6a"));

/// Whenever you materialize another character, this character gains
/// {-gained-spark(n:2)}.
pub const TEST_TRIGGER_GAIN_TWO_SPARK_ON_PLAY_CARD_ENEMY_TURN: BaseCardId =
    BaseCardId(uuid!("82759c0b-5161-4f6f-91b3-d42c2b4e0f9f"));

/// {a} {-energy-cost(e:1)}: Draw {-cards(n:1)}.
pub const TEST_ACTIVATED_ABILITY_DRAW_CARD: BaseCardId =
    BaseCardId(uuid!("8dfeb2c1-2d72-411c-a8cc-7f84ca532c63"));

/// {ma} {-energy-cost(e:1)}: Draw {-cards(n:1)}.
pub const TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER: BaseCardId =
    BaseCardId(uuid!("56f944bb-333b-4e2c-9c8c-2068f41998c2"));

/// {fa} {-energy-cost(e:1)}: Draw {-cards(n:1)}.
pub const TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER: BaseCardId =
    BaseCardId(uuid!("44aa4a1a-e8c6-4969-94bc-5fdbe010395e"));

/// {a} {-energy-cost(e:2)}: {Dissolve} an enemy character.
pub const TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER: BaseCardId =
    BaseCardId(uuid!("785e0341-fdd8-4e05-acb4-cbceed70ea6c"));

/// {ability}{a} {-energy-cost(e:1)}: Draw {-cards(n:1)}.{end-ability}
/// {ability}{a} {-energy-cost(e:2)}: Draw {-cards(n:2)}.{end-ability}
pub const TEST_DUAL_ACTIVATED_ABILITY_CHARACTER: BaseCardId =
    BaseCardId(uuid!("3af84464-874a-4fd2-89cb-1986dee59ae1"));

/// {-Foresee(n:1)}.
pub const TEST_FORESEE_ONE: BaseCardId = BaseCardId(uuid!("8217b59b-6573-484b-9f3b-203e86e1d841"));

/// {-Foresee(n:2)}.
pub const TEST_FORESEE_TWO: BaseCardId = BaseCardId(uuid!("89e34264-a69c-48a4-867e-add7b811394b"));

/// {-Foresee(n:1)}. Draw {-cards(n:1)}.
pub const TEST_FORESEE_ONE_DRAW_A_CARD: BaseCardId =
    BaseCardId(uuid!("820faab3-37c1-46fa-a314-5f023ec739a1"));

/// {ability}Draw {-cards(n:1)}.{end-ability}
/// {ability}{-Reclaim-Cost(e:1)}{end-ability}
pub const TEST_DRAW_ONE_RECLAIM: BaseCardId =
    BaseCardId(uuid!("0cba1386-d1b6-4f57-8ccc-d92f8be01d7c"));

/// {ability}{-Foresee(n:1)}.{end-ability}
/// {ability}{-Reclaim-Cost(e:3)}{end-ability}
pub const TEST_FORESEE_ONE_RECLAIM: BaseCardId =
    BaseCardId(uuid!("86ffc58b-96db-4106-a892-8ae2a70719e6"));

/// Return a card from your void to your hand.
pub const TEST_RETURN_VOID_CARD_TO_HAND: BaseCardId =
    BaseCardId(uuid!("46e20fe4-36ca-438d-91a6-fac880ee9495"));

/// {choose-one} {mode}{-energy-cost(e:1)}: Draw {-cards(n:1)}.{end-mode}
/// {mode}{-energy-cost(e:3)}: Draw {-cards(n:2)}.{end-mode}
pub const TEST_MODAL_DRAW_ONE_OR_DRAW_TWO: BaseCardId =
    BaseCardId(uuid!("029889e9-25bc-438f-a492-8813febd65d8"));

/// {choose-one} {mode}{-energy-cost(e:1)}: Draw {-cards(n:1)}.{end-mode}
/// {mode}{-energy-cost(e:2)}: {Dissolve} an enemy character.{end-mode}
pub const TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY: BaseCardId =
    BaseCardId(uuid!("9847b3fc-1e7f-44e5-90af-1240ae12aaee"));

/// Return an enemy character to hand.
pub const TEST_RETURN_TO_HAND: BaseCardId =
    BaseCardId(uuid!("cf2f292b-f02c-4130-aff7-3f48fd147633"));
