use bon::Builder;
use masonry::flex_enums::{TextAlign, WhiteSpace};
use masonry::flex_node::{FlexNode, NodeType, TextNode};
use masonry::flex_style::{FlexGrow, FlexShrink, FlexStyle};

use crate::component::{Component, NodeComponent};
use crate::style_options::{self, StyleOptions};
use crate::typography::{self, Typography};

#[derive(Clone, Builder)]
pub struct TextComponent {
    /// The text to display
    #[builder(into)]
    pub text: String,

    /// Controls the visual display of the text (size, color, etc)
    pub typography: Typography,

    pub style_options: Option<StyleOptions>,

    #[builder(into)]
    pub flex_grow: Option<FlexGrow>,

    #[builder(into)]
    pub flex_shrink: Option<FlexShrink>,

    pub text_align: Option<TextAlign>,

    pub white_space: Option<WhiteSpace>,
}

impl Component for TextComponent {
    fn render(self) -> Option<impl Component> {
        Some(NodeComponent)
    }

    fn flex_node(&self) -> Option<FlexNode> {
        let mut style = FlexStyle::default();

        typography::apply(&self.typography, &mut style);
        style_options::apply(&self.style_options, &mut style);

        if let Some(flex_grow) = self.flex_grow {
            style.flex_grow = Some(flex_grow);
        }

        if let Some(flex_shrink) = self.flex_shrink {
            style.flex_shrink = Some(flex_shrink);
        }

        if let Some(text_align) = self.text_align {
            style.text_align = Some(text_align);
        }

        if let Some(white_space) = self.white_space {
            style.white_space = Some(white_space);
        }

        Some(FlexNode {
            node_type: Some(NodeType::Text(TextNode { label: self.text.clone() })),
            style: Some(style),
            ..Default::default()
        })
    }
}
