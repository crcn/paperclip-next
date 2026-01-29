/// Comprehensive tests for new expression types (Binary, Call, Template)
use crate::*;
use paperclip_parser::parse_with_path;

#[test]
fn test_binary_comparison_less_than() {
    let source = r#"
public component Test {
    render div {
        text {count < 10}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("count".to_string(), Value::Number(5.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "true"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_binary_comparison_greater_than() {
    let source = r#"
public component Test {
    render div {
        text {count > 10}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("count".to_string(), Value::Number(15.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "true"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_binary_comparison_less_than_or_equal() {
    let source = r#"
public component Test {
    render div {
        text {count <= 10}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("count".to_string(), Value::Number(10.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "true"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_binary_comparison_greater_than_or_equal() {
    let source = r#"
public component Test {
    render div {
        text {count >= 10}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("count".to_string(), Value::Number(10.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "true"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_binary_logical_and() {
    let source = r#"
public component Test {
    render div {
        text {isActive && isVisible}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("isActive".to_string(), Value::Boolean(true));
    evaluator.context.set_variable("isVisible".to_string(), Value::Boolean(true));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "true"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_binary_logical_and_short_circuit() {
    let source = r#"
public component Test {
    render div {
        text {isActive && isVisible}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("isActive".to_string(), Value::Boolean(false));
    evaluator.context.set_variable("isVisible".to_string(), Value::Boolean(true));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "false"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_binary_logical_or() {
    let source = r#"
public component Test {
    render div {
        text {isActive || isVisible}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("isActive".to_string(), Value::Boolean(false));
    evaluator.context.set_variable("isVisible".to_string(), Value::Boolean(true));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "true"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_binary_logical_or_short_circuit() {
    let source = r#"
public component Test {
    render div {
        text {isActive || isVisible}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("isActive".to_string(), Value::Boolean(true));
    evaluator.context.set_variable("isVisible".to_string(), Value::Boolean(false));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "true"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_function_call_is_noop() {
    let source = r#"
public component Test {
    render div {
        text {formatDate(date, "YYYY-MM-DD")}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("date".to_string(), Value::String("2024-01-01".to_string()));

    // Should succeed (not error) and return empty string
    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, ""),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_template_string_basic() {
    let source = r#"
public component Test {
    render div {
        text "Hello ${name}!"
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("name".to_string(), Value::String("World".to_string()));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "Hello World!"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_template_string_multiple_interpolations() {
    let source = r#"
public component Test {
    render div {
        text "User ${firstName} ${lastName} (${age} years old)"
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("firstName".to_string(), Value::String("John".to_string()));
    evaluator.context.set_variable("lastName".to_string(), Value::String("Doe".to_string()));
    evaluator.context.set_variable("age".to_string(), Value::Number(30.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "User John Doe (30 years old)"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_template_string_with_expression() {
    let source = r#"
public component Test {
    render div {
        text "Count plus one: ${count + 1}"
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("count".to_string(), Value::Number(5.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "Count plus one: 6"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_operator_precedence_multiply_before_add() {
    let source = r#"
public component Test {
    render div {
        text {a + b * c}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("a".to_string(), Value::Number(2.0));
    evaluator.context.set_variable("b".to_string(), Value::Number(3.0));
    evaluator.context.set_variable("c".to_string(), Value::Number(4.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    // Should be 2 + (3 * 4) = 14, not (2 + 3) * 4 = 20
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "14"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_operator_precedence_comparison_before_logical() {
    let source = r#"
public component Test {
    render div {
        text {a > 5 && b < 10}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("a".to_string(), Value::Number(7.0));
    evaluator.context.set_variable("b".to_string(), Value::Number(3.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    // Should be (a > 5) && (b < 10) = true && true = true
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "true"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_complex_expression_combination() {
    let source = r#"
public component Test {
    render div {
        text {count > 0 && count <= 10 || isSpecial}
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator.context.set_variable("count".to_string(), Value::Number(5.0));
    evaluator.context.set_variable("isSpecial".to_string(), Value::Boolean(false));

    let vdoc = evaluator.evaluate(&doc).unwrap();
    // Should be (5 > 0 && 5 <= 10) || false = true || false = true
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => assert_eq!(content, "true"),
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}
