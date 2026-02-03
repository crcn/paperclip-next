//! # VDOM Extension Traits
//!
//! Extension traits that add builder methods and helpers to proto-generated VDOM types.
//! These provide ergonomic APIs for constructing and pattern-matching proto types.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::vdom_ext::{VNodeExt, ElementNodeExt, as_element};
//! use crate::vdom_differ::proto::vdom as proto;
//!
//! // Building nodes
//! let node = proto::VNode::element("div", "root")
//!     .with_attr("class", "container")
//!     .with_child(proto::VNode::text("Hello"));
//!
//! // Pattern matching
//! if let Some(elem) = as_element(&node) {
//!     println!("Tag: {}", elem.tag);
//! }
//! ```

use crate::vdom_differ::proto::vdom as proto;
use std::collections::HashMap;

// ============================================================================
// Value Builders (for flexible metadata)
// ============================================================================

/// Extension trait for building Value instances
pub trait ValueExt {
    /// Create a null value
    fn null() -> proto::Value;
    /// Create a number value
    fn number(n: f64) -> proto::Value;
    /// Create a string value
    fn string(s: impl Into<String>) -> proto::Value;
    /// Create a boolean value
    fn bool(b: bool) -> proto::Value;
    /// Create an object value from field pairs
    fn object(fields: impl IntoIterator<Item = (impl Into<String>, proto::Value)>) -> proto::Value;
    /// Create a list value
    fn list(items: impl IntoIterator<Item = proto::Value>) -> proto::Value;
}

impl ValueExt for proto::Value {
    fn null() -> proto::Value {
        proto::Value {
            kind: Some(proto::value::Kind::NullValue(proto::NullValue::NullValue as i32)),
        }
    }

    fn number(n: f64) -> proto::Value {
        proto::Value {
            kind: Some(proto::value::Kind::NumberValue(n)),
        }
    }

    fn string(s: impl Into<String>) -> proto::Value {
        proto::Value {
            kind: Some(proto::value::Kind::StringValue(s.into())),
        }
    }

    fn bool(b: bool) -> proto::Value {
        proto::Value {
            kind: Some(proto::value::Kind::BoolValue(b)),
        }
    }

    fn object(fields: impl IntoIterator<Item = (impl Into<String>, proto::Value)>) -> proto::Value {
        let mut map = HashMap::new();
        for (k, v) in fields {
            map.insert(k.into(), v);
        }
        proto::Value {
            kind: Some(proto::value::Kind::ObjectValue(proto::ObjectValue { fields: map })),
        }
    }

    fn list(items: impl IntoIterator<Item = proto::Value>) -> proto::Value {
        proto::Value {
            kind: Some(proto::value::Kind::ListValue(proto::ListValue {
                values: items.into_iter().collect(),
            })),
        }
    }
}

// ============================================================================
// VNode Builders
// ============================================================================

/// Extension trait for building VNode instances
pub trait VNodeExt {
    /// Create an element node
    fn element(tag: impl Into<String>, semantic_id: impl Into<String>) -> proto::VNode;
    /// Create a text node
    fn text(content: impl Into<String>) -> proto::VNode;
    /// Create a comment node
    fn comment(content: impl Into<String>) -> proto::VNode;
    /// Create an error node
    fn error(message: impl Into<String>, semantic_id: impl Into<String>) -> proto::VNode;
    /// Create an error node with span
    fn error_with_span(
        message: impl Into<String>,
        semantic_id: impl Into<String>,
        span: proto::Span,
    ) -> proto::VNode;
}

impl VNodeExt for proto::VNode {
    fn element(tag: impl Into<String>, semantic_id: impl Into<String>) -> proto::VNode {
        proto::VNode {
            node_type: Some(proto::v_node::NodeType::Element(proto::ElementNode {
                tag: tag.into(),
                attributes: HashMap::new(),
                styles: HashMap::new(),
                children: Vec::new(),
                semantic_id: semantic_id.into(),
                key: None,
                source_id: None,
                metadata: None,
            })),
        }
    }

    fn text(content: impl Into<String>) -> proto::VNode {
        proto::VNode {
            node_type: Some(proto::v_node::NodeType::Text(proto::TextNode {
                content: content.into(),
            })),
        }
    }

    fn comment(content: impl Into<String>) -> proto::VNode {
        proto::VNode {
            node_type: Some(proto::v_node::NodeType::Comment(proto::CommentNode {
                content: content.into(),
            })),
        }
    }

    fn error(message: impl Into<String>, semantic_id: impl Into<String>) -> proto::VNode {
        proto::VNode {
            node_type: Some(proto::v_node::NodeType::Error(proto::ErrorNode {
                message: message.into(),
                semantic_id: semantic_id.into(),
                span: None,
            })),
        }
    }

