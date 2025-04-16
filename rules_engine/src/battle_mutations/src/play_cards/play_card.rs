use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::{ObjectId, StackCardId};
use battle_data::battle_cards::zone::Zone;
use core_data::effect_source::EffectSource;
use core_data::identifiers::CardId;
use core_data::types::PlayerName;

use crate::play_cards::target_prompt::add_target_prompt;
use crate::player_mutations::energy;
use crate::zone_mutations::move_card;

/// Attempts to play a card to the stack as `player` by paying its costs. If the
/// card requires targets, a prompt for valid targets will be added.
///
/// Returns the [ObjectId] of the card in its new zone if the card was played
/// successfully, otherwise returns `None`, e.g. if this card is prevented from
/// being played or no longer exists.
pub fn execute(
    battle: &mut BattleData,
    player: PlayerName,
    source: EffectSource,
    card_id: CardId,
) -> Option<ObjectId> {
    if let Some(energy_cost) = battle.cards.card(card_id)?.properties.cost {
        energy::spend(battle, player, source, energy_cost);
    }
    battle.cards.card_mut(card_id)?.revealed_to_opponent = true;
    let object_id = move_card::run(battle, source, card_id, Zone::Stack)?;
    add_target_prompt(battle, source, StackCardId::new(object_id, card_id));
    Some(object_id)
}
