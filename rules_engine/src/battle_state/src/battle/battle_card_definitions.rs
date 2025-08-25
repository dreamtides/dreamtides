use std::sync::Arc;

use core_data::identifiers::CardIdentity;
use serde::{Deserialize, Serialize};
use tabula_data::card_definition::CardDefinition;

use crate::battle::all_cards::CreatedCard;
use crate::battle_cards::ability_list::AbilityList;

/// Stores the [CardDefinition]s and [AbilityList]s for cards in this battle
/// keyed by their [CardIdentity].
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BattleCardDefinitions {
    #[serde(default)]
    cache_by_identity: Vec<Arc<AbilityList>>,
    #[serde(default)]
    definitions_by_identity: Vec<Arc<CardDefinition>>,
}

/// Describes a card definition to add to the cache.
pub struct BattleCardDefinitionsCard {
    pub ability_list: Arc<AbilityList>,
    pub definition: Arc<CardDefinition>,
}

/// Returns the result of a [BattleCardDefinitions::build] operation.
pub struct BattleCardDefinitionsResponse {
    pub cache: BattleCardDefinitions,
    pub created: Vec<CreatedCard>,
}

impl BattleCardDefinitions {
    pub fn get_ability_list(&self, identity: CardIdentity) -> Arc<AbilityList> {
        self.cache_by_identity[identity.0].clone()
    }

    pub fn get_definition(&self, identity: CardIdentity) -> Arc<CardDefinition> {
        self.definitions_by_identity[identity.0].clone()
    }

    /// Builds a new [BattleCardDefinitions] from a list of
    /// [BattleCardDefinitionsCard]s.
    pub fn build(cards: Vec<BattleCardDefinitionsCard>) -> BattleCardDefinitionsResponse {
        let initial_lists: Vec<Arc<AbilityList>> = Vec::new();
        let initial_defs: Vec<Arc<CardDefinition>> = Vec::new();
        Self::build_with_initial(&initial_lists, &initial_defs, cards)
    }

    /// Builds a new [BattleCardDefinitions] by appending a list of
    /// [BattleCardDefinitionsCard]s to this [BattleCardDefinitions].
    pub fn append(
        self: &BattleCardDefinitions,
        cards: Vec<BattleCardDefinitionsCard>,
    ) -> BattleCardDefinitionsResponse {
        Self::build_with_initial(&self.cache_by_identity, &self.definitions_by_identity, cards)
    }

    fn build_with_initial(
        initial_lists: &[Arc<AbilityList>],
        initial_defs: &[Arc<CardDefinition>],
        cards: Vec<BattleCardDefinitionsCard>,
    ) -> BattleCardDefinitionsResponse {
        let start = initial_lists.len();
        let mut cache_by_identity = Vec::with_capacity(start + cards.len());
        let mut definitions_by_identity = Vec::with_capacity(start + cards.len());
        cache_by_identity.extend(initial_lists.iter().cloned());
        definitions_by_identity.extend(initial_defs.iter().cloned());

        let mut created = Vec::with_capacity(cards.len());
        for (i, card) in cards.into_iter().enumerate() {
            let identity = CardIdentity(start + i);
            cache_by_identity.push(card.ability_list.clone());
            definitions_by_identity.push(card.definition.clone());
            created.push(CreatedCard {
                identity,
                can_play_restriction: card.ability_list.can_play_restriction,
                base_energy_cost: card.definition.energy_cost,
                base_spark: card.definition.spark,
                card_type: card.definition.card_type,
                is_fast: card.definition.is_fast,
            });
        }

        BattleCardDefinitionsResponse {
            cache: BattleCardDefinitions { cache_by_identity, definitions_by_identity },
            created,
        }
    }
}
