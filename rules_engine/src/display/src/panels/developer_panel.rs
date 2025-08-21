use action_data::battle_display_action::BattleDisplayAction;
use action_data::debug_action_data::DebugAction;
use action_data::game_action_data::GameAction;
use action_data::panel_address::PanelAddress;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use battle_state::battle_player::battle_player_state::TestDeckName;
use bon::Builder;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use masonry::flex_enums::{FlexAlign, FlexDirection, FlexJustify, FlexWrap};
use masonry::flex_style::FlexStyle;
use tabula_ids::test_card;
use ui_components::box_component::BoxComponent;
use ui_components::button_component::ButtonComponent;
use ui_components::component::Component;
use ui_components::icon;
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
                                .action(GameAction::BattleDisplayAction(
                                    BattleDisplayAction::OpenPanel(PanelAddress::SetOpponentAgent),
                                ))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Add Card to Hand")
                                .action(GameAction::BattleDisplayAction(
                                    BattleDisplayAction::OpenPanel(PanelAddress::AddCardToHand),
                                ))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Play Opponent Card")
                                .action(GameAction::BattleDisplayAction(
                                    BattleDisplayAction::OpenPanel(PanelAddress::PlayOpponentCard),
                                ))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Opponent Continue")
                                .action(BattleAction::Debug(DebugBattleAction::OpponentContinue))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Draw Card")
                                .action(BattleAction::Debug(DebugBattleAction::DrawCard {
                                    player: self.user_player,
                                }))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Enemy Character")
                                .action(BattleAction::Debug(
                                    DebugBattleAction::AddCardToBattlefield {
                                        player: self.user_player.opponent(),
                                        card: test_card::TEST_VANILLA_CHARACTER,
                                    },
                                ))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label(format!("99 {}", icon::ENERGY))
                                .action(BattleAction::Debug(DebugBattleAction::SetEnergy {
                                    player: self.user_player,
                                    energy: Energy(99),
                                }))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Restart Battle")
                                .action(DebugAction::RestartBattle)
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Core 11")
                                .action(DebugAction::RestartBattleWithDecks {
                                    one: TestDeckName::CoreEleven,
                                    two: TestDeckName::CoreEleven,
                                })
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("View Logs")
                                .action(GameAction::BattleDisplayAction(
                                    BattleDisplayAction::OpenPanel(PanelAddress::ViewLogs(None)),
                                ))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Deck->1")
                                .action(BattleAction::Debug(
                                    DebugBattleAction::SetCardsRemainingInDeck {
                                        player: self.user_player,
                                        cards: 1,
                                    },
                                ))
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
