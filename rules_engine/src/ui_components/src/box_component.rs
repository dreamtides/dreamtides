use action_data::game_action_data::GameAction;
use core_data::display_types::Milliseconds;
use masonry::flex_enums::FlexDirection;
use masonry::flex_node::{EventHandlers, FlexNode};
use masonry::flex_style::FlexStyle;

use crate::component::{Component, NodeComponent};

/// Represents a generic flexbox which can contain other UI components.
#[derive(Clone)]
pub struct BoxComponent(pub FlexNode);

impl Component for BoxComponent {
    fn render(self) -> Option<impl Component> {
        Some(NodeComponent)
    }

    fn flex_node(&self) -> Option<FlexNode> {
        Some(self.0.clone())
    }
}

pub struct Unnamed;

pub struct Named(pub String);

pub struct BoxComponentBuilder<T> {
    name: T,
    pub children: Vec<FlexNode>,
    event_handlers: Option<EventHandlers>,
    style: Option<FlexStyle>,
    hover_style: Option<FlexStyle>,
    pressed_style: Option<FlexStyle>,
    on_attach_style: Option<FlexStyle>,
    on_attach_style_duration: Option<Milliseconds>,
}

impl BoxComponent {
    pub fn builder() -> BoxComponentBuilder<Unnamed> {
        BoxComponentBuilder {
            name: Unnamed,
            children: Vec::new(),
            event_handlers: None,
            style: None,
            hover_style: None,
            pressed_style: None,
            on_attach_style: None,
            on_attach_style_duration: None,
        }
    }
}

impl BoxComponentBuilder<Unnamed> {
    pub fn name(self, name: impl Into<String>) -> BoxComponentBuilder<Named> {
        BoxComponentBuilder {
            name: Named(name.into()),
            children: self.children,
            event_handlers: self.event_handlers,
            style: self.style,
            hover_style: self.hover_style,
            pressed_style: self.pressed_style,
            on_attach_style: self.on_attach_style,
            on_attach_style_duration: self.on_attach_style_duration,
        }
    }
}

impl BoxComponentBuilder<Named> {
    pub fn child(mut self, child: impl Component) -> Self {
        if let Some(child) = child.flex_node() {
            self.children.push(child);
        }
        self
    }

    pub fn children(mut self, children: Vec<FlexNode>) -> Self {
        self.children = children;
        self
    }

    pub fn on_click(mut self, action: impl Into<GameAction>) -> Self {
        self.event_handlers.get_or_insert(EventHandlers::default()).on_click = Some(action.into());
        self
    }

    pub fn event_handlers(mut self, event_handlers: EventHandlers) -> Self {
        self.event_handlers = Some(event_handlers);
        self
    }

    pub fn style(mut self, style: FlexStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn hover_style(mut self, hover_style: FlexStyle) -> Self {
        self.hover_style = Some(hover_style);
        self
    }

    pub fn pressed_style(mut self, pressed_style: FlexStyle) -> Self {
        self.pressed_style = Some(pressed_style);
        self
    }

    pub fn on_attach_style(self, on_attach_style: FlexStyle) -> Self {
        self.maybe_on_attach_style(Some(on_attach_style))
    }

    pub fn maybe_on_attach_style(mut self, on_attach_style: Option<FlexStyle>) -> Self {
        self.on_attach_style = on_attach_style;
        self
    }

    pub fn on_attach_style_duration(self, duration: Milliseconds) -> Self {
        self.maybe_on_attach_style_duration(Some(duration))
    }

    pub fn maybe_on_attach_style_duration(mut self, duration: Option<Milliseconds>) -> Self {
        self.on_attach_style_duration = duration;
        self
    }
}

impl BoxComponentBuilder<Named> {
    pub fn build(self) -> BoxComponent {
        // Default flex direction to Row.
        let style = match self.style {
            Some(mut style) if style.flex_direction.is_none() => {
                style.flex_direction = Some(FlexDirection::Row);
                Some(style)
            }
            other => other,
        };

        BoxComponent(FlexNode {
            name: Some(self.name.0),
            node_type: None,
            children: self.children,
            event_handlers: self.event_handlers,
            style,
            hover_style: self.hover_style,
            pressed_style: self.pressed_style,
            on_attach_style: self.on_attach_style,
            on_attach_style_duration: self.on_attach_style_duration,
        })
    }
}
