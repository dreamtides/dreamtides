use masonry::flex_node::FlexNode;

use crate::component::{Component, NodeComponent};

/// Implements type erasure for components.
///
/// If you want to interact with a set of components in a generic way, you can
/// wrap them in wrapper::wrap to provide a uniform static type for all
/// components.
pub fn wrap(child: impl Component) -> Option<WrapperComponent> {
    child.flex_node().map(|child| WrapperComponent { child })
}

#[derive(Clone)]
pub struct WrapperComponent {
    child: FlexNode,
}

impl Component for WrapperComponent {
    fn render(self) -> Option<impl Component> {
        Some(NodeComponent)
    }

    fn flex_node(&self) -> Option<FlexNode> {
        Some(self.child.clone())
    }
}
