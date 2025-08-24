use std::sync::Arc;

use core_data::identifiers::CardIdentity;
use tabula_data::card_definition::CardDefinition;

use crate::battle::all_cards::CreatedCard;
use crate::battle_cards::ability_list::AbilityList;

/// Stores the [CardDefinition] and [AbilityList]s for cards in this battle
/// keyed by their [CardIdentity].
#[derive(Default, Debug)]
pub struct AbilityCache {
    cache_by_identity: Vec<Arc<AbilityList>>,
    definitions_by_identity: Vec<Arc<CardDefinition>>,
}

/// Describes a card definition to add to the cache.
pub struct AbilityCacheCard {
    pub ability_list: Arc<AbilityList>,
    pub definition: Arc<CardDefinition>,
}

/// Returns the result of a [AbilityCache::build] operation.
pub struct AbilityCacheResponse {
    pub cache: AbilityCache,
    pub created: Vec<CreatedCard>,
}

impl AbilityCache {
    pub fn try_get_by_identity(&self, identity: CardIdentity) -> Option<Arc<AbilityList>> {
        self.cache_by_identity.get(identity.0).cloned()
    }

    pub fn try_get_definition(&self, identity: CardIdentity) -> Option<Arc<CardDefinition>> {
        self.definitions_by_identity.get(identity.0).cloned()
    }

    /// Builds a new [AbilityCache] from a list of [AbilityCacheCard]s.
    pub fn build(cards: Vec<AbilityCacheCard>) -> AbilityCacheResponse {
        let initial_lists: Vec<Arc<AbilityList>> = Vec::new();
        let initial_defs: Vec<Arc<CardDefinition>> = Vec::new();
        Self::build_with_initial(&initial_lists, &initial_defs, cards)
    }

    /// Builds a new [AbilityCache] by appending a list of [AbilityCacheCard]s
    /// to this [AbilityCache].
    pub fn append(self: &AbilityCache, cards: Vec<AbilityCacheCard>) -> AbilityCacheResponse {
        Self::build_with_initial(&self.cache_by_identity, &self.definitions_by_identity, cards)
    }

    fn build_with_initial(
        initial_lists: &[Arc<AbilityList>],
        initial_defs: &[Arc<CardDefinition>],
        cards: Vec<AbilityCacheCard>,
    ) -> AbilityCacheResponse {
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

        AbilityCacheResponse {
            cache: AbilityCache { cache_by_identity, definitions_by_identity },
            created,
        }
    }
}
