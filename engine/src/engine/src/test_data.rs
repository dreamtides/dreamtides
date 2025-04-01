use std::sync::{LazyLock, Mutex};

use action_data::battle_action::{
    BattleAction, CardBrowserType, CardOrderSelectionTarget, SelectCardOrder,
};
use action_data::debug_action::DebugAction;
use action_data::user_action::UserAction;
use core_data::display_color::{self, DisplayColor};
use core_data::display_types::{
    AudioClipAddress, EffectAddress, Milliseconds, ProjectileAddress, SpriteAddress, Url,
};
use core_data::identifiers::{BattleId, CardId};
use core_data::numerics::{Energy, Points, Spark};
use core_data::types::{CardFacing, PlayerName};
use display_data::battle_view::{
    BattleView, ButtonView, CardOrderSelectorView, InterfaceView, PlayerView,
};
use display_data::card_view::{
    CardActions, CardEffects, CardFrame, CardPrefab, CardView, DisplayImage, RevealedCardView,
};
use display_data::command::{
    Command, CommandSequence, DisplayDreamwellActivationCommand, DisplayEffectCommand,
    DisplayJudgmentCommand, DissolveCardCommand, DrawUserCardsCommand, FireProjectileCommand,
    GameMessageType, GameObjectId, ParallelCommandGroup, UpdateBattleCommand,
};
use display_data::object_position::{ObjectPosition, Position, SelectingTargets};
use display_data::request_data::{
    ConnectRequest, ConnectResponse, Metadata, PerformActionRequest, PerformActionResponse,
};
use masonry::flex_enums::{FlexPosition, TextAlign, WhiteSpace};
use masonry::flex_node::{FlexNode, NodeType, Text};
use masonry::flex_style::{
    BorderRadius, Dimension, DimensionGroup, DimensionUnit, FlexInsets, FlexStyle, FlexVector3,
};
use uuid::Uuid;

static RESOLVING_MULLIGAN: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
static CURRENT_BATTLE: LazyLock<Mutex<Option<BattleView>>> = LazyLock::new(|| Mutex::new(None));
static CARD_BROWSER_SOURCE: LazyLock<Mutex<Option<Position>>> = LazyLock::new(|| Mutex::new(None));
static ORDER_SELECTOR_VISIBLE: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
static CARD_ORDER_ORIGINAL_POSITIONS: LazyLock<Mutex<std::collections::HashMap<CardId, Position>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));
const STUFF_TO_DO: u32 = 3;

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

