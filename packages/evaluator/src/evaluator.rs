use crate::vdom::{VDocument, VNode};
use paperclip_parser::ast::*;
use std::collections::HashMap;
use thiserror::Error;

pub type EvalResult<T> = Result<T, EvalError>;

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Evaluation error: {0}")]
    EvaluationError(String),
}

/// Context for evaluation
#[derive(Clone)]
pub struct EvalContext {
    components: HashMap<String, Component>,
    tokens: HashMap<String, String>,
    variables: HashMap<String, Value>,
}

impl EvalContext {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            tokens: HashMap::new(),
            variables: HashMap::new(),
        }
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
        Self {
            context: EvalContext::new(),
        }
    }

    /// Evaluate a document to virtual DOM
    pub fn evaluate(&mut self, doc: &Document) -> EvalResult<VDocument> {
        // Register tokens
        for token in &doc.tokens {
            self.context.add_token(token.name.clone(), token.value.clone());
        }

        // Register components
        for component in &doc.components {
            self.context.add_component(component.clone());
        }

        let mut vdoc = VDocument::new();

        // Evaluate all public components
        for component in &doc.components {
            if component.public {
                let vnode = self.evaluate_component(&component.name)?;
                vdoc.add_node(vnode);
            }
        }

        Ok(vdoc)
    }

    /// Evaluate a component by name
    fn evaluate_component(&self, name: &str) -> EvalResult<VNode> {
        self.evaluate_component_with_props(name, &HashMap::new())
    }

    /// Evaluate a component with props
    fn evaluate_component_with_props(
        &self,
        name: &str,
        props: &HashMap<String, Value>,
    ) -> EvalResult<VNode> {
        let component = self
            .context
            .components
            .get(name)
            .ok_or_else(|| EvalError::ComponentNotFound(name.to_string()))?;

        // Create a new context scope with props as variables
        let mut scoped_evaluator = Evaluator {
            context: self.context.clone(),
        };

        // Bind props to variables
        for (key, value) in props {
            scoped_evaluator.context.set_variable(key.clone(), value.clone());
        }

        if let Some(body) = &component.body {
            scoped_evaluator.evaluate_element(body)
        } else {
            Ok(VNode::element("div"))
        }
    }

    /// Evaluate an element
    fn evaluate_element(&self, element: &Element) -> EvalResult<VNode> {
        match element {
            Element::Tag {
                name,
                attributes,
                styles,
                children,
                ..
            } => {
                let mut vnode = VNode::element(name);

                // Evaluate attributes
                for (key, expr) in attributes {
                    let value = self.evaluate_expression(expr)?;
                    vnode = vnode.with_attr(key, value.to_string());
                }

                // Evaluate styles
                for style_block in styles {
                    for (key, value) in &style_block.properties {
                        vnode = vnode.with_style(key, value);
                    }
                }

                // Evaluate children
                for child in children {
                    let child_vnode = self.evaluate_element(child)?;
                    vnode = vnode.with_child(child_vnode);
                }

                Ok(vnode)
            }

            Element::Text { content, .. } => {
                let value = self.evaluate_expression(content)?;
                Ok(VNode::text(value.to_string()))
            }

            Element::Instance {
                name,
                props,
                children: _children,
                ..
            } => {
                // Evaluate component with props
                // Props are evaluated in current context, then passed to component
                let mut evaluated_props = HashMap::new();
                for (key, expr) in props {
                    let value = self.evaluate_expression(expr)?;
                    evaluated_props.insert(key.clone(), value);
                }

                // Expand component - this returns the component's body with props applied
                // The result is pure DOM elements, not a Component VNode
                self.evaluate_component_with_props(name, &evaluated_props)
            }

            Element::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let condition_value = self.evaluate_expression(condition)?;

                if condition_value.is_truthy() {
                    // Evaluate then branch (return first node, or wrapper if multiple)
                    if then_branch.len() == 1 {
                        self.evaluate_element(&then_branch[0])
                    } else {
                        let mut wrapper = VNode::element("div");
                        for child in then_branch {
                            let child_vnode = self.evaluate_element(child)?;
                            wrapper = wrapper.with_child(child_vnode);
                        }
                        Ok(wrapper)
                    }
                } else if let Some(else_branch) = else_branch {
                    if else_branch.len() == 1 {
                        self.evaluate_element(&else_branch[0])
                    } else {
                        let mut wrapper = VNode::element("div");
                        for child in else_branch {
                            let child_vnode = self.evaluate_element(child)?;
                            wrapper = wrapper.with_child(child_vnode);
                        }
                        Ok(wrapper)
                    }
                } else {
                    // Return empty comment
                    Ok(VNode::Comment {
                        content: "conditional false".to_string(),
                    })
                }
            }

            Element::Repeat {
                item_name: _item_name,
                collection,
                body,
                ..
            } => {
                // For now, simplified: assume collection is an array variable
                let collection_value = self.evaluate_expression(collection)?;

                let mut wrapper = VNode::element("div");

                if let Value::Array(items) = collection_value {
                    for _item in items {
                        // TODO: Set item variable in context
                        for child in body {
                            let child_vnode = self.evaluate_element(child)?;
                            wrapper = wrapper.with_child(child_vnode);
                        }
                    }
                }

                Ok(wrapper)
            }

            Element::SlotInsert { name, .. } => {
                // For now, return comment
                Ok(VNode::Comment {
                    content: format!("slot: {}", name),
                })
            }
        }
    }

    /// Evaluate an expression
    fn evaluate_expression(&self, expr: &Expression) -> EvalResult<Value> {
        match expr {
            Expression::Literal { value, .. } => Ok(Value::String(value.clone())),

            Expression::Number { value, .. } => Ok(Value::Number(*value)),

            Expression::Boolean { value, .. } => Ok(Value::Boolean(*value)),

            Expression::Variable { name, .. } => self
                .context
                .get_variable(name)
                .cloned()
                .ok_or_else(|| EvalError::VariableNotFound(name.clone())),

            Expression::Member {
                object, property, ..
            } => {
                let obj_value = self.evaluate_expression(object)?;

                match obj_value {
                    Value::Object(map) => map
                        .get(property)
                        .cloned()
                        .ok_or_else(|| EvalError::VariableNotFound(property.clone())),
                    _ => Err(EvalError::EvaluationError(format!(
                        "Cannot access property {} on non-object",
                        property
                    ))),
                }
            }

            Expression::Binary {
                left,
                operator,
                right,
                ..
            } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;

                match operator {
                    BinaryOp::Add => match (left_val, right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                        _ => Err(EvalError::EvaluationError("Invalid operands for +".to_string())),
                    },
                    BinaryOp::Subtract => match (left_val, right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                        _ => Err(EvalError::EvaluationError("Invalid operands for -".to_string())),
                    },
                    BinaryOp::Multiply => match (left_val, right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                        _ => Err(EvalError::EvaluationError("Invalid operands for *".to_string())),
                    },
                    BinaryOp::Divide => match (left_val, right_val) {
                        (Value::Number(a), Value::Number(b)) => {
                            if b != 0.0 {
                                Ok(Value::Number(a / b))
                            } else {
                                Err(EvalError::EvaluationError("Division by zero".to_string()))
                            }
                        }
                        _ => Err(EvalError::EvaluationError("Invalid operands for /".to_string())),
                    },
                    BinaryOp::Equals => Ok(Value::Boolean(left_val == right_val)),
                    BinaryOp::NotEquals => Ok(Value::Boolean(left_val != right_val)),
                    _ => Err(EvalError::EvaluationError(format!(
                        "Operator {:?} not implemented",
                        operator
                    ))),
                }
            }

            Expression::Call { .. } => {
                // TODO: Implement function calls
                Ok(Value::String("function call".to_string()))
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
    use paperclip_parser::parse;

    #[test]
    fn test_evaluate_simple_component() {
        let source = r#"
            public component Button {
                render button {
                    text "Click me"
                }
            }
        "#;

        let doc = parse(source).expect("Failed to parse");
        let mut evaluator = Evaluator::new();
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

        let doc = parse(source).expect("Failed to parse");
        let mut evaluator = Evaluator::new();
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

        let doc = parse(source).expect("Failed to parse");
        let mut evaluator = Evaluator::new();
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should evaluate to a single public component (App)
        assert_eq!(vdoc.nodes.len(), 1);

        // App renders: <div><div>World</div></div>
        // The outer div is from App, inner div is from Greeting expansion
        if let VNode::Element { tag: outer_tag, children: outer_children, .. } = &vdoc.nodes[0] {
            assert_eq!(outer_tag, "div");
            assert_eq!(outer_children.len(), 1);

            if let VNode::Element { tag: inner_tag, children: inner_children, .. } = &outer_children[0] {
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

        let doc = parse(source).expect("Failed to parse");
        let mut evaluator = Evaluator::new();
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        assert_eq!(vdoc.nodes.len(), 1);

        // App -> <div> -> Button -> <button> -> Label -> <span> -> "Click me"
        if let VNode::Element { tag: div_tag, children: div_children, .. } = &vdoc.nodes[0] {
            assert_eq!(div_tag, "div");
            assert_eq!(div_children.len(), 1);

            if let VNode::Element { tag: button_tag, children: button_children, .. } = &div_children[0] {
                assert_eq!(button_tag, "button");
                assert_eq!(button_children.len(), 1);

                if let VNode::Element { tag: span_tag, children: span_children, .. } = &button_children[0] {
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
