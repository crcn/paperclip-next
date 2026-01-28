use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Virtual DOM node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VNode {
    /// HTML element
    Element {
        tag: String,
        attributes: HashMap<String, String>,
        styles: HashMap<String, String>,
        children: Vec<VNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },

    /// Text node
    Text {
        content: String,
    },

    /// Comment node
    Comment {
        content: String,
    },
}

impl VNode {
    pub fn element(tag: impl Into<String>) -> Self {
        VNode::Element {
            tag: tag.into(),
            attributes: HashMap::new(),
            styles: HashMap::new(),
            children: Vec::new(),
            id: None,
        }
    }

    pub fn text(content: impl Into<String>) -> Self {
        VNode::Text {
            content: content.into(),
        }
    }

    pub fn with_attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let VNode::Element { ref mut attributes, .. } = self {
            attributes.insert(key.into(), value.into());
        }
        self
    }

    pub fn with_style(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let VNode::Element { ref mut styles, .. } = self {
            styles.insert(key.into(), value.into());
        }
        self
    }

    pub fn with_child(mut self, child: VNode) -> Self {
        if let VNode::Element { ref mut children, .. } = self {
            children.push(child);
        }
        self
    }

    pub fn with_children(mut self, new_children: Vec<VNode>) -> Self {
        if let VNode::Element { ref mut children, .. } = self {
            children.extend(new_children);
        }
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        if let VNode::Element { id: ref mut node_id, .. } = self {
            *node_id = Some(id.into());
        }
        self
    }
}

/// Virtual Document (collection of root nodes with metadata)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VDocument {
    pub nodes: Vec<VNode>,
    pub styles: Vec<CssRule>,
}

/// CSS Rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CssRule {
    pub selector: String,
    pub properties: HashMap<String, String>,
}

impl VDocument {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            styles: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: VNode) {
        self.nodes.push(node);
    }

    pub fn add_style(&mut self, selector: impl Into<String>, properties: HashMap<String, String>) {
        self.styles.push(CssRule {
            selector: selector.into(),
            properties,
        });
    }
}

impl Default for VDocument {
    fn default() -> Self {
        Self::new()
    }
}
