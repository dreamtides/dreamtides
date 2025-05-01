pub mod basic_scene;

use std::sync::{LazyLock, Mutex};

use action_data::battle_action_data::{
    BattleAction, CardBrowserType, CardOrderSelectionTarget, SelectCardOrder,
};
use action_data::debug_action_data::DebugAction;
use action_data::game_action_data::GameAction;
use battle_data::battle_cards::card_id::{CardIdType, CharacterId};
use core_data::display_color::{self, DisplayColor};
use core_data::display_types::{
    AudioClipAddress, EffectAddress, MaterialAddress, Milliseconds, ProjectileAddress,
};
use core_data::identifiers::{BattleId, CardId};
use core_data::numerics::{Energy, Spark};
use display_data::battle_view::{BattleView, ButtonView, CardOrderSelectorView, DisplayPlayer};
use display_data::card_view::CardView;
use display_data::command::{
    ArrowStyle, Command, CommandSequence, DisplayArrow, DisplayArrowsCommand,
    DisplayDreamwellActivationCommand, DisplayEffectCommand, DisplayJudgmentCommand,
    DissolveCardCommand, FireProjectileCommand, GameMessageType, GameObjectId,
    ParallelCommandGroup, UpdateBattleCommand,
};
use display_data::object_position::{Position, StackType};
use display_data::request_data::{
    ConnectRequest, ConnectResponse, Metadata, PerformActionRequest, PerformActionResponse,
};
use masonry::borders::BorderRadius;
use masonry::dimension::{Dimension, DimensionGroup, DimensionUnit, FlexInsets};
use masonry::flex_enums::{FlexPosition, TextAlign, WhiteSpace};
use masonry::flex_node::{FlexNode, NodeType, TextNode};
use masonry::flex_style::{FlexStyle, FlexVector3};
use uuid::Uuid;

static CURRENT_BATTLE: LazyLock<Mutex<Option<BattleView>>> = LazyLock::new(|| Mutex::new(None));
static CARD_BROWSER_SOURCE: LazyLock<Mutex<Option<Position>>> = LazyLock::new(|| Mutex::new(None));
static ORDER_SELECTOR_VISIBLE: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
static CARD_ORDER_ORIGINAL_POSITIONS: LazyLock<Mutex<std::collections::HashMap<CardId, Position>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

pub fn connect(request: &ConnectRequest, _scenario: &str) -> ConnectResponse {
    let battle = basic_scene::create(BattleId(Uuid::new_v4()));
    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
    ConnectResponse {
        metadata: request.metadata,
        commands: CommandSequence::from_command(Command::UpdateBattle(UpdateBattleCommand::new(
            battle,
        ))),
    }
}

pub fn perform_action(request: &PerformActionRequest, scenario: &str) -> PerformActionResponse {
    match &request.action {
        GameAction::BattleAction(action) => {
            perform_battle_action(*action, request.metadata, scenario)
        }
        GameAction::DebugAction(action) => {
            perform_debug_action(*action, request.metadata, scenario)
        }
        _ => PerformActionResponse {
            metadata: request.metadata,
            commands: CommandSequence::default(),
        },
    }
}

fn perform_battle_action(
    action: BattleAction,
    metadata: Metadata,
    scenario: &str,
) -> PerformActionResponse {
    let commands = match action {
        BattleAction::PlayCardFromHand(id) => play_card(id.card_id(), scenario),
        BattleAction::BrowseCards(card_browser) => browse_cards(card_browser),
        BattleAction::CloseCardBrowser => close_card_browser(),
        BattleAction::SelectCharacterTarget(id) => select_card(id.card_id()),
        BattleAction::SelectCardOrder(select_order) => select_card_order(select_order),
        _ => {
            panic!("Not implemented: {:?}", action);
        }
    };

    PerformActionResponse { metadata, commands }
}

fn perform_debug_action(
    action: DebugAction,
    metadata: Metadata,
    scenario: &str,
) -> PerformActionResponse {
    let commands = match action {
        DebugAction::ApplyTestScenarioAction => apply_test_scenario_action(scenario),
        _ => {
            panic!("Not implemented: {:?}", action);
        }
    };

    PerformActionResponse { metadata, commands }
}

