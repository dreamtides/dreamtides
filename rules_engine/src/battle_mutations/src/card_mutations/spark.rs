use battle_queries::battle_card_queries::card_properties;
use battle_queries::panic_with;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::Spark;

/// Adds `amount` to the spark of the character with the given ID.
///
/// Panics if the character is not on the battlefield.
pub fn gain(
    battle: &mut BattleState,
    source: EffectSource,
    character_id: CharacterId,
    amount: Spark,
) {
    battle.push_animation(source, || BattleAnimation::GainSpark { character_id, spark: amount });

    let Some(character_state) = battle
        .cards
        .battlefield_state_mut(card_properties::controller(battle, character_id))
        .get_mut(&character_id)
    else {
        panic_with!("Character not found on battlefield", battle, character_id);
    };
    character_state.spark += amount;
}
