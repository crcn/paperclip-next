//! # Paperclip Evaluator
//!
//! Evaluates Paperclip AST to Virtual DOM and CSS.
//!
//! ## Purpose
//!
//! The evaluator transforms parsed Paperclip documents into virtual DOM trees and CSS rules.
//! It handles component instantiation, prop passing, slot composition, conditionals, and
//! repeat blocks.
//!
//! ## Determinism Contract
//!
//! **INVARIANT: Evaluation is fully deterministic.**
//!
//! For any Document + EvalContext state, `evaluate()` MUST produce identical output on every invocation:
//!
//! - Same AST → Same VDOM structure (byte-for-byte identical)
//! - Same AST → Same semantic IDs
//! - Same AST → Same CSS output
//! - No HashMap iteration order leaks (Rust's SipHash provides deterministic iteration)
//! - No non-deterministic ID generation (CRC32 is deterministic, counters reset per-eval)
//! - No time/random/environment dependence
//! - No floating-point non-associativity issues
//!
//! **Why determinism matters:**
//! - **Diffing correctness**: Client-side differ assumes deterministic VDOMs
//! - **Caching**: Memoization requires pure functions
//! - **Collaboration**: CRDT merges require deterministic resolution
//! - **Reproducibility**: AI tools need predictable outputs
//! - **Testing**: Snapshot tests break with non-determinism
//!
//! **Tests**: See `tests/test_determinism.rs` for validation.
//!
//! ## Recursion Protection
//!
//! Component recursion is detected and prevented via component stack tracking.
//! See `EvalContext::component_stack` and the cycle detection at component instantiation.
//!
//! **Structural recursion** (direct or indirect cycles) is always an error:
//! ```paperclip
//! component A { render A() }  // Error: recursive component
//! ```
//!
//! **Data-driven recursion** (bounded by data structures) is allowed:
//! ```paperclip
//! component TreeNode {
//!   render div {
//!     text node.label
//!     repeat child in node.children {
//!       TreeNode(node=child)  // OK: bounded by data
//!     }
//!   }
//! }
//! ```
//!
//! ## Error Recovery Boundaries
//!
//! Error recovery is **only allowed at expression and leaf-node boundaries**, not at structural boundaries:
//!
//! **✅ Recoverable** (expression/leaf boundary):
//! - `text { user.invalid.property }` → VNode::Error
//! - `style { color: bad.value }` → Style-level error or VNode::Error
//!
//! **❌ NOT recoverable** (structural boundary):
//! - `SomeComponent()` → Component resolution failure is FATAL
//! - `slot unknownSlot` → Slot resolution failure is FATAL
//!
//! **Why**: Component instantiation failure means no context to safely continue.
//! Half-constructed trees break semantic guarantees.
//!
//! ## Semantic Identity System
//!
//! Every VNode has a `semantic_id` that uniquely identifies it within the VDOM tree.
//! Semantic IDs are stable across refactoring (rename component, move elements, etc.).
//!
//! **Uniqueness scope**: Per-VDOM-tree (evaluation output), NOT per-source-file.
//! See `packages/semantics/src/identity.rs` for full details.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use paperclip_evaluator::Evaluator;
//! use paperclip_parser::parse;
//!
//! let source = "component Button { render button { text \"Click\" } }";
//! let doc = parse(source)?;
//!
//! let mut evaluator = Evaluator::new();
//! let vdom = evaluator.evaluate(&doc)?;
//! ```

use crate::css_evaluator::CssEvaluator;
use crate::css_optimizer::optimize_css_rules;
use crate::css_minifier::minify_css_rules;
use crate::utils::get_style_namespace;
use crate::vdom::{CssRule, VNode, VirtualDomDocument};
use paperclip_bundle::Bundle;
use paperclip_parser::ast::*;
use paperclip_semantics::{SemanticID, SemanticSegment};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};

pub type EvalResult<T> = Result<T, EvalError>;

/// Check if a name is a known HTML tag
fn is_html_tag(name: &str) -> bool {
    matches!(
        name,
        // Common HTML tags
        "a" | "abbr" | "address" | "area" | "article" | "aside" | "audio" |
        "b" | "base" | "bdi" | "bdo" | "blockquote" | "body" | "br" | "button" |
        "canvas" | "caption" | "cite" | "code" | "col" | "colgroup" |
        "data" | "datalist" | "dd" | "del" | "details" | "dfn" | "dialog" | "div" | "dl" | "dt" |
        "em" | "embed" |
        "fieldset" | "figcaption" | "figure" | "footer" | "form" |
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "head" | "header" | "hgroup" | "hr" | "html" |
        "i" | "iframe" | "img" | "input" | "ins" |
        "kbd" |
        "label" | "legend" | "li" | "link" |
        "main" | "map" | "mark" | "menu" | "meta" | "meter" |
        "nav" | "noscript" |
        "object" | "ol" | "optgroup" | "option" | "output" |
        "p" | "picture" | "pre" | "progress" |
        "q" |
        "rp" | "rt" | "ruby" |
        "s" | "samp" | "script" | "search" | "section" | "select" | "slot" | "small" | "source" | "span" | "strong" | "style" | "sub" | "summary" | "sup" | "svg" |
        "table" | "tbody" | "td" | "template" | "textarea" | "tfoot" | "th" | "thead" | "time" | "title" | "tr" | "track" |
        "u" | "ul" |
        "var" | "video" |
        "wbr"
    )
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Component '{name}' not found at {span:?}")]
    ComponentNotFound { name: String, span: Span },

    #[error("Variable '{name}' not found at {span:?}")]
    VariableNotFound { name: String, span: Span },

    #[error("Division by zero at {span:?}")]
    DivisionByZero { span: Span },

    #[error("Invalid operands for operator {operator} at {span:?}: {details}")]
    InvalidOperands {
        operator: String,
        details: String,
        span: Span,
    },

    #[error("Type error at {span:?}: {message}")]
    TypeError { message: String, span: Span },

    #[error("Evaluation error at {span:?}: {message}")]
    EvaluationError { message: String, span: Span },

    #[error("Recursive component detected: {component}\nCall stack: {}\n{}", call_stack.join(" → "), hint.as_ref().unwrap_or(&String::new()))]
    RecursiveComponent {
        component: String,
        call_stack: Vec<String>,
        hint: Option<String>,
    },
}

