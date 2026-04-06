// ============================================================
// Core primitives — Rust newtype wrappers serialize transparently
// ============================================================

/** Rust Energy(u32) — serializes as plain number */
export type Energy = number;
/** Rust Spark(u32) — serializes as plain number */
export type Spark = number;
/** Rust Points(u32) — serializes as plain number */
export type Points = number;
/** Rust TurnId(u32) — serializes as plain number */
export type TurnId = number;
/** Rust Milliseconds { milliseconds_value: u32 } */
export interface Milliseconds {
  milliseconds_value: number;
}
/** Rust UserId(Uuid) — serializes as UUID string */
export type UserId = string;
/** Rust BattleId(Uuid) — serializes as UUID string */
export type BattleId = string;
/** Rust ClientCardId = String */
export type ClientCardId = string;

// ============================================================
// Display types
// ============================================================

export interface DisplayColor {
  red: number;
  green: number;
  blue: number;
  alpha: number;
}

export interface SpriteAddress {
  sprite: string;
}

export interface PrefabAddress {
  prefab: string;
}

export interface AudioClipAddress {
  audio_clip: string;
}

export interface EffectAddress {
  effect: string;
}

// DisplayImage: externally tagged enum
// { "Sprite": { "sprite": "..." } } or { "Prefab": { ... } }
export type DisplayImage =
  | { Sprite: SpriteAddress }
  | { Prefab: { prefab: PrefabAddress; studio_type: string } };

// ============================================================
// Positions
// ============================================================

export type DisplayPlayer = "User" | "Enemy";

export type StackType =
  | "Default"
  | "TargetingUserBattlefield"
  | "TargetingEnemyBattlefield"
  | "TargetingBothBattlefields";

// Position: externally tagged enum — unit variants are strings,
// data variants are { "VariantName": data }
export type Position =
  | "Default"
  | "Offscreen"
  | { OnStack: StackType }
  | "Drawn"
  | { InHand: DisplayPlayer }
  | { InDeck: DisplayPlayer }
  | { InVoid: DisplayPlayer }
  | { InBanished: DisplayPlayer }
  | { OnBattlefield: [DisplayPlayer, "Front" | "Back", number] }
  | { InPlayerStatus: DisplayPlayer }
  | "Browser"
  | { CardOrderSelector: string }
  | "HandStorage"
  | { InDreamwell: DisplayPlayer }
  | "DreamwellActivation"
  | "GameModifier"
  | "OnScreenStorage"
  | { AboveVoid: DisplayPlayer }
  | string; // catch-all for quest-specific positions

export interface ObjectPosition {
  position: Position;
  sorting_key: number;
}

// ============================================================
// Card types
// ============================================================

export type CardFacing = "FaceUp" | "FaceDown";

export type CardPrefab =
  | "Character"
  | "Event"
  | "Identity"
  | "Token"
  | "Dreamwell"
  | "Enemy"
  | "Dreamsign"
  | "IconCard"
  | "Journey"
  | "OfferCost";

// GameAction: externally tagged enum
// Unit variants: "NoOp", "PassPriority", etc.
// Data variants: { "BattleAction": { "PlayCardFromHand": ... } }
// We use a loose type since actions are opaque — we receive them from
// the server and send them back without inspecting internals.
export type GameAction = unknown;

export interface CardActions {
  can_play?: GameAction;
  can_select_order?: unknown;
  on_play_sound?: AudioClipAddress;
  on_click?: GameAction;
  play_effect_preview?: unknown;
  button_attachment?: ButtonView;
}

export interface CardEffects {
  looping_effect?: EffectAddress;
  reverse_dissolve_on_appear?: unknown;
}

export interface RevealedCardView {
  image: DisplayImage;
  name: string;
  cost?: string;
  produced?: string;
  spark?: string;
  card_type: string;
  rules_text: string;
  outline_color?: DisplayColor;
  is_fast: boolean;
  summoning_sick: boolean;
  actions: CardActions;
  effects: CardEffects;
  info_zoom_data?: unknown;
}

export interface CardView {
  id: ClientCardId;
  position: ObjectPosition;
  revealed?: RevealedCardView;
  revealed_to_opponents: boolean;
  card_facing: CardFacing;
  backless: boolean;
  create_position?: ObjectPosition;
  create_sound?: AudioClipAddress;
  destroy_position?: ObjectPosition;
  prefab: CardPrefab;
}

