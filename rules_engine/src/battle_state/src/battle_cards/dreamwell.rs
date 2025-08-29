use std::sync::Arc;

use ability_data::ability::{Ability, EventAbility};
use core_data::identifiers::AbilityNumber;
use core_data::numerics::Energy;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use tabula_data::card_definitions::dreamwell_card_definition::DreamwellCardDefinition;
use tabula_data::tabula::Tabula;
use tabula_ids::card_lists::{self, DreamwellCardIdList};

use crate::battle_cards::ability_list::AbilityData;

#[derive(Debug, Clone, Serialize)]
pub struct DreamwellCard {
    pub definition: DreamwellCardDefinition,
    #[serde(skip)]
    pub produced_energy: Energy,
    #[serde(skip)]
    pub effects: Vec<AbilityData<EventAbility>>,
}

impl<'de> Deserialize<'de> for DreamwellCard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DreamwellBattleCardSerde {
            definition: DreamwellCardDefinition,
        }

        let s = DreamwellBattleCardSerde::deserialize(deserializer)?;
        Ok(build_card(s.definition))
    }
}

/// The dreamwell is a deck of cards that is used during the dreamwell phase to
/// give players energy production and apply random effects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dreamwell {
    /// Cards in the dreamwell.
    #[serde(default)]
    pub cards: Arc<Vec<Arc<DreamwellCard>>>,

    /// Index of the next card to be drawn from the dreamwell.
    #[serde(default)]
    pub next_index: usize,
}

impl Dreamwell {
    /// Creates a new dreamwell from a [DreamwellCardIdList].
    pub fn from_card_list(tabula: &Tabula, list: DreamwellCardIdList) -> Self {
        let mut cards = Vec::new();
        for card_id in card_lists::dreamwell_card_id_list(list) {
            cards.push(Arc::new(build_card(
                tabula
                    .dreamwell_cards
                    .get(card_id)
                    .unwrap_or_else(|| panic!("Card {card_id:?} not found in tabula"))
                    .clone(),
            )));
        }
        Self { cards: Arc::new(cards), next_index: 0 }
    }
}

fn build_card(definition: DreamwellCardDefinition) -> DreamwellCard {
    DreamwellCard {
        produced_energy: definition.energy_produced,
        effects: definition
            .abilities
            .iter()
            .enumerate()
            .filter_map(|(i, a)| match a {
                Ability::Event(e) => {
                    Some(AbilityData { ability_number: AbilityNumber(i), ability: e.clone() })
                }
                _ => None,
            })
            .collect(),
        definition,
    }
}