/// Context for evaluation
#[derive(Clone)]
pub struct EvalContext {
    components: HashMap<String, Component>,
    tokens: HashMap<String, String>,
    variables: HashMap<String, Value>,
    current_component: Option<String>,
    document_id: String,
    /// Semantic path - tracks current position in component tree for building semantic IDs
    semantic_path: Vec<SemanticSegment>,
    /// Component instance key counters for auto-generating keys
    component_key_counters: HashMap<String, usize>,
    /// Slot content - maps slot name to inserted content
    slot_content: HashMap<String, Vec<Element>>,
    /// Component call stack for cycle detection (prevents infinite recursion)
    component_stack: Vec<String>,
}

impl EvalContext {
    pub fn new(document_id: String) -> Self {
        Self {
            components: HashMap::new(),
            tokens: HashMap::new(),
            variables: HashMap::new(),
            current_component: None,
            document_id,
            semantic_path: Vec::new(),
            component_key_counters: HashMap::new(),
            slot_content: HashMap::new(),
            component_stack: Vec::new(),
        }
    }

    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.insert(component.name.clone(), component);
    }

    pub fn add_token(&mut self, name: String, value: String) {
        self.tokens.insert(name, value);
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Get current semantic ID from the path
    pub fn get_semantic_id(&self) -> SemanticID {
        SemanticID::new(self.semantic_path.clone())
    }

    /// Push a segment onto the semantic path
    pub fn push_segment(&mut self, segment: SemanticSegment) {
        self.semantic_path.push(segment);
    }

    /// Pop a segment from the semantic path
    pub fn pop_segment(&mut self) {
        self.semantic_path.pop();
    }

    /// Generate an auto-key for a component instance
    pub fn generate_component_key(&mut self, component_name: &str) -> String {
        let counter = self
            .component_key_counters
            .entry(component_name.to_string())
            .or_insert(0);
        let key = format!("{}-{}", component_name, counter);
        *counter += 1;
        key
    }
}

/// Runtime value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => String::new(),
            Value::Array(_) | Value::Object(_) => format!("{:?}", self),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::String(s) => !s.is_empty(),
            Value::Number(n) => *n != 0.0,
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        }
    }
}

/// Evaluator
pub struct Evaluator {
    pub context: EvalContext,
}

impl Evaluator {
    pub fn new() -> Self {
        Self::with_document_id("<anonymous>")
    }

    pub fn with_document_id(path: &str) -> Self {
        let document_id = paperclip_parser::get_document_id(path);
        Self {
            context: EvalContext::new(document_id),
        }
    }

    /// Evaluate a document to virtual DOM
    #[instrument(skip(self, doc), fields(components = doc.components.len(), tokens = doc.tokens.len()))]
    pub fn evaluate(&mut self, doc: &Document) -> EvalResult<VirtualDomDocument> {
        info!("Starting document evaluation");

        // Register tokens
        for token in &doc.tokens {
            debug!(token_name = %token.name, token_value = %token.value, "Registering token");
            self.context
                .add_token(token.name.clone(), token.value.clone());
        }

        // Register components
        for component in &doc.components {
            debug!(component_name = %component.name, public = component.public, "Registering component");
            self.context.add_component(component.clone());
        }

        let mut vdoc = VirtualDomDocument::new();

        // Evaluate ALL components for preview rendering.
        // The `public` keyword only affects cross-file imports, not preview.
        // Designers need to see all components while working.
        info!(
            total_components = doc.components.len(),
            "Evaluating all components for preview"
        );

        for component in &doc.components {
            debug!(component_name = %component.name, public = component.public, "Evaluating component");
            let vnode = self.evaluate_component(&component.name)?;
            vdoc.add_node(vnode);
        }

        // Evaluate top-level renders
        for render in &doc.renders {
            debug!("Evaluating top-level render element");
            let vnode = self.evaluate_element(render)?;
            vdoc.add_node(vnode);
        }

        // Evaluate CSS
        debug!("Starting CSS evaluation");
        let mut css_evaluator = CssEvaluator::with_document_id(&self.context.document_id);
        match css_evaluator.evaluate(doc) {
            Ok(css_doc) => {
                let original_count = css_doc.rules.len();
                debug!(rules = original_count, "CSS evaluation succeeded");

                // Convert CSS evaluator rules to VDOM rules
                let mut css_rules = Vec::new();
                for css_rule in css_doc.rules {
                    css_rules.push(CssRule {
                        selector: css_rule.selector,
                        properties: css_rule.properties,
                        media_query: css_rule.media_query,
                    });
                }

                // Optimize CSS rules (deduplicate, merge)
                debug!(before = original_count, "Optimizing CSS rules");
                css_rules = optimize_css_rules(css_rules);
                let optimized_count = css_rules.len();
                debug!(
                    after = optimized_count,
                    saved = original_count - optimized_count,
                    "CSS optimization complete"
                );

                // Minify CSS (remove whitespace, shorten values)
                minify_css_rules(&mut css_rules);
                debug!("CSS minification complete");

                vdoc.styles = css_rules;
            }
            Err(e) => {
                warn!(error = %e, "CSS evaluation failed - continuing without styles");
                // Don't fail the entire evaluation if CSS fails
            }
        }

        info!(nodes = vdoc.nodes.len(), styles = vdoc.styles.len(), "Document evaluation complete");
        Ok(vdoc)
    }

