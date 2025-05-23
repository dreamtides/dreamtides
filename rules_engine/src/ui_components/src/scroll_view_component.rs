use masonry::flex_node::{FlexNode, NodeType, ScrollViewNode};
use masonry::flex_style::FlexStyle;

use crate::component::{Component, NodeComponent};

#[derive(Clone)]
pub struct ScrollViewComponent(pub FlexNode);

impl Component for ScrollViewComponent {
    fn render(self) -> Option<impl Component> {
        Some(NodeComponent)
    }

    fn flex_node(&self) -> Option<FlexNode> {
        Some(self.0.clone())
    }
}

pub struct ScrollViewComponentBuilder {
    content: Option<FlexNode>,
}

impl ScrollViewComponent {
    pub fn builder() -> ScrollViewComponentBuilder {
        ScrollViewComponentBuilder { content: None }
    }
}

impl ScrollViewComponentBuilder {
    pub fn child(mut self, child: impl Component) -> Self {
        if let Some(child_node) = child.flex_node() {
            self.content = Some(child_node);
        }
        self
    }

    pub fn build(self) -> ScrollViewComponent {
        let children = if let Some(content) = self.content { vec![content] } else { vec![] };

        ScrollViewComponent(FlexNode {
            name: Some("ScrollView".to_string()),
            node_type: Some(NodeType::ScrollViewNode(Box::new(ScrollViewNode {
                ..Default::default()
            }))),
            children,
            style: Some(FlexStyle {
                margin: Some(0.into()),
                padding: Some(0.into()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
