use std::sync::{LazyLock, Mutex};

use action_data::battle_action::BattleAction;
use action_data::debug_action::DebugAction;
use action_data::user_action::UserAction;
use core_data::display_types::{
    AudioClipAddress, DisplayColor, EffectAddress, Milliseconds, ProjectileAddress, SpriteAddress,
    Url,
};
use core_data::identifiers::{BattleId, CardId};
use core_data::numerics::{Energy, Points, Spark};
use core_data::types::CardFacing;
use display_data::battle_view::{BattleView, DisplayPlayer, InterfaceView, PlayerView};
use display_data::card_view::{
    CardActions, CardEffects, CardFrame, CardPrefab, CardView, DisplayImage, RevealedCardStatus,
    RevealedCardView,
};
use display_data::command::{
    Command, CommandSequence, DisplayEffectCommand, DissolveCardCommand, DrawUserCardsCommand,
    FireProjectileCommand, GameObjectId, ParallelCommandGroup, UpdateBattleCommand,
};
use display_data::object_position::{ObjectPosition, Position};
use display_data::request_data::{
    ConnectRequest, ConnectResponse, Metadata, PerformActionRequest, PerformActionResponse,
};
use masonry::flex_enums::{FlexPosition, TextAlign, WhiteSpace};
use masonry::flex_node::{FlexNode, NodeType, Text};
use masonry::flex_style::{
    BorderRadius, Dimension, DimensionGroup, DimensionUnit, FlexInsets, FlexStyle, FlexVector3,
};
use uuid::Uuid;

static CURRENT_BATTLE: LazyLock<Mutex<Option<BattleView>>> = LazyLock::new(|| Mutex::new(None));

pub fn connect(request: &ConnectRequest) -> ConnectResponse {
    let battle = scene_0(BattleId(Uuid::new_v4()));
    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
    ConnectResponse {
        metadata: request.metadata,
        commands: CommandSequence::from_command(Command::UpdateBattle(UpdateBattleCommand::new(
            battle,
        ))),
    }
}

pub fn perform_action(request: &PerformActionRequest) -> PerformActionResponse {
    match request.action {
        UserAction::DebugAction(action) => perform_debug_action(action, request.metadata),
        UserAction::BattleAction(action) => perform_battle_action(action, request.metadata),
    }
}