pub fn perform_debug_action(action: DebugAction, metadata: Metadata) -> PerformActionResponse {
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
        DebugAction::TriggerUserJudgment => {
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();

            battle.user.energy = Energy(3);
            battle.user.produced_energy = Energy(3);
            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

            let dreamwell_card_id = battle
                .cards
                .iter()
                .find(|card| {
                    matches!(card.position.position, Position::InDreamwell(PlayerName::User))
                })
                .map(|card| card.id)
                .unwrap_or(battle.cards[0].id);

            PerformActionResponse {
                metadata,
                commands: CommandSequence::sequential(vec![
                    Command::DisplayGameMessage(GameMessageType::YourTurn),
                    Command::DisplayJudgment(DisplayJudgmentCommand {
                        player: PlayerName::User,
                        new_score: None,
                    }),
                    Command::DisplayDreamwellActivation(DisplayDreamwellActivationCommand {
                        card_id: dreamwell_card_id,
                        player: PlayerName::User,
                        new_energy: Some(Energy(3)),
                        new_produced_energy: Some(Energy(3)),
                    }),
                    Command::UpdateBattle(UpdateBattleCommand::new(battle)),
                ]),
            }
        }
        DebugAction::TriggerEnemyJudgment => {
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();

            battle.enemy.energy = Energy(3);
            battle.enemy.produced_energy = Energy(3);
            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
            let dreamwell_card_id = battle
                .cards
                .iter()
                .find(|card| {
                    matches!(card.position.position, Position::InDreamwell(PlayerName::Enemy))
                })
                .map(|card| card.id)
                .unwrap_or(battle.cards[0].id);

            PerformActionResponse {
                metadata,
                commands: CommandSequence::sequential(vec![
                    Command::DisplayGameMessage(GameMessageType::EnemyTurn),
                    Command::DisplayJudgment(DisplayJudgmentCommand {
                        player: PlayerName::Enemy,
                        new_score: Some(Points(10)),
                    }),
                    Command::DisplayDreamwellActivation(DisplayDreamwellActivationCommand {
                        card_id: dreamwell_card_id,
                        player: PlayerName::Enemy,
                        new_energy: Some(Energy(3)),
                        new_produced_energy: Some(Energy(3)),
                    }),
                    Command::UpdateBattle(UpdateBattleCommand::new(battle)),
                ]),
            }
        }
        DebugAction::PerformSomeAction => {
            if STUFF_TO_DO == 1 {
                // Set mulligan state
                let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();

                // Set the global RESOLVING_MULLIGAN flag to true
                *RESOLVING_MULLIGAN.lock().unwrap() = true;

                // Update all cards in the user's hand to have "select card" action and disable
                // "can play"
                for card in battle.cards.iter_mut() {
                    if matches!(card.position.position, Position::InHand(PlayerName::User)) {
                        if let Some(revealed) = &mut card.revealed {
                            // Set on_click action to select this card
                            revealed.actions.on_click =
                                Some(UserAction::BattleAction(BattleAction::SelectCard(card.id)));
                            // Disable can_play action
                            revealed.actions.can_play = false;
                            // Set visual status to indicate selectable
                            revealed.outline_color = Some(display_color::WHITE);
                        }
                    }
                }

                // Add a screen overlay with mulligan instructions
                battle.interface.screen_overlay = Some(mulligan_message());

                // Disable the primary action button during mulligan
                battle.interface.primary_action_button = None;

                let commands =
                    vec![Command::UpdateBattle(UpdateBattleCommand::new(battle.clone()))];
                *CURRENT_BATTLE.lock().unwrap() = Some(battle);

                PerformActionResponse { metadata, commands: CommandSequence::sequential(commands) }
            } else if STUFF_TO_DO == 3 {
                // Find the first card in enemy hand
                let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
                let mut commands = Vec::new();

                if let Some((card_index, card)) = battle.cards.iter().enumerate().find(|(_, c)| {
                    matches!(c.position.position, Position::InHand(PlayerName::Enemy))
                }) {
                    let sorting_key = card.position.sorting_key;

                    battle.cards[card_index] = card_view(
                        Position::SelectingTargets(SelectingTargets {
                            actor: PlayerName::Enemy,
                            targeting: PlayerName::User,
                        }),
                        sorting_key,
                    );
                    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));
                    *CURRENT_BATTLE.lock().unwrap() = Some(battle);
                }

                PerformActionResponse { metadata, commands: CommandSequence::sequential(commands) }
            } else {
                // Find the first card in enemy hand
                let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
                let mut commands = Vec::new();

                if let Some((card_index, card)) = battle.cards.iter().enumerate().find(|(_, c)| {
                    matches!(c.position.position, Position::InHand(PlayerName::Enemy))
                }) {
                    let sorting_key = card.position.sorting_key;

                    // Move card to stack
                    battle.cards[card_index] = card_view(Position::OnStack, sorting_key);
                    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));

                    // Wait for animation
                    commands.push(Command::Wait(Milliseconds::new(1500)));

                    // Move card to battlefield
                    battle.cards[card_index] =
                        card_view(Position::OnBattlefield(PlayerName::Enemy), sorting_key);
                    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));

                    *CURRENT_BATTLE.lock().unwrap() = Some(battle);
                }

                PerformActionResponse { metadata, commands: CommandSequence::sequential(commands) }
            }
        }
    }
}

