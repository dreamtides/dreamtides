# Client Masonry & UI Panels

How Rust-defined UI is built, serialized, and rendered in Unity. The masonry
system is a server-side UI framework: the Rust rules engine builds a FlexNode
tree (a serializable CSS-flexbox-like DOM), serializes it to JSON, and sends it
to the Unity client for rendering via UIToolkit VisualElements. A virtual-DOM
reconciler on the client diffs each new tree against the previous one and patches
the live UI.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [FlexNode Structure](#flexnode-structure)
- [FlexStyle Properties](#flexstyle-properties)
- [The Component Trait](#the-component-trait)
- [Available Components](#available-components)
- [Interface Rendering](#interface-rendering)
- [The Panel System](#the-panel-system)
- [Client-Side Reconciler](#client-side-reconciler)
- [Client-Side Style Application](#client-side-style-application)
- [DocumentService and Overlay Containers](#documentservice-and-overlay-containers)
- [Event Handling Flow](#event-handling-flow)
- [DisplayProperties and Mobile Adaptation](#displayproperties-and-mobile-adaptation)
- [Key Files Reference](#key-files-reference)

## Architecture Overview

Three Rust crates participate, layered bottom-up:

- **masonry** -- Pure data crate defining FlexNode, FlexStyle, EventHandlers,
  and dimension types. No logic, just serializable structs. Everything derives
  serde and schemars traits for JSON transport and C# schema generation.
- **ui_components** -- Reusable Component trait and concrete component types
  (boxes, text, buttons, panels, scroll views). Components compose into FlexNode
  trees via a render/resolve pattern.
- **display** -- Game-specific rendering logic. The interface_rendering and panel
  modules assemble components into the actual UI sent each frame.

On the Unity side, three C# files form the rendering layer:

- **Reconciler.cs** -- Virtual-DOM diffing by index position and node type tag.
- **MasonRenderer.cs** -- Maps FlexStyle properties to Unity IStyle, creates
  VisualElement instances, and wires event handlers.
- **DocumentService.cs** -- Manages four overlay containers and bridges game
  commands to the reconciler.

Data flows in one direction: BattleState in Rust produces Component trees, which
resolve to FlexNode trees, which serialize to JSON, which Unity deserializes and
reconciles into live VisualElements. User interactions flow back as GameAction
values embedded in the FlexNode event handlers.

## FlexNode Structure

FlexNode (masonry/src/flex_node.rs) is the fundamental layout unit. Each node
has:

- **name** -- Optional string for debugging and Unity USS selectors. Not used
  for reconciliation matching.
- **node_type** -- Determines specialized behavior. Variants: Text (label
  string), TypewriterTextNode (animated character-by-character reveal with sound
  effects), ScrollViewNode (scrollable container with elasticity, page size,
  scrollbar visibility, and touch behavior settings), DraggableNode (drag-and-drop
  with target identifiers, drop actions, and custom indicators), TextFieldNode
  (text input with multiline, password, max length, and selection settings), and
  SliderNode (slider with range, direction, preference persistence, and
  sub-element styles for label, tracker, dragger, and drag container).
- **children** -- A Vec of child FlexNodes forming the recursive tree.
- **event_handlers** -- Optional EventHandlers struct. Each field is an
  Option of GameAction: on_click, on_long_press, on_mouse_enter,
  on_mouse_leave, on_mouse_down, on_mouse_up, and on_field_changed. The Rust
  renderer pre-bakes the exact action each interaction should trigger.
- **style** -- Base FlexStyle for the node.
- **hover_style** -- Merged over the base style on mouse enter, reverted on
  mouse leave.
- **pressed_style** -- Merged over the base style on mouse down, reverted on
  mouse up.
- **on_attach_style** -- Applied when the element first attaches to a panel,
  optionally reverting after on_attach_style_duration milliseconds. Used for
  entrance animations like fade-ins.

All Option fields use skip_serializing_if for compact JSON output.

## FlexStyle Properties

FlexStyle (masonry/src/flex_style.rs) is a builder-pattern struct with
approximately 50 optional properties, mapping directly to Unity UIToolkit style
properties. It uses the bon Builder derive for ergonomic construction. Property
groups:

**Layout and flexbox:** align_content, align_items, align_self (FlexAlign with
Auto, FlexStart, Center, FlexEnd, Stretch), display (Flex or None),
flex_basis, flex_direction (Column, ColumnReverse, Row, RowReverse), flex_grow,
flex_shrink, flex_wrap (NoWrap, Wrap, WrapReverse), justify_content (FlexStart,
Center, FlexEnd, SpaceBetween, SpaceAround), and position (Relative, Absolute).

**Sizing:** width, height, min_width, min_height, max_width, max_height -- all
Dimension values.

**Spacing:** margin and padding as DimensionGroup (supports uniform, two-axis,
or four-side values), plus inset as FlexInsets (top, right, bottom, left).

**Borders:** border_color (per-side DisplayColor), border_radius (per-corner
Dimension), and border_width (per-side float). All support uniform shorthand
via From impls.

**Visual:** background_color, background_image (SpriteAddress),
background_image_tint_color, color, opacity, visibility, overflow, and
picking_mode (Position passes events, Ignore lets them through).

**Typography:** font (FontAddress), font_size, font_style (Normal, Bold,
Italic, BoldAndItalic), letter_spacing, word_spacing, paragraph_spacing,
text_align (nine variants from UpperLeft to LowerRight), text_overflow (Clip or
Ellipsis), text_overflow_position (End, Start, Middle), text_outline_color,
text_outline_width, text_shadow (offset, blur_radius, color), and white_space
(Normal, NoWrap).

**Transforms:** rotate (degrees), scale (uniform or per-axis), translate (x/y
Dimension plus z float), and transform_origin.

**Transitions:** transition_properties (list of property names),
transition_durations, transition_delays (both in Milliseconds), and
transition_easing_modes (22 variants including Ease, Linear, EaseInBounce,
EaseOutElastic, and others).

### Dimension Units

The Dimension type pairs a float value with a DimensionUnit. Six units are
available: Pixels (default for integer and float conversions), Percentage (via
the Percent helper), ViewportWidth, ViewportHeight, and four safe area inset
units (SafeAreaTopInset, SafeAreaRightInset, SafeAreaBottomInset,
SafeAreaLeftInset). Viewport and safe area units are converted to actual pixel
values on the client side using DocumentService screen calculations.

## The Component Trait

The Component trait (ui_components/src/component.rs) provides a React-like
compositional model with three methods:

- **render(self)** -- Consumes the component and returns another component
  (composition), or None for leaf nodes. Composite components like
  ButtonComponent return a configured BoxComponent from render. Leaf components
  like BoxComponent return a sentinel NodeComponent.
- **flex_node(&self)** -- Resolves the component to a FlexNode. The default
  implementation calls render recursively until reaching a leaf, then returns
  that leaf's FlexNode. Leaf components override this to return their FlexNode
  directly.
- **wrap(self)** -- Type erasure. Eagerly resolves to a FlexNode and wraps it
  in a WrapperComponent. This allows heterogeneous component collections since
  all WrapperComponent values share one concrete type.

Component requires Clone. There is a blanket impl for Option that delegates to
the inner component or returns None, enabling conditional rendering.

WrapperComponent (ui_components/src/wrapper.rs) stores a pre-rendered FlexNode.
Its flex_node method simply returns the stored node. Use wrap when building
lists of mixed component types.

## Available Components

### BoxComponent

The foundational layout container (ui_components/src/box_component.rs). Wraps a
raw FlexNode. Uses a custom typestate builder: calling builder() produces an
Unnamed state, then name() transitions to Named, which unlocks child, children,
on_click, event_handlers, style, hover_style, pressed_style, on_attach_style,
and on_attach_style_duration. The name is required -- every box must be named for
debugging. Default flex_direction is Row.

### TextComponent

Displays text with typography styling (ui_components/src/text_component.rs).
Uses bon Builder. Fields: text (the label string), typography (a Typography enum
preset), and optional flex_grow, flex_shrink, text_align, and white_space. The
flex_node override produces a FlexNode with NodeType::Text. Defaults text_align
to MiddleCenter.

### ButtonComponent

A styled interactive button (ui_components/src/button_component.rs). Uses bon
Builder. Fields: label, action (a GameAction to fire on click), optional
flex_grow, and is_primary (defaults false). The render method returns a
BoxComponent with background image (primary or secondary variant), fixed height,
center alignment, hover style (gray-300 tint), pressed style (gray-500 tint
plus 0.97 scale), and a TextComponent child with ButtonLabel typography.

### PanelComponent

A modal overlay window (ui_components/src/panel_component.rs). Generic over
content (any Component type). Uses bon Builder. Fields: title, content, and
show_close_button. Renders as an absolutely-positioned BoxComponent with safe
area insets, window background image with 500px image slice, and two children: a
CloseButtonComponent in the top-right corner and the provided content component.

### ScrollViewComponent

A scrollable container (ui_components/src/scroll_view_component.rs). Uses a
custom builder with a single child method. Produces a FlexNode with
NodeType::ScrollViewNode. Horizontal scrollbar is always hidden. Vertical
scrollbar is hidden on mobile devices (via DisplayProperties check) and uses
default visibility on desktop.

### CloseButtonComponent

A unit struct with no fields (ui_components/src/close_button_component.rs).
Always renders as an 18x18 BoxComponent with close-button background, hover and
pressed states, and an on_click handler that fires
BattleDisplayAction::CloseCurrentPanel. The icon is the XMARK constant from the
icon font.

### Typography Presets

The Typography enum (ui_components/src/typography.rs) provides named font
presets: StackTrace (size 6), ButtonLabel (size 8, mobile scaling disabled),
InterfaceMessage (size 10), SupplementalCardInfo (size 10), and Body2 (size 10).
All default to white. On mobile devices, font sizes scale down by 0.65x unless
the preset disables mobile scaling.

### Icon Constants

The icon module (ui_components/src/icon.rs) defines Unicode code points mapped
to an icon font on the client: ENERGY, WARNING, CHEVRON_UP, XMARK, EYE_SLASH,
EYE, NON_NUMERIC, FAST, ACTIVATED, and MULTI_ACTIVATED. Used as text content in
TextComponent for icon rendering.

## Interface Rendering

The interface_rendering module (display/src/rendering/interface_rendering.rs)
builds the InterfaceView, the primary UI overlay sent with each UpdateBattle
command. The InterfaceView contains:

- **screen_overlay** -- The FlexNode tree for the full-screen masonry overlay.
- **has_open_panels** -- Whether a panel is currently displayed.
- **Action buttons** -- primary_action_button, secondary_action_button,
  increment_button, decrement_button, dev_button, and undo_button. Each is a
  ButtonView (label plus optional GameAction), rendered by Unity as fixed-position
  native buttons separate from the masonry overlay.
- **browser** -- Optional CardBrowserView for deck/void browsing.
- **card_order_selector** -- Optional view for deck card reordering prompts.

The interface_view function has two modes:

**Animation mode:** During animations, the overlay includes only the current
panel (if any) and disabled dev/undo buttons. No interactive buttons appear.

**Interactive mode:** Builds the full overlay with a prompt message at the top
(telling the player what to do), a show-battlefield toggle button in the
bottom-right (visible when the stack or card browser is active), the current
panel overlay, and all action buttons.

The prompt message reads from the front of the battle prompts queue and either
uses the card's prompt message or falls back to a generic message based on the
PromptType variant (such as "Choose a character" for ChooseCharacter or "Choose
energy amount" for ChooseEnergyValue).

The primary action button follows a cascading priority: prompt choice zero,
energy cost submission, void/hand/deck card target submission, pass priority
(resolve stack), end turn, then start next turn. The secondary action button
shows prompt choice one when a Choose prompt has multiple options. Increment and
decrement buttons appear only during ChooseEnergyValue prompts.

An InterfaceMessage component (display/src/rendering/interface_message.rs)
renders text anchored at the top or bottom of the screen. It supports a temporary
mode with CSS opacity transitions: fading in over 300 milliseconds, then fading
out after a configurable on_attach_style_duration.

## The Panel System

Panels are full-screen modal overlays built using PanelComponent. The
panel_rendering module (display/src/panels/panel_rendering.rs) routes
PanelAddress enum values to specific panel implementations:

- **DeveloperPanel** -- Debug menu with buttons for setting AI opponent, adding
  cards to hand, playing opponent cards, drawing cards, granting energy,
  restarting the battle, running benchmark decks, viewing logs, and resizing the
  deck. Each button dispatches a GameAction (typically DebugAction or
  BattleDisplayAction::OpenPanel).
- **SetOpponentAgentPanel** -- Scrollable list of AI agent configurations
  (MonteCarlo with various iteration counts, RandomAction,
  FirstAvailableAction, WaitFiveSeconds, Human). Each dispatches
  DebugAction::SetOpponentAgent.
- **AddCardToHandPanel** -- Shows current hand count and a scrollable list of
  all test cards with Add buttons dispatching DebugBattleAction::AddCardToHand.
- **PlayOpponentCardPanel** -- Scrollable list of test cards with Play buttons
  dispatching DebugBattleAction::OpponentPlayCard.
- **ViewLogsPanel** -- Reads the last 1000 lines of the log file with
  emoji-based filter buttons. Uses ScrollViewComponent for content. Filter
  buttons dispatch BattleDisplayAction::OpenPanel with a ViewLogs filter.

All panels use PanelComponent as their root, which provides the window
background, absolute positioning with safe area insets, and a close button.

## Client-Side Reconciler

The Reconciler (client/Assets/Dreamtides/Masonry/Reconciler.cs) implements a
virtual-DOM diffing algorithm. Its single public method, Update, takes a new
FlexNode and an optional previous IMasonElement and returns either null (element
reused in place) or a new element (replacement needed).

The algorithm matches by **index position and node type tag**, not by name or
key. The NodeTypeTag enum has six values: VisualElement, Draggable, Text,
TextField, Slider, and ScrollView. The tag is computed from which NodeType
variant is populated on the FlexNode.

When types match, the existing element is reused: children are recursively
updated in place by index, then MasonRenderer.ApplyToElement updates styles and
properties. When types differ, a new element is created from scratch.

Child reconciliation walks the new children by index. If the index exists in the
old element, the child is recursively updated (and replaced if the type changed).
New indices beyond the old child count create new elements. Old indices beyond
the new child count are removed. Elements with internal children (TextField,
Slider) skip child updates to avoid clobbering Unity's internal sub-elements.

## Client-Side Style Application

MasonRenderer (client/Assets/Dreamtides/Masonry/MasonRenderer.cs) handles two
responsibilities: creating the correct VisualElement subclass from a FlexNode,
and mapping all FlexStyle properties to Unity's IStyle API.

Element creation uses the NodeType to select the class: no type or null produces
a plain NodeVisualElement, Text produces a NodeLabel (Unity Label),
TypewriterTextNode produces a NodeTypewriterText (animated label that reveals
characters one at a time), ScrollViewNode produces a NodeScrollView,
DraggableNode produces a Draggable, TextFieldNode produces a NodeTextField, and
SliderNode produces a NodeSlider.

Style application maps every FlexStyle property to its Unity counterpart.
Dimension units are resolved: Pixels pass through directly, Percentage maps to
Unity percent lengths, ViewportWidth and ViewportHeight convert via
DocumentService.ScreenPxToElementPx, and safe area inset units account for
device notches and system bars.

Interactive style layering works through event callbacks. On mouse enter, the
renderer merges the base style with on_attach_style and hover_style and applies
the result. On mouse leave, it reverts to base plus on_attach_style. Mouse down
adds pressed_style on top; mouse up reverts. The merging uses a last-non-null-
wins strategy per property via Mason.MergeStyles.

Non-interactive elements (those without event handlers) receive
pickingMode.Ignore so mouse events pass through to elements behind them.

All IMasonElement types implement INodeCallbacks, which provides a lazily-
initialized Callbacks instance. Callbacks are allocated only when an event
handler is actually set, avoiding overhead on passive elements. The Callbacks
class supports Click, MouseDown, MouseUp, MouseEnter, MouseLeave, LongPress
(fires after 0.5 seconds of held mouse down, suppressing the subsequent click),
Change, FieldChanged, and AttachToPanel events.

## DocumentService and Overlay Containers

DocumentService (client/Assets/Dreamtides/Services/DocumentService.cs) manages
the UIDocument and provides four overlay containers:

- **InfoZoomContainer** -- For zoomed-in card detail views. Used when the player
  inspects a card to show supplemental help text (keyword explanations) built by
  the SupplementalCardInfo component in Rust.
- **ScreenOverlay** -- The primary overlay for the battle interface. Receives
  the FlexNode from InterfaceView.screen_overlay. This is where panels, prompt
  messages, and toggle buttons live.
- **ScreenAnchoredNode** -- For UI anchored to world-space positions, such as
  floating text above a character. Supports auto-hide with fade-out after a
  configurable duration.
- **EffectPreviewOverlay** -- For effect preview displays showing potential
  outcomes.

Each container is an absolutely-positioned wrapper spanning the full screen with
pickingMode Ignore, containing a NodeVisualElement as the reconciliation target.
The private Reconcile method calls Reconciler.Update and swaps in any new element
if the node type changed.

DocumentService also handles coordinate conversion between screen pixels, Unity
UIToolkit logical pixels, and world-space positions. It computes safe area insets
in element-space pixels for the MasonRenderer dimension unit conversion.

## Event Handling Flow

The event handling architecture is server-authoritative. The Rust engine decides
what actions are legal, pre-computes them, and embeds them as GameAction values
in FlexNode EventHandlers. The Unity client does not determine what a button
does -- it simply fires the pre-baked action.

The flow from click to state change:

- On the Rust side, rendering code sets EventHandlers fields (typically via
  BoxComponent.on_click or ButtonComponent.action) to a GameAction value. The
  GameAction enum has five variants: NoOp, DebugAction, BattleAction,
  BattleDisplayAction, and Undo.
- The FlexNode tree serializes to JSON and reaches Unity.
- MasonRenderer.ApplyNode reads the EventHandlers and for each non-null handler,
  calls SetCallback on the element's Callbacks instance with a closure that
  invokes ActionService.PerformAction with the action.
- When the user interacts, UIToolkit fires the event, Callbacks looks up the
  current action closure and invokes it.
- ActionService.PerformAction constructs a PerformActionRequest with the
  GameAction, user metadata, and a response version UUID (for duplicate
  prevention). In plugin mode it calls the native FFI function and polls for
  results; in dev server mode it sends an HTTP POST.
- The Rust engine dispatches on the GameAction variant. BattleAction mutates
  BattleState and re-renders. BattleDisplayAction modifies only display state
  (panel address, card browser source, energy selector, stack visibility) and
  re-renders without changing game state. Undo pops the undo stack.
- The engine returns a CommandSequence. The primary command is UpdateBattle,
  carrying a complete BattleView with the new InterfaceView, updated card views,
  and new EventHandlers embedded for the next interaction cycle.

BattleDisplayAction has six variants: BrowseCards (opens a card browser for a
deck, void, or status zone), CloseCardBrowser, SetSelectedEnergyAdditionalCost,
OpenPanel (with a PanelAddress), CloseCurrentPanel, and ToggleStackVisibility.

The same ActionService.PerformAction entry point also serves native Unity
GameObjects (cards, action buttons) which store their GameAction from CardView or
ButtonView data and fire it on mouse up.

## DisplayProperties and Mobile Adaptation

DisplayProperties (ui_components/src/display_properties.rs) captures client
screen information: screen_width, screen_height, and is_mobile_device. Defaults
to 1920x1080 desktop. The client sends these values in the ConnectRequest using
Screen.width, Screen.height, and Application.isMobilePlatform.

The Rust side stores properties globally per user via a mutex-protected HashMap.
Two consumers use them:

- **Typography** -- On mobile devices, font sizes scale down by 0.65x unless the
  Typography preset disables mobile scaling (only ButtonLabel does).
- **ScrollViewComponent** -- Hides vertical scrollbars on mobile devices.

On the client side, DocumentService uses a reference resolution of 225x400 at
16:9 (height-matched) for converting viewport dimension units to element pixels.

## Key Files Reference

**Rust -- masonry crate:**
- masonry/src/flex_node.rs -- FlexNode, NodeType, EventHandlers
- masonry/src/flex_style.rs -- FlexStyle with all CSS flexbox properties
- masonry/src/flex_enums.rs -- All style enums (FlexAlign, FlexDirection, etc.)
- masonry/src/dimension.rs -- Dimension, DimensionUnit, DimensionGroup
- masonry/src/borders.rs -- BorderWidth, BorderColor, BorderRadius

**Rust -- ui_components crate:**
- ui_components/src/component.rs -- Component trait, NodeComponent
- ui_components/src/box_component.rs -- BoxComponent and typestate builder
- ui_components/src/text_component.rs -- TextComponent
- ui_components/src/button_component.rs -- ButtonComponent
- ui_components/src/panel_component.rs -- PanelComponent
- ui_components/src/scroll_view_component.rs -- ScrollViewComponent
- ui_components/src/close_button_component.rs -- CloseButtonComponent
- ui_components/src/wrapper.rs -- WrapperComponent for type erasure
- ui_components/src/typography.rs -- Typography enum and font presets
- ui_components/src/icon.rs -- Icon font constants
- ui_components/src/display_properties.rs -- DisplayProperties

**Rust -- display crate:**
- display/src/rendering/interface_rendering.rs -- InterfaceView assembly
- display/src/rendering/interface_message.rs -- InterfaceMessage component
- display/src/rendering/supplemental_card_info.rs -- Card info zoom overlay
- display/src/panels/panel_rendering.rs -- PanelAddress routing
- display/src/panels/developer_panel.rs -- Developer debug panel
- display/src/panels/set_opponent_agent_panel.rs -- AI selection panel
- display/src/panels/add_card_to_hand_panel.rs -- Card addition panel
- display/src/panels/play_opponent_card_panel.rs -- Opponent card panel
- display/src/panels/view_logs_panel.rs -- Log viewer panel
- display/src/display_actions/apply_battle_display_action.rs -- Display action dispatch

**Unity -- Masonry:**
- client/Assets/Dreamtides/Masonry/Reconciler.cs -- Virtual-DOM diffing
- client/Assets/Dreamtides/Masonry/MasonRenderer.cs -- Style application
- client/Assets/Dreamtides/Masonry/Elements.cs -- IMasonElement, Callbacks
- client/Assets/Dreamtides/Masonry/Mason.cs -- Static utilities and builders
- client/Assets/Dreamtides/Masonry/Draggable.cs -- Drag-and-drop element
- client/Assets/Dreamtides/Masonry/ScrollView.cs -- Scroll view element
- client/Assets/Dreamtides/Masonry/Slider.cs -- Slider element
- client/Assets/Dreamtides/Masonry/TextField.cs -- Text field element
- client/Assets/Dreamtides/Masonry/TypewriterText.cs -- Typewriter text element

**Unity -- Services:**
- client/Assets/Dreamtides/Services/DocumentService.cs -- Overlay containers
- client/Assets/Dreamtides/Services/ActionServiceImpl.cs -- Action dispatch