    /// Evaluate a bundle to virtual DOM (supports cross-file imports)
    #[instrument(skip(self, bundle), fields(entry = %entry_path.display()))]
    pub fn evaluate_bundle(
        &mut self,
        bundle: &Bundle,
        entry_path: &Path,
    ) -> EvalResult<VirtualDomDocument> {
        info!("Starting bundle DOM evaluation");

        // Get entry document
        let entry_doc = bundle.get_document(entry_path).ok_or_else(|| {
            error!("Entry document not found");
            EvalError::EvaluationError {
                message: format!("Entry document not found: {}", entry_path.display()),
                span: Span::new(0, 0, "error".to_string()),
            }
        })?;

        // Update document_id for the entry file
        if let Some(doc_id) = bundle.get_document_id(entry_path) {
            self.context.document_id = doc_id.to_string();
        }

        // Register tokens from entry file
        for token in &entry_doc.tokens {
            debug!(token_name = %token.name, token_value = %token.value, "Registering token");
            self.context
                .add_token(token.name.clone(), token.value.clone());
        }

        // Register tokens from imported files
        if let Some(deps) = bundle.get_dependencies(entry_path) {
            for dep_path in deps {
                if let Some(dep_doc) = bundle.get_document(dep_path) {
                    for token in &dep_doc.tokens {
                        if token.public {
                            debug!(token_name = %token.name, from_file = %dep_path.display(), "Registering imported token");
                            self.context
                                .add_token(token.name.clone(), token.value.clone());
                        }
                    }
                }
            }
        }

        // Register components from entry file
        for component in &entry_doc.components {
            debug!(component_name = %component.name, public = component.public, "Registering component");
            self.context.add_component(component.clone());
        }

        // Register components from imported files
        if let Some(deps) = bundle.get_dependencies(entry_path) {
            for dep_path in deps {
                if let Some(dep_doc) = bundle.get_document(dep_path) {
                    for component in &dep_doc.components {
                        if component.public {
                            debug!(component_name = %component.name, from_file = %dep_path.display(), "Registering imported component");
                            self.context.add_component(component.clone());
                        }
                    }
                }
            }
        }

        let mut vdoc = VirtualDomDocument::new();

        // Evaluate ALL components from entry file for preview rendering.
        // The `public` keyword only affects cross-file imports, not preview.
        info!(
            total_components = entry_doc.components.len(),
            "Evaluating all components for preview"
        );

        for component in &entry_doc.components {
            debug!(component_name = %component.name, public = component.public, "Evaluating component");
            let vnode = self.evaluate_component(&component.name)?;
            vdoc.add_node(vnode);
        }

        info!(nodes = vdoc.nodes.len(), "Bundle DOM evaluation complete");
        Ok(vdoc)
    }

    /// Evaluate a component by name (for top-level public components)
    fn evaluate_component(&mut self, name: &str) -> EvalResult<VNode> {
        // Generate key and push component segment for top-level component
        let component_key = self.context.generate_component_key(name);
        self.context.push_segment(SemanticSegment::Component {
            name: name.to_string(),
            key: Some(component_key),
        });

        let result = self.evaluate_component_with_props(name, &HashMap::new());

        // Pop component segment
        self.context.pop_segment();

        result
    }

    /// Evaluate a component with props
    /// NOTE: Caller is responsible for pushing/popping the component segment
    #[instrument(skip(self, props), fields(component_name = name, prop_count = props.len()))]
    fn evaluate_component_with_props(
        &mut self,
        name: &str,
        props: &HashMap<String, Value>,
    ) -> EvalResult<VNode> {
        self.evaluate_component_with_props_and_children(name, props, &[])
    }

    /// Evaluate a component with props and slot children
    /// NOTE: Caller is responsible for pushing/popping the component segment
    #[instrument(skip(self, props, children), fields(component_name = name, prop_count = props.len(), child_count = children.len()))]
    fn evaluate_component_with_props_and_children(
        &mut self,
        name: &str,
        props: &HashMap<String, Value>,
        children: &[Element],
    ) -> EvalResult<VNode> {
        debug!(
            component = name,
            stack = ?self.context.component_stack,
            "Evaluating component with props and children"
        );

        // Check for circular component dependency
        if self.context.component_stack.contains(&name.to_string()) {
            // Build full call stack for error message
            let mut call_stack = self.context.component_stack.clone();
            call_stack.push(name.to_string());

            // Generate helpful hint based on the cycle
            let hint = if call_stack.len() == 2 && call_stack[0] == name {
                // Direct recursion (A → A)
                Some(format!(
                    "Component '{}' renders itself unconditionally. Add a conditional (if), a repeat loop with different data, or ensure recursion is data-dependent.",
                    name
                ))
            } else {
                // Indirect recursion (A → B → A)
                Some(format!(
                    "Component cycle detected. Ensure components do not form circular dependencies."
                ))
            };

            error!(
                component = name,
                stack = ?call_stack,
                "Circular component dependency detected"
            );

            return Err(EvalError::RecursiveComponent {
                component: name.to_string(),
                call_stack,
                hint,
            });
        }

        let component = self.context.components.get(name).ok_or_else(|| {
            error!(component_name = name, "Component not found");
            EvalError::ComponentNotFound {
                name: name.to_string(),
                span: Span::new(0, 0, "error".to_string()), // TODO: Pass span from call site
            }
        })?;

        // Clone component body to avoid borrow checker issues
        let component_body = component.body.clone();

        // Push component to call stack BEFORE cloning context
        // This ensures the cloned context includes the updated stack
        self.context.component_stack.push(name.to_string());

        // Create a new context scope with props as variables
        let mut scoped_evaluator = Evaluator {
            context: self.context.clone(),
        };

        // Set current component for class name scoping
        scoped_evaluator.context.current_component = Some(name.to_string());

        // Bind props to variables
        for (key, value) in props {
            scoped_evaluator
                .context
                .set_variable(key.clone(), value.clone());
        }

        // Register slot content
        // If children are provided, they become the default slot content
        // Named slots can be extracted from children if they have slot attributes
        if !children.is_empty() {
            // For now, all children go to the default "children" slot
            scoped_evaluator
                .context
                .slot_content
                .insert("children".to_string(), children.to_vec());
        }

        let result = if let Some(body) = &component_body {
            scoped_evaluator.evaluate_element(body)
        } else {
            // Empty component - return empty div with semantic ID
            let semantic_id = scoped_evaluator.context.get_semantic_id();
            Ok(VNode::element("div", semantic_id))
        };

        // Pop component from call stack in SELF context (not scoped)
        self.context.component_stack.pop();

        result
    }