fn play_card(card_id: CardId, scenario: &str) -> CommandSequence {
    let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
    let Some((card_index, _)) = battle.cards.iter().enumerate().find(|(_, c)| c.id == card_id)
    else {
        panic!("Card not found: {:?}", card_id);
    };
    let mut commands = Vec::new();

    match scenario {
        "basic" => {
            battle.cards[card_index].position.position =
                Position::OnBattlefield(DisplayPlayer::User);
        }
        "play_card_with_targets" => {
            play_card_with_targets(&mut battle, card_id, StackType::TargetingEnemyBattlefield);
        }
        "play_card_with_order_selector" => {
            play_card_with_order_selector(&mut battle, &mut commands, card_id);
        }
        "respond_to_enemy_card" => {
            play_card_with_targets(&mut battle, card_id, StackType::TargetingBothBattlefields);
        }
        _ => {
            panic!("Scenario not implemented: {:?}", scenario);
        }
    }

    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));
    *CURRENT_BATTLE.lock().unwrap() = Some(battle);
    CommandSequence::sequential(commands)
}

fn browse_cards(card_browser: CardBrowserType) -> CommandSequence {
    let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();

    let source_position = match card_browser {
        CardBrowserType::UserDeck => Position::InDeck(DisplayPlayer::User),
        CardBrowserType::EnemyDeck => Position::InDeck(DisplayPlayer::Enemy),
        CardBrowserType::UserVoid => Position::InVoid(DisplayPlayer::User),
        CardBrowserType::EnemyVoid => Position::InVoid(DisplayPlayer::Enemy),
        CardBrowserType::UserStatus => Position::InPlayerStatus(DisplayPlayer::User),
        CardBrowserType::EnemyStatus => Position::InPlayerStatus(DisplayPlayer::Enemy),
    };

    // Store the source position for later use when closing browser
    *CARD_BROWSER_SOURCE.lock().unwrap() = Some(source_position);

    for card in battle.cards.iter_mut() {
        if card.position.position == source_position {
            card.position.position = Position::Browser;
        }
    }

    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

    CommandSequence::sequential(vec![Command::UpdateBattle(UpdateBattleCommand::new(battle))])
}

fn close_card_browser() -> CommandSequence {
    let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
    let source_position = *CARD_BROWSER_SOURCE.lock().unwrap();

    if let Some(position) = source_position {
        for card in battle.cards.iter_mut() {
            if card.position.position == Position::Browser {
                card.position.position = position;
            }
        }
    }

    *CARD_BROWSER_SOURCE.lock().unwrap() = None;
    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

    CommandSequence::sequential(vec![Command::UpdateBattle(UpdateBattleCommand::new(battle))])
}

