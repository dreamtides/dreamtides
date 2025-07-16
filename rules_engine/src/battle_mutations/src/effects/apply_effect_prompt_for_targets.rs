use ability_data::effect::Effect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use battle_state::core::effect_source::EffectSource;

/// Applies an effect to the [BattleState], prompting for effect targets if
/// required.
///
/// This is used when resolving triggered effects, prompt effects, 'if you do'
/// effects, and similar actions which happen on resolution. It is NOT used for
/// playing cards or activating abilities, which have their targets selected
/// before adding them to the stack.
///
/// The predicate "This" will be interpreted by this function as being the card
/// associated with the provided [EffectSource]. The predicate "That" will be
/// applied based on the `that_card` parameter to this function.
pub fn execute(
    _battle: &mut BattleState,
    _source: EffectSource,
    _effect: &Effect,
    _that_card: Option<CardId>,
) {
}
