use std::sync::Arc;

use core_data::identifiers::CardIdentity;
use tabula_data::card_definition::CardDefinition;

use crate::battle_cards::ability_list::AbilityList;

#[derive(Default, Debug)]
pub struct AbilityCache {
    cache_by_identity: Vec<Arc<AbilityList>>,
    definitions_by_identity: Vec<Arc<CardDefinition>>,
}

impl AbilityCache {
    pub fn try_get_by_identity(&self, identity: CardIdentity) -> Option<Arc<AbilityList>> {
        self.cache_by_identity.get(identity.0).cloned()
    }

    pub fn try_get_definition(&self, identity: CardIdentity) -> Option<Arc<CardDefinition>> {
        self.definitions_by_identity.get(identity.0).cloned()
    }

    pub fn from_pairs(
        mut pairs: Vec<(CardIdentity, Arc<AbilityList>, Arc<CardDefinition>)>,
    ) -> Self {
        if pairs.is_empty() {
            return Self::default();
        }

        pairs.sort_by_key(|(id, _, _)| id.0);
        for (index, (id, _, _)) in pairs.iter().enumerate() {
            if id.0 != index {
                panic!(
                    "AbilityCache::from_pairs expected sequential identities starting at 0; found gap at index {index} (id {id:?})"
                );
            }
        }

        let mut cache_by_identity = Vec::with_capacity(pairs.len());
        let mut definitions_by_identity = Vec::with_capacity(pairs.len());
        for (_, list, definition) in pairs {
            cache_by_identity.push(list);
            definitions_by_identity.push(definition);
        }
        Self { cache_by_identity, definitions_by_identity }
    }
}