// ============================================================
// Player and battle view
// ============================================================

export type DisplayedTurnIndicator = "Left" | "Right";

export interface PlayerView {
  score: Points;
  can_act: boolean;
  energy: Energy;
  produced_energy: Energy;
  total_spark: Spark;
  turn_indicator?: DisplayedTurnIndicator;
  is_victory_imminent: boolean;
}

export interface ButtonView {
  label: string;
  action?: GameAction;
}

export interface CardBrowserView {
  close_button?: GameAction;
}

export interface CardOrderSelectorView {
  include_deck: boolean;
  include_void: boolean;
}

export interface InterfaceView {
  has_open_panels: boolean;
  screen_overlay?: FlexNode;
  primary_action_button?: ButtonView;
  primary_action_show_on_idle_duration?: Milliseconds;
  secondary_action_button?: ButtonView;
  increment_button?: ButtonView;
  decrement_button?: ButtonView;
  dev_button?: ButtonView;
  undo_button?: ButtonView;
  browser?: CardBrowserView;
  card_order_selector?: CardOrderSelectorView;
}

export interface DisplayArrow {
  source: unknown;
  target: unknown;
  color: string;
}

export type BattlePreviewState = "None" | "Pending" | { Active: unknown };

export interface BattleView {
  id: BattleId;
  user: PlayerView;
  enemy: PlayerView;
  cards: CardView[];
  interface: InterfaceView;
  arrows: DisplayArrow[];
  preview: BattlePreviewState;
  turn_number: TurnId;
  game_over: boolean;
}

// ============================================================
// FlexNode (for overlay parsing)
// ============================================================

export interface TextNode {
  label: string;
}

export type NodeType =
  | { Text: TextNode }
  | { TypewriterTextNode: { label: string } }
  | { ScrollViewNode: unknown }
  | { DraggableNode: unknown }
  | { TextFieldNode: unknown }
  | { SliderNode: unknown };

export interface EventHandlers {
  on_click?: GameAction;
  on_long_press?: GameAction;
  on_mouse_enter?: GameAction;
  on_mouse_leave?: GameAction;
  on_mouse_down?: GameAction;
  on_mouse_up?: GameAction;
  on_field_changed?: GameAction;
}

export interface FlexNode {
  name?: string;
  node_type?: NodeType;
  children: FlexNode[];
  event_handlers?: EventHandlers;
  style?: unknown;
  hover_style?: unknown;
  pressed_style?: unknown;
  on_attach_style?: unknown;
  on_attach_style_duration?: Milliseconds;
}

// ============================================================
// Commands
// ============================================================

export interface UpdateBattleCommand {
  battle: BattleView;
  update_sound?: AudioClipAddress;
}

// Command: externally tagged enum. We only care about UpdateBattle.
export type Command =
  | { UpdateBattle: UpdateBattleCommand }
  | Record<string, unknown>;

export interface ParallelCommandGroup {
  commands: Command[];
}

export interface CommandSequence {
  groups: ParallelCommandGroup[];
}

// ============================================================
// Request / Response types
// ============================================================

export type PollResponseType = "None" | "Incremental" | "Final";

export interface Metadata {
  user_id: UserId;
  battle_id?: BattleId;
  request_id?: string;
  integration_test_id?: string;
}

export interface ConnectRequest {
  metadata: Metadata;
  persistent_data_path: string;
  streaming_assets_path: string;
  vs_opponent?: UserId;
  display_properties?: unknown;
  debug_configuration?: DebugConfiguration;
}

export interface DebugConfiguration {
  enemy?: unknown;
  seed?: number;
  deck_override?: TestDeckName;
  dreamwell_override?: unknown;
}

export type TestDeckName = "Vanilla" | "StartingFive" | "Benchmark1" | "Core11";

export interface ConnectResponse {
  metadata: Metadata;
  commands: CommandSequence;
  response_version: string;
}

export interface PerformActionRequest {
  metadata: Metadata;
  action: GameAction;
  save_file_id?: UserId;
  last_response_version?: string;
}

export interface PerformActionResponse {
  metadata: Metadata;
  commands: CommandSequence;
}

export interface PollRequest {
  metadata: Metadata;
}

export interface PollResponse {
  metadata: Metadata;
  commands?: CommandSequence;
  response_type: PollResponseType;
  response_version?: string;
}
