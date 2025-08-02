use core_data::types::PlayerName;
use enumset::{EnumSet, EnumSetType};

use crate::battle::card_id::HandCardId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlayCardsInvalidation {
    EnergyChanged(PlayerName),
    HandChanged(PlayerName),
    BattlefieldChanged(PlayerName),
    StackChanged,
    VoidChanged(PlayerName),
    PreventDissolveEffectToggled,
}

#[derive(EnumSetType, Debug)]
pub enum PlayCardsInvalidationFlag {
    OwnerBattlefieldChanged,
    OpponentBattlefieldChanged,
    StackChanged,
    OwnerVoidChanged,
    PreventDissolveEffectToggled,
}

#[derive(Default, Clone, Debug)]
pub struct CanPlayCardsData {
    pub invalidations: EnumSet<PlayCardsInvalidationFlag>,
    pub play_from_hand: Vec<HandCardId>,
    pub play_from_hand_fast: Vec<HandCardId>,
}