fn select_card(card_id: CardId) -> CommandSequence {
    let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();

    let cards_to_move: Vec<(usize, u32)> = battle
        .cards
        .iter()
        .enumerate()
        .filter_map(|(index, card)| {
            if card.id == card_id || matches!(card.position.position, Position::OnStack(_)) {
                Some((index, card.position.sorting_key))
            } else {
                None
            }
        })
        .collect();

    let source_card_id = battle
        .cards
        .iter()
        .find_map(|card| {
            if matches!(card.position.position, Position::OnStack(_)) {
                Some(card.id)
            } else {
                None
            }
        })
        .unwrap();

    for (index, sorting_key) in cards_to_move {
        let position = if battle.cards[index].id == card_id {
            Position::InVoid(DisplayPlayer::Enemy)
        } else {
            Position::InVoid(DisplayPlayer::User)
        };
        battle.cards[index] = basic_scene::card_view(position, sorting_key);
        battle.cards[index].position.sorting_key = 999;
    }

    battle.interface.screen_overlay = None;
    battle.interface.primary_action_button = Some(ButtonView {
        label: "End Turn".to_string(),
        action: Some(GameAction::DebugAction(DebugAction::ApplyTestScenarioAction)),
    });

    clear_all_statuses(&mut battle);
    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

    let fire_projectile = Command::FireProjectile(FireProjectileCommand {
        source_id: GameObjectId::CardId(source_card_id),
        target_id: GameObjectId::CardId(card_id),
        projectile: ProjectileAddress { projectile: "Assets/ThirdParty/Hovl Studio/AAA Projectiles Vol 1/Prefabs/Dreamtides/Projectile 2 electro.prefab".to_string() },
        travel_duration: None,
        fire_sound: Some(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Electric Magic/RPG3_ElectricMagic_Cast02.wav")),
        impact_sound: Some(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Electric Magic/RPG3_ElectricMagic2_LightImpact01.wav")),
        additional_hit: None,
        additional_hit_delay: None,
        wait_duration: None,
        hide_on_hit: false,
        jump_to_position: None,
    });

    CommandSequence {
        groups: vec![
            ParallelCommandGroup { commands: vec![fire_projectile] },
            ParallelCommandGroup {
                commands: vec![Command::DissolveCard(DissolveCardCommand {
                    target: card_id,
                    reverse: false,
                    material: MaterialAddress::new(
                        "Assets/Content/Dissolves/Dissolve15.mat".to_string(),
                    ),
                    color: display_color::BLUE_500,
                    dissolve_speed: None,
                })],
            },
            ParallelCommandGroup {
                commands: vec![
                    Command::UpdateBattle(UpdateBattleCommand {
                        battle,
                        update_sound: Some(AudioClipAddress::new(
                            "Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Generic Magic and Impacts/RPG3_Generic_SubtleWhoosh04.wav")),
                    }),
                    Command::DissolveCard(DissolveCardCommand {
                        target: card_id,
                        reverse: true,
                        material: MaterialAddress::new("Assets/Content/Dissolves/Dissolve15.mat".to_string()),
                        color: display_color::BLUE_500,
                        dissolve_speed: None,
                    }),
                ],
            },
            ParallelCommandGroup { commands: vec![] },
        ],
    }
}

fn play_card_with_targets(battle: &mut BattleView, card_id: CardId, stack: StackType) {
    let Some((card_index, card)) = battle.cards.iter().enumerate().find(|(_, c)| c.id == card_id)
    else {
        panic!("Card not found: {:?}", card_id);
    };
    let sorting_key = card.position.sorting_key;
    battle.cards[card_index] = basic_scene::card_view(Position::OnStack(stack), sorting_key);
    if stack == StackType::TargetingBothBattlefields {
        battle.cards[card_index].position.sorting_key = 999;
    }

    // Move any other cards currently on any stack position to the new stack
    // position
    for card in battle.cards.iter_mut() {
        if matches!(card.position.position, Position::OnStack(_)) && card.id != card_id {
            card.position.position = Position::OnStack(stack);
        }
    }

    battle.interface.screen_overlay = Some(select_target_message());
    battle.interface.primary_action_button = None;
    set_can_play_to_false(battle);
    for card in battle.cards.iter_mut() {
        if matches!(card.position.position, Position::OnBattlefield(DisplayPlayer::Enemy)) {
            if let Some(revealed) = &mut card.revealed {
                revealed.actions.on_click = Some(GameAction::BattleAction(
                    BattleAction::SelectCharacterTarget(CharacterId(card.id)),
                ));
                revealed.outline_color = Some(display_color::RED_500);
            }
        }
    }
}

fn play_card_with_order_selector(
    battle: &mut BattleView,
    commands: &mut Vec<Command>,
    card_id: CardId,
) {
    let Some((card_index, card)) = battle.cards.iter().enumerate().find(|(_, c)| c.id == card_id)
    else {
        panic!("Card not found: {:?}", card_id);
    };
    let sorting_key = card.position.sorting_key;
    battle.cards[card_index] =
        basic_scene::card_view(Position::OnStack(StackType::Default), sorting_key);
    battle.cards[card_index].position.sorting_key = 500;
    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));

    commands.push(Command::DisplayEffect(DisplayEffectCommand {
        target: GameObjectId::CardId(card_id),
        effect: EffectAddress::new(
            "Assets/ThirdParty/Hovl Studio/Magic hits/Prefabs/_Hit 10.prefab",
        ),
        duration: Milliseconds::new(500),
        scale: FlexVector3::new(5.0, 5.0, 5.0),
        sound: Some(AudioClipAddress::new(
            "Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Generic Magic and Impacts/RPG3_Generic_SubtleWhoosh04.wav",
        )),
    }));
    battle.cards[card_index] =
        basic_scene::card_view(Position::InVoid(DisplayPlayer::User), sorting_key);
    commands.push(Command::UpdateBattle(
        UpdateBattleCommand::new(battle.clone()).with_update_sound(
            AudioClipAddress::new(
                "Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Generic Magic and Impacts/RPG3_Magic2_Projectiles02.wav",
            ),
        ),
    ));
    let c1 = draw_card(battle);
    let c2 = draw_card(battle);
    *ORDER_SELECTOR_VISIBLE.lock().unwrap() = true;
    *CARD_ORDER_ORIGINAL_POSITIONS.lock().unwrap() = std::collections::HashMap::new();
    battle.interface.bottom_right_button = Some(ButtonView {
        label: "\u{f070} Hide Browser".to_string(),
        action: Some(GameAction::BattleAction(BattleAction::ToggleOrderSelectorVisibility)),
    });

    if let (Some(c1_view), Some(c2_view)) = (c1, c2) {
        for card in battle.cards.iter_mut() {
            if card.id == c1_view.id || card.id == c2_view.id {
                card.position.position =
                    Position::CardOrderSelector(CardOrderSelectionTarget::Deck);
                card.revealed.as_mut().unwrap().actions.can_select_order = true;
            }
        }
    }
    battle.interface.card_order_selector =
        Some(CardOrderSelectorView { include_deck: true, include_void: true });
    battle.interface.primary_action_button = Some(ButtonView {
        label: "End Turn".to_string(),
        action: Some(GameAction::DebugAction(DebugAction::ApplyTestScenarioAction)),
    });

    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())
        .with_update_sound(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Electric Magic/RPG3_ElectricMagic_Cast02.wav"))));
}

