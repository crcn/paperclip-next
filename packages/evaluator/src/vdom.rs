//! # Virtual DOM (VDOM)
//!
//! Paperclip's Virtual DOM representation for hot module reload and client-side rendering.
//!
//! ## Purpose
//!
//! The VDOM provides an intermediate representation between Paperclip's AST and rendered output.
//! It enables efficient diffing and patching for live preview updates.
//!
//! ## Core Types
//!
//! - **VNode**: Virtual DOM node (Element, Text, Comment, or Error)
//! - **VirtualDomDocument**: Complete VDOM tree with associated CSS rules
//! - **CssRule**: CSS rule with selector and properties
//!
//! ## Identity System
//!
//! Every `VNode::Element` has a **semantic_id** (required) that uniquely identifies it within
//! the VDOM tree. Semantic IDs are stable across refactoring (component renames, element moves, etc.).
//!
//! **Key Field**: Elements in repeat blocks should have explicit keys for stable diffing.
//! Auto-generated keys may not survive data reordering.
//!
//! ## Error Nodes
//!
//! `VNode::Error` nodes represent evaluation errors that don't crash the entire preview.
//! They include the error message, source span, and semantic ID for proper diffing.
//!
//! ## Usage
//!
//! ```rust
//! use paperclip_evaluator::VNode;
//! use paperclip_semantics::SemanticID;
//!
//! let node = VNode::element("div", SemanticID::root())
//!     .with_attr("class", "container")
//!     .with_style("color", "red")
//!     .with_child(VNode::text("Hello"));
//! ```

use paperclip_parser::ast::Span;
use paperclip_semantics::SemanticID;
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
        /// Semantic identity (stable across refactoring) - REQUIRED
        semantic_id: SemanticID,
        /// Source ID mapping back to AST span.id (for mutations)
        #[serde(skip_serializing_if = "Option::is_none")]
        source_id: Option<String>,
        /// Explicit key for repeat items (from key attribute)
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
    },

    /// Text node
    Text { content: String },

    /// Comment node
    Comment { content: String },

    /// Error node (for partial evaluation - shows errors inline instead of crashing)
    Error {
        message: String,
        /// Source location where error occurred
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
        /// Semantic ID for diffing (errors at same location should match)
        semantic_id: SemanticID,
    },
}

impl VNode {
    pub fn element(tag: impl Into<String>, semantic_id: SemanticID) -> Self {
        VNode::Element {
            tag: tag.into(),
            attributes: HashMap::new(),
            styles: HashMap::new(),
            children: Vec::new(),
            semantic_id,
            source_id: None,
            key: None,
        }
    }

    pub fn text(content: impl Into<String>) -> Self {
        VNode::Text {
            content: content.into(),
        }
    }

    pub fn error(message: impl Into<String>, span: Option<Span>, semantic_id: SemanticID) -> Self {
        VNode::Error {
            message: message.into(),
            span,
            semantic_id,
        }
    }

    pub fn with_attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let VNode::Element {
            ref mut attributes, ..
        } = self
        {
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
        if let VNode::Element {
            ref mut children, ..
        } = self
        {
            children.push(child);
        }
        self
    }

    pub fn with_children(mut self, new_children: Vec<VNode>) -> Self {
        if let VNode::Element {
            ref mut children, ..
        } = self
        {
            children.extend(new_children);
        }
        self
    }

    pub fn with_semantic_id(mut self, new_semantic_id: SemanticID) -> Self {
        if let VNode::Element {
            semantic_id: ref mut sid,
            ..
        } = self
        {
            *sid = new_semantic_id;
        }
        self
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        if let VNode::Element {
            key: ref mut node_key,
            ..
        } = self
        {
            *node_key = Some(key.into());
        }
        self
    }

    pub fn with_source_id(mut self, id: impl Into<String>) -> Self {
        if let VNode::Element {
            source_id: ref mut sid,
            ..
        } = self
        {
            *sid = Some(id.into());
        }
        self
    }
}

/// Component metadata for designer use (frames, annotations, descriptions)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Component name
    pub name: String,
    /// Description from doc comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Frame positioning for designer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<FrameMetadata>,
    /// All annotations from doc comment (for extensibility)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<AnnotationMetadata>,
    /// Source span ID for mutations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
}

/// Frame metadata from @frame annotation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
}

/// Generic annotation metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationMetadata {
    pub name: String,
    pub params: std::collections::HashMap<String, serde_json::Value>,
}

/// Virtual Document (collection of root nodes with metadata)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualDomDocument {
    pub nodes: Vec<VNode>,
    pub styles: Vec<CssRule>,
    /// Component metadata for designer (frames, descriptions, annotations)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<ComponentMetadata>,
}

/// CSS Rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CssRule {
    pub selector: String,
    pub properties: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_query: Option<String>,
}

impl VirtualDomDocument {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            styles: Vec::new(),
            components: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: VNode) {
        self.nodes.push(node);
    }

    pub fn add_style(&mut self, selector: impl Into<String>, properties: HashMap<String, String>) {
        self.styles.push(CssRule {
            selector: selector.into(),
            properties,
            media_query: None,
        });
    }
}

impl Default for VirtualDomDocument {
    fn default() -> Self {
        Self::new()
    }
}
