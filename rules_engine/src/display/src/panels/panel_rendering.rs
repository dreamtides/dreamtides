use action_data::panel_address::PanelAddress;
use battle_data::battle::battle_data::BattleData;
use masonry::flex_node::FlexNode;
use ui_components::component::Component;

use crate::core::response_builder::ResponseBuilder;
use crate::panels::developer_panel::DeveloperPanel;
use crate::panels::set_opponent_agent_panel::SetOpponentAgentPanel;

/// Renders a panel based on its [PanelAddress].
pub fn render_panel(
    address: PanelAddress,
    builder: &ResponseBuilder,
    battle: &BattleData,
) -> Option<FlexNode> {
    match address {
        PanelAddress::Developer => {
            DeveloperPanel::builder().user_player(builder.display_for_player()).build().flex_node()
        }
        PanelAddress::SetOpponentAgent => SetOpponentAgentPanel::builder()
            .user_player(builder.display_for_player())
            .battle(battle)
            .build()
            .flex_node(),
    }
}
