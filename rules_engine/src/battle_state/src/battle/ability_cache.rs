use std::fmt;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::battle::card_id::CardIdType;
use crate::battle_cards::ability_list::AbilityList;

pub struct AbilityCache {
    /// Cache of abilities for cards in this battle.
    ///
    /// Indices in this vector correspond to the `CardId` of each card in
    /// the battle.
    cache: RwLock<Vec<Arc<AbilityList>>>,
}

impl AbilityCache {
    pub fn get(&self, id: impl CardIdType) -> Arc<AbilityList> {
        self.cache.read()[id.card_id().0].clone()
    }

    pub fn append<I: IntoIterator<Item = Arc<AbilityList>>>(&self, lists: I) {
        let mut guard = self.cache.write();
        guard.extend(lists);
    }
}

impl Default for AbilityCache {
    fn default() -> Self {
        Self { cache: RwLock::new(Vec::new()) }
    }
}

impl fmt::Debug for AbilityCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AbilityCache").finish()
    }
}
