//! Virtual DOM definitions for evaluated Paperclip output
//!
//! This is what the evaluator produces - a platform-agnostic representation
//! that can be:
//! 1. Rendered directly in the preview (fast patching)
//! 2. Compiled to React/Vue/etc.
//! 3. Diffed for incremental updates

use serde::{Deserialize, Serialize};

/// Virtual HTML node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VirtualNode {
    Element(VirtualElement),
    Text(String),
    Fragment(Vec<VirtualNode>),
}

/// Virtual element with source mapping
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualElement {
    /// HTML tag name
    pub tag: String,

    /// Unique ID for designer selection (maps back to AST)
    pub source_id: String,

    /// HTML attributes
    pub attributes: Vec<(String, String)>,

    /// CSS class names (generated)
    pub class_names: Vec<String>,

    /// Inline styles (for dynamic values)
    pub inline_styles: Vec<(String, String)>,

    /// Child nodes
    pub children: Vec<VirtualNode>,

    /// If this is a live component placeholder
    pub live_component: Option<LiveComponentRef>,
}

/// Reference to a live component that needs real JS mounting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveComponentRef {
    /// Component identifier (e.g., "@app/GoogleMap")
    pub component_id: String,
    /// Props to pass (serialized to JSON)
    pub props: String,
}

/// Virtual CSS output
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualCSS {
    /// CSS rules
    pub rules: Vec<CSSRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CSSRule {
    /// Selector (e.g., ".pc-abc123")
    pub selector: String,
    /// Declarations
    pub declarations: Vec<(String, String)>,
    /// Media query wrapper if any
    pub media: Option<String>,
}

/// Evaluated module output
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatedModule {
    /// File path
    pub path: String,
    /// Root components with their frames
    pub components: Vec<EvaluatedComponent>,
    /// Generated CSS
    pub css: VirtualCSS,
}

/// A single evaluated component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatedComponent {
    /// Component name
    pub name: String,
    /// Frame position on canvas (from @frame annotation)
    pub frame: Option<Frame>,
    /// Rendered virtual DOM
    pub vdom: VirtualNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: Option<f64>,
}

/// Patch operation for incremental updates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Patch {
    Insert {
        path: Vec<usize>,
        node: VirtualNode,
    },
    Remove {
        path: Vec<usize>,
    },
    Replace {
        path: Vec<usize>,
        node: VirtualNode,
    },
    UpdateAttributes {
        path: Vec<usize>,
        attributes: Vec<(String, String)>,
    },
    UpdateText {
        path: Vec<usize>,
        text: String,
    },
}
