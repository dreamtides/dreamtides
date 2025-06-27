use masonry::flex_node::FlexNode;

use crate::wrapper::{self, WrapperComponent};

/// A component is any reusable piece of UI.
///
/// Typically this is a struct that has one or more properties settable via a
/// builder pattern.
///
/// Components can either return another component, typically by invoking its
/// `build` method, or can create and return UI node directly
pub trait Component: Clone {
    fn render(self) -> Option<impl Component>;

    /// Renders this component to a [FlexNode].
    fn flex_node(&self) -> Option<FlexNode> {
        self.clone().render().and_then(|c| c.flex_node())
    }

    /// Erases the type of this component.
    ///
    /// If you want to interact with a set of components in a generic way, you
    /// can wrap() them to provide a uniform static type for all components.
    fn wrap(self) -> Option<WrapperComponent> {
        wrapper::wrap(self)
    }
}

impl<T: Component> Component for Option<T> {
    fn render(self) -> Option<impl Component> {
        if let Some(c) = self { c.render() } else { None }
    }

    fn flex_node(&self) -> Option<FlexNode> {
        if let Some(c) = self { c.flex_node() } else { None }
    }
}

#[derive(Clone)]
pub struct NodeComponent;

impl Component for NodeComponent {
    fn render(self) -> Option<impl Component> {
        None::<NodeComponent>
    }
}
