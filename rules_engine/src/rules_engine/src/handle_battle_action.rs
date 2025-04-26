use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use std::{panic, thread};

use action_data::battle_action::BattleAction;
use actions::battle_actions;
use ai_agents::agent_search;
use ai_data::game_ai::GameAI;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::animation_data::AnimationData;
use battle_data::battle_player::player_data::PlayerType;
use battle_queries::legal_action_queries::legal_actions::{self, LegalActions};
use core_data::identifiers::UserId;
use core_data::types::PlayerName;
use display::rendering::renderer;
use display_data::command::CommandSequence;
use logging::battle_trace;
use tokio::task::{self};
use tracing::{error, instrument};

static PENDING_UPDATES: LazyLock<Mutex<HashMap<UserId, Vec<CommandSequence>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[instrument(name = "handle_battle_action", level = "debug", skip(battle))]
pub fn execute(
    battle: &mut BattleData,
    initiated_by: UserId,
    player: PlayerName,
    action: BattleAction,
) -> CommandSequence {
    let mut current_player = player;
    let mut current_action = action;

    loop {
        battle_trace!("Executing battle action", battle, current_action);
        battle_actions::execute(battle, current_player, current_action);

        let Some(next_player) = legal_actions::next_to_act(battle) else {
            battle_trace!("Rendering updates for game over", battle);
            return render_updates(battle, initiated_by);
        };

        let legal_actions = legal_actions::compute(battle, next_player, LegalActions::default());
        if legal_actions == vec![BattleAction::ResolveStack] {
            battle_trace!("Automatically executing ResolveStack", battle);
            current_player = next_player;
            current_action = BattleAction::ResolveStack;
            continue;
        }

        if let PlayerType::Agent(agent) = battle.player(next_player).player_type.clone() {
            battle_trace!("Starting async agent action selection", battle);
            let result = render_updates(battle, initiated_by);
            battle.animations = Some(AnimationData::default());
            spawn_agent_task(battle.clone(), next_player, agent, initiated_by);
            return result;
        } else {
            battle_trace!("Rendering updates for player", battle);
            let result = render_updates(battle, initiated_by);
            battle.animations = Some(AnimationData::default());
            return result;
        }
    }
}

fn spawn_agent_task(battle: BattleData, player: PlayerName, agent: GameAI, initiated_by: UserId) {
    task::spawn(async move {
        let task_result =
            task::spawn_blocking(move || execute_agent_action(battle, player, agent, initiated_by))
                .await;

        if let Err(e) = task_result {
            error!("Agent task panicked: {}", e);
        }
    });
}

fn execute_agent_action(
    mut battle: BattleData,
    player: PlayerName,
    agent: GameAI,
    initiated_by: UserId,
) {
    let panic_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        battle_trace!("Selecting action for AI player", battle);
        let next_action = agent_search::select_action(&battle, player, &agent);
        battle_trace!("Executing action for AI player", battle, next_action);

        thread::sleep(Duration::from_secs(2));

        battle_actions::execute(&mut battle, player, next_action);

        let updates = render_updates(&battle, initiated_by);

        if let PlayerType::User(opponent_id) = battle.player(player.opponent()).player_type {
            let mut pending_updates = PENDING_UPDATES.lock().unwrap();
            pending_updates.entry(opponent_id).or_default().push(updates.clone());
            pending_updates.entry(initiated_by).or_default().push(updates);
        } else {
            let mut pending_updates = PENDING_UPDATES.lock().unwrap();
            pending_updates.entry(initiated_by).or_default().push(updates);
        }
    }));

    if let Err(e) = panic_result {
        error!("Agent action selection panicked: {:?}", e);
    }
}

pub fn poll(user_id: UserId) -> Option<CommandSequence> {
    let mut updates = PENDING_UPDATES.lock().unwrap();
    if let Some(user_updates) = updates.get_mut(&user_id) {
        if !user_updates.is_empty() {
            return Some(user_updates.remove(0));
        }
    }
    None
}

fn render_updates(battle: &BattleData, user_id: UserId) -> CommandSequence {
    let player_name = renderer::player_name_for_user(battle, user_id);
    let player_updates = renderer::render_updates(battle, user_id);

    if let PlayerType::User(opponent_id) = &battle.player(player_name.opponent()).player_type {
        let opponent_updates = renderer::render_updates(battle, *opponent_id);
        let mut updates = PENDING_UPDATES.lock().unwrap();
        updates.entry(*opponent_id).or_default().push(opponent_updates);
    }

    player_updates
}
