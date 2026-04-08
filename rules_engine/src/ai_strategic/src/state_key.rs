use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::{StackCardAdditionalCostsPaid, StackItemState};
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::types::PlayerName;

pub fn key(battle: &BattleState) -> u64 {
    let mut hasher = DefaultHasher::new();

    battle.status.is_game_over().hash(&mut hasher);
    battle.turn.active_player.hash(&mut hasher);
    battle.turn.turn_id.hash(&mut hasher);
    battle.turn.judgment_position.hash(&mut hasher);
    battle.turn.positioning_started.hash(&mut hasher);
    battle.turn.positioning_character.hash(&mut hasher);
    battle.turn.moved_this_turn.hash(&mut hasher);
    battle.stack_priority.hash(&mut hasher);
    battle.phase.hash(&mut hasher);

    for player in [PlayerName::One, PlayerName::Two] {
        let player_state = battle.players.player(player);
        player_state.points.hash(&mut hasher);
        player_state.current_energy.hash(&mut hasher);
        player_state.produced_energy.hash(&mut hasher);
        battle.cards.hand(player).len().hash(&mut hasher);
        battle.cards.void(player).len().hash(&mut hasher);
        battle.cards.top_of_deck(player).len().hash(&mut hasher);
        battle.cards.shuffled_into_deck(player).len().hash(&mut hasher);
        battle.cards.hand(player).iter().collect::<Vec<_>>().hash(&mut hasher);
        battle.cards.void(player).iter().collect::<Vec<_>>().hash(&mut hasher);

        let battlefield = battle.cards.battlefield(player);
        battlefield.front.hash(&mut hasher);
        battlefield.back.hash(&mut hasher);

        for character_id in battlefield.all_characters() {
            if let Some(state) = battle.cards.battlefield_state(player).get(&character_id) {
                state.spark.hash(&mut hasher);
                state.played_turn.hash(&mut hasher);
            }
        }
    }

    battle.cards.all_items_on_stack().len().hash(&mut hasher);
    for item in battle.cards.all_items_on_stack() {
        item.controller.hash(&mut hasher);
        item.id.hash(&mut hasher);
        item.additional_costs_paid_hash(&mut hasher);
        item.targets_hash(&mut hasher);
        item.modal_choice.hash(&mut hasher);
    }

    battle.prompts.len().hash(&mut hasher);
    if let Some(prompt) = battle.prompts.front() {
        prompt.player.hash(&mut hasher);
        prompt.configuration.optional.hash(&mut hasher);
        prompt.prompt_description.hash(&mut hasher);
        hash_prompt_type(&prompt.prompt_type, &mut hasher);
    }

    hasher.finish()
}

trait StrategicHashExt {
    fn additional_costs_paid_hash(&self, hasher: &mut DefaultHasher);
    fn targets_hash(&self, hasher: &mut DefaultHasher);
}

impl StrategicHashExt for StackItemState {
    fn additional_costs_paid_hash(&self, hasher: &mut DefaultHasher) {
        match &self.additional_costs_paid {
            StackCardAdditionalCostsPaid::None => {
                0u8.hash(hasher);
            }
            StackCardAdditionalCostsPaid::Energy(energy) => {
                1u8.hash(hasher);
                energy.hash(hasher);
            }
        }
    }

    fn targets_hash(&self, hasher: &mut DefaultHasher) {
        match &self.targets {
            None => 0u8.hash(hasher),
            Some(targets) => {
                1u8.hash(hasher);
                targets.card_ids().hash(hasher);
            }
        }
    }
}

fn hash_prompt_type(prompt_type: &PromptType, hasher: &mut DefaultHasher) {
    match prompt_type {
        PromptType::ChooseCharacter { valid, .. } => {
            0u8.hash(hasher);
            valid.hash(hasher);
        }
        PromptType::ChooseStackCard { valid, .. } => {
            1u8.hash(hasher);
            valid.hash(hasher);
        }
        PromptType::ChooseVoidCard(prompt) => {
            2u8.hash(hasher);
            prompt.valid.hash(hasher);
            prompt.selected.hash(hasher);
            prompt.maximum_selection.hash(hasher);
        }
        PromptType::ChooseHandCards(prompt) => {
            3u8.hash(hasher);
            prompt.valid.hash(hasher);
            prompt.selected.hash(hasher);
            prompt.maximum_selection.hash(hasher);
        }
        PromptType::Choose { choices } => {
            4u8.hash(hasher);
            choices.len().hash(hasher);
        }
        PromptType::ChooseEnergyValue { minimum, maximum } => {
            5u8.hash(hasher);
            minimum.hash(hasher);
            maximum.hash(hasher);
        }
        PromptType::ModalEffect(prompt) => {
            6u8.hash(hasher);
            prompt.choices.len().hash(hasher);
        }
        PromptType::ChooseActivatedAbility { character_id, abilities } => {
            7u8.hash(hasher);
            character_id.hash(hasher);
            abilities.len().hash(hasher);
        }
        PromptType::SelectDeckCardOrder { prompt } => {
            8u8.hash(hasher);
            prompt.initial.hash(hasher);
            prompt.moved.hash(hasher);
            prompt.deck.hash(hasher);
            prompt.void.hash(hasher);
        }
    }
}