fn select_card_order(select_order: SelectCardOrder) -> CommandSequence {
    let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
    if let Some(card_index) = battle.cards.iter().position(|card| card.id == select_order.card_id) {
        battle.cards[card_index].position.position =
            Position::CardOrderSelector(select_order.target);
    }

    let mut selector_cards: Vec<(usize, CardId, u32)> = battle
        .cards
        .iter()
        .enumerate()
        .filter_map(|(idx, card)| {
            if card.position.position == Position::CardOrderSelector(select_order.target) {
                Some((idx, card.id, card.position.sorting_key))
            } else {
                None
            }
        })
        .collect();

    selector_cards.sort_by_key(|(_, _, key)| *key);

    if let Some(selected_idx) =
        selector_cards.iter().position(|(_, id, _)| *id == select_order.card_id)
    {
        let (card_index, _, _) = selector_cards.remove(selected_idx);
        let new_position = select_order.position.min(selector_cards.len());
        selector_cards.insert(new_position, (card_index, select_order.card_id, 0));
        for (i, (idx, _, _)) in selector_cards.iter().enumerate() {
            battle.cards[*idx].position.position = Position::CardOrderSelector(select_order.target);
            battle.cards[*idx].position.sorting_key = i as u32;
        }
    }

    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

    CommandSequence::sequential(vec![Command::UpdateBattle(UpdateBattleCommand::new(battle))])
}

fn apply_test_scenario_action(scenario: &str) -> CommandSequence {
    let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
    let mut commands = Vec::new();
    match scenario {
        "user_judgment_phase" => trigger_user_judgment_phase(&mut battle, &mut commands),
        "respond_to_enemy_card" => respond_to_enemy_card(&mut battle, &mut commands),
        _ => {
            panic!("Scenario not implemented: {:?}", scenario);
        }
    }
    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));
    *CURRENT_BATTLE.lock().unwrap() = Some(battle);
    CommandSequence::sequential(commands)
}

fn respond_to_enemy_card(battle: &mut BattleView, commands: &mut Vec<Command>) {
    if let Some((card_index, card)) = battle
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| matches!(c.position.position, Position::InHand(DisplayPlayer::Enemy)))
    {
        let sorting_key = card.position.sorting_key;
        let card_id = card.id;
        let target_id = battle
            .cards
            .iter()
            .find(|c| matches!(c.position.position, Position::OnBattlefield(DisplayPlayer::User)))
            .map(|c| c.id)
            .unwrap_or(card_id); // Fallback to the card's own ID if no target found
        battle.cards[card_index] =
            basic_scene::card_view(Position::OnStack(StackType::Default), sorting_key);
        commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));
        commands.push(Command::Wait(Milliseconds::new(500)));
        battle.cards[card_index] = basic_scene::card_view(
            Position::OnStack(StackType::TargetingUserBattlefield),
            sorting_key,
        );
        commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));
        commands.push(Command::DisplayArrows(DisplayArrowsCommand {
            arrows: vec![DisplayArrow {
                source: GameObjectId::CardId(card_id),
                target: GameObjectId::CardId(target_id),
                color: ArrowStyle::Red,
            }],
        }));
    }
}

