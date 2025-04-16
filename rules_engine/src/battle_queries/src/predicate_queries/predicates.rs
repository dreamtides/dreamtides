use ability_data::predicate::{CardPredicate, Predicate};
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::CharacterId;
use core_data::effect_source::EffectSource;
use core_data::types::PlayerName;

/// Returns the set of characters on the battlefield matching this `predicate`.
pub fn matching_characters(
    battle: &BattleData,
    controller: PlayerName,
    source: EffectSource,
    predicate: Predicate,
) -> Vec<CharacterId> {
    match predicate {
        Predicate::Enemy(card_predicate) => on_battlefield(
            battle,
            controller,
            source,
            battle.cards.battlefield(controller.opponent()).iter().cloned().collect::<Vec<_>>(),
            card_predicate,
        ),
        _ => todo!("Implement {:?}", predicate),
    }
}

/// Returns the set of characters on the battlefield from `collection` which
/// match `predicate`.
fn on_battlefield(
    _battle: &BattleData,
    _controller: PlayerName,
    _source: EffectSource,
    collection: Vec<CharacterId>,
    predicate: CardPredicate,
) -> Vec<CharacterId> {
    match predicate {
        CardPredicate::Card | CardPredicate::Character => collection,
        _ => todo!("Implement {:?}", predicate),
    }
}
