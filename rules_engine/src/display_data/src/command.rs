use bon::Builder;
use core_data::display_color::DisplayColor;
use core_data::display_types::{
    AudioClipAddress, EffectAddress, MaterialAddress, Milliseconds, ProjectileAddress,
    StudioAnimation,
};
use core_data::numerics::{Energy, Points};
use masonry::flex_node::FlexNode;
use masonry::flex_style::FlexVector3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use crate::battle_view::{BattleView, DisplayPlayer};
use crate::card_view::{CardView, ClientCardId};
use crate::object_position::{ObjectPosition, Position};
use crate::quest_view::QuestView;

/// A list of [ParallelCommandGroup]s to execute sequentially.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct CommandSequence {
    pub groups: Vec<ParallelCommandGroup>,
}

impl CommandSequence {
    pub fn from_command(command: Command) -> Self {
        Self { groups: vec![ParallelCommandGroup { commands: vec![command] }] }
    }

    pub fn sequential(sequence: Vec<Command>) -> Self {
        Self {
            groups: sequence
                .into_iter()
                .map(|c| ParallelCommandGroup { commands: vec![c] })
                .collect(),
        }
    }

    pub fn from_vecs(vecs: Vec<Vec<Command>>) -> Self {
        Self { groups: vecs.into_iter().map(|c| ParallelCommandGroup { commands: c }).collect() }
    }

    pub fn parallel(commands: Vec<Command>) -> Self {
        Self { groups: vec![ParallelCommandGroup { commands }] }
    }

    pub fn optional_sequential(sequence: Vec<Option<Command>>) -> Self {
        Self {
            groups: sequence
                .into_iter()
                .filter_map(|c| c.map(|c| ParallelCommandGroup { commands: vec![c] }))
                .collect(),
        }
    }
}

/// A set of [Command]s to execute simultaneously.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ParallelCommandGroup {
    pub commands: Vec<Command>,
}

/// Represents an animated update to the visual state of the game.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, EnumDiscriminants)]
#[strum_discriminants()]
pub enum Command {
    UpdateBattle(Box<UpdateBattleCommand>),
    UpdateQuest(Box<UpdateQuestCommand>),
    Wait(Milliseconds),
    FireProjectile(FireProjectileCommand),
    DissolveCard(DissolveCardCommand),
    DisplayGameMessage(GameMessageType),
    DisplayEffect(DisplayEffectCommand),
    PlayAudioClip(PlayAudioClipCommand),
    DrawUserCards(DrawUserCardsCommand),
    DisplayJudgment(DisplayJudgmentCommand),
    DisplayDreamwellActivation(DisplayDreamwellActivationCommand),
    DisplayEnemyMessage(DisplayEnemyMessageCommand),
    PlayStudioAnimation(PlayStudioAnimationCommand),
    SetCardTrail(SetCardTrailCommand),
    ShuffleVoidIntoDeck(ShuffleVoidIntoDeckCommand),
    UpdateScreenOverlay(Box<UpdateScreenOverlayCommand>),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateBattleCommand {
    /// The battle to update.
    pub battle: BattleView,

    /// Sound to play when the battle is updated.
    pub update_sound: Option<AudioClipAddress>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateQuestCommand {
    /// The quest to update.
    pub quest: QuestView,

    /// Sound to play when the quest is updated.
    pub update_sound: Option<AudioClipAddress>,
}

impl UpdateBattleCommand {
    pub fn new(battle: BattleView) -> Self {
        Self { battle, update_sound: None }
    }

    pub fn with_update_sound(mut self, update_sound: AudioClipAddress) -> Self {
        self.update_sound = Some(update_sound);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Builder)]
pub struct FireProjectileCommand {
    // The source to fire the projectile from.
    pub source_id: GameObjectId,

    // The target to fire the projectile to.
    pub target_id: GameObjectId,

    // Projectile to fire from the 'source_id' card to 'target_id'
    pub projectile: ProjectileAddress,

    // How long the projectile should take to hit its target. Defaults to 300ms.
    pub travel_duration: Option<Milliseconds>,

    // Sound to play when the projectile is fired.
    pub fire_sound: Option<AudioClipAddress>,

    // Sound to play when the projectile hits its target.
    pub impact_sound: Option<AudioClipAddress>,

    // Additional effect to display on the target on hit.
    pub additional_hit: Option<EffectAddress>,

    // Delay before showing the additional hit. If provided, the original
    // projectile Hit effect will be hidden before showing the new hit effect.
    pub additional_hit_delay: Option<Milliseconds>,

    // During to wait for the project's impact effect before continuing
    pub wait_duration: Option<Milliseconds>,

    // If true, the target will be hidden after being hit during the
    // 'wait_duration' and before jumping to 'jump_to_position'.
    #[builder(default)]
    pub hide_on_hit: bool,

    // Position for the target to jump to after being hit.
    pub jump_to_position: Option<ObjectPosition>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Builder)]
pub struct DissolveCardCommand {
    /// The card to dissolve.
    ///
    /// Once a card is dissolved, it will be invisible until a reverse dissolve
    /// is applied to it.
    pub target: ClientCardId,

