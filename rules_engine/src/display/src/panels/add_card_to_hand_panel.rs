use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use battle_state::battle::battle_state::BattleState;
use bon::Builder;
use core_data::identifiers::BaseCardId;
use core_data::types::PlayerName;
use masonry::flex_enums::{FlexAlign, FlexDirection, FlexJustify};
use masonry::flex_style::FlexStyle;
use tabula_ids::test_card;
use ui_components::box_component::BoxComponent;
use ui_components::button_component::ButtonComponent;
use ui_components::component::Component;
use ui_components::panel_component::PanelComponent;
use ui_components::scroll_view_component::ScrollViewComponent;
use ui_components::text_component::TextComponent;
use ui_components::typography::Typography;

#[derive(Clone, Builder)]
pub struct AddCardToHandPanel<'a> {
    pub user_player: PlayerName,
    pub battle: &'a BattleState,
}

impl Component for AddCardToHandPanel<'_> {
    fn render(self) -> Option<impl Component> {
        Some(
            PanelComponent::builder()
                .title("Add Card to Hand")
                .content(
                    ScrollViewComponent::builder()
                        .child(
                            BoxComponent::builder()
                                .name("Card Options")
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
                                        .name("Current Hand Count Display")
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
                                                    "Cards in hand: {}",
                                                    self.battle.cards.hand(self.user_player).len()
                                                ))
                                                .typography(Typography::Body2)
                                                .build(),
                                        )
                                        .build(),
                                )
                                .child(
                                    BoxComponent::builder()
                                        .name("All Test Cards")
                                        .style(
                                            FlexStyle::builder()
                                                .align_items(FlexAlign::Stretch)
                                                .flex_direction(FlexDirection::Column)
                                                .flex_grow(1)
                                                .justify_content(FlexJustify::FlexStart)
                                                .build(),
                                        )
                                        .children(
                                            test_card::ALL_TEST_CARD_IDS
                                                .iter()
                                                .filter_map(|id| {
                                                    AddCardCell::builder()
                                                        .battle(self.battle)
                                                        .card(*id)
                                                        .user_player(self.user_player)
                                                        .build()
                                                        .flex_node()
                                                })
                                                .collect(),
                                        )
                                        .build(),
                                )
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
    }
}

#[derive(Clone, Builder)]
pub struct AddCardCell<'a> {
    pub battle: &'a BattleState,
    pub card: BaseCardId,
    pub user_player: PlayerName,
}

impl Component for AddCardCell<'_> {
    fn render(self) -> Option<impl Component> {
        let name = self
            .battle
            .tabula
            .cards
            .get(&self.card)
            .expect("definition missing for identity")
            .displayed_name
            .clone();
        Some(
            BoxComponent::builder()
                .name(format!("{:?} Card Cell", self.card))
                .style(
                    FlexStyle::builder()
                        .align_items(FlexAlign::Center)
                        .justify_content(FlexJustify::SpaceBetween)
                        .margin(6)
                        .build(),
                )
                .child(TextComponent::builder().text(name).typography(Typography::Body2).build())
                .child(
                    ButtonComponent::builder()
                        .label("Add")
                        .action(BattleAction::Debug(DebugBattleAction::AddCardToHand {
                            player: self.user_player,
                            card: self.card,
                        }))
                        .build(),
                )
                .build(),
        )
    }
}