fn perform_debug_action(action: DebugAction, metadata: Metadata) -> PerformActionResponse {
    match action {
        DebugAction::DrawCard => {
            let mut commands = vec![];
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
            let card = draw_card(&mut battle);
            commands.push(Command::DrawUserCards(DrawUserCardsCommand {
                cards: vec![card.unwrap()],
                stagger_interval: Milliseconds::new(100),
                pause_duration: Milliseconds::new(100),
            }));
            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
            commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle)));
            PerformActionResponse { metadata, commands: CommandSequence::sequential(commands) }
        }
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
        if let Some(card_index) = battle.cards.iter().position(|c| c.id == card_id) {
            let mut shown_drawn = battle.clone();
            shown_drawn.cards[card_index] = card_view(Position::Drawn, sorting_key);
            let view = card_view(Position::InHand(DisplayPlayer::User), sorting_key);
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

fn perform_battle_action(action: BattleAction, metadata: Metadata) -> PerformActionResponse {
    match action {
        BattleAction::PlayCard(card_id) => {
            let mut commands = Vec::new();
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
            if let Some((card_index, card)) =
                battle.cards.iter().enumerate().find(|(_, c)| c.id == card_id)
            {
                let sorting_key = card.position.sorting_key;
                let position = if sorting_key % 5 == 1 {
                    battle.interface.screen_overlay = Some(select_target_message());
                    battle.interface.primary_action_button = None;
                    set_can_play_to_false(&mut battle);
                    for card in battle.cards.iter_mut() {
                        if matches!(
                            card.position.position,
                            Position::OnBattlefield(DisplayPlayer::Enemy)
                        ) {
                            if let Some(revealed) = &mut card.revealed {
                                revealed.actions.on_click = Some(UserAction::BattleAction(
                                    BattleAction::SelectTarget(card.id),
                                ));
                                revealed.status = Some(RevealedCardStatus::CanSelectNegative);
                            }
                        }
                    }
                    Position::SelectingTargets(DisplayPlayer::Enemy)
                } else if sorting_key % 5 == 2 {
                    battle.cards[card_index] = card_view(Position::OnStack, sorting_key);
                    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));
                    let c1 = draw_card(&mut battle);
                    let c2 = draw_card(&mut battle);
                    commands.push(Command::DisplayEffect(DisplayEffectCommand {
                        target: GameObjectId::Deck(DisplayPlayer::User),
                        effect: EffectAddress::new("Assets/ThirdParty/Hovl Studio/Magic circles/Prefabs/Magic circle 1 Variant.prefab"),
                        duration: Milliseconds::new(100),
                        scale: FlexVector3::one(),
                        sound: Some(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Generic Magic and Impacts/RPG3_Magic2_Cast03v1.wav"))
                    }));
                    commands.push(Command::DrawUserCards(DrawUserCardsCommand {
                        cards: vec![c1.unwrap(), c2.unwrap()],
                        stagger_interval: Milliseconds::new(300),
                        pause_duration: Milliseconds::new(100),
                    }));
                    Position::InVoid(DisplayPlayer::User)
                } else if sorting_key % 5 == 0 {
                    let mut battle_clone = battle.clone();
                    battle_clone.cards[card_index] =
                        card_view(Position::OnBattlefield(DisplayPlayer::User), sorting_key);
                    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(
                        battle_clone.clone(),
                    )));
                    let mut trigger_card = card_view(Position::OnStack, 1234);
                    trigger_card.prefab = CardPrefab::Token;
                    trigger_card.create_position = Some(ObjectPosition {
                        position: Position::HiddenWithinCard(card.id),
                        sorting_key: 1,
                        sorting_sub_key: 0,
                    });
                    trigger_card.destroy_position = Some(ObjectPosition {
                        position: Position::HiddenWithinCard(card.id),
                        sorting_key: 1,
                        sorting_sub_key: 0,
                    });
                    battle_clone.cards.push(trigger_card);
                    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle_clone)));
                    commands.push(Command::Wait(Milliseconds::new(1000)));
                    let c1 = draw_card(&mut battle);
                    commands.push(Command::DisplayEffect(DisplayEffectCommand {
                        target: GameObjectId::Deck(DisplayPlayer::User),
                        effect: EffectAddress::new("Assets/ThirdParty/Hovl Studio/Magic circles/Prefabs/Magic circle 1 Variant.prefab"),
                        duration: Milliseconds::new(100),
                        scale: FlexVector3::one(),
                        sound: Some(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Generic Magic and Impacts/RPG3_Magic2_Cast03v1.wav"))
                    }));
                    commands.push(Command::DrawUserCards(DrawUserCardsCommand {
                        cards: vec![c1.unwrap()],
                        stagger_interval: Milliseconds::new(300),
                        pause_duration: Milliseconds::new(100),
                    }));
                    Position::OnBattlefield(DisplayPlayer::User)
                } else {
                    Position::OnBattlefield(DisplayPlayer::User)
                };
                battle.cards[card_index] = card_view(position, sorting_key);
            }

            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
            commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle)));
            PerformActionResponse { metadata, commands: CommandSequence::sequential(commands) }
        }
        BattleAction::SelectTarget(card_id) => {
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();

            // First collect all the cards we need to move
            let cards_to_move: Vec<(usize, u32)> = battle
                .cards
                .iter()
                .enumerate()
                .filter_map(|(index, card)| {
                    if card.id == card_id
                        || matches!(
                            card.position.position,
                            Position::SelectingTargets(DisplayPlayer::Enemy)
                        )
                    {
                        Some((index, card.position.sorting_key))
                    } else {
                        None
                    }
                })
                .collect();

            // Find the source card (the one in SelectingTargets position)
            let source_card_id = battle
                .cards
                .iter()
                .find_map(|card| {
                    if matches!(
                        card.position.position,
                        Position::SelectingTargets(DisplayPlayer::Enemy)
                    ) {
                        Some(card.id)
                    } else {
                        None
                    }
                })
                .unwrap();

            // Now update all the cards
            for (index, sorting_key) in cards_to_move {
                let position = if battle.cards[index].id == card_id {
                    Position::InVoid(DisplayPlayer::Enemy)
                } else {
                    Position::InVoid(DisplayPlayer::User)
                };
                battle.cards[index] = card_view(position, sorting_key);
                set_sorting_key(&mut battle.cards[index], 999);
            }

            // Reset interface state
            battle.interface.screen_overlay = None;
            battle.interface.primary_action_button = Some("End Turn".to_string());

            clear_all_statuses(&mut battle);
            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

            // Create the fire projectile command
            let fire_projectile = Command::FireProjectile(FireProjectileCommand {
                source_id: GameObjectId::CardId(source_card_id),
                target_id: GameObjectId::CardId(card_id),
                projectile: ProjectileAddress { projectile: "Assets/ThirdParty/Hovl Studio/AAA Projectiles Vol 1/Prefabs/Projectiles(transform)/Projectile 2 electro.prefab".to_string() },
                travel_duration: None,
                fire_sound: Some(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Electric Magic/RPG3_ElectricMagic_Cast02.wav")),
                impact_sound: Some(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Electric Magic/RPG3_ElectricMagic2_LightImpact01.wav")),
                additional_hit: None,
                additional_hit_delay: None,
                wait_duration: None,
                hide_on_hit: false,
                jump_to_position: None,
            });

            PerformActionResponse {
                metadata,
                commands: CommandSequence {
                    groups: vec![
                        ParallelCommandGroup { commands: vec![fire_projectile] },
                        ParallelCommandGroup { commands: vec![
                            Command::DissolveCard(DissolveCardCommand { target: card_id, reverse: false }),
                        ] },
                        ParallelCommandGroup { commands: vec![
                            Command::UpdateBattle(UpdateBattleCommand {
                                battle,
                                update_sound: Some(AudioClipAddress::new(
                                    "Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Generic Magic and Impacts/RPG3_Generic_SubtleWhoosh04.wav")),
                            }),
                            Command::DissolveCard(DissolveCardCommand {
                                target: card_id,
                                reverse: true,
                            }),
                        ] },
                        ParallelCommandGroup { commands: vec![] },
                    ],
                },
            }
        }
    }
}

