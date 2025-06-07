use battle_state::battle::battle_state::{BattleState, RequestContext};
use core_data::identifiers::{BattleId, UserId};
use core_data::types::PlayerName;
use display::core::response_builder::ResponseBuilder;
use display::rendering::battle_rendering;
use display_data::command::{Command, CommandSequence, UpdateBattleCommand};
use game_creation::new_battle;
use masonry::flex_enums::{TextAlign, WhiteSpace};
use ui_components::component::Component;
use ui_components::panel_component::PanelComponent;
use ui_components::text_component::TextComponent;
use ui_components::typography::Typography;
use uuid::Uuid;

/// Attempts to display an error message to the player describing a rules engine
/// error.
pub fn display_error_message(
    battle: Option<&BattleState>,
    message: impl Into<String>,
) -> CommandSequence {
    let message = message.into();
    match battle {
        Some(existing_battle) => display_error_message_with_battle(existing_battle, message),
        None => {
            let id = BattleId(Uuid::new_v4());
            let dummy_battle =
                new_battle::create_and_start(UserId::default(), id, RequestContext {
                    developer_mode: false,
                });
            display_error_message_with_battle(&dummy_battle, message)
        }
    }
}

fn display_error_message_with_battle(battle: &BattleState, message: String) -> CommandSequence {
    let mut builder = ResponseBuilder::new(PlayerName::One, false);
    let mut view = battle_rendering::battle_view(&builder, battle);
    view.interface.screen_overlay = render_message(message).flex_node();
    builder.push(Command::UpdateBattle(UpdateBattleCommand { battle: view, update_sound: None }));
    builder.commands()
}

fn render_message(text: String) -> impl Component {
    PanelComponent::builder()
        .title("Error Message")
        .content(
            TextComponent::builder()
                .text(text)
                .text_align(TextAlign::UpperLeft)
                .typography(Typography::StackTrace)
                .white_space(WhiteSpace::Normal)
                .build(),
        )
        .build()
}
