use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Span information for source location tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub id: String,
}

impl Span {
    pub fn new(start: usize, end: usize, id: String) -> Self {
        Self { start, end, id }
    }
}

/// Root document node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub imports: Vec<Import>,
    pub tokens: Vec<TokenDecl>,
    pub triggers: Vec<TriggerDecl>,
    pub styles: Vec<StyleDecl>,
    pub components: Vec<Component>,
}

/// Import statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
    pub span: Span,
}

/// Token declaration (design tokens)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenDecl {
    pub public: bool,
    pub name: String,
    pub value: String,
    pub span: Span,
}

/// Trigger declaration (reusable CSS selectors/media queries)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerDecl {
    pub public: bool,
    pub name: String,
    pub selectors: Vec<String>,
    pub span: Span,
}

/// Style declaration (reusable style mixin)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleDecl {
    pub public: bool,
    pub name: String,
    pub extends: Vec<String>,
    pub properties: HashMap<String, String>,
    pub span: Span,
}

/// Component definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub public: bool,
    pub name: String,
    pub script: Option<ScriptDirective>,
    pub frame: Option<FrameAnnotation>,
    pub variants: Vec<Variant>,
    pub slots: Vec<Slot>,
    pub body: Option<Element>,
    pub span: Span,
}

/// Script directive for binding to external code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptDirective {
    pub src: String,
    pub target: String,
    pub name: Option<String>,
    pub span: Span,
}

/// Frame annotation for designer positioning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameAnnotation {
    pub x: f64,
    pub y: f64,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub span: Span,
}

/// Variant definition (state-based styling)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub name: String,
    pub triggers: Vec<String>,
    pub span: Span,
}

/// Slot definition (content insertion points)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    pub name: String,
    pub default_content: Vec<Element>,
    pub span: Span,
}

/// Element node (render tree)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Element {
    /// HTML element (div, button, etc.)
    Tag {
        tag_name: String,
        name: Option<String>,
        attributes: HashMap<String, Expression>,
        styles: Vec<StyleBlock>,
        children: Vec<Element>,
        span: Span,
    },

    /// Text node
    Text { content: Expression, span: Span },

    /// Component instance
    Instance {
        name: String,
        props: HashMap<String, Expression>,
        children: Vec<Element>,
        span: Span,
    },

    /// Conditional rendering
    Conditional {
        condition: Expression,
        then_branch: Vec<Element>,
        else_branch: Option<Vec<Element>>,
        span: Span,
    },

    /// Iteration
    Repeat {
        item_name: String,
        collection: Expression,
        body: Vec<Element>,
        span: Span,
    },

    /// Slot insertion
    SlotInsert { name: String, span: Span },

    /// Insert directive (explicit slot content)
    Insert {
        slot_name: String,
        content: Vec<Element>,
        span: Span,
    },
}

/// Style block (inline styles)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleBlock {
    pub variants: Vec<String>,
    pub extends: Vec<String>,
    pub properties: HashMap<String, String>,
    pub span: Span,
}

/// Expression (used in bindings and attributes)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Expression {
    /// String literal
    Literal { value: String, span: Span },

    /// Number literal
    Number { value: f64, span: Span },

    /// Boolean literal
    Boolean { value: bool, span: Span },

    /// Variable reference
    Variable { name: String, span: Span },

    /// Member access (obj.prop)
    Member {
        object: Box<Expression>,
        property: String,
        span: Span,
    },

    /// Binary operation (a + b)
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
        span: Span,
    },

    /// Function call
    Call {
        function: String,
        arguments: Vec<Expression>,
        span: Span,
    },

    /// String template interpolation
    Template {
        parts: Vec<TemplatePart>,
        span: Span,
    },
}

/// Binary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

/// Template string parts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemplatePart {
    Literal(String),
    Expression(Expression),
}

impl Document {
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            tokens: Vec::new(),
            triggers: Vec::new(),
            styles: Vec::new(),
            components: Vec::new(),
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
