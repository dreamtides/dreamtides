use action_data::panel_address::PanelAddress;
use battle_state::battle::battle_state::BattleState;
use ui_components::component::Component;

use crate::core::response_builder::ResponseBuilder;
use crate::panels::add_card_to_hand_panel::AddCardToHandPanel;
use crate::panels::developer_panel::DeveloperPanel;
use crate::panels::set_opponent_agent_panel::SetOpponentAgentPanel;
use crate::panels::view_logs_panel::ViewLogsPanel;

/// Renders a panel based on its [PanelAddress].
pub fn render_panel(
    address: PanelAddress,
    builder: &ResponseBuilder,
    battle: &BattleState,
) -> impl Component {
    match address {
        PanelAddress::Developer => {
            DeveloperPanel::builder().user_player(builder.display_for_player()).build().wrap()
        }
        PanelAddress::SetOpponentAgent => SetOpponentAgentPanel::builder()
            .user_player(builder.display_for_player())
            .battle(battle)
            .build()
            .wrap(),
        PanelAddress::AddCardToHand => AddCardToHandPanel::builder()
            .user_player(builder.display_for_player())
            .battle(battle)
            .build()
            .wrap(),
        PanelAddress::ViewLogs(filter) => ViewLogsPanel::builder()
            .maybe_log_directory(battle.request_context.logging_options.log_directory.clone())
            .maybe_filter(filter)
            .build()
            .wrap(),
    }
}
