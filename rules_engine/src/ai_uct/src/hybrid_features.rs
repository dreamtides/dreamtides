use battle_queries::battle_card_queries::card_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use core_data::types::PlayerName;

use crate::hybrid_dataset::FeatureMap;
use crate::hybrid_eval;

pub fn merge_policy_features(
    state_features: &FeatureMap,
    action_features: &FeatureMap,
    legal_action_count: usize,
) -> FeatureMap {
    let mut features = FeatureMap::new();
    for (key, value) in state_features {
        features.insert(format!("state_{key}"), *value);
    }
    for (key, value) in action_features {
        features.insert(format!("action_{key}"), *value);
    }
    features.insert("global_legal_action_count".to_string(), legal_action_count as f64);
    features
}

pub fn extract_state_features(battle: &BattleState, perspective: PlayerName) -> FeatureMap {
    let mut features = FeatureMap::new();
    let opponent = perspective.opponent();
    let our_player = battle.players.player(perspective);
    let their_player = battle.players.player(opponent);
    let points_to_win = battle.rules_config.points_to_win.0 as f64;

    features.insert("bias".to_string(), 1.0);
    features.insert("turn_id".to_string(), battle.turn.turn_id.0 as f64);
    features.insert(
        "is_active_player".to_string(),
        f64::from((battle.turn.active_player == perspective) as u8),
    );
    features.insert(
        "phase_judgment".to_string(),
        f64::from((battle.phase == BattleTurnPhase::Judgment) as u8),
    );
    features
        .insert("phase_main".to_string(), f64::from((battle.phase == BattleTurnPhase::Main) as u8));
    features.insert(
        "phase_ending".to_string(),
        f64::from((battle.phase == BattleTurnPhase::Ending) as u8),
    );
    features.insert("has_stack".to_string(), f64::from(battle.cards.has_stack() as u8));
    features.insert("has_prompt".to_string(), f64::from((!battle.prompts.is_empty()) as u8));
    features.insert("stack_priority".to_string(), stack_priority_value(battle, perspective));
    features.insert("our_points".to_string(), our_player.points.0 as f64);
    features.insert("their_points".to_string(), their_player.points.0 as f64);
    features.insert(
        "points_margin".to_string(),
        our_player.points.0 as f64 - their_player.points.0 as f64,
    );
    features.insert(
        "distance_margin".to_string(),
        (points_to_win - our_player.points.0 as f64)
            - (points_to_win - their_player.points.0 as f64),
    );
    features.insert("our_energy".to_string(), our_player.current_energy.0 as f64);
    features.insert("their_energy".to_string(), their_player.current_energy.0 as f64);
    features.insert(
        "energy_margin".to_string(),
        our_player.current_energy.0 as f64 - their_player.current_energy.0 as f64,
    );
    features.insert(
        "produced_margin".to_string(),
        our_player.produced_energy.0 as f64 - their_player.produced_energy.0 as f64,
    );
    features.insert("our_hand_size".to_string(), battle.cards.hand(perspective).len() as f64);
    features.insert("their_hand_size".to_string(), battle.cards.hand(opponent).len() as f64);
    features.insert(
        "hand_size_margin".to_string(),
        battle.cards.hand(perspective).len() as f64 - battle.cards.hand(opponent).len() as f64,
    );
    features.insert("our_void_size".to_string(), battle.cards.void(perspective).len() as f64);
    features.insert("their_void_size".to_string(), battle.cards.void(opponent).len() as f64);
    features.insert(
        "void_size_margin".to_string(),
        battle.cards.void(perspective).len() as f64 - battle.cards.void(opponent).len() as f64,
    );
    features.insert(
        "our_front_count".to_string(),
        battle.cards.battlefield(perspective).front.iter().flatten().count() as f64,
    );
    features.insert(
        "their_front_count".to_string(),
        battle.cards.battlefield(opponent).front.iter().flatten().count() as f64,
    );
    features.insert(
        "our_back_count".to_string(),
        battle.cards.battlefield(perspective).back.iter().flatten().count() as f64,
    );
    features.insert(
        "their_back_count".to_string(),
        battle.cards.battlefield(opponent).back.iter().flatten().count() as f64,
    );
    features.insert("our_front_spark".to_string(), row_spark_sum(battle, perspective, true));
    features.insert("their_front_spark".to_string(), row_spark_sum(battle, opponent, true));
    features.insert("our_back_spark".to_string(), row_spark_sum(battle, perspective, false));
    features.insert("their_back_spark".to_string(), row_spark_sum(battle, opponent, false));
    features.insert("our_ready_back".to_string(), ready_back_count(battle, perspective));
    features.insert("their_ready_back".to_string(), ready_back_count(battle, opponent));
    features.insert("open_attack_lanes".to_string(), open_attack_lanes(battle, perspective));
    features.insert(
        "can_begin_positioning".to_string(),
        f64::from(can_begin_positioning(battle, perspective) as u8),
    );

    for column in 0..battle.rules_config.front_row_size {
        features.insert(
            format!("our_front_col_{column}_spark"),
            front_column_spark(battle, perspective, column),
        );
        features.insert(
            format!("their_front_col_{column}_spark"),
            front_column_spark(battle, opponent, column),
        );
    }

    for feature in hybrid_eval::breakdown(battle, perspective).features {
        features.insert(format!("eval_{}", feature.name), feature.value);
    }

    features
}