fn draw_card(battle: &mut BattleView) -> Option<CardView> {
    if let Some(deck_card) = battle
        .cards
        .iter()
        .find(|c| matches!(c.position.position, Position::InDeck(PlayerName::User)))
    {
        let card_id = deck_card.id;
        let sorting_key = deck_card.position.sorting_key;
        battle.user.total_spark += Spark(11);
        if let Some(card_index) = battle.cards.iter().position(|c| c.id == card_id) {
            let mut shown_drawn = battle.clone();
            shown_drawn.cards[card_index] = card_view(Position::Drawn, sorting_key);
            let view = card_view(Position::InHand(PlayerName::User), sorting_key);
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
                            Position::OnBattlefield(PlayerName::Enemy)
                        ) {
                            if let Some(revealed) = &mut card.revealed {
                                revealed.actions.on_click = Some(UserAction::BattleAction(
                                    BattleAction::SelectCard(card.id),
                                ));
                                revealed.outline_color = Some(display_color::RED_500);
                            }
                        }
                    }
                    Position::SelectingTargets(SelectingTargets {
                        actor: PlayerName::User,
                        targeting: PlayerName::Enemy,
                    })
                } else if sorting_key % 5 == 2 {
                    battle.cards[card_index] = card_view(Position::OnStack, sorting_key);
                    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));
                    commands.push(Command::Wait(Milliseconds::new(500)));
                    battle.cards[card_index] =
                        card_view(Position::InVoid(PlayerName::User), sorting_key);
                    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));
                    let c1 = draw_card(&mut battle);
                    let c2 = draw_card(&mut battle);
                    *ORDER_SELECTOR_VISIBLE.lock().unwrap() = true;
                    *CARD_ORDER_ORIGINAL_POSITIONS.lock().unwrap() =
                        std::collections::HashMap::new();
                    battle.interface.bottom_right_button = Some(ButtonView {
                        label: "\u{f070} Hide Browser".to_string(),
                        action: UserAction::BattleAction(
                            BattleAction::ToggleOrderSelectorVisibility,
                        ),
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

                    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
                    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));
                    Position::InVoid(PlayerName::User)
                } else if sorting_key % 5 == 3 {
                    battle.cards[card_index] = card_view(Position::OnStack, sorting_key);
                    commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle.clone())));
                    let c1 = draw_card(&mut battle);
                    let c2 = draw_card(&mut battle);
                    commands.push(Command::DisplayEffect(DisplayEffectCommand {
                        target: GameObjectId::Deck(PlayerName::User),
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
                    Position::InVoid(PlayerName::User)
                } else if sorting_key % 5 == 0 {
                    let mut battle_clone = battle.clone();
                    battle_clone.cards[card_index] =
                        card_view(Position::OnBattlefield(PlayerName::User), sorting_key);
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
                        target: GameObjectId::Deck(PlayerName::User),
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
                    Position::OnBattlefield(PlayerName::User)
                } else {
                    Position::OnBattlefield(PlayerName::User)
                };
                battle.cards[card_index] = card_view(position, sorting_key);
            }

            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
            commands.push(Command::UpdateBattle(UpdateBattleCommand::new(battle)));
            PerformActionResponse { metadata, commands: CommandSequence::sequential(commands) }
        }
        BattleAction::SelectCard(card_id) => {
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();

            // Check if we're in mulligan mode
            if *RESOLVING_MULLIGAN.lock().unwrap() {
                // Handle card selection during mulligan
                if let Some(card_index) = battle.cards.iter().position(|card| card.id == card_id) {
                    // Toggle the selection status
                    if let Some(revealed) = &mut battle.cards[card_index].revealed {
                        if revealed.outline_color == Some(display_color::WHITE) {
                            revealed.outline_color = Some(display_color::GREEN);
                        } else {
                            revealed.outline_color = Some(display_color::WHITE);
                        }
                    }
                }

                // Add a "Confirm Mulligan" button if it doesn't exist
                if battle.interface.primary_action_button.is_none() {
                    battle.interface.primary_action_button = Some("Confirm Mulligan".to_string());
                }

                *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

                return PerformActionResponse {
                    metadata,
                    commands: CommandSequence::sequential(vec![Command::UpdateBattle(
                        UpdateBattleCommand::new(battle),
                    )]),
                };
            }

            // First collect all the cards we need to move
            let cards_to_move: Vec<(usize, u32)> = battle
                .cards
                .iter()
                .enumerate()
                .filter_map(|(index, card)| {
                    if card.id == card_id
                        || matches!(card.position.position, Position::SelectingTargets(_))
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
                    if matches!(card.position.position, Position::SelectingTargets(_)) {
                        Some(card.id)
                    } else {
                        None
                    }
                })
                .unwrap();

            // Now update all the cards
            for (index, sorting_key) in cards_to_move {
                let position = if battle.cards[index].id == card_id {
                    Position::InVoid(PlayerName::Enemy)
                } else {
                    Position::InVoid(PlayerName::User)
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
        BattleAction::SelectCardOrder(SelectCardOrder { target, card_id, position }) => {
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
            if let Some(card_index) = battle.cards.iter().position(|card| card.id == card_id) {
                battle.cards[card_index].position.position = Position::CardOrderSelector(target);
            }

            // Find all cards in the CardOrderSelector
            let mut selector_cards: Vec<(usize, CardId, u32)> = battle
                .cards
                .iter()
                .enumerate()
                .filter_map(|(idx, card)| {
                    if card.position.position == Position::CardOrderSelector(target) {
                        Some((idx, card.id, card.position.sorting_key))
                    } else {
                        None
                    }
                })
                .collect();

            // Sort cards by their current sorting key
            selector_cards.sort_by_key(|(_, _, key)| *key);

            // Find the index of the selected card
            if let Some(selected_idx) = selector_cards.iter().position(|(_, id, _)| *id == card_id)
            {
                // Remove the card from its current position
                let (card_index, _, _) = selector_cards.remove(selected_idx);

                // Insert it at the new position, capped by the number of cards
                let new_position = position.min(selector_cards.len());
                selector_cards.insert(new_position, (card_index, card_id, 0));

                // Update sorting keys for all cards in the selector
                for (i, (idx, _, _)) in selector_cards.iter().enumerate() {
                    battle.cards[*idx].position.position = Position::CardOrderSelector(target);
                    battle.cards[*idx].position.sorting_key = i as u32;
                }
            }

            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

            PerformActionResponse {
                metadata,
                commands: CommandSequence::sequential(vec![Command::UpdateBattle(
                    UpdateBattleCommand::new(battle),
                )]),
            }
        }
        BattleAction::BrowseCards(card_browser) => {
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();

            let source_position = match card_browser {
                CardBrowserType::UserDeck => Position::InDeck(PlayerName::User),
                CardBrowserType::EnemyDeck => Position::InDeck(PlayerName::Enemy),
                CardBrowserType::UserVoid => Position::InVoid(PlayerName::User),
                CardBrowserType::EnemyVoid => Position::InVoid(PlayerName::Enemy),
                CardBrowserType::UserStatus => Position::InPlayerStatus(PlayerName::User),
                CardBrowserType::EnemyStatus => Position::InPlayerStatus(PlayerName::Enemy),
            };

            // Store the source position for later use when closing browser
            *CARD_BROWSER_SOURCE.lock().unwrap() = Some(source_position);

            for card in battle.cards.iter_mut() {
                if card.position.position == source_position {
                    card.position.position = Position::Browser;
                }
            }

            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

            PerformActionResponse {
                metadata,
                commands: CommandSequence::sequential(vec![Command::UpdateBattle(
                    UpdateBattleCommand::new(battle),
                )]),
            }
        }
        BattleAction::CloseCardBrowser => {
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();

            // Get the source position from our stored global state
            let source_position = *CARD_BROWSER_SOURCE.lock().unwrap();

            if let Some(position) = source_position {
                // Move cards from browser back to the original position
                for card in battle.cards.iter_mut() {
                    if card.position.position == Position::Browser {
                        card.position.position = position;
                    }
                }
            }

            // Clear the stored source position
            *CARD_BROWSER_SOURCE.lock().unwrap() = None;
            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

            PerformActionResponse {
                metadata,
                commands: CommandSequence::sequential(vec![Command::UpdateBattle(
                    UpdateBattleCommand::new(battle),
                )]),
            }
        }
        BattleAction::ToggleOrderSelectorVisibility => {
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
            let mut is_visible = ORDER_SELECTOR_VISIBLE.lock().unwrap();
            let mut original_positions = CARD_ORDER_ORIGINAL_POSITIONS.lock().unwrap();

            // Toggle the visibility state
            *is_visible = !*is_visible;

            if *is_visible {
                battle.interface.bottom_right_button = Some(ButtonView {
                    label: "\u{f070} Hide Browser".to_string(),
                    action: UserAction::BattleAction(BattleAction::ToggleOrderSelectorVisibility),
                });

                // If toggling to visible, restore cards from storage to their original
                // positions
                for card in battle.cards.iter_mut() {
                    if card.position.position == Position::OnScreenStorage {
                        println!("Restoring original position for card {:?}", card.id);
                        if let Some(original_position) = original_positions.remove(&card.id) {
                            card.position.position = original_position;
                        }
                    }
                }
                // Clear the saved positions as they're no longer needed
                original_positions.clear();
            } else {
                battle.interface.bottom_right_button = Some(ButtonView {
                    label: "\u{f06e} Show Browser".to_string(),
                    action: UserAction::BattleAction(BattleAction::ToggleOrderSelectorVisibility),
                });

                // If toggling to hidden, move cards to storage and save original positions
                for card in battle.cards.iter_mut() {
                    if let Position::CardOrderSelector(_) = card.position.position {
                        println!("Saving original position for card {:?}", card.id);
                        original_positions.insert(card.id, card.position.position);
                        card.position.position = Position::OnScreenStorage;
                    }
                }
            }

            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

            PerformActionResponse {
                metadata,
                commands: CommandSequence::sequential(vec![Command::UpdateBattle(
                    UpdateBattleCommand::new(battle),
                )]),
            }
        }
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

fn set_sorting_key(card: &mut CardView, sorting_key: u32) {
    card.position.sorting_key = sorting_key;
}

fn scene_0(id: BattleId) -> BattleView {
    BattleView {
        id,
        user: PlayerView {
            score: Points(0),
            can_act: false,
            energy: Energy(2),
            produced_energy: Energy(2),
            total_spark: Spark(0),
        },
        enemy: PlayerView {
            score: Points(0),
            can_act: false,
            energy: Energy(2),
            produced_energy: Energy(2),
            total_spark: Spark(0),
        },
        cards: [
            cards_in_position(Position::InHand(PlayerName::User), 5, 3),
            cards_in_position(Position::InVoid(PlayerName::User), 8, 6),
            cards_in_position(Position::InDeck(PlayerName::User), 14, 20),
            cards_in_position(Position::InHand(PlayerName::Enemy), 105, 3),
            cards_in_position(Position::InVoid(PlayerName::Enemy), 108, 6),
            cards_in_position(Position::InDeck(PlayerName::Enemy), 114, 20),
            cards_in_position(Position::InVoid(PlayerName::User), 150, 4),
            cards_in_position(Position::OnBattlefield(PlayerName::User), 533, 8),
            cards_in_position(Position::OnBattlefield(PlayerName::Enemy), 633, 8),
            cards_in_position(Position::InHand(PlayerName::Enemy), 733, 5),
            vec![enemy_card(Position::InPlayerStatus(PlayerName::Enemy), 738)],
            vec![dreamsign_card(Position::InPlayerStatus(PlayerName::User), 739)],
            vec![dreamwell_card(Position::InDreamwell(PlayerName::User), 740)],
            vec![dreamwell_card(Position::InDreamwell(PlayerName::Enemy), 741)],
            vec![game_modifier_card(Position::GameModifier, 742)],
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
    let mut card = if sorting_key % 5 == 0 {
        card1(position, sorting_key)
    } else if sorting_key % 5 == 1 {
        card2(position, sorting_key)
    } else if sorting_key % 5 == 2 {
        card3(position, sorting_key)
    } else if sorting_key % 5 == 3 {
        card4(position, sorting_key)
    } else {
        card5(position, sorting_key)
    };

    // If the card is in the enemy's hand, it shouldn't be revealed to the player
    if position == Position::InHand(PlayerName::Enemy) {
        card.revealed = None;
        card.revealed_to_opponents = false;
    }

    card
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
                address: SpriteAddress::new("Assets/ThirdParty/GameAssets/CardImages/Standard/2521694543.png"),
            },
            name: "Titan of Forgotten Echoes".to_string(),
            rules_text: "When you materialize your second character in a turn, return this character from your void to play.".to_string(),
            outline_color: (position == Position::InHand(PlayerName::User)).then_some(display_color::GREEN),
            cost: Some(Energy(6)),
            produced: None,
            spark: Some(Spark(4)),
            card_type: "Ancient".to_string(),
            frame: CardFrame::Character,
            supplemental_card_info: flex_node("<b>Materialize</b>: A character entering play."),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(PlayerName::User),
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
                    "Assets/ThirdParty/GameAssets/CardImages/Standard/1633431262.png",
                ),
            },
            name: "Beacon of Tomorrow".to_string(),
            rules_text: "Discover a card with cost (2).".to_string(),
            outline_color: (position == Position::InHand(PlayerName::User)).then_some(display_color::GREEN),
            cost: Some(Energy(2)),
            produced: None,
            spark: None,
            card_type: "Event".to_string(),
            frame: CardFrame::Event,
            supplemental_card_info: flex_node(
                "<b>Discover</b>: Pick one of 4 cards with different types to put into your hand.",
            ),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(PlayerName::User),
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
                    "Assets/ThirdParty/GameAssets/CardImages/Standard/2269064817.png",
                ),
            },
            name: "Scrap Reclaimer".to_string(),
            rules_text: "Judgment: Return this character from your void to your hand. Born from rust and resilience.".to_string(),
            outline_color: (position == Position::InHand(PlayerName::User)).then_some(display_color::GREEN),
            cost: Some(Energy(4)),
            produced: None,
            spark: Some(Spark(0)),
            card_type: "Tinkerer".to_string(),
            frame: CardFrame::Character,
            supplemental_card_info: flex_node(
                "<b>Judgment</b>: Triggers at the start of your turn."),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(PlayerName::User),
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
                    "Assets/ThirdParty/GameAssets/CardImages/Standard/2269064809.png",
                ),
            },
            name: "Evacuation Enforcer".to_string(),
            rules_text: "> Draw 2 cards. Discard 3 cards.\nPromises under a stormy sky."
                .to_string(),
            outline_color: (position == Position::InHand(PlayerName::User))
                .then_some(display_color::GREEN),
            cost: Some(Energy(2)),
            produced: None,
            spark: Some(Spark(0)),
            card_type: "Trooper".to_string(),
            frame: CardFrame::Character,
            supplemental_card_info: None,
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(PlayerName::User),
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
                    "Assets/ThirdParty/GameAssets/CardImages/Standard/2027158310.png",
                ),
            },
            name: "Moonlit Voyage".to_string(),
            rules_text: "Draw 2 cards. Discard 2 cards.\nReclaim".to_string(),
            outline_color: (position == Position::InHand(PlayerName::User))
                .then_some(display_color::GREEN),
            cost: Some(Energy(2)),
            produced: None,
            spark: None,
            card_type: "Event".to_string(),
            frame: CardFrame::Event,
            supplemental_card_info: flex_node(
                "<b>Reclaim</b>: You may play this card from your void, then banish it.",
            ),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(PlayerName::User),
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

