use action_data::debug_action_data::DebugAction;
use ai_data::game_ai::GameAI;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_player::player_data::PlayerType;
use bon::Builder;
use core_data::types::PlayerName;
use masonry::flex_enums::{FlexAlign, FlexDirection, FlexJustify};
use masonry::flex_style::FlexStyle;
use ui_components::box_component::BoxComponent;
use ui_components::button_component::ButtonComponent;
use ui_components::component::Component;
use ui_components::panel_component::PanelComponent;
use ui_components::text_component::TextComponent;
use ui_components::typography::Typography;

#[derive(Clone, Builder)]
pub struct SetOpponentAgentPanel<'a> {
    pub user_player: PlayerName,
    pub battle: &'a BattleData,
}

impl Component for SetOpponentAgentPanel<'_> {
    fn render(self) -> Option<impl Component> {
        Some(
            PanelComponent::builder()
                .title("Set Opponent Agent")
                .content(
                    BoxComponent::builder()
                        .name("Opponent Agent Options")
                        .style(
                            FlexStyle::builder()
                                .align_items(FlexAlign::Stretch)
                                .flex_direction(FlexDirection::Column)
                                .flex_grow(1)
                                .justify_content(FlexJustify::FlexStart)
                                .max_width(300)
                                .padding((8, 8, 8, 8))
                                .build(),
                        )
                        .child(
                            BoxComponent::builder()
                                .name("Current Agent Display")
                                .style(
                                    FlexStyle::builder()
                                        .align_items(FlexAlign::Center)
                                        .justify_content(FlexJustify::Center)
                                        .margin((0, 0, 12, 0))
                                        .width(200)
                                        .build(),
                                )
                                .child(
                                    TextComponent::builder()
                                        .text(format!(
                                            "Current agent: {:?}",
                                            self.get_opponent_agent()
                                        ))
                                        .typography(Typography::Body2)
                                        .build(),
                                )
                                .build(),
                        )
                        .child(
                            SetAgentCell::builder().agent(GameAI::Uct1MaxIterations(1000)).build(),
                        )
                        .child(
                            SetAgentCell::builder().agent(GameAI::Uct1MaxIterations(5000)).build(),
                        )
                        .child(
                            SetAgentCell::builder().agent(GameAI::Uct1MaxIterations(10000)).build(),
                        )
                        .child(
                            SetAgentCell::builder().agent(GameAI::Uct1MaxIterations(50000)).build(),
                        )
                        .child(SetAgentCell::builder().agent(GameAI::IterativeDeepening).build())
                        .child(SetAgentCell::builder().agent(GameAI::RandomAction).build())
                        .child(SetAgentCell::builder().agent(GameAI::FirstAvailableAction).build())
                        .build(),
                )
                .build(),
        )
    }
}

impl SetOpponentAgentPanel<'_> {
    fn get_opponent_agent(&self) -> String {
        match self.battle.player(self.user_player.opponent()).player_type {
            PlayerType::User(id) => format!("User: {:?}", id),
            PlayerType::Agent(agent) => format!("{:?}", agent),
        }
    }
}

#[derive(Clone, Builder)]
pub struct SetAgentCell {
    pub agent: GameAI,
}

impl Component for SetAgentCell {
    fn render(self) -> Option<impl Component> {
        Some(
            BoxComponent::builder()
                .name(format!("{:?} Agent Cell", self.agent))
                .style(
                    FlexStyle::builder()
                        .align_items(FlexAlign::Center)
                        .justify_content(FlexJustify::SpaceBetween)
                        .margin(6)
                        .build(),
                )
                .child(
                    TextComponent::builder()
                        .text(format!("{:?}", self.agent))
                        .typography(Typography::Body2)
                        .build(),
                )
                .child(
                    ButtonComponent::builder()
                        .label("Select")
                        .action(DebugAction::SetOpponentAgent(self.agent))
                        .build(),
                )
                .build(),
        )
    }
}
