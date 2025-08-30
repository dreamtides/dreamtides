use ability_data::effect::ModelEffectChoiceIndex;
use core_data::identifiers::AbilityNumber;
use core_data::numerics::{Energy, Points, Spark};
use core_data::types::PlayerName;
use strum::{Display, EnumDiscriminants};

use crate::battle::card_id::{
    ActivatedAbilityId, BattleDeckCardId, CardId, CharacterId, HandCardId,
};
use crate::battle_cards::stack_card_state::{EffectTargets, StackItemId};
use crate::battle_cards::zone::Zone;
use crate::prompt_types::prompt_data::PromptChoiceLabel;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum TargetedEffectName {
    Counterspell,
    Dissolve,
    ReturnToHand,
}

/// Records events during rules engine execution for display as game animations.
#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(Display))]
pub enum BattleAnimation {
    ActivatedAbility {
        player: PlayerName,
        activated_ability_id: ActivatedAbilityId,
    },
    ApplyTargetedEffect {
        effect_name: TargetedEffectName,
        targets: Vec<CardId>,
    },
    DrawCards {
        player: PlayerName,
        cards: Vec<HandCardId>,
    },
    DreamwellActivation {
        player: PlayerName,
        dreamwell_card_id: CardId,
        new_energy: Energy,
        new_produced_energy: Energy,
    },
    GainSpark {
        character_id: CharacterId,
        spark: Spark,
    },
    Judgment {
        player: PlayerName,
        new_score: Option<Points>,
    },
    MakeChoice {
        player: PlayerName,
        choice: PromptChoiceLabel,
    },
    PlayCard {
        player: PlayerName,
        card_id: CardId,
        from_zone: Zone,
    },
    PlayedCard {
        player: PlayerName,
        card_id: CardId,
        from_zone: Zone,
    },
    PreventedEffect {
        card_id: CardId,
    },
    PutCardsFromDeckIntoVoid {
        player: PlayerName,
        cards: Vec<BattleDeckCardId>,
    },
    ResolveCharacter {
        character_id: CharacterId,
    },
    SelectModalEffectChoice {
        player: PlayerName,
        item_id: StackItemId,
        choice_index: ModelEffectChoiceIndex,
    },
    SelectedTargetsForCard {
        player: PlayerName,
        source_id: StackItemId,
        targets: EffectTargets,
    },
    SetActiveTriggers {
        triggers: Vec<TriggerAnimation>,
    },
    StartTurn {
        player: PlayerName,
    },
}

/// A trigger which is currently awaiting resolution.
///
/// The display layer keeps track of the set of currently active triggers and
/// displays them visually while other effects are animating.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TriggerAnimation {
    pub controller: PlayerName,
    pub character_id: CharacterId,
    pub ability_number: AbilityNumber,
}
