use action_data::game_action::GameAction;
use masonry::flex_node::{EventHandlers, FlexNode};
use masonry::flex_style::FlexStyle;

/// Represents a generic flexbox which can contain other UI components.
pub struct BoxComponent(pub FlexNode);

#[derive(Default)]
pub struct BoxComponentBuilder {
    pub name: Option<String>,
    pub children: Vec<FlexNode>,
    pub event_handlers: Option<EventHandlers>,
    pub style: Option<FlexStyle>,
    pub hover_style: Option<FlexStyle>,
    pub pressed_style: Option<FlexStyle>,
    pub on_attach_style: Option<FlexStyle>,
}

impl BoxComponent {
    pub fn builder() -> BoxComponentBuilder {
        BoxComponentBuilder::default()
    }
}

impl From<BoxComponent> for Option<FlexNode> {
    fn from(component: BoxComponent) -> Self {
        Some(component.0)
    }
}

impl BoxComponentBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn child(mut self, child: impl Into<Option<FlexNode>>) -> Self {
        if let Some(child) = child.into() {
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

    pub fn on_attach_style(mut self, on_attach_style: FlexStyle) -> Self {
        self.on_attach_style = Some(on_attach_style);
        self
    }

    pub fn build(self) -> BoxComponent {
        BoxComponent(FlexNode {
            name: self.name,
            node_type: None,
            children: self.children,
            event_handlers: self.event_handlers,
            style: self.style,
            hover_style: self.hover_style,
            pressed_style: self.pressed_style,
            on_attach_style: self.on_attach_style,
        })
    }
}