    fn error_with_span(
        message: impl Into<String>,
        semantic_id: impl Into<String>,
        span: proto::Span,
    ) -> proto::VNode {
        proto::VNode {
            node_type: Some(proto::v_node::NodeType::Error(proto::ErrorNode {
                message: message.into(),
                semantic_id: semantic_id.into(),
                span: Some(span),
            })),
        }
    }
}

/// Extension trait for element node builder methods
pub trait ElementNodeExt {
    /// Add an attribute
    fn with_attr(self, key: impl Into<String>, value: impl Into<String>) -> Self;
    /// Add a style property
    fn with_style(self, key: impl Into<String>, value: impl Into<String>) -> Self;
    /// Add a child node
    fn with_child(self, child: proto::VNode) -> Self;
    /// Add multiple children
    fn with_children(self, children: Vec<proto::VNode>) -> Self;
    /// Set the key
    fn with_key(self, key: impl Into<String>) -> Self;
    /// Set the source_id
    fn with_source_id(self, id: impl Into<String>) -> Self;
    /// Set the semantic_id
    fn with_semantic_id(self, id: impl Into<String>) -> Self;
    /// Set arbitrary metadata
    fn with_metadata(self, metadata: proto::Value) -> Self;
    /// Set frame metadata (convenience method)
    fn with_frame(self, x: f64, y: f64, width: Option<f64>, height: Option<f64>) -> Self;
}

impl ElementNodeExt for proto::VNode {
    fn with_attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let Some(proto::v_node::NodeType::Element(ref mut elem)) = self.node_type {
            elem.attributes.insert(key.into(), value.into());
        }
        self
    }

    fn with_style(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let Some(proto::v_node::NodeType::Element(ref mut elem)) = self.node_type {
            elem.styles.insert(key.into(), value.into());
        }
        self
    }

    fn with_child(mut self, child: proto::VNode) -> Self {
        if let Some(proto::v_node::NodeType::Element(ref mut elem)) = self.node_type {
            elem.children.push(child);
        }
        self
    }

    fn with_children(mut self, children: Vec<proto::VNode>) -> Self {
        if let Some(proto::v_node::NodeType::Element(ref mut elem)) = self.node_type {
            elem.children.extend(children);
        }
        self
    }

    fn with_key(mut self, key: impl Into<String>) -> Self {
        if let Some(proto::v_node::NodeType::Element(ref mut elem)) = self.node_type {
            elem.key = Some(key.into());
        }
        self
    }

    fn with_source_id(mut self, id: impl Into<String>) -> Self {
        if let Some(proto::v_node::NodeType::Element(ref mut elem)) = self.node_type {
            elem.source_id = Some(id.into());
        }
        self
    }

    fn with_semantic_id(mut self, id: impl Into<String>) -> Self {
        if let Some(proto::v_node::NodeType::Element(ref mut elem)) = self.node_type {
            elem.semantic_id = id.into();
        }
        self
    }

    fn with_metadata(mut self, metadata: proto::Value) -> Self {
        if let Some(proto::v_node::NodeType::Element(ref mut elem)) = self.node_type {
            // Merge metadata if existing, otherwise set
            if let Some(existing) = &mut elem.metadata {
                // Merge object fields if both are objects
                match (&mut existing.kind, &metadata.kind) {
                    (
                        Some(proto::value::Kind::ObjectValue(existing_obj)),
                        Some(proto::value::Kind::ObjectValue(new_obj)),
                    ) => {
                        for (k, v) in &new_obj.fields {
                            existing_obj.fields.insert(k.clone(), v.clone());
                        }
                    }
                    _ => {
                        // Otherwise replace
                        elem.metadata = Some(metadata);
                    }
                }
            } else {
                elem.metadata = Some(metadata);
            }
        }
        self
    }

    fn with_frame(self, x: f64, y: f64, width: Option<f64>, height: Option<f64>) -> Self {
        let mut fields: Vec<(String, proto::Value)> = vec![
            ("x".to_string(), proto::Value::number(x)),
            ("y".to_string(), proto::Value::number(y)),
        ];
        if let Some(w) = width {
            fields.push(("width".to_string(), proto::Value::number(w)));
        }
        if let Some(h) = height {
            fields.push(("height".to_string(), proto::Value::number(h)));
        }

        let frame_value = proto::Value::object(fields);
        self.with_metadata(proto::Value::object(vec![("frame".to_string(), frame_value)]))
    }
}

// ============================================================================
// Pattern Matching Helpers
// ============================================================================

