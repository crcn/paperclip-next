//! AST definitions for Paperclip .pc files
//!
//! The AST preserves source spans for:
//! 1. Error reporting with precise locations
//! 2. Roundtrip serialization (edit AST → regenerate source)
//! 3. Designer click-to-source mapping

use serde::{Deserialize, Serialize};

/// Source location span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// A node with source location information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

/// Root document containing all top-level items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub items: Vec<Spanned<Item>>,
}

/// Top-level items in a .pc file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Import(Import),
    Token(Token),
    Style(StyleDefinition),
    Component(Component),
}

/// Import statement: `import "./path.pc" as alias`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Import {
    pub path: Spanned<String>,
    pub alias: Option<Spanned<String>>,
}

/// Design token: `public token primaryColor #3366FF`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub is_public: bool,
    pub name: Spanned<String>,
    pub value: Spanned<TokenValue>,
}

/// Token value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenValue {
    Color(String),
    Dimension(f64, String), // e.g., 16px, 1.5rem
    String(String),
    Number(f64),
    Reference(String), // var(otherToken)
}

/// Style definition (mixin): `public style buttonBase { ... }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleDefinition {
    pub is_public: bool,
    pub name: Spanned<String>,
    pub declarations: Vec<Spanned<StyleDeclaration>>,
}

/// Component definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub is_public: bool,
    pub name: Spanned<String>,
    pub doc_comment: Option<DocComment>,
    pub variants: Vec<Spanned<Variant>>,
    pub render: Option<Spanned<RenderNode>>,
}

/// Doc comment with metadata like @frame
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocComment {
    pub content: String,
    pub frame: Option<FrameAnnotation>,
    pub samples: Vec<SampleData>,
}

/// @frame(x: 100, y: 50, width: 320, height: 480)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameAnnotation {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: Option<f64>,
}

/// @sample for preview data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SampleData {
    pub name: String,
    pub data: String, // JSON string
}

/// Variant definition: `variant hover trigger { ":hover" }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub name: Spanned<String>,
    pub triggers: Vec<Spanned<String>>,
}

/// Render tree node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderNode {
    Element(Element),
    Text(TextNode),
    Slot(Slot),
    Insert(Insert),
    Condition(Condition),
    Repeat(Repeat),
    ComponentInstance(ComponentInstance),
}

/// Element node: `div { ... }` or `button { ... }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub tag: Spanned<String>,
    pub attributes: Vec<Spanned<Attribute>>,
    pub styles: Vec<Spanned<StyleBlock>>,
    pub children: Vec<Spanned<RenderNode>>,
}

/// Text node: `text "Hello"` or `text "Hello {name}"`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextNode {
    pub parts: Vec<TextPart>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextPart {
    Literal(String),
    Expression(Expression),
}

/// Slot definition: `slot children { ... }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    pub name: Spanned<String>,
    pub default: Vec<Spanned<RenderNode>>,
}

/// Slot insertion: `insert slotName { ... }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Insert {
    pub slot_name: Spanned<String>,
    pub children: Vec<Spanned<RenderNode>>,
}

/// Conditional: `if condition { ... }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub condition: Spanned<Expression>,
    pub then_branch: Vec<Spanned<RenderNode>>,
    pub else_branch: Option<Vec<Spanned<RenderNode>>>,
}

/// Repeat/loop: `repeat items as item { ... }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repeat {
    pub source: Spanned<Expression>,
    pub iterator: Spanned<String>,
    pub index: Option<Spanned<String>>,
    pub body: Vec<Spanned<RenderNode>>,
    pub empty: Option<Vec<Spanned<RenderNode>>>,
}

/// Component instance: `Button(label="Save")`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentInstance {
    pub name: Spanned<String>,
    pub namespace: Option<Spanned<String>>, // For imported: `theme.Button`
    pub props: Vec<Spanned<Prop>>,
    pub inserts: Vec<Spanned<Insert>>,
}

/// Attribute on an element: `class="primary"` or `onClick={handler}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: Spanned<String>,
    pub value: Spanned<AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Expression(Expression),
    Boolean(bool),
}

/// Prop on a component instance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prop {
    pub name: Spanned<String>,
    pub value: Spanned<Expression>,
}

/// Style block within an element
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleBlock {
    pub extends: Vec<Spanned<String>>,
    pub variant: Option<Spanned<String>>,
    pub declarations: Vec<Spanned<StyleDeclaration>>,
}

/// CSS declaration: `background: #fff`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleDeclaration {
    pub property: Spanned<String>,
    pub value: Spanned<StyleValue>,
}

/// CSS value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StyleValue {
    Keyword(String),
    Color(String),
    Dimension(f64, String),
    Number(f64),
    String(String),
    Function(String, Vec<StyleValue>),
    Reference(String), // var(tokenName)
    List(Vec<StyleValue>),
}

/// Expression (formula-like, no control flow)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Identifier(String),
    Number(f64),
    String(String),
    Boolean(bool),
    MemberAccess(Box<Spanned<Expression>>, Spanned<String>),
    FunctionCall(Spanned<String>, Vec<Spanned<Expression>>),
    BinaryOp(Box<Spanned<Expression>>, BinaryOperator, Box<Spanned<Expression>>),
    UnaryOp(UnaryOperator, Box<Spanned<Expression>>),
    Ternary(Box<Spanned<Expression>>, Box<Spanned<Expression>>, Box<Spanned<Expression>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Neg,
}
