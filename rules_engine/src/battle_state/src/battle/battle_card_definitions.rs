use std::collections::BTreeMap;
use std::sync::Arc;

use core_data::types::PlayerName;
use quest_state::quest::deck::QuestDeckCardId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tabula_data::card_definition::CardDefinition;

use crate::battle::all_cards::CreatedCard;
use crate::battle_cards::ability_list::AbilityList;

/// Identifies a card with given rules text during a battle.
///
/// This is used as a key to look up the cached definition of a card's rules to
/// improve performance. Two cards with the same identity are guaranteed to be
/// identical in play.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct BattleCardIdentity(usize);

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct QuestDeckIdentity {
    pub id: QuestDeckCardId,
    pub owner: PlayerName,
}

/// Stores the [CardDefinition]s and [AbilityList]s for cards in this battle
/// keyed by their [BattleCardIdentity].
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BattleCardDefinitions {
    #[serde(skip)]
    cache_by_identity: Vec<Arc<AbilityList>>,
    #[serde(skip)]
    definitions_by_identity: Vec<Arc<CardDefinition>>,
    #[serde(default)]
    quest_deck_card_ids: BTreeMap<BattleCardIdentity, QuestDeckIdentity>,
}

/// Describes a card definition to add to the cache.
pub struct BattleCardDefinitionsCard {
    pub ability_list: Arc<AbilityList>,
    pub definition: Arc<CardDefinition>,
    pub quest_deck_card_id: QuestDeckCardId,
    pub owner: PlayerName,
}

/// Returns the result of a [BattleCardDefinitions::build] operation.
pub struct BattleCardDefinitionsResponse {
    pub cache: BattleCardDefinitions,
    pub created: Vec<CreatedCard>,
}

impl BattleCardDefinitions {
    pub fn get_ability_list(&self, identity: BattleCardIdentity) -> Arc<AbilityList> {
        self.cache_by_identity[identity.0].clone()
    }

    pub fn get_definition(&self, identity: BattleCardIdentity) -> Arc<CardDefinition> {
        self.definitions_by_identity[identity.0].clone()
    }

    /// Builds a new [BattleCardDefinitions] from a list of
    /// [BattleCardDefinitionsCard]s.
    pub fn build(cards: Vec<BattleCardDefinitionsCard>) -> BattleCardDefinitionsResponse {
        let initial_lists: Vec<Arc<AbilityList>> = Vec::new();
        let initial_defs: Vec<Arc<CardDefinition>> = Vec::new();
        let initial_map: BTreeMap<BattleCardIdentity, QuestDeckIdentity> = BTreeMap::new();
        Self::build_with_initial(&initial_lists, &initial_defs, &initial_map, cards)
    }

    /// Builds a new [BattleCardDefinitions] by appending a list of
    /// [BattleCardDefinitionsCard]s to this [BattleCardDefinitions].
    pub fn append(
        self: &BattleCardDefinitions,
        cards: Vec<BattleCardDefinitionsCard>,
    ) -> BattleCardDefinitionsResponse {
        Self::build_with_initial(
            &self.cache_by_identity,
            &self.definitions_by_identity,
            &self.quest_deck_card_ids,
            cards,
        )
    }

    /// Builds a new [BattleCardDefinitions] by repopulating the cached values
    /// using the provided builder functions.
    ///
    /// Card identities will be reconstructed based on the internal quest deck
    /// card ids.
    pub fn rebuild(
        definitions: &BattleCardDefinitions,
        definition_fn: impl Fn(QuestDeckCardId, PlayerName) -> Arc<CardDefinition>,
        ability_list_fn: impl Fn(&CardDefinition) -> AbilityList,
    ) -> BattleCardDefinitions {
        let mut cache_by_identity = vec![None; definitions.quest_deck_card_ids.len()];
        let mut definitions_by_identity = vec![None; definitions.quest_deck_card_ids.len()];
        for (identity, quest_deck_identity) in definitions.quest_deck_card_ids.iter() {
            let definition = definition_fn(quest_deck_identity.id, quest_deck_identity.owner);
            let ability_list = ability_list_fn(&definition);
            cache_by_identity[identity.0] = Some(Arc::new(ability_list));
            definitions_by_identity[identity.0] = Some(definition);
        }

        let ability_list = cache_by_identity.into_iter().flatten().collect::<Vec<_>>();
        assert_eq!(
            ability_list.len(),
            definitions.cache_by_identity.len(),
            "Ability list length mismatch"
        );
        let definition_list = definitions_by_identity.into_iter().flatten().collect::<Vec<_>>();
        assert_eq!(
            definition_list.len(),
            definitions.definitions_by_identity.len(),
            "Definition list length mismatch"
        );

        BattleCardDefinitions {
            cache_by_identity: ability_list,
            definitions_by_identity: definition_list,
            quest_deck_card_ids: definitions.quest_deck_card_ids.clone(),
        }
    }

    fn build_with_initial(
        initial_lists: &[Arc<AbilityList>],
        initial_defs: &[Arc<CardDefinition>],
        initial_map: &BTreeMap<BattleCardIdentity, QuestDeckIdentity>,
        cards: Vec<BattleCardDefinitionsCard>,
    ) -> BattleCardDefinitionsResponse {
        let start = initial_lists.len();
        let mut cache_by_identity = Vec::with_capacity(start + cards.len());
        let mut definitions_by_identity = Vec::with_capacity(start + cards.len());
        let mut quest_deck_card_ids = initial_map.clone();
        cache_by_identity.extend(initial_lists.iter().cloned());
        definitions_by_identity.extend(initial_defs.iter().cloned());

        let mut created = Vec::with_capacity(cards.len());
        for (i, card) in cards.into_iter().enumerate() {
            let identity = BattleCardIdentity(start + i);
            cache_by_identity.push(card.ability_list.clone());
            definitions_by_identity.push(card.definition.clone());
            quest_deck_card_ids.insert(identity, QuestDeckIdentity {
                id: card.quest_deck_card_id,
                owner: card.owner,
            });
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
            cache: BattleCardDefinitions {
                cache_by_identity,
                definitions_by_identity,
                quest_deck_card_ids,
            },
            created,
        }
    }
}
