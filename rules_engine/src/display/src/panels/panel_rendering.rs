use action_data::panel_address::PanelAddress;
use battle_state::battle::battle_state::BattleState;
use masonry::flex_node::FlexNode;
use ui_components::component::Component;

use crate::core::response_builder::ResponseBuilder;
use crate::panels::add_card_to_hand_panel::AddCardToHandPanel;
use crate::panels::developer_panel::DeveloperPanel;
use crate::panels::set_opponent_agent_panel::SetOpponentAgentPanel;

/// Renders a panel based on its [PanelAddress].
pub fn render_panel(
    address: PanelAddress,
    builder: &ResponseBuilder,
    battle: &BattleState,
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
        PanelAddress::AddCardToHand => AddCardToHandPanel::builder()
            .user_player(builder.display_for_player())
            .battle(battle)
            .build()
            .flex_node(),
    }
}