fn set_can_play_to_false(battle: &mut BattleView) {
    for card in battle.cards.iter_mut() {
        if let Some(revealed) = card.revealed.as_mut() {
            revealed.actions.can_play = false;
        }
    }
}

fn clear_all_statuses(battle: &mut BattleView) {
    for card in battle.cards.iter_mut() {
        if let Some(revealed) = card.revealed.as_mut() {
            revealed.status = None;
        }
    }
}

fn set_sorting_key(card: &mut CardView, sorting_key: u32) {
    card.position.sorting_key = sorting_key;
}

fn scene_0(id: BattleId) -> BattleView {
    BattleView {
        id,
        user: PlayerView { score: Points(0), can_act: false },
        enemy: PlayerView { score: Points(0), can_act: false },
        cards: [
            cards_in_position(Position::OnBattlefield(DisplayPlayer::User), 0, 5),
            cards_in_position(Position::InHand(DisplayPlayer::User), 5, 3),
            cards_in_position(Position::InVoid(DisplayPlayer::User), 8, 6),
            cards_in_position(Position::InDeck(DisplayPlayer::User), 14, 20),
            cards_in_position(Position::OnBattlefield(DisplayPlayer::Enemy), 100, 8),
            cards_in_position(Position::InHand(DisplayPlayer::Enemy), 105, 3),
            cards_in_position(Position::InVoid(DisplayPlayer::Enemy), 108, 6),
            cards_in_position(Position::InDeck(DisplayPlayer::Enemy), 114, 20),
        ]
        .concat()
        .to_vec(),
        status_description: "Status".to_string(),
        interface: InterfaceView {
            primary_action_button: Some("End Turn".to_string()),
            ..Default::default()
        },
    }
}

fn cards_in_position(position: Position, start_key: u32, count: u32) -> Vec<CardView> {
    (0..count).map(|i| card_view(position, start_key + i)).collect()
}

fn card_view(position: Position, sorting_key: u32) -> CardView {
    if sorting_key % 5 == 0 {
        card1(position, sorting_key)
    } else if sorting_key % 5 == 1 {
        card2(position, sorting_key)
    } else if sorting_key % 5 == 2 {
        card3(position, sorting_key)
    } else if sorting_key % 5 == 3 {
        card4(position, sorting_key)
    } else {
        card5(position, sorting_key)
    }
}

fn card1(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        card_back: Url::new("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new("Assets/ThirdParty/GameAssets/CardImages/2521694543.png"),
            },
            name: "Titan of Forgotten Echoes".to_string(),
            rules_text: "When you materialize your second character in a turn, return this character from your void to play.".to_string(),
            status: None,
            cost: Energy(6),
            spark: Some(Spark(4)),
            card_type: "Ancient".to_string(),
            frame: CardFrame::Character,
            supplemental_card_info: flex_node("<b>Materialize</b>: A character entering play."),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(DisplayPlayer::User),
                ..Default::default()
            },
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Default,
    }
}