/// Extract element from a VNode
#[inline]
pub fn as_element(node: &proto::VNode) -> Option<&proto::ElementNode> {
    match &node.node_type {
        Some(proto::v_node::NodeType::Element(e)) => Some(e),
        _ => None,
    }
}

/// Extract mutable element from a VNode
#[inline]
pub fn as_element_mut(node: &mut proto::VNode) -> Option<&mut proto::ElementNode> {
    match &mut node.node_type {
        Some(proto::v_node::NodeType::Element(e)) => Some(e),
        _ => None,
    }
}

/// Extract text node from a VNode
#[inline]
pub fn as_text(node: &proto::VNode) -> Option<&proto::TextNode> {
    match &node.node_type {
        Some(proto::v_node::NodeType::Text(t)) => Some(t),
        _ => None,
    }
}

/// Extract comment node from a VNode
#[inline]
pub fn as_comment(node: &proto::VNode) -> Option<&proto::CommentNode> {
    match &node.node_type {
        Some(proto::v_node::NodeType::Comment(c)) => Some(c),
        _ => None,
    }
}

/// Extract error node from a VNode
#[inline]
pub fn as_error(node: &proto::VNode) -> Option<&proto::ErrorNode> {
    match &node.node_type {
        Some(proto::v_node::NodeType::Error(e)) => Some(e),
        _ => None,
    }
}

/// Extract component node from a VNode
#[inline]
pub fn as_component(node: &proto::VNode) -> Option<&proto::ComponentNode> {
    match &node.node_type {
        Some(proto::v_node::NodeType::Component(c)) => Some(c),
        _ => None,
    }
}

/// Check if node is an element
#[inline]
pub fn is_element(node: &proto::VNode) -> bool {
    matches!(&node.node_type, Some(proto::v_node::NodeType::Element(_)))
}

/// Check if node is a text node
#[inline]
pub fn is_text(node: &proto::VNode) -> bool {
    matches!(&node.node_type, Some(proto::v_node::NodeType::Text(_)))
}

/// Check if node is a comment
#[inline]
pub fn is_comment(node: &proto::VNode) -> bool {
    matches!(&node.node_type, Some(proto::v_node::NodeType::Comment(_)))
}

/// Check if node is an error
#[inline]
pub fn is_error(node: &proto::VNode) -> bool {
    matches!(&node.node_type, Some(proto::v_node::NodeType::Error(_)))
}

// ============================================================================
// Semantic ID Extraction
// ============================================================================

/// Get the semantic ID from any node type that has one
#[inline]
pub fn get_semantic_id(node: &proto::VNode) -> Option<&str> {
    match &node.node_type {
        Some(proto::v_node::NodeType::Element(e)) => Some(&e.semantic_id),
        Some(proto::v_node::NodeType::Error(e)) => Some(&e.semantic_id),
        Some(proto::v_node::NodeType::Component(c)) => Some(&c.semantic_id),
        _ => None,
    }
}

// ============================================================================
// Metadata Extraction Helpers
// ============================================================================

/// Extract frame metadata from an element node
pub fn get_frame(node: &proto::VNode) -> Option<(f64, f64, Option<f64>, Option<f64>)> {
    let elem = as_element(node)?;
    let metadata = elem.metadata.as_ref()?;
    let obj = match &metadata.kind {
        Some(proto::value::Kind::ObjectValue(o)) => o,
        _ => return None,
    };
    let frame = obj.fields.get("frame")?;
    let frame_obj = match &frame.kind {
        Some(proto::value::Kind::ObjectValue(o)) => o,
        _ => return None,
    };

    let x = get_number_from_fields(&frame_obj.fields, "x")?;
    let y = get_number_from_fields(&frame_obj.fields, "y")?;
    let width = get_number_from_fields(&frame_obj.fields, "width");
    let height = get_number_from_fields(&frame_obj.fields, "height");

    Some((x, y, width, height))
}

/// Extract a number value from a field map
fn get_number_from_fields(fields: &HashMap<String, proto::Value>, key: &str) -> Option<f64> {
    match fields.get(key)?.kind.as_ref()? {
        proto::value::Kind::NumberValue(n) => Some(*n),
        _ => None,
    }
}

// ============================================================================
// VDocument Builders
// ============================================================================

/// Extension trait for VDocument
pub trait VDocumentExt {
    /// Create an empty document
    fn new() -> proto::VDocument;
    /// Add a node to the document
    fn add_node(&mut self, node: proto::VNode);
    /// Add a CSS rule to the document
    fn add_style(&mut self, rule: proto::CssRule);
    /// Add component metadata
    fn add_component_metadata(&mut self, meta: proto::ComponentMetadata);
}