    /// Evaluate an element
    fn evaluate_element(&mut self, element: &Element) -> EvalResult<VNode> {
        match element {
            Element::Tag {
                tag_name,
                name: _element_name,
                attributes,
                styles,
                children,
                span,
            } => {
                // Extract role before pushing segment
                let role = attributes
                    .get("data-role")
                    .and_then(|expr| match expr {
                        Expression::Literal { value, .. } => Some(value.clone()),
                        _ => None,
                    })
                    .or_else(|| {
                        // Fallback to first class name
                        attributes.get("class").and_then(|expr| match expr {
                            Expression::Literal { value, .. } => {
                                value.split_whitespace().next().map(String::from)
                            }
                            _ => None,
                        })
                    });

                // Push element segment
                self.context.push_segment(SemanticSegment::Element {
                    tag: tag_name.clone(),
                    role,
                    ast_id: span.id.clone(),
                });

                // Build semantic ID from current context (includes this element)
                let semantic_id = self.context.get_semantic_id();

                let mut vnode = VNode::element(tag_name, semantic_id);

                // Generate and apply class name for CSS synchronization
                let class_name = get_style_namespace(
                    Some(tag_name.as_str()),
                    &span.id,
                    self.context.current_component.as_deref(),
                );

                // Check if attributes contain a class
                let mut has_class = false;

                // Evaluate attributes
                for (key, expr) in attributes {
                    match self.evaluate_expression(expr) {
                        Ok(value) => {
                            // Merge with generated class name if this is the class attribute
                            if key == "class" {
                                let merged_class = format!("{} {}", class_name, value.to_string());
                                vnode = vnode.with_attr(key, merged_class);
                                has_class = true;
                            } else {
                                vnode = vnode.with_attr(key, value.to_string());
                            }
                        }
                        Err(err) => {
                            // In dev mode, set attribute to error message
                            warn!(attribute = key, error = %err, "Expression evaluation failed in attribute");
                            vnode = vnode.with_attr(key, format!("[Error: {}]", err));
                        }
                    }
                }

                // If no class attribute was set, add the generated class name
                if !has_class {
                    vnode = vnode.with_attr("class", class_name);
                }

                // Evaluate styles
                for style_block in styles {
                    for (key, value) in &style_block.properties {
                        vnode = vnode.with_style(key, value);
                    }
                }

                // Evaluate children
                for child in children {
                    match self.evaluate_element(child) {
                        Ok(child_vnode) => {
                            vnode = vnode.with_child(child_vnode);
                        }
                        Err(err) => {
                            // RecursiveComponent errors are fatal - propagate immediately
                            if matches!(err, EvalError::RecursiveComponent { .. }) {
                                return Err(err);
                            }

                            // Other errors: In dev mode, add error node for failed child
                            warn!(error = %err, "Child element evaluation failed");
                            let span = match child {
                                Element::Tag { span, .. } => Some(span.clone()),
                                Element::Text { span, .. } => Some(span.clone()),
                                Element::Instance { span, .. } => Some(span.clone()),
                                Element::Conditional { span, .. } => Some(span.clone()),
                                Element::Repeat { span, .. } => Some(span.clone()),
                                Element::SlotInsert { span, .. } => Some(span.clone()),
                                Element::Insert { span, .. } => Some(span.clone()),
                            };
                            let semantic_id = self.context.get_semantic_id();
                            vnode = vnode.with_child(VNode::error(
                                format!("Error: {}", err),
                                span,
                                semantic_id,
                            ));
                        }
                    }
                }

                // Pop element segment
                self.context.pop_segment();

                Ok(vnode)
            }

            Element::Text {
                content,
                styles,
                span,
            } => {
                match self.evaluate_expression(content) {
                    Ok(value) => {
                        let text_value = value.to_string();
                        if styles.is_empty() {
                            Ok(VNode::text(text_value))
                        } else {
                            // Wrap text in span when it has styles
                            let semantic_id = self.context.get_semantic_id();
                            let mut vnode = VNode::element("span", semantic_id);
                            for style_block in styles {
                                for (key, value) in &style_block.properties {
                                    vnode = vnode.with_style(key, value);
                                }
                            }
                            vnode = vnode.with_child(VNode::text(text_value));
                            Ok(vnode)
                        }
                    }
                    Err(err) => {
                        // In dev mode, show error inline instead of crashing
                        warn!(error = %err, "Expression evaluation failed in text node");
                        let semantic_id = self.context.get_semantic_id();
                        Ok(VNode::error(
                            format!("Error: {}", err),
                            Some(span.clone()),
                            semantic_id,
                        ))
                    }
                }
            }

            Element::Instance {
                name,
                props,
                children,
                span: _span,
            } => {
                // Check if this is an HTML tag (not a component)
                if is_html_tag(name) {
                    // Treat as HTML tag element
                    self.context.push_segment(SemanticSegment::Element {
                        tag: name.clone(),
                        role: None,
                        ast_id: format!("{}-html-tag", name),
                    });

                    let semantic_id = self.context.get_semantic_id();
                    let mut vnode = VNode::element(name, semantic_id);

                    // Evaluate props as attributes
                    for (key, expr) in props {
                        match self.evaluate_expression(expr) {
                            Ok(value) => {
                                vnode = vnode.with_attr(key, value.to_string());
                            }
                            Err(err) => {
                                warn!(attr = key, error = %err, "Attribute evaluation failed");
                            }
                        }
                    }

                    // Evaluate children
                    for child in children {
                        match self.evaluate_element(child) {
                            Ok(child_vnode) => {
                                vnode = vnode.with_child(child_vnode);
                            }
                            Err(err) => {
                                // RecursiveComponent errors are fatal - propagate immediately
                                if matches!(err, EvalError::RecursiveComponent { .. }) {
                                    return Err(err);
                                }

                                warn!(error = %err, "Child element evaluation failed");
                                let child_span = match child {
                                    Element::Tag { span, .. } => Some(span.clone()),
                                    Element::Text { span, .. } => Some(span.clone()),
                                    Element::Instance { span, .. } => Some(span.clone()),
                                    Element::Conditional { span, .. } => Some(span.clone()),
                                    Element::Repeat { span, .. } => Some(span.clone()),
                                    Element::SlotInsert { span, .. } => Some(span.clone()),
                                    Element::Insert { span, .. } => Some(span.clone()),
                                };
                                let error_id = self.context.get_semantic_id();
                                vnode = vnode.with_child(VNode::error(
                                    format!("Error: {}", err),
                                    child_span,
                                    error_id,
                                ));
                            }
                        }
                    }

                    self.context.pop_segment();
                    return Ok(vnode);
                }

                // Otherwise, treat as component instance
                // Extract or generate component key
                let key = props
                    .get("key")
                    .and_then(|expr| match expr {
                        Expression::Literal { value, .. } => Some(value.clone()),
                        _ => None,
                    })
                    .or_else(|| Some(self.context.generate_component_key(name)));

                // Push component segment
                self.context.push_segment(SemanticSegment::Component {
                    name: name.clone(),
                    key,
                });

                // Evaluate component with props
                // Props are evaluated in current context, then passed to component
                let mut evaluated_props = HashMap::new();
                for (key, expr) in props {
                    match self.evaluate_expression(expr) {
                        Ok(value) => {
                            evaluated_props.insert(key.clone(), value);
                        }
                        Err(err) => {
                            warn!(prop = key, error = %err, "Prop evaluation failed");
                            // Use null as fallback value for failed prop
                            evaluated_props.insert(key.clone(), Value::Null);
                        }
                    }
                }

                // Expand component - this returns the component's body with props applied
                // The result is pure DOM elements, not a Component VNode
                // Pass children as slot content
                let result = self.evaluate_component_with_props_and_children(
                    name,
                    &evaluated_props,
                    children,
                );

                // Pop component segment
                self.context.pop_segment();

                result
            }

            Element::Conditional {
                condition,
                then_branch,
                else_branch,
                span,
            } => {
                let condition_value = match self.evaluate_expression(condition) {
                    Ok(value) => value,
                    Err(err) => {
                        // If condition fails, emit error node
                        warn!(error = %err, "Conditional expression evaluation failed");
                        let semantic_id = self.context.get_semantic_id();
                        return Ok(VNode::error(
                            format!("Error in conditional: {}", err),
                            Some(span.clone()),
                            semantic_id,
                        ));
                    }
                };

                if condition_value.is_truthy() {
                    // Push ConditionalBranch segment for then branch
                    self.context
                        .push_segment(SemanticSegment::ConditionalBranch {
                            condition_id: span.id.clone(),
                            branch: paperclip_semantics::Branch::Then,
                        });

                    // Evaluate then branch (return first node, or wrapper if multiple)
                    let result = if then_branch.len() == 1 {
                        self.evaluate_element(&then_branch[0])
                    } else {
                        let semantic_id = self.context.get_semantic_id();
                        let mut wrapper = VNode::element("div", semantic_id);
                        for child in then_branch {
                            match self.evaluate_element(child) {
                                Ok(child_vnode) => {
                                    wrapper = wrapper.with_child(child_vnode);
                                }
                                Err(err) => {
                                    warn!(error = %err, "Then branch child evaluation failed");
                                    let child_span = match child {
                                        Element::Tag { span, .. } => Some(span.clone()),
                                        Element::Text { span, .. } => Some(span.clone()),
                                        Element::Instance { span, .. } => Some(span.clone()),
                                        Element::Conditional { span, .. } => Some(span.clone()),
                                        Element::Repeat { span, .. } => Some(span.clone()),
                                        Element::SlotInsert { span, .. } => Some(span.clone()),
                                        Element::Insert { span, .. } => Some(span.clone()),
                                    };
                                    let error_id = self.context.get_semantic_id();
                                    wrapper = wrapper.with_child(VNode::error(
                                        format!("Error: {}", err),
                                        child_span,
                                        error_id,
                                    ));
                                }
                            }
                        }
                        Ok(wrapper)
                    };

                    self.context.pop_segment();
                    result
                } else if let Some(else_branch) = else_branch {
                    // Push ConditionalBranch segment for else branch
                    self.context
                        .push_segment(SemanticSegment::ConditionalBranch {
                            condition_id: span.id.clone(),
                            branch: paperclip_semantics::Branch::Else,
                        });

                    let result = if else_branch.len() == 1 {
                        self.evaluate_element(&else_branch[0])
                    } else {
                        let semantic_id = self.context.get_semantic_id();
                        let mut wrapper = VNode::element("div", semantic_id);
                        for child in else_branch {
                            match self.evaluate_element(child) {
                                Ok(child_vnode) => {
                                    wrapper = wrapper.with_child(child_vnode);
                                }
                                Err(err) => {
                                    warn!(error = %err, "Else branch child evaluation failed");
                                    let child_span = match child {
                                        Element::Tag { span, .. } => Some(span.clone()),
                                        Element::Text { span, .. } => Some(span.clone()),
                                        Element::Instance { span, .. } => Some(span.clone()),
                                        Element::Conditional { span, .. } => Some(span.clone()),
                                        Element::Repeat { span, .. } => Some(span.clone()),
                                        Element::SlotInsert { span, .. } => Some(span.clone()),
                                        Element::Insert { span, .. } => Some(span.clone()),
                                    };
                                    let error_id = self.context.get_semantic_id();
                                    wrapper = wrapper.with_child(VNode::error(
                                        format!("Error: {}", err),
                                        child_span,
                                        error_id,
                                    ));
                                }
                            }
                        }
                        Ok(wrapper)
                    };

                    self.context.pop_segment();
                    result
                } else {
                    // Return empty comment
                    Ok(VNode::Comment {
                        content: "conditional false".to_string(),
                    })
                }
            }

            Element::Repeat {
                item_name,
                collection,
                body,
                span,
            } => {
                // For now, simplified: assume collection is an array variable
                let collection_value = match self.evaluate_expression(collection) {
                    Ok(value) => value,
                    Err(err) => {
                        // If collection fails, emit error node
                        warn!(error = %err, "Repeat collection evaluation failed");
                        let semantic_id = self.context.get_semantic_id();
                        return Ok(VNode::error(
                            format!("Error in repeat collection: {}", err),
                            Some(span.clone()),
                            semantic_id,
                        ));
                    }
                };

                // Push repeat wrapper semantic ID
                let semantic_id = self.context.get_semantic_id();
                let mut wrapper = VNode::element("div", semantic_id);

                if let Value::Array(items) = collection_value {
                    for (index, item) in items.iter().enumerate() {
                        // Check if first child has explicit key attribute
                        let explicit_key =
                            if let Some(Element::Tag { attributes, .. }) = body.first() {
                                attributes.get("key").and_then(|expr| {
                                    match self.evaluate_expression(expr) {
                                        Ok(Value::String(s)) => Some(s),
                                        Ok(Value::Number(n)) => Some(n.to_string()),
                                        Ok(other) => Some(other.to_string()),
                                        Err(_) => None,
                                    }
                                })
                            } else {
                                None
                            };

                        // Track whether we have an explicit key before consuming it
                        let has_explicit_key = explicit_key.is_some();

                        // Use explicit key if available, otherwise auto-generate
                        let item_key = explicit_key.unwrap_or_else(|| format!("item-{}", index));

                        // Push RepeatItem segment
                        self.context.push_segment(SemanticSegment::RepeatItem {
                            repeat_id: span.id.clone(),
                            key: item_key.clone(),
                        });

                        // Set item variable in context for this iteration
                        // Save old value if it exists (to restore later)
                        let old_value = self.context.variables.get(item_name).cloned();
                        self.context.set_variable(item_name.clone(), item.clone());

                        for (child_idx, child) in body.iter().enumerate() {
                            match self.evaluate_element(child) {
                                Ok(mut child_vnode) => {
                                    // Apply explicit key to first child if it was extracted from attributes
                                    if child_idx == 0 && has_explicit_key {
                                        child_vnode = child_vnode.with_key(item_key.clone());
                                    }
                                    wrapper = wrapper.with_child(child_vnode);
                                }
                                Err(err) => {
                                    warn!(error = %err, "Repeat body child evaluation failed");
                                    let child_span = match child {
                                        Element::Tag { span, .. } => Some(span.clone()),
                                        Element::Text { span, .. } => Some(span.clone()),
                                        Element::Instance { span, .. } => Some(span.clone()),
                                        Element::Conditional { span, .. } => Some(span.clone()),
                                        Element::Repeat { span, .. } => Some(span.clone()),
                                        Element::SlotInsert { span, .. } => Some(span.clone()),
                                        Element::Insert { span, .. } => Some(span.clone()),
                                    };
                                    let error_id = self.context.get_semantic_id();
                                    wrapper = wrapper.with_child(VNode::error(
                                        format!("Error: {}", err),
                                        child_span,
                                        error_id,
                                    ));
                                }
                            }
                        }

                        // Restore old value or remove variable
                        if let Some(old) = old_value {
                            self.context.set_variable(item_name.clone(), old);
                        } else {
                            self.context.variables.remove(item_name);
                        }

                        self.context.pop_segment();
                    }
                }

                Ok(wrapper)
            }

            Element::SlotInsert { name, span } => {
                // Get the component definition to access slot defaults
                let component_name = match self.context.current_component.as_ref() {
                    Some(name) => name,
                    None => {
                        warn!("SlotInsert outside of component context");
                        let semantic_id = self.context.get_semantic_id();
                        return Ok(VNode::error(
                            "Error: SlotInsert outside of component context".to_string(),
                            Some(span.clone()),
                            semantic_id,
                        ));
                    }
                };

                // Check if we have inserted content for this slot (clone to avoid borrow issues)
                let inserted_content = self.context.slot_content.get(name).cloned();

                if let Some(inserted_content) = inserted_content {
                    // Use inserted content
                    self.context.push_segment(SemanticSegment::Slot {
                        name: name.clone(),
                        variant: paperclip_semantics::SlotVariant::Inserted,
                    });

                    // If single child, return it directly
                    // If multiple children, wrap in fragment
                    let result = if inserted_content.len() == 1 {
                        self.evaluate_element(&inserted_content[0])
                    } else {
                        // Wrap multiple children in a div
                        let semantic_id = self.context.get_semantic_id();
                        let mut wrapper = VNode::element("div", semantic_id);
                        for child in &inserted_content {
                            match self.evaluate_element(child) {
                                Ok(child_vnode) => {
                                    wrapper = wrapper.with_child(child_vnode);
                                }
                                Err(err) => {
                                    warn!(error = %err, "Slot inserted content child evaluation failed");
                                    let child_span = match child {
                                        Element::Tag { span, .. } => Some(span.clone()),
                                        Element::Text { span, .. } => Some(span.clone()),
                                        Element::Instance { span, .. } => Some(span.clone()),
                                        Element::Conditional { span, .. } => Some(span.clone()),
                                        Element::Repeat { span, .. } => Some(span.clone()),
                                        Element::SlotInsert { span, .. } => Some(span.clone()),
                                        Element::Insert { span, .. } => Some(span.clone()),
                                    };
                                    let error_id = self.context.get_semantic_id();
                                    wrapper = wrapper.with_child(VNode::error(
                                        format!("Error: {}", err),
                                        child_span,
                                        error_id,
                                    ));
                                }
                            }
                        }
                        Ok(wrapper)
                    };

                    self.context.pop_segment();
                    result
                } else {
                    // Use default content from slot definition
                    let component = match self.context.components.get(component_name) {
                        Some(comp) => comp,
                        None => {
                            warn!(component = component_name, "Component not found for slot");
                            let semantic_id = self.context.get_semantic_id();
                            return Ok(VNode::error(
                                format!("Error: Component '{}' not found", component_name),
                                Some(span.clone()),
                                semantic_id,
                            ));
                        }
                    };

                    let slot = match component.slots.iter().find(|s| &s.name == name) {
                        Some(s) => s,
                        None => {
                            warn!(
                                slot_name = name,
                                component = component_name,
                                "Slot not found in component"
                            );
                            let semantic_id = self.context.get_semantic_id();
                            return Ok(VNode::error(
                                format!(
                                    "Error: Slot '{}' not found in component '{}'",
                                    name, component_name
                                ),
                                Some(span.clone()),
                                semantic_id,
                            ));
                        }
                    };

                    // Clone default content to avoid borrow issues
                    let default_content = slot.default_content.clone();

                    self.context.push_segment(SemanticSegment::Slot {
                        name: name.clone(),
                        variant: paperclip_semantics::SlotVariant::Default,
                    });

                    // If single default child, return it directly
                    // If multiple defaults, wrap in fragment
                    let result = if default_content.len() == 1 {
                        self.evaluate_element(&default_content[0])
                    } else if default_content.is_empty() {
                        // Empty slot - return comment
                        Ok(VNode::Comment {
                            content: format!("empty slot: {}", name),
                        })
                    } else {
                        // Wrap multiple children in a div
                        let semantic_id = self.context.get_semantic_id();
                        let mut wrapper = VNode::element("div", semantic_id);
                        for child in &default_content {
                            match self.evaluate_element(child) {
                                Ok(child_vnode) => {
                                    wrapper = wrapper.with_child(child_vnode);
                                }
                                Err(err) => {
                                    warn!(error = %err, "Slot default content child evaluation failed");
                                    let child_span = match child {
                                        Element::Tag { span, .. } => Some(span.clone()),
                                        Element::Text { span, .. } => Some(span.clone()),
                                        Element::Instance { span, .. } => Some(span.clone()),
                                        Element::Conditional { span, .. } => Some(span.clone()),
                                        Element::Repeat { span, .. } => Some(span.clone()),
                                        Element::SlotInsert { span, .. } => Some(span.clone()),
                                        Element::Insert { span, .. } => Some(span.clone()),
                                    };
                                    let error_id = self.context.get_semantic_id();
                                    wrapper = wrapper.with_child(VNode::error(
                                        format!("Error: {}", err),
                                        child_span,
                                        error_id,
                                    ));
                                }
                            }
                        }
                        Ok(wrapper)
                    };

                    self.context.pop_segment();
                    result
                }
            }

            Element::Insert {
                slot_name,
                content,
                span,
            } => {
                // Insert directive is used to explicitly provide slot content
                // This should typically be handled at the component instance level
                // For now, we'll evaluate the content as a fragment
                warn!("Insert directive evaluated directly (should be handled at instance level)");

                let semantic_id = self.context.get_semantic_id();
                let mut wrapper = VNode::element("div", semantic_id);

                for child in content {
                    match self.evaluate_element(child) {
                        Ok(child_vnode) => {
                            wrapper = wrapper.with_child(child_vnode);
                        }
                        Err(err) => {
                            warn!(error = %err, "Insert content child evaluation failed");
                            let child_span = match child {
                                Element::Tag { span, .. } => Some(span.clone()),
                                Element::Text { span, .. } => Some(span.clone()),
                                Element::Instance { span, .. } => Some(span.clone()),
                                Element::Conditional { span, .. } => Some(span.clone()),
                                Element::Repeat { span, .. } => Some(span.clone()),
                                Element::SlotInsert { span, .. } => Some(span.clone()),
                                Element::Insert { span, .. } => Some(span.clone()),
                            };
                            let error_id = self.context.get_semantic_id();
                            wrapper = wrapper.with_child(VNode::error(
                                format!("Error: {}", err),
                                child_span,
                                error_id,
                            ));
                        }
                    }
                }

                Ok(wrapper)
            }
        }
    }

