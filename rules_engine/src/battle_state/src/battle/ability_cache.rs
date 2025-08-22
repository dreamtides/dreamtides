use std::sync::Arc;

use core_data::identifiers::CardIdentity;

use crate::battle_cards::ability_list::AbilityList;

#[derive(Default, Debug)]
pub struct AbilityCache {
    cache_by_identity: Vec<Arc<AbilityList>>,
}

impl AbilityCache {
    pub fn try_get_by_identity(&self, identity: CardIdentity) -> Option<Arc<AbilityList>> {
        self.cache_by_identity.get(identity.0).cloned()
    }

    pub fn from_pairs(pairs: Vec<(CardIdentity, Arc<AbilityList>)>) -> Self {
        let max = pairs.iter().map(|(id, _)| id.0).max().unwrap_or(0);
        let mut cache_by_identity = Vec::with_capacity(max + 1);
        cache_by_identity.resize_with(max + 1, || Arc::new(AbilityList::default()));
        for (identity, list) in pairs {
            cache_by_identity[identity.0] = list;
        }
        Self { cache_by_identity }
    }
}
