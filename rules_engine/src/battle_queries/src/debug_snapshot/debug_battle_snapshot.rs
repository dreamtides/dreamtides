use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType, CharacterId, StackCardId};
use battle_state::battle_cards::stack_card_state::StackCardState;
use battle_state::battle_cards::zone::Zone;
use battle_state::battle_player::battle_player_state::BattlePlayerState;
use battle_state::debug::debug_all_cards::DebugAllCards;
use battle_state::debug::debug_battle_player_state::DebugBattlePlayerState;
use battle_state::debug::debug_battle_state::DebugBattleState;
use battle_state::debug::debug_card_state::{
    DebugCardProperties, DebugCardState, DebugStackCardState,
};
use battle_state::debug::debug_prompt_data::DebugPromptData;
use battle_state::prompt_types::prompt_data::{PromptData, PromptType};
use core_data::types::PlayerName;
use strum::IntoDiscriminant;

use crate::battle_card_queries::{card_abilities, card_properties};

/// Builds a human-readable representation of the state of the battle suitable
/// for use in logging & debugging.
pub fn capture(state: &BattleState) -> DebugBattleState {
    DebugBattleState {
        id: format!("{:?}", state.id),
        player_one: debug_player_state(PlayerName::One, &state.players.one),
        player_two: debug_player_state(PlayerName::Two, &state.players.two),
        cards: debug_all_cards(state),
        status: format!("{:?}", state.status),
        stack_priority: format!("{:?}", state.stack_priority),
        turn: format!("{:?}", state.turn),
        phase: format!("{:?}", state.phase),
        prompt: debug_prompt_data(&state.prompt),
    }
}

fn debug_player_state(name: PlayerName, state: &BattlePlayerState) -> DebugBattlePlayerState {
    DebugBattlePlayerState {
        name: format!("{:?}", name),
        points: format!("{:?}", state.points),
        current_energy: format!("{:?}", state.current_energy),
        produced_energy: format!("{:?}", state.produced_energy),
        spark_bonus: format!("{:?}", state.spark_bonus),
    }
}

fn debug_all_cards(battle: &BattleState) -> DebugAllCards {
    DebugAllCards {
        p1_battlefield: debug_zone(
            battle,
            Zone::Battlefield,
            PlayerName::One,
            battle.cards.battlefield(PlayerName::One).iter().map(|c| c.card_id()),
        ),
        p2_battlefield: debug_zone(
            battle,
            Zone::Battlefield,
            PlayerName::Two,
            battle.cards.battlefield(PlayerName::Two).iter().map(|c| c.card_id()),
        ),
        p1_void: debug_zone(
            battle,
            Zone::Void,
            PlayerName::One,
            battle.cards.void(PlayerName::One).iter().map(|c| c.card_id()),
        ),
        p2_void: debug_zone(
            battle,
            Zone::Void,
            PlayerName::Two,
            battle.cards.void(PlayerName::Two).iter().map(|c| c.card_id()),
        ),
        p1_hand: debug_zone(
            battle,
            Zone::Hand,
            PlayerName::One,
            battle.cards.hand(PlayerName::One).iter().map(|c| c.card_id()),
        ),
        p2_hand: debug_zone(
            battle,
            Zone::Stack,
            PlayerName::One,
            battle.cards.all_cards_on_stack().iter().map(|state| state.id.card_id()),
        ),
        p1_deck: debug_zone(
            battle,
            Zone::Deck,
            PlayerName::One,
            battle.cards.deck(PlayerName::One).iter().map(|c| c.card_id()),
        ),
        p2_deck: debug_zone(
            battle,
            Zone::Deck,
            PlayerName::Two,
            battle.cards.deck(PlayerName::Two).iter().map(|c| c.card_id()),
        ),
        stack: debug_zone(
            battle,
            Zone::Stack,
            PlayerName::One,
            battle.cards.all_cards_on_stack().iter().map(|state| state.id.card_id()),
        ),
        p1_banished: debug_zone(
            battle,
            Zone::Banished,
            PlayerName::One,
            battle.cards.banished(PlayerName::One).iter().map(|c| c.card_id()),
        ),
        p2_banished: debug_zone(
            battle,
            Zone::Banished,
            PlayerName::Two,
            battle.cards.banished(PlayerName::Two).iter().map(|c| c.card_id()),
        ),
    }
}

fn debug_zone(
    battle: &BattleState,
    zone: Zone,
    player: PlayerName,
    contents: impl Iterator<Item = CardId>,
) -> Vec<DebugCardState> {
    contents.map(|card_id| debug_card_state(battle, player, zone, card_id)).collect()
}

fn debug_card_state(
    battle: &BattleState,
    controller: PlayerName,
    current_zone: Zone,
    card_id: CardId,
) -> DebugCardState {
    DebugCardState {
        id: format!("{:?}", card_id),
        controller: format!("{:?}", controller),
        current_zone: format!("{:?}", current_zone),
        properties: DebugCardProperties {
            card_type: format!("{:?}", card_properties::card_type(battle, card_id)),
            spark: format!("{:?}", battle.cards.spark(controller, CharacterId(card_id))),
            cost: format!("{:?}", card_properties::cost(battle, card_id)),
            is_fast: format!("{:?}", card_properties::is_fast(battle, card_id)),
        },
        abilities: card_abilities::query(battle, card_id)
            .iter()
            .map(|(_, ability)| format!("{:?}", ability))
            .collect(),
        stack_state: debug_stack_card_state(battle.cards.stack_card(StackCardId(card_id))),
    }
}

fn debug_stack_card_state(state: Option<&StackCardState>) -> DebugStackCardState {
    let Some(state) = state else {
        return DebugStackCardState {
            has_stack_card_state: false,
            id: String::new(),
            controller: String::new(),
            targets: String::new(),
            additional_costs_paid: String::new(),
        };
    };

    DebugStackCardState {
        has_stack_card_state: true,
        id: format!("{:?}", state.id),
        controller: format!("{:?}", state.controller),
        targets: format!("{:?}", state.targets),
        additional_costs_paid: format!("{:?}", state.additional_costs_paid),
    }
}

fn debug_prompt_data(prompt: &Option<PromptData>) -> DebugPromptData {
    let Some(prompt_data) = prompt else {
        return DebugPromptData {
            is_active: false,
            player: String::new(),
            prompt_kind: String::new(),
            choices: Vec::new(),
            configuration: String::new(),
            context: String::new(),
        };
    };

    DebugPromptData {
        is_active: true,
        player: format!("{:?}", prompt_data.player),
        prompt_kind: format!("{:?}", prompt_data.prompt_type.discriminant()),
        choices: format_prompt_choices(&prompt_data.prompt_type),
        configuration: format!("{:?}", prompt_data.configuration),
        context: format!("{:?}", prompt_data.context),
    }
}

fn format_prompt_choices(prompt: &PromptType) -> Vec<String> {
    match prompt {
        PromptType::ChooseCharacter { valid } => {
            valid.iter().map(|id| format!("{:?}", id)).collect()
        }
        PromptType::ChooseStackCard { valid } => {
            valid.iter().map(|id| format!("{:?}", id)).collect()
        }
        PromptType::Choose { choices } => {
            choices.iter().map(|choice| format!("{:?}", choice)).collect()
        }
        PromptType::ChooseEnergyValue { minimum, current, maximum } => {
            vec![
                format!("min {}", minimum),
                format!("current {}", current),
                format!("max {}", maximum),
            ]
        }
    }
}