fn enemy_card(position: Position, sorting_key: u32) -> CardView {
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        card_back: Url::new("".to_string()),
        revealed: Some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Enemy/Korrak.png",
                ),
            },
            name: "<size=200%>Korrak</size>\nHellfire Sovereign".to_string(),
            rules_text: ">Judgment: A character you control gains +2 spark.".to_string(),
            outline_color: None,
            cost: None,
            produced: None,
            spark: None,
            card_type: "Enemy".to_string(),
            frame: CardFrame::Default,
            supplemental_card_info: flex_node(
                "<b>Judgment</b>: Triggers at the start of enemy's turn.",
            ),
            is_fast: false,
            actions: CardActions::default(),
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Enemy,
    }
}

fn dreamsign_card(position: Position, sorting_key: u32) -> CardView {
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        card_back: Url::new("".to_string()),
        revealed: Some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Dreamsign/DragonEgg.png",
                ),
            },
            name: "Dragon Egg".to_string(),
            rules_text: ">Judgment: If you control 3 characters with the same type, draw a card."
                .to_string(),
            outline_color: None,
            cost: None,
            produced: None,
            spark: None,
            card_type: "Dreamsign".to_string(),
            frame: CardFrame::Default,
            supplemental_card_info: flex_node(
                "<b>Judgment</b>: Triggers at the start of enemy's turn.",
            ),
            is_fast: false,
            actions: CardActions::default(),
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Dreamsign,
    }
}

