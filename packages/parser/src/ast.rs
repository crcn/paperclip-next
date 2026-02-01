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
    /// Top-level render elements (text, div, etc.)
    pub renders: Vec<Element>,
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
    pub overrides: Vec<Override>,
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

/// Override definition (target nested instances)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Override {
    /// Dot-separated path to target (e.g., ["Button", "Icon"])
    pub path: Vec<String>,

    /// Styles to apply
    pub styles: Vec<StyleBlock>,

    /// Attributes to override
    pub attributes: HashMap<String, Expression>,

    /// Source location
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
    Text {
        content: Expression,
        styles: Vec<StyleBlock>,
        span: Span,
    },

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
            renders: Vec::new(),
        }
    }

    /// Find an element by its span ID
    pub fn find_element(&self, id: &str) -> Option<&Element> {
        // Search in components
        for component in &self.components {
            if let Some(body) = &component.body {
                if let Some(elem) = Self::find_element_recursive(body, id) {
                    return Some(elem);
                }
            }
        }
        // Search in top-level renders
        for render in &self.renders {
            if let Some(elem) = Self::find_element_recursive(render, id) {
                return Some(elem);
            }
        }
        None
    }

    /// Find an element by ID (mutable)
    pub fn find_element_mut(&mut self, id: &str) -> Option<&mut Element> {
        // Search in components
        for component in &mut self.components {
            if let Some(body) = &mut component.body {
                if let Some(elem) = Self::find_element_recursive_mut(body, id) {
                    return Some(elem);
                }
            }
        }
        // Search in top-level renders
        for render in &mut self.renders {
            if let Some(elem) = Self::find_element_recursive_mut(render, id) {
                return Some(elem);
            }
        }
        None
    }

    fn find_element_recursive<'a>(elem: &'a Element, id: &str) -> Option<&'a Element> {
        if elem.span().id == id {
            return Some(elem);
        }

        match elem {
            Element::Tag { children, .. } => {
                for child in children {
                    if let Some(found) = Self::find_element_recursive(child, id) {
                        return Some(found);
                    }
                }
            }
            Element::Instance { children, .. } => {
                for child in children {
                    if let Some(found) = Self::find_element_recursive(child, id) {
                        return Some(found);
                    }
                }
            }
            Element::Conditional {
                then_branch,
                else_branch,
                ..
            } => {
                for child in then_branch {
                    if let Some(found) = Self::find_element_recursive(child, id) {
                        return Some(found);
                    }
                }
                if let Some(else_elems) = else_branch {
                    for child in else_elems {
                        if let Some(found) = Self::find_element_recursive(child, id) {
                            return Some(found);
                        }
                    }
                }
            }
            Element::Repeat { body, .. } => {
                for child in body {
                    if let Some(found) = Self::find_element_recursive(child, id) {
                        return Some(found);
                    }
                }
            }
            Element::Insert { content, .. } => {
                for child in content {
                    if let Some(found) = Self::find_element_recursive(child, id) {
                        return Some(found);
                    }
                }
            }
            Element::Text { .. } | Element::SlotInsert { .. } => {}
        }

        None
    }

    fn find_element_recursive_mut<'a>(elem: &'a mut Element, id: &str) -> Option<&'a mut Element> {
        if elem.span().id == id {
            return Some(elem);
        }

        match elem {
            Element::Tag { children, .. } => {
                for child in children {
                    if let Some(found) = Self::find_element_recursive_mut(child, id) {
                        return Some(found);
                    }
                }
            }
            Element::Instance { children, .. } => {
                for child in children {
                    if let Some(found) = Self::find_element_recursive_mut(child, id) {
                        return Some(found);
                    }
                }
            }
            Element::Conditional {
                then_branch,
                else_branch,
                ..
            } => {
                for child in then_branch {
                    if let Some(found) = Self::find_element_recursive_mut(child, id) {
                        return Some(found);
                    }
                }
                if let Some(else_elems) = else_branch {
                    for child in else_elems {
                        if let Some(found) = Self::find_element_recursive_mut(child, id) {
                            return Some(found);
                        }
                    }
                }
            }
            Element::Repeat { body, .. } => {
                for child in body {
                    if let Some(found) = Self::find_element_recursive_mut(child, id) {
                        return Some(found);
                    }
                }
            }
            Element::Insert { content, .. } => {
                for child in content {
                    if let Some(found) = Self::find_element_recursive_mut(child, id) {
                        return Some(found);
                    }
                }
            }
            Element::Text { .. } | Element::SlotInsert { .. } => {}
        }

        None
    }

    /// Check if an element is inside a repeat template
    pub fn is_in_repeat_template(&self, id: &str) -> bool {
        for component in &self.components {
            if let Some(body) = &component.body {
                if Self::is_in_repeat_recursive(body, id, false) {
                    return true;
                }
            }
        }
        false
    }

    fn is_in_repeat_recursive(elem: &Element, target_id: &str, in_repeat: bool) -> bool {
        if elem.span().id == target_id {
            return in_repeat;
        }

        match elem {
            Element::Repeat { body, .. } => {
                // Inside a repeat template
                for child in body {
                    if Self::is_in_repeat_recursive(child, target_id, true) {
                        return true;
                    }
                }
            }
            Element::Tag { children, .. } | Element::Instance { children, .. } => {
                for child in children {
                    if Self::is_in_repeat_recursive(child, target_id, in_repeat) {
                        return true;
                    }
                }
            }
            Element::Conditional {
                then_branch,
                else_branch,
                ..
            } => {
                for child in then_branch {
                    if Self::is_in_repeat_recursive(child, target_id, in_repeat) {
                        return true;
                    }
                }
                if let Some(else_elems) = else_branch {
                    for child in else_elems {
                        if Self::is_in_repeat_recursive(child, target_id, in_repeat) {
                            return true;
                        }
                    }
                }
            }
            Element::Insert { content, .. } => {
                for child in content {
                    if Self::is_in_repeat_recursive(child, target_id, in_repeat) {
                        return true;
                    }
                }
            }
            Element::Text { .. } | Element::SlotInsert { .. } => {}
        }

        false
    }

    /// Check if moving node to parent would create a cycle
    pub fn would_create_cycle(&self, node_id: &str, parent_id: &str) -> bool {
        // A cycle occurs if parent is a descendant of node
        self.is_descendant_of(parent_id, node_id)
    }

    /// Check if potential_descendant is a descendant of ancestor
    fn is_descendant_of(&self, potential_descendant: &str, ancestor: &str) -> bool {
        if potential_descendant == ancestor {
            return true;
        }

        if let Some(ancestor_elem) = self.find_element(ancestor) {
            return Self::contains_element(ancestor_elem, potential_descendant);
        }

        false
    }

    fn contains_element(elem: &Element, target_id: &str) -> bool {
        if elem.span().id == target_id {
            return true;
        }

        match elem {
            Element::Tag { children, .. } | Element::Instance { children, .. } => children
                .iter()
                .any(|child| Self::contains_element(child, target_id)),
            Element::Conditional {
                then_branch,
                else_branch,
                ..
            } => {
                then_branch
                    .iter()
                    .any(|child| Self::contains_element(child, target_id))
                    || else_branch.as_ref().map_or(false, |els| {
                        els.iter()
                            .any(|child| Self::contains_element(child, target_id))
                    })
            }
            Element::Repeat { body, .. } => body
                .iter()
                .any(|child| Self::contains_element(child, target_id)),
            Element::Insert { content, .. } => content
                .iter()
                .any(|child| Self::contains_element(child, target_id)),
            Element::Text { .. } | Element::SlotInsert { .. } => false,
        }
    }

    /// Check if child can be validly placed inside parent
    pub fn is_valid_parent_child(&self, _parent: &Element, _child: &Element) -> bool {
        // For now, allow all parent-child relationships
        // Can add specific rules later (e.g., no text in certain elements)
        true
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Element {
    /// Get the span of this element
    pub fn span(&self) -> &Span {
        match self {
            Element::Tag { span, .. } => span,
            Element::Text { span, .. } => span,
            Element::Instance { span, .. } => span,
            Element::Conditional { span, .. } => span,
            Element::Repeat { span, .. } => span,
            Element::SlotInsert { span, .. } => span,
            Element::Insert { span, .. } => span,
        }
    }

    /// Get mutable span
    pub fn span_mut(&mut self) -> &mut Span {
        match self {
            Element::Tag { span, .. } => span,
            Element::Text { span, .. } => span,
            Element::Instance { span, .. } => span,
            Element::Conditional { span, .. } => span,
            Element::Repeat { span, .. } => span,
            Element::SlotInsert { span, .. } => span,
            Element::Insert { span, .. } => span,
        }
    }

    /// Get children if this is a container element
    pub fn children(&self) -> Option<&Vec<Element>> {
        match self {
            Element::Tag { children, .. } => Some(children),
            Element::Instance { children, .. } => Some(children),
            _ => None,
        }
    }

    /// Get mutable children if this is a container element
    pub fn children_mut(&mut self) -> Option<&mut Vec<Element>> {
        match self {
            Element::Tag { children, .. } => Some(children),
            Element::Instance { children, .. } => Some(children),
            _ => None,
        }
    }

    /// Get tag name if this is a Tag element
    pub fn tag_name(&self) -> Option<&str> {
        match self {
            Element::Tag { tag_name, .. } => Some(tag_name),
            _ => None,
        }
    }
}
