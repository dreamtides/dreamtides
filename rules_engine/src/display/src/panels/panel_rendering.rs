use action_data::panel_address::PanelAddress;
use battle_state::battle::battle_state::BattleState;
use ui_components::box_component::BoxComponent;
use ui_components::component::Component;
use ui_components::wrapper;

use crate::core::response_builder::ResponseBuilder;
use crate::panels::add_card_to_hand_panel::AddCardToHandPanel;
use crate::panels::developer_panel::DeveloperPanel;
use crate::panels::set_opponent_agent_panel::SetOpponentAgentPanel;

/// Renders a panel based on its [PanelAddress].
pub fn render_panel(
    address: PanelAddress,
    builder: &ResponseBuilder,
    battle: &BattleState,
) -> impl Component {
    match address {
        PanelAddress::Developer => wrapper::wrap(
            DeveloperPanel::builder().user_player(builder.display_for_player()).build(),
        ),
        PanelAddress::SetOpponentAgent => wrapper::wrap(
            SetOpponentAgentPanel::builder()
                .user_player(builder.display_for_player())
                .battle(battle)
                .build(),
        ),
        PanelAddress::AddCardToHand => wrapper::wrap(
            AddCardToHandPanel::builder()
                .user_player(builder.display_for_player())
                .battle(battle)
                .build(),
        ),
    }
}