impl VDocumentExt for proto::VDocument {
    fn new() -> proto::VDocument {
        proto::VDocument {
            nodes: Vec::new(),
            styles: Vec::new(),
            components: Vec::new(),
            metadata: None,
        }
    }

    fn add_node(&mut self, node: proto::VNode) {
        self.nodes.push(node);
    }

    fn add_style(&mut self, rule: proto::CssRule) {
        self.styles.push(rule);
    }

    fn add_component_metadata(&mut self, meta: proto::ComponentMetadata) {
        self.components.push(meta);
    }
}

// ============================================================================
// CssRule Builders
// ============================================================================

/// Extension trait for CssRule
pub trait CssRuleExt {
    /// Create a new CSS rule
    fn new(selector: impl Into<String>, properties: HashMap<String, String>) -> proto::CssRule;
    /// Create a new CSS rule with media query
    fn new_with_media(
        selector: impl Into<String>,
        properties: HashMap<String, String>,
        media_query: impl Into<String>,
    ) -> proto::CssRule;
}

impl CssRuleExt for proto::CssRule {
    fn new(selector: impl Into<String>, properties: HashMap<String, String>) -> proto::CssRule {
        proto::CssRule {
            selector: selector.into(),
            properties,
            media_query: None,
            metadata: None,
        }
    }

    fn new_with_media(
        selector: impl Into<String>,
        properties: HashMap<String, String>,
        media_query: impl Into<String>,
    ) -> proto::CssRule {
        proto::CssRule {
            selector: selector.into(),
            properties,
            media_query: Some(media_query.into()),
            metadata: None,
        }
    }
}

// ============================================================================
// Span Builders
// ============================================================================

/// Extension trait for Span
pub trait SpanExt {
    /// Create a new span
    fn new(start: u32, end: u32, id: impl Into<String>) -> proto::Span;
}

impl SpanExt for proto::Span {
    fn new(start: u32, end: u32, id: impl Into<String>) -> proto::Span {
        proto::Span {
            start,
            end,
            id: id.into(),
        }
    }
}

/// Convert parser Span to proto Span
pub fn span_to_proto(span: &paperclip_parser::ast::Span) -> proto::Span {
    proto::Span {
        start: span.start as u32,
        end: span.end as u32,
        id: span.id.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_builder() {
        let node = proto::VNode::element("div", "root")
            .with_attr("class", "container")
            .with_style("padding", "16px")
            .with_child(proto::VNode::text("Hello"));

        let elem = as_element(&node).expect("Should be element");
        assert_eq!(elem.tag, "div");
        assert_eq!(elem.semantic_id, "root");
        assert_eq!(elem.attributes.get("class"), Some(&"container".to_string()));
        assert_eq!(elem.styles.get("padding"), Some(&"16px".to_string()));
        assert_eq!(elem.children.len(), 1);
    }

    #[test]
    fn test_text_builder() {
        let node = proto::VNode::text("Hello World");
        let text = as_text(&node).expect("Should be text");
        assert_eq!(text.content, "Hello World");
    }

    #[test]
    fn test_error_builder() {
        let node = proto::VNode::error("Something went wrong", "error-id");
        let error = as_error(&node).expect("Should be error");
        assert_eq!(error.message, "Something went wrong");
        assert_eq!(error.semantic_id, "error-id");
    }

    #[test]
    fn test_frame_metadata() {
        let node = proto::VNode::element("div", "root").with_frame(100.0, 200.0, Some(400.0), Some(300.0));

        let (x, y, width, height) = get_frame(&node).expect("Should have frame");
        assert_eq!(x, 100.0);
        assert_eq!(y, 200.0);
        assert_eq!(width, Some(400.0));
        assert_eq!(height, Some(300.0));
    }

    #[test]
    fn test_value_builders() {
        let num = proto::Value::number(42.0);
        assert!(matches!(num.kind, Some(proto::value::Kind::NumberValue(n)) if n == 42.0));

        let s = proto::Value::string("hello");
        assert!(matches!(s.kind, Some(proto::value::Kind::StringValue(ref s)) if s == "hello"));

        let b = proto::Value::bool(true);
        assert!(matches!(b.kind, Some(proto::value::Kind::BoolValue(true))));

        let obj = proto::Value::object(vec![("key".to_string(), proto::Value::number(1.0))]);
        assert!(matches!(obj.kind, Some(proto::value::Kind::ObjectValue(_))));
    }

    #[test]
    fn test_vdocument_builder() {
        let mut doc = proto::VDocument::new();
        doc.add_node(proto::VNode::element("div", "root"));
        doc.add_style(proto::CssRule::new(
            ".test",
            [("color".to_string(), "red".to_string())].into_iter().collect(),
        ));

        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(doc.styles.len(), 1);
    }
}