fn trigger_user_judgment_phase(battle: &mut BattleView, commands: &mut Vec<Command>) {
    battle.user.energy = Energy(3);
    battle.user.produced_energy = Energy(3);
    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

    let dreamwell_card_id = battle
        .cards
        .iter()
        .find(|card| matches!(card.position.position, Position::InDreamwell(DisplayPlayer::User)))
        .map(|card| card.id)
        .unwrap_or(battle.cards[0].id);

    commands.extend(vec![
        Command::DisplayGameMessage(GameMessageType::YourTurn),
        Command::DisplayJudgment(DisplayJudgmentCommand {
            player: DisplayPlayer::User,
            new_score: None,
        }),
        Command::DisplayDreamwellActivation(DisplayDreamwellActivationCommand {
            card_id: dreamwell_card_id,
            player: DisplayPlayer::User,
            new_energy: Some(Energy(3)),
            new_produced_energy: Some(Energy(3)),
        }),
    ]);
}

fn select_target_message() -> FlexNode {
    let style = FlexStyle {
        background_color: Some(DisplayColor { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.95 }),
        border_radius: Some(BorderRadius {
            top_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            top_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        padding: Some(DimensionGroup {
            top: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        color: Some(DisplayColor { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 }),
        font_size: Some(Dimension { unit: DimensionUnit::Pixels, value: 10.0 }),
        min_height: Some(Dimension { unit: DimensionUnit::Pixels, value: 22.0 }),
        white_space: Some(WhiteSpace::Normal),
        text_align: Some(TextAlign::MiddleCenter),
        ..Default::default()
    };

    let message = FlexNode {
        node_type: Some(NodeType::Text(TextNode { label: "Choose an enemy character".into() })),
        style: Some(style),
        ..Default::default()
    };

    FlexNode {
        style: Some(FlexStyle {
            position: Some(FlexPosition::Absolute),
            inset: Some(FlexInsets {
                top: None,
                right: Some(Dimension { unit: DimensionUnit::Pixels, value: 32.0 }),
                bottom: Some(Dimension { unit: DimensionUnit::Pixels, value: 50.0 }),
                left: Some(Dimension { unit: DimensionUnit::Pixels, value: 32.0 }),
            }),
            ..Default::default()
        }),
        children: vec![message],
        ..Default::default()
    }
}

fn draw_card(battle: &mut BattleView) -> Option<CardView> {
    if let Some(deck_card) = battle
        .cards
        .iter()
        .find(|c| matches!(c.position.position, Position::InDeck(DisplayPlayer::User)))
    {
        let card_id = deck_card.id;
        let sorting_key = deck_card.position.sorting_key;
        battle.user.total_spark += Spark(11);
        if let Some(card_index) = battle.cards.iter().position(|c| c.id == card_id) {
            let mut shown_drawn = battle.clone();
            shown_drawn.cards[card_index] = basic_scene::card_view(Position::Drawn, sorting_key);
            let view = basic_scene::card_view(Position::InHand(DisplayPlayer::User), sorting_key);
            battle.cards[card_index] = view.clone();
            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
            Some(view)
        } else {
            None
        }
    } else {
        None
    }
}

fn set_can_play_to_false(battle: &mut BattleView) {
    for card in battle.cards.iter_mut() {
        if let Some(revealed) = card.revealed.as_mut() {
            revealed.actions.can_play = false;
            revealed.outline_color = None;
        }
    }
}

fn clear_all_statuses(battle: &mut BattleView) {
    for card in battle.cards.iter_mut() {
        if let Some(revealed) = card.revealed.as_mut() {
            revealed.outline_color = None;
        }
    }
}
