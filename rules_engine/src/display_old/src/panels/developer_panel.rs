use action_data_old::debug_action_data::DebugAction;
use action_data_old::game_action_data::GameAction;
use action_data_old::panel_address::PanelAddress;
use battle_data_old::actions::battle_action_data::{BattleAction, DebugBattleAction};
use bon::Builder;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use masonry_old::flex_enums::{FlexAlign, FlexDirection, FlexJustify, FlexWrap};
use masonry_old::flex_style::FlexStyle;
use ui_components_old::box_component::BoxComponent;
use ui_components_old::button_component::ButtonComponent;
use ui_components_old::component::Component;
use ui_components_old::icon;
use ui_components_old::panel_component::PanelComponent;

#[derive(Clone, Builder)]
pub struct DeveloperPanel {
    pub user_player: PlayerName,
}

impl Component for DeveloperPanel {
    fn render(self) -> Option<impl Component> {
        Some(
            PanelComponent::builder()
                .title("Developer")
                .content(
                    BoxComponent::builder()
                        .name("Developer Buttons")
                        .style(
                            FlexStyle::builder()
                                .align_items(FlexAlign::Center)
                                .flex_direction(FlexDirection::Column)
                                .flex_grow(1)
                                .justify_content(FlexJustify::Center)
                                .wrap(FlexWrap::Wrap)
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Set AI")
                                .action(GameAction::OpenPanel(PanelAddress::SetOpponentAgent))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Draw Card")
                                .action(BattleAction::Debug(DebugBattleAction::DrawCard(
                                    self.user_player,
                                )))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label(format!("99 {}", icon::ENERGY))
                                .action(BattleAction::Debug(DebugBattleAction::SetEnergy(
                                    self.user_player,
                                    Energy(99),
                                )))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Restart Battle")
                                .action(DebugAction::RestartBattle)
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
    }
}

#[derive(Clone, Builder)]
pub struct DebugButton {
    #[builder(into)]
    pub label: String,
    #[builder(into)]
    pub action: GameAction,
}

impl Component for DebugButton {
    fn render(self) -> Option<impl Component> {
        Some(
            BoxComponent::builder()
                .name(format!("{} Button", self.label))
                .style(FlexStyle::builder().margin(4).build())
                .child(ButtonComponent::builder().label(self.label).action(self.action).build())
                .build(),
        )
    }
}
