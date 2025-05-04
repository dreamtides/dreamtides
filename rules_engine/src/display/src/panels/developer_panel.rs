use action_data::debug_action_data::DebugAction;
use action_data::game_action_data::GameAction;
use action_data::panel_address::PanelAddress;
use battle_data::actions::battle_action_data::{BattleAction, DebugBattleAction};
use bon::Builder;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use masonry::flex_enums::{FlexAlign, FlexDirection, FlexJustify, FlexWrap};
use masonry::flex_style::FlexStyle;
use ui_components::box_component::BoxComponent;
use ui_components::button_component::ButtonComponent;
use ui_components::component::Component;
use ui_components::panel_component::PanelComponent;

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
                                .label("99\u{f7e4}")
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