    /// Evaluate an expression
    pub(crate) fn evaluate_expression(&self, expr: &Expression) -> EvalResult<Value> {
        match expr {
            Expression::Literal { value, .. } => Ok(Value::String(value.clone())),

            Expression::Number { value, .. } => Ok(Value::Number(*value)),

            Expression::Boolean { value, .. } => Ok(Value::Boolean(*value)),

            Expression::Variable { name, span } => {
                self.context.get_variable(name).cloned().ok_or_else(|| {
                    warn!(variable_name = name, span = ?span, "Variable not found in context");
                    EvalError::VariableNotFound {
                        name: name.clone(),
                        span: span.clone(),
                    }
                })
            }

            Expression::Member {
                object,
                property,
                span,
            } => {
                let obj_value = self.evaluate_expression(object)?;

                match obj_value {
                    Value::Object(map) => {
                        map.get(property)
                            .cloned()
                            .ok_or_else(|| EvalError::VariableNotFound {
                                name: property.clone(),
                                span: span.clone(),
                            })
                    }
                    _ => Err(EvalError::TypeError {
                        message: format!("Cannot access property {} on non-object", property),
                        span: span.clone(),
                    }),
                }
            }

            Expression::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;

                match operator {
                    BinaryOp::Add => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                        (Value::String(a), Value::String(b)) => {
                            Ok(Value::String(format!("{}{}", a, b)))
                        }
                        _ => Err(EvalError::InvalidOperands {
                            operator: "+".to_string(),
                            details: format!(
                                "Expected number + number or string + string, got {:?} + {:?}",
                                left_val, right_val
                            ),
                            span: span.clone(),
                        }),
                    },
                    BinaryOp::Subtract => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                        _ => Err(EvalError::InvalidOperands {
                            operator: "-".to_string(),
                            details: format!(
                                "Expected number - number, got {:?} - {:?}",
                                left_val, right_val
                            ),
                            span: span.clone(),
                        }),
                    },
                    BinaryOp::Multiply => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                        _ => Err(EvalError::InvalidOperands {
                            operator: "*".to_string(),
                            details: format!(
                                "Expected number * number, got {:?} * {:?}",
                                left_val, right_val
                            ),
                            span: span.clone(),
                        }),
                    },
                    BinaryOp::Divide => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => {
                            if *b != 0.0 {
                                Ok(Value::Number(a / b))
                            } else {
                                Err(EvalError::DivisionByZero { span: span.clone() })
                            }
                        }
                        _ => Err(EvalError::InvalidOperands {
                            operator: "/".to_string(),
                            details: format!(
                                "Expected number / number, got {:?} / {:?}",
                                left_val, right_val
                            ),
                            span: span.clone(),
                        }),
                    },
                    BinaryOp::Equals => Ok(Value::Boolean(left_val == right_val)),
                    BinaryOp::NotEquals => Ok(Value::Boolean(left_val != right_val)),
                    BinaryOp::LessThan => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
                        _ => Err(EvalError::InvalidOperands {
                            operator: "<".to_string(),
                            details: format!(
                                "Expected number < number, got {:?} < {:?}",
                                left_val, right_val
                            ),
                            span: span.clone(),
                        }),
                    },
                    BinaryOp::LessThanOrEqual => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
                        _ => Err(EvalError::InvalidOperands {
                            operator: "<=".to_string(),
                            details: format!(
                                "Expected number <= number, got {:?} <= {:?}",
                                left_val, right_val
                            ),
                            span: span.clone(),
                        }),
                    },
                    BinaryOp::GreaterThan => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
                        _ => Err(EvalError::InvalidOperands {
                            operator: ">".to_string(),
                            details: format!(
                                "Expected number > number, got {:?} > {:?}",
                                left_val, right_val
                            ),
                            span: span.clone(),
                        }),
                    },
                    BinaryOp::GreaterThanOrEqual => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
                        _ => Err(EvalError::InvalidOperands {
                            operator: ">=".to_string(),
                            details: format!(
                                "Expected number >= number, got {:?} >= {:?}",
                                left_val, right_val
                            ),
                            span: span.clone(),
                        }),
                    },
                    BinaryOp::And => {
                        // Logical AND with short-circuit evaluation
                        let left_bool = match left_val {
                            Value::Boolean(b) => b,
                            _ => {
                                return Err(EvalError::InvalidOperands {
                                    operator: "&&".to_string(),
                                    details: format!(
                                        "Expected boolean && boolean, got {:?} && {:?}",
                                        left_val, right_val
                                    ),
                                    span: span.clone(),
                                })
                            }
                        };

                        if !left_bool {
                            // Short circuit - left is false, don't evaluate right
                            Ok(Value::Boolean(false))
                        } else {
                            match right_val {
                                Value::Boolean(b) => Ok(Value::Boolean(b)),
                                _ => Err(EvalError::InvalidOperands {
                                    operator: "&&".to_string(),
                                    details: format!(
                                        "Expected boolean && boolean, got {:?} && {:?}",
                                        left_val, right_val
                                    ),
                                    span: span.clone(),
                                }),
                            }
                        }
                    }
                    BinaryOp::Or => {
                        // Logical OR with short-circuit evaluation
                        let left_bool = match left_val {
                            Value::Boolean(b) => b,
                            _ => {
                                return Err(EvalError::InvalidOperands {
                                    operator: "||".to_string(),
                                    details: format!(
                                        "Expected boolean || boolean, got {:?} || {:?}",
                                        left_val, right_val
                                    ),
                                    span: span.clone(),
                                })
                            }
                        };

                        if left_bool {
                            // Short circuit - left is true, don't evaluate right
                            Ok(Value::Boolean(true))
                        } else {
                            match right_val {
                                Value::Boolean(b) => Ok(Value::Boolean(b)),
                                _ => Err(EvalError::InvalidOperands {
                                    operator: "||".to_string(),
                                    details: format!(
                                        "Expected boolean || boolean, got {:?} || {:?}",
                                        left_val, right_val
                                    ),
                                    span: span.clone(),
                                }),
                            }
                        }
                    }
                }
            }

            Expression::Call {
                function,
                arguments,
                span,
            } => {
                // Function calls are not yet implemented - return empty string as no-op
                // Log warning so developers know this feature is pending
                warn!(
                    function = function,
                    arg_count = arguments.len(),
                    span = ?span,
                    "Function call not yet implemented - returning empty string"
                );
                Ok(Value::String(String::new()))
            }

            Expression::Template { parts, .. } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        TemplatePart::Literal(s) => result.push_str(s),
                        TemplatePart::Expression(expr) => {
                            let value = self.evaluate_expression(expr)?;
                            result.push_str(&value.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            }
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse_with_path;

    #[test]
    fn test_evaluate_simple_component() {
        let source = r#"
            public component Button {
                render button {
                    text "Click me"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        assert_eq!(vdoc.nodes.len(), 1);

        if let VNode::Element { tag, children, .. } = &vdoc.nodes[0] {
            assert_eq!(tag, "button");
            assert_eq!(children.len(), 1);

            if let VNode::Text { content } = &children[0] {
                assert_eq!(content, "Click me");
            } else {
                panic!("Expected text node");
            }
        } else {
            panic!("Expected element node");
        }
    }

    #[test]
    fn test_evaluate_with_styles() {
        let source = r#"
            public component Card {
                render div {
                    style {
                        padding: 16px
                        background: #FF0000
                    }
                    text "Hello"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        assert_eq!(vdoc.nodes.len(), 1);

        if let VNode::Element { tag, styles, .. } = &vdoc.nodes[0] {
            assert_eq!(tag, "div");
            assert_eq!(styles.get("padding"), Some(&"16px".to_string()));
            assert_eq!(styles.get("background"), Some(&"#FF0000".to_string()));
        } else {
            panic!("Expected element node");
        }
    }

    #[test]
    fn test_component_expansion_with_props() {
        let source = r#"
            component Greeting {
                render div {
                    text {name}
                }
            }

            public component App {
                render div {
                    Greeting(name="World")
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // All components render for preview (Greeting + App)
        assert_eq!(vdoc.nodes.len(), 2);

        // First node is Greeting rendered standalone: <div>{name}</div>
        // (props not passed, so {name} evaluates to empty or error)

        // Second node is App: <div><div>World</div></div>
        // The outer div is from App, inner div is from Greeting expansion
        if let VNode::Element {
            tag: outer_tag,
            children: outer_children,
            ..
        } = &vdoc.nodes[1]
        {
            assert_eq!(outer_tag, "div");
            assert_eq!(outer_children.len(), 1);

            if let VNode::Element {
                tag: inner_tag,
                children: inner_children,
                ..
            } = &outer_children[0]
            {
                assert_eq!(inner_tag, "div");
                assert_eq!(inner_children.len(), 1);

                if let VNode::Text { content } = &inner_children[0] {
                    assert_eq!(content, "World");
                } else {
                    panic!("Expected text node with 'World'");
                }
            } else {
                panic!("Expected inner div element (from Greeting)");
            }
        } else {
            panic!("Expected outer div element (from App)");
        }
    }

    #[test]
    fn test_nested_component_expansion() {
        let source = r#"
            component Label {
                render span {
                    text {message}
                }
            }

            component Button {
                render button {
                    Label(message={label})
                }
            }

            public component App {
                render div {
                    Button(label="Click me")
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // All 3 components render for preview (Label, Button, App)
        assert_eq!(vdoc.nodes.len(), 3);

        // App is the last node: <div> -> Button -> <button> -> Label -> <span> -> "Click me"
        if let VNode::Element {
            tag: div_tag,
            children: div_children,
            ..
        } = &vdoc.nodes[2]
        {
            assert_eq!(div_tag, "div");
            assert_eq!(div_children.len(), 1);

            if let VNode::Element {
                tag: button_tag,
                children: button_children,
                ..
            } = &div_children[0]
            {
                assert_eq!(button_tag, "button");
                assert_eq!(button_children.len(), 1);

                if let VNode::Element {
                    tag: span_tag,
                    children: span_children,
                    ..
                } = &button_children[0]
                {
                    assert_eq!(span_tag, "span");
                    assert_eq!(span_children.len(), 1);

                    if let VNode::Text { content } = &span_children[0] {
                        assert_eq!(content, "Click me");
                    } else {
                        panic!("Expected text node");
                    }
                } else {
                    panic!("Expected span element");
                }
            } else {
                panic!("Expected button element");
            }
        } else {
            panic!("Expected div element");
        }
    }
}
