pub mod basic_scene;

use std::sync::{LazyLock, Mutex};

use action_data::battle_action::{BattleAction, CardBrowserType};
use action_data::user_action::UserAction;
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use display_data::battle_view::BattleView;
use display_data::command::{Command, CommandSequence, UpdateBattleCommand};
use display_data::object_position::Position;
use display_data::request_data::{
    ConnectRequest, ConnectResponse, Metadata, PerformActionRequest, PerformActionResponse,
};
use uuid::Uuid;

static CURRENT_BATTLE: LazyLock<Mutex<Option<BattleView>>> = LazyLock::new(|| Mutex::new(None));
static CARD_BROWSER_SOURCE: LazyLock<Mutex<Option<Position>>> = LazyLock::new(|| Mutex::new(None));

pub fn connect(request: &ConnectRequest, _scenario: &str) -> ConnectResponse {
    let battle = basic_scene::create(BattleId(Uuid::new_v4()));
    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
    ConnectResponse {
        metadata: request.metadata,
        commands: CommandSequence::from_command(Command::UpdateBattle(UpdateBattleCommand::new(
            battle,
        ))),
    }
}

pub fn perform_action(request: &PerformActionRequest, _scenario: &str) -> PerformActionResponse {
    match &request.action {
        UserAction::BattleAction(action) => {
            perform_battle_action(*action, request.metadata, _scenario)
        }
        _ => {
            panic!("Not implemented: {:?}", request);
        }
    }
}

fn perform_battle_action(
    action: BattleAction,
    metadata: Metadata,
    _scenario: &str,
) -> PerformActionResponse {
    let commands = match action {
        BattleAction::BrowseCards(card_browser) => browse_cards(card_browser),
        BattleAction::CloseCardBrowser => close_card_browser(),
        _ => {
            panic!("Not implemented: {:?}", action);
        }
    };

    PerformActionResponse { metadata, commands }
}

fn browse_cards(card_browser: CardBrowserType) -> CommandSequence {
    let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();

    let source_position = match card_browser {
        CardBrowserType::UserDeck => Position::InDeck(PlayerName::User),
        CardBrowserType::EnemyDeck => Position::InDeck(PlayerName::Enemy),
        CardBrowserType::UserVoid => Position::InVoid(PlayerName::User),
        CardBrowserType::EnemyVoid => Position::InVoid(PlayerName::Enemy),
        CardBrowserType::UserStatus => Position::InPlayerStatus(PlayerName::User),
        CardBrowserType::EnemyStatus => Position::InPlayerStatus(PlayerName::Enemy),
    };

    // Store the source position for later use when closing browser
    *CARD_BROWSER_SOURCE.lock().unwrap() = Some(source_position);

    for card in battle.cards.iter_mut() {
        if card.position.position == source_position {
            card.position.position = Position::Browser;
        }
    }

    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

    CommandSequence::sequential(vec![Command::UpdateBattle(UpdateBattleCommand::new(battle))])
}

fn close_card_browser() -> CommandSequence {
    let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
    let source_position = *CARD_BROWSER_SOURCE.lock().unwrap();

    if let Some(position) = source_position {
        for card in battle.cards.iter_mut() {
            if card.position.position == Position::Browser {
                card.position.position = position;
            }
        }
    }

    *CARD_BROWSER_SOURCE.lock().unwrap() = None;
    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

    CommandSequence::sequential(vec![Command::UpdateBattle(UpdateBattleCommand::new(battle))])
}