pub fn extract_action_features(
    battle: &BattleState,
    perspective: PlayerName,
    action: BattleAction,
) -> FeatureMap {
    let mut features = FeatureMap::new();
    features.insert("bias".to_string(), 1.0);
    features.insert(
        "is_play_hand".to_string(),
        f64::from(matches!(action, BattleAction::PlayCardFromHand(..)) as u8),
    );
    features.insert(
        "is_play_void".to_string(),
        f64::from(matches!(action, BattleAction::PlayCardFromVoid(..)) as u8),
    );
    features.insert(
        "is_activate".to_string(),
        f64::from(matches!(action, BattleAction::ActivateAbilityForCharacter(..)) as u8),
    );
    features.insert("is_pass".to_string(), f64::from((action == BattleAction::PassPriority) as u8));
    features.insert("is_end_turn".to_string(), f64::from((action == BattleAction::EndTurn) as u8));
    features.insert(
        "is_start_next_turn".to_string(),
        f64::from((action == BattleAction::StartNextTurn) as u8),
    );
    features.insert(
        "is_begin_positioning".to_string(),
        f64::from((action == BattleAction::BeginPositioning) as u8),
    );
    features.insert(
        "is_move_front".to_string(),
        f64::from(matches!(action, BattleAction::MoveCharacterToFrontRank(..)) as u8),
    );
    features.insert(
        "is_move_back".to_string(),
        f64::from(matches!(action, BattleAction::MoveCharacterToBackRank(..)) as u8),
    );

    match action {
        BattleAction::PlayCardFromHand(card_id) => {
            card_features(&mut features, battle, card_id);
        }
        BattleAction::PlayCardFromVoid(card_id) => {
            card_features(&mut features, battle, card_id);
        }
        BattleAction::ActivateAbilityForCharacter(character_id) => {
            features.insert(
                "character_spark".to_string(),
                card_properties::spark(battle, perspective, character_id).unwrap_or_default().0
                    as f64,
            );
        }
        BattleAction::MoveCharacterToFrontRank(character_id, column) => {
            features.insert("move_column".to_string(), column as f64);
            features.insert(
                "character_spark".to_string(),
                card_properties::spark(battle, perspective, character_id).unwrap_or_default().0
                    as f64,
            );
        }
        BattleAction::MoveCharacterToBackRank(character_id, column) => {
            features.insert("move_column".to_string(), column as f64);
            features.insert(
                "character_spark".to_string(),
                card_properties::spark(battle, perspective, character_id).unwrap_or_default().0
                    as f64,
            );
        }
        _ => {}
    }

    features
}

fn can_begin_positioning(battle: &BattleState, perspective: PlayerName) -> bool {
    matches!(
        legal_actions::compute(battle, perspective),
        LegalActions::Standard { actions } if actions.can_begin_positioning
    )
}

fn card_features(
    features: &mut FeatureMap,
    battle: &BattleState,
    card_id: impl battle_state::battle::card_id::CardIdType,
) {
    features.insert(
        "energy_cost".to_string(),
        card_properties::converted_energy_cost(battle, card_id).0 as f64,
    );
    features.insert(
        "base_spark".to_string(),
        card_properties::base_spark(battle, card_id).unwrap_or_default().0 as f64,
    );
    features
        .insert("is_fast".to_string(), f64::from(card_properties::is_fast(battle, card_id) as u8));
}

fn front_column_spark(battle: &BattleState, player: PlayerName, column: usize) -> f64 {
    battle.cards.battlefield(player).front[column]
        .map(|character_id| {
            card_properties::spark(battle, player, character_id).unwrap_or_default().0 as f64
        })
        .unwrap_or_default()
}

fn open_attack_lanes(battle: &BattleState, perspective: PlayerName) -> f64 {
    battle.cards.battlefield(perspective).front[..battle.rules_config.front_row_size]
        .iter()
        .zip(battle.cards.battlefield(perspective.opponent()).front.iter())
        .filter(|(our, theirs)| our.is_none() && theirs.is_none())
        .count() as f64
}

fn ready_back_count(battle: &BattleState, perspective: PlayerName) -> f64 {
    let current_turn = battle.turn.turn_id.0;
    battle
        .cards
        .battlefield(perspective)
        .back
        .iter()
        .flatten()
        .filter(|character_id| {
            battle.cards.battlefield_state(perspective).get(character_id).is_some_and(|state| {
                state.played_turn != current_turn
                    && !battle.turn.moved_this_turn.contains(character_id)
            })
        })
        .count() as f64
}

fn row_spark_sum(battle: &BattleState, player: PlayerName, front: bool) -> f64 {
    let row = if front {
        &battle.cards.battlefield(player).front
    } else {
        &battle.cards.battlefield(player).back
    };
    row.iter()
        .flatten()
        .map(|character_id| {
            card_properties::spark(battle, player, *character_id).unwrap_or_default().0 as f64
        })
        .sum()
}

fn stack_priority_value(battle: &BattleState, perspective: PlayerName) -> f64 {
    match battle.stack_priority {
        Some(player) if player == perspective => 1.0,
        Some(_) => -1.0,
        None => 0.0,
    }
}