fn card2(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        card_back: Url::new("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/1633431262.png",
                ),
            },
            name: "Beacon of Tomorrow".to_string(),
            rules_text: "Discover a card with cost (2).".to_string(),
            status: None,
            cost: Energy(2),
            spark: None,
            card_type: "Event".to_string(),
            frame: CardFrame::Event,
            supplemental_card_info: flex_node(
                "<b>Discover</b>: Pick one of 4 cards with different types to put into your hand.",
            ),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(DisplayPlayer::User),
                on_play_sound: Some(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Electric Magic/RPG3_ElectricMagic_Cast01.wav")),
                ..Default::default()
            },
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Default,
    }
}

fn card3(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        card_back: Url::new("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/2269064817.png",
                ),
            },
            name: "Scrap Reclaimer".to_string(),
            rules_text: "Judgment: Return this character from your void to your hand. Born from rust and resilience.".to_string(),
            status: None,
            cost: Energy(4),
            spark: Some(Spark(0)),
            card_type: "Tinkerer".to_string(),
            frame: CardFrame::Character,
            supplemental_card_info: flex_node(
                "<b>Judgment</b>: Triggers at the start of your turn."),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(DisplayPlayer::User),
                ..Default::default()
            },
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Default,
    }
}

fn card4(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        card_back: Url::new("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/2269064809.png",
                ),
            },
            name: "Evacuation Enforcer".to_string(),
            rules_text: "> Draw 2 cards. Discard 3 cards.\nPromises under a stormy sky."
                .to_string(),
            status: None,
            cost: Energy(2),
            spark: Some(Spark(0)),
            card_type: "Trooper".to_string(),
            frame: CardFrame::Character,
            supplemental_card_info: None,
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(DisplayPlayer::User),
                ..Default::default()
            },
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Default,
    }
}

fn card5(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        card_back: Url::new("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/2027158310.png",
                ),
            },
            name: "Moonlit Voyage".to_string(),
            rules_text: "Draw 2 cards. Discard 2 cards.\nReclaim".to_string(),
            status: None,
            cost: Energy(2),
            spark: None,
            card_type: "Event".to_string(),
            frame: CardFrame::Event,
            supplemental_card_info: flex_node(
                "<b>Reclaim</b>: You may play this card from your void, then banish it.",
            ),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(DisplayPlayer::User),
                ..Default::default()
            },
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Token,
    }
}

fn flex_node(text: impl Into<String>) -> Option<FlexNode> {
    let style = FlexStyle {
        background_color: Some(DisplayColor { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.95 }),
        border_radius: Some(BorderRadius {
            top_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            top_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        padding: Some(DimensionGroup {
            top: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
            right: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
            bottom: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
            left: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
        }),
        color: Some(DisplayColor { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 }),
        font_size: Some(Dimension { unit: DimensionUnit::Pixels, value: 14.0 }),
        max_width: Some(Dimension { unit: DimensionUnit::Percentage, value: 80.0 }),
        min_height: Some(Dimension { unit: DimensionUnit::Pixels, value: 44.0 }),
        white_space: Some(WhiteSpace::Normal),
        ..Default::default()
    };
    Some(FlexNode {
        node_type: Some(NodeType::Text(Text { label: text.into() })),
        style: Some(style),
        ..Default::default()
    })
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
            top: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
            right: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
            bottom: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
            left: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
        }),
        color: Some(DisplayColor { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 }),
        font_size: Some(Dimension { unit: DimensionUnit::Pixels, value: 20.0 }),
        min_height: Some(Dimension { unit: DimensionUnit::Pixels, value: 44.0 }),
        white_space: Some(WhiteSpace::Normal),
        text_align: Some(TextAlign::MiddleCenter),
        ..Default::default()
    };

    let message = FlexNode {
        node_type: Some(NodeType::Text(Text { label: "Choose an enemy character".into() })),
        style: Some(style),
        ..Default::default()
    };

    FlexNode {
        style: Some(FlexStyle {
            position: Some(FlexPosition::Absolute),
            inset: Some(FlexInsets {
                top: None,
                right: Some(Dimension { unit: DimensionUnit::Pixels, value: 32.0 }),
                bottom: Some(Dimension { unit: DimensionUnit::Pixels, value: 150.0 }),
                left: Some(Dimension { unit: DimensionUnit::Pixels, value: 32.0 }),
            }),
            ..Default::default()
        }),
        children: vec![message],
        ..Default::default()
    }
}
