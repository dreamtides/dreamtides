use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::can_play_cards_data::{
    CanPlayCardsData, PlayCardsInvalidation, PlayCardsInvalidationFlag,
};
use core_data::types::PlayerName;

pub fn invalidate(battle: &mut BattleState, invalidation: PlayCardsInvalidation) {
    match invalidation {
        PlayCardsInvalidation::EnergyChanged(player) => {
            *battle.can_play_cards.player_mut(player) = compute_legal_cards(battle, player);
        }
        PlayCardsInvalidation::HandChanged(player) => {
            *battle.can_play_cards.player_mut(player) = compute_legal_cards(battle, player);
        }
        PlayCardsInvalidation::BattlefieldChanged(player) => {
            if should_invalidate(battle, player, PlayCardsInvalidationFlag::OwnerBattlefieldChanged)
            {
                *battle.can_play_cards.player_mut(player) = compute_legal_cards(battle, player);
            }
            if should_invalidate(
                battle,
                player.opponent(),
                PlayCardsInvalidationFlag::OpponentBattlefieldChanged,
            ) {
                *battle.can_play_cards.player_mut(player.opponent()) =
                    compute_legal_cards(battle, player.opponent());
            }
        }
        PlayCardsInvalidation::StackChanged => {
            if should_invalidate(battle, PlayerName::One, PlayCardsInvalidationFlag::StackChanged) {
                *battle.can_play_cards.player_mut(PlayerName::One) =
                    compute_legal_cards(battle, PlayerName::One);
            }
            if should_invalidate(battle, PlayerName::Two, PlayCardsInvalidationFlag::StackChanged) {
                *battle.can_play_cards.player_mut(PlayerName::Two) =
                    compute_legal_cards(battle, PlayerName::Two);
            }
        }
        PlayCardsInvalidation::VoidChanged(player) => {
            if should_invalidate(battle, player, PlayCardsInvalidationFlag::OwnerVoidChanged) {
                *battle.can_play_cards.player_mut(player) = compute_legal_cards(battle, player);
            }
        }
        PlayCardsInvalidation::PreventDissolveEffectToggled => {
            if should_invalidate(
                battle,
                PlayerName::One,
                PlayCardsInvalidationFlag::PreventDissolveEffectToggled,
            ) {
                *battle.can_play_cards.player_mut(PlayerName::One) =
                    compute_legal_cards(battle, PlayerName::One);
            }

            if should_invalidate(
                battle,
                PlayerName::Two,
                PlayCardsInvalidationFlag::PreventDissolveEffectToggled,
            ) {
                *battle.can_play_cards.player_mut(PlayerName::Two) =
                    compute_legal_cards(battle, PlayerName::Two);
            }
        }
    }
}

fn should_invalidate(
    battle: &BattleState,
    player: PlayerName,
    flag: PlayCardsInvalidationFlag,
) -> bool {
    battle.can_play_cards.player(player).invalidations.contains(flag)
}

fn compute_legal_cards(_battle: &BattleState, _player: PlayerName) -> CanPlayCardsData {
    todo!("")
}
