use display_data::battle_view::InterfaceView;
use masonry::flex_node::{FlexNode, NodeType};

#[derive(Default)]
pub struct TestInterfaceView {
    pub view: Option<InterfaceView>,
}

impl TestInterfaceView {
    pub fn new(view: Option<InterfaceView>) -> Self {
        Self { view }
    }

    /// Get text from the screen overlay
    pub fn screen_overlay_text(&self) -> String {
        self.view
            .as_ref()
            .and_then(|v| v.screen_overlay.as_ref())
            .map(extract_text_from_node)
            .unwrap_or_default()
    }

    /// Get text from the primary action button
    pub fn primary_action_button_text(&self) -> String {
        self.view
            .as_ref()
            .and_then(|v| v.primary_action_button.as_ref())
            .map(|button| button.label.clone())
            .unwrap_or_default()
    }

    /// Get text from the secondary action button
    pub fn secondary_action_button_text(&self) -> String {
        self.view
            .as_ref()
            .and_then(|v| v.secondary_action_button.as_ref())
            .map(|button| button.label.clone())
            .unwrap_or_default()
    }

    /// Get text from the increment button
    pub fn increment_button_text(&self) -> String {
        self.view
            .as_ref()
            .and_then(|v| v.increment_button.as_ref())
            .map(|button| button.label.clone())
            .unwrap_or_default()
    }

    /// Get text from the decrement button
    pub fn decrement_button_text(&self) -> String {
        self.view
            .as_ref()
            .and_then(|v| v.decrement_button.as_ref())
            .map(|button| button.label.clone())
            .unwrap_or_default()
    }

    /// Get text from the dev button
    pub fn dev_button_text(&self) -> String {
        self.view
            .as_ref()
            .and_then(|v| v.dev_button.as_ref())
            .map(|button| button.label.clone())
            .unwrap_or_default()
    }

    /// Get text from the undo button
    pub fn undo_button_text(&self) -> String {
        self.view
            .as_ref()
            .and_then(|v| v.undo_button.as_ref())
            .map(|button| button.label.clone())
            .unwrap_or_default()
    }

    /// Get text from the bottom right button
    pub fn bottom_right_button_text(&self) -> String {
        self.view
            .as_ref()
            .and_then(|v| v.bottom_right_button.as_ref())
            .map(|button| button.label.clone())
            .unwrap_or_default()
    }

    /// Check if screen overlay contains the given substring
    pub fn screen_overlay_contains(&self, substring: &str) -> bool {
        self.screen_overlay_text().contains(substring)
    }

    /// Check if primary action button contains the given substring
    pub fn primary_action_button_contains(&self, substring: &str) -> bool {
        self.primary_action_button_text().contains(substring)
    }

    /// Check if secondary action button contains the given substring
    pub fn secondary_action_button_contains(&self, substring: &str) -> bool {
        self.secondary_action_button_text().contains(substring)
    }

    /// Check if increment button contains the given substring
    pub fn increment_button_contains(&self, substring: &str) -> bool {
        self.increment_button_text().contains(substring)
    }

    /// Check if decrement button contains the given substring
    pub fn decrement_button_contains(&self, substring: &str) -> bool {
        self.decrement_button_text().contains(substring)
    }

    /// Check if dev button contains the given substring
    pub fn dev_button_contains(&self, substring: &str) -> bool {
        self.dev_button_text().contains(substring)
    }

    /// Check if undo button contains the given substring
    pub fn undo_button_contains(&self, substring: &str) -> bool {
        self.undo_button_text().contains(substring)
    }

    /// Check if bottom right button contains the given substring
    pub fn bottom_right_button_contains(&self, substring: &str) -> bool {
        self.bottom_right_button_text().contains(substring)
    }
}

/// Recursively extract all text content from a FlexNode and its children
pub fn extract_text_from_node(node: &FlexNode) -> String {
    let mut texts = Vec::new();
    collect_text_recursive(node, &mut texts);
    texts.join(" ")
}

/// Recursively collect text from a FlexNode hierarchy
fn collect_text_recursive(node: &FlexNode, texts: &mut Vec<String>) {
    if let Some(NodeType::Text(text_node)) = &node.node_type {
        if !text_node.label.is_empty() {
            texts.push(text_node.label.clone());
        }
    }

    for child in &node.children {
        collect_text_recursive(child, texts);
    }
}