    /// The material to use for the dissolve effect.
    pub material: MaterialAddress,

    /// If true, dissolve will be played backwards to "create" the card.
    pub reverse: bool,

    /// The color to use for the dissolve effect.
    pub color: DisplayColor,

    /// The speed multiplier of the dissolve effect. Defaults to 1.
    pub dissolve_speed: Option<f64>,

    /// Sound to play
    pub sound: Option<AudioClipAddress>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DisplayEffectCommand {
    /// The target to display the effect on.
    pub target: GameObjectId,

    /// The effect to display.
    pub effect: EffectAddress,

    /// How long to wait before continuing with animations.
    pub duration: Milliseconds,

    /// Local scale to apply to this effect
    pub scale: FlexVector3,

    /// Sound to play along with effect
    pub sound: Option<AudioClipAddress>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PlayAudioClipCommand {
    /// Sound to play
    pub sound: AudioClipAddress,

    /// How long to pause before continuing with animations.
    pub pause_duration: Milliseconds,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DrawUserCardsCommand {
    /// Cards to draw. Must already be present in user deck.
    pub cards: Vec<CardView>,

    /// Time to wait between drawing subsequent cards.
    pub stagger_interval: Milliseconds,

    /// Time to display each card before moving it to hand.
    ///
    /// Should be less than stagger_interval for best results.
    pub pause_duration: Milliseconds,

    /// Destination position to move the cards to after drawing them.
    pub destination: Position,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DisplayJudgmentCommand {
    /// The player to display the judgment animation for.
    pub player: DisplayPlayer,

    /// The new score for the player, if it has changed.
    pub new_score: Option<Points>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DisplayDreamwellActivationCommand {
    /// The player to display the dreamwell activation for.
    pub player: DisplayPlayer,

    /// The card to display an activation for. This card will be moved from its
    /// current position (assumed to be the 'Dreamwell' position) to the
    /// DreamwellActivation position, and an update to the player's produced
    /// energy value will be displayed.
    ///
    /// If there are triggered events as a result of this activation, the card
    /// should be kept in the DreamwellActivation position for the next
    /// update. Otherwise it's typical to return the card to the Dreamwell
    /// position.
    pub card_id: ClientCardId,

    /// New energy available to this player, if it has changed.
    pub new_energy: Option<Energy>,

    /// New energy produced by this player at the start of the turn, if it has
    /// changed.
    pub new_produced_energy: Option<Energy>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DisplayArrow {
    pub source: GameObjectId,
    pub target: GameObjectId,
    pub color: ArrowStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum ArrowStyle {
    Red,
    Blue,
    Green,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DisplayEnemyMessageCommand {
    pub message: String,
    pub show_duration: Milliseconds,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PlayStudioAnimationCommand {
    pub studio_type: StudioType,
    pub enter_animation: Option<StudioAnimation>,
    pub animation: StudioAnimation,
    pub exit_animation: Option<StudioAnimation>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum StudioType {
    UserStatus,
    EnemyStatus,
    UserIdentityCard,
    EnemyIdentityCard,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SetCardTrailCommand {
    pub card_ids: Vec<ClientCardId>,
    pub trail: ProjectileAddress,
    pub duration: Milliseconds,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ShuffleVoidIntoDeckCommand {
    pub player: DisplayPlayer,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateScreenOverlayCommand {
    /// New screen overlay to set. If None clears the current overlay.
    pub screen_overlay: Option<FlexNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub enum GameObjectId {
    CardId(ClientCardId),
    Deck(DisplayPlayer),
    Void(DisplayPlayer),
    Avatar(DisplayPlayer),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub enum GameMessageType {
    YourTurn,
    EnemyTurn,
    Victory,
    Defeat,
}
