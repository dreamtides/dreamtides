use std::sync::Arc;

use battle_queries::battle_card_queries::card_abilities;
use battle_queries::battle_trace;
use battle_queries::legal_action_queries::legal_actions_cache;
use battle_state::battle::animation_data::AnimationData;
use battle_state::battle::battle_card_definitions::BattleCardDefinitions;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_trace::battle_tracing::BattleTracing;
use core_data::identifiers::QuestId;
use database::save_file::SaveFile;
use state_provider::state_provider::StateProvider;
use tracing::instrument;

/// Returns a deserialized [BattleState] for the battle in this save
/// file, if one is present.
#[instrument(skip_all, level = "debug")]
pub fn battle<P>(provider: &P, file: &SaveFile) -> Option<(BattleState, QuestId)>
where
    P: StateProvider + 'static,
{
    match file {
        SaveFile::V1(v1) => {
            let quest = v1.quest.as_ref()?;
            let quest_id = quest.id;

            let mut battle = quest.battle.clone()?;
            battle.tabula = provider.tabula();
            battle.tracing = Some(BattleTracing::default());
            battle.animations = Some(AnimationData::default());
            battle.card_definitions = Arc::new(BattleCardDefinitions::rebuild(
                &battle.card_definitions,
                |quest_deck_card_id, owner| {
                    let definition =
                        battle.players.player(owner).quest.deck.get_card(quest_deck_card_id);
                    Arc::new(definition.clone())
                },
                card_abilities::build_from_definition,
            ));
            legal_actions_cache::populate(&mut battle);
            battle_trace!("Loaded battle from save", &mut battle);
            Some((battle, quest_id))
        }
    }
}

// undo logic moved to undo.rs