fn dreamwell_card(position: Position, sorting_key: u32) -> CardView {
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        card_back: Url::new("".to_string()),
        revealed: Some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Dreamwell/1963305268.png",
                ),
            },
            name: "Rising Dawn".to_string(),
            rules_text: "Draw a card.".to_string(),
            outline_color: None,
            cost: None,
            produced: Some(Energy(2)),
            spark: None,
            card_type: "Dreamwell".to_string(),
            frame: CardFrame::Default,
            supplemental_card_info: None,
            is_fast: false,
            actions: CardActions::default(),
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Dreamwell,
    }
}

fn game_modifier_card(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        card_back: Url::new("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Standard/2027158310.png",
                ),
            },
            name: "Celestial Reverie".to_string(),
            rules_text: "Until end of turn, whenever you play a character, draw a card. "
                .to_string(),
            outline_color: None,
            cost: None,
            produced: None,
            spark: None,
            card_type: "Game Modifier".to_string(),
            frame: CardFrame::Event,
            supplemental_card_info: None,
            is_fast: false,
            actions: CardActions::default(),
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
            top_left: Dimension { unit: DimensionUnit::Pixels, value: 2.0 },
            top_right: Dimension { unit: DimensionUnit::Pixels, value: 2.0 },
            bottom_right: Dimension { unit: DimensionUnit::Pixels, value: 2.0 },
            bottom_left: Dimension { unit: DimensionUnit::Pixels, value: 2.0 },
        }),
        padding: Some(DimensionGroup {
            top: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        color: Some(DisplayColor { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 }),
        font_size: Some(Dimension { unit: DimensionUnit::Pixels, value: 5.0 }),
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
                bottom: Some(Dimension { unit: DimensionUnit::Pixels, value: 50.0 }),
                left: Some(Dimension { unit: DimensionUnit::Pixels, value: 32.0 }),
            }),
            ..Default::default()
        }),
        children: vec![message],
        ..Default::default()
    }
}

fn mulligan_message() -> FlexNode {
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
        node_type: Some(NodeType::Text(Text {
            label: "Resolve Mulligan: Select cards in hand to replace.".into(),
        })),
        style: Some(style),
        ..Default::default()
    };

    FlexNode {
        style: Some(FlexStyle {
            position: Some(FlexPosition::Absolute),
            inset: Some(FlexInsets {
                top: None,
                right: Some(Dimension { unit: DimensionUnit::Pixels, value: 32.0 }),
                bottom: Some(Dimension { unit: DimensionUnit::Pixels, value: 135.0 }),
                left: Some(Dimension { unit: DimensionUnit::Pixels, value: 32.0 }),
            }),
            ..Default::default()
        }),
        children: vec![message],
        ..Default::default()
    }
}
