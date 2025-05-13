use masonry::flex_node::FlexNode;

/// A component is any reusable piece of UI.
///
/// Typically this is a struct that has one or more properties settable via a
/// builder pattern.
///
/// Components can either return another component, typically by invoking its
/// `build` method, or can create and return UI node directly
pub trait Component: Clone {
    fn render(self) -> Option<impl Component>;

    fn flex_node(&self) -> Option<FlexNode> {
        self.clone().render().and_then(|c| c.flex_node())
    }
}

impl<T: Component> Component for Option<T> {
    fn render(self) -> Option<impl Component> {
        if let Some(c) = self {
            c.render()
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct NodeComponent;

impl Component for NodeComponent {
    fn render(self) -> Option<impl Component> {
        Some(self)
    }
}
