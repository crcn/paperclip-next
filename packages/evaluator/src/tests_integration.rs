/// Integration tests: parse -> evaluate -> verify output
/// Tests complete flow for all new features
use crate::*;
use paperclip_parser::parse_with_path;

#[test]
fn test_integration_binary_operations_in_conditionals() {
    let source = r#"
public component Badge {
    render div {
        if count > 0 {
            span {
                text {count}
            }
        }
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator
        .context
        .set_variable("count".to_string(), Value::Number(5.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Should render the span since count > 0
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            assert_eq!(children.len(), 1);
            match &children[0] {
                VNode::Element { tag, .. } => assert_eq!(tag, "span"),
                _ => panic!("Expected span element"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_logical_operators_in_conditionals() {
    let source = r#"
public component StatusBadge {
    render div {
        if isActive && isVisible {
            span {
                text "Active"
            }
        }
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator
        .context
        .set_variable("isActive".to_string(), Value::Boolean(true));
    evaluator
        .context
        .set_variable("isVisible".to_string(), Value::Boolean(true));

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Should render the span since both are true
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            assert_eq!(children.len(), 1);
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_template_strings_in_text() {
    let source = r#"
public component UserGreeting {
    render div {
        text "Welcome, ${firstName} ${lastName}!"
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    evaluator
        .context
        .set_variable("firstName".to_string(), Value::String("Alice".to_string()));
    evaluator
        .context
        .set_variable("lastName".to_string(), Value::String("Smith".to_string()));

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Check greeting rendered correctly
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => match &children[0] {
            VNode::Text { content } => assert_eq!(content, "Welcome, Alice Smith!"),
            _ => panic!("Expected text node"),
        },
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_binary_ops_in_text() {
    let source = r#"
public component Calculator {
    render div {
        text "Progress: ${progress * 100}%"
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator
        .context
        .set_variable("progress".to_string(), Value::Number(0.75));

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Should have calculated percentage
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => match &children[0] {
            VNode::Text { content } => {
                assert_eq!(content, "Progress: 75%");
            }
            _ => panic!("Expected text node"),
        },
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_combination_variants_with_css() {
    let source = r#"
public component Button {
    variant primary
    variant hover

    render button {
        style variant primary {
            background: blue
        }

        style variant hover {
            opacity: 0.8
        }

        style variant primary + hover {
            background: darkblue
        }

        text "Click me"
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Enable both variants
    evaluator
        .context
        .set_variable("primary".to_string(), Value::Boolean(true));
    evaluator
        .context
        .set_variable("hover".to_string(), Value::Boolean(true));

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Should render button with combined styles applied
    match &vdoc.nodes[0] {
        VNode::Element { tag, styles, .. } => {
            assert_eq!(tag, "button");
            // All three style blocks should be applied when both variants are true
            assert!(styles.len() > 0);
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_element_names_preserved_in_vdom() {
    let source = r#"
public component Card {
    render div container {
        div header {
            text "Title"
        }
        div body {
            text "Content"
        }
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Element names don't affect VDOM structure but should parse and evaluate correctly
    match &vdoc.nodes[0] {
        VNode::Element { tag, children, .. } => {
            assert_eq!(tag, "div");
            assert_eq!(children.len(), 2);

            // Verify both header and body rendered
            match &children[0] {
                VNode::Element { tag, .. } => assert_eq!(tag, "div"),
                _ => panic!("Expected element"),
            }
            match &children[1] {
                VNode::Element { tag, .. } => assert_eq!(tag, "div"),
                _ => panic!("Expected element"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_function_calls_are_noop() {
    let source = r#"
public component FormattedDate {
    render div {
        text "Date: ${formatDate(date, format)}"
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator
        .context
        .set_variable("date".to_string(), Value::String("2024-01-01".to_string()));
    evaluator.context.set_variable(
        "format".to_string(),
        Value::String("YYYY-MM-DD".to_string()),
    );

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Function call should be no-op (empty string)
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            match &children[0] {
                VNode::Text { content } => {
                    // Should have "Date: " but function call result is empty
                    assert_eq!(content, "Date: ");
                }
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_complex_expression_evaluation() {
    let source = r#"
public component PriceDisplay {
    render div {
        if price > 0 && quantity > 0 {
            div {
                text "Total: ${price * quantity}"
            }
        }
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator
        .context
        .set_variable("price".to_string(), Value::Number(25.0));
    evaluator
        .context
        .set_variable("quantity".to_string(), Value::Number(5.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Should render the total since conditions are true
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            // The conditional wraps in a div
            assert!(children.len() > 0);

            // Check total calculation exists
            match &children[0] {
                VNode::Element {
                    children: total_children,
                    ..
                } => match &total_children[0] {
                    VNode::Text { content } => assert_eq!(content, "Total: 125"),
                    _ => panic!("Expected text node"),
                },
                _ => panic!("Expected element"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_operator_precedence_in_evaluation() {
    let source = r#"
public component Calculator {
    render div {
        text "Result: ${a + b * c}"
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    evaluator
        .context
        .set_variable("a".to_string(), Value::Number(2.0));
    evaluator
        .context
        .set_variable("b".to_string(), Value::Number(3.0));
    evaluator
        .context
        .set_variable("c".to_string(), Value::Number(4.0));

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Should evaluate as 2 + (3 * 4) = 14, not (2 + 3) * 4 = 20
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => match &children[0] {
            VNode::Text { content } => assert_eq!(content, "Result: 14"),
            _ => panic!("Expected text node"),
        },
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_short_circuit_evaluation() {
    let source = r#"
public component ShortCircuit {
    render div {
        if true || isUndefined {
            div {
                text "Should render due to short-circuit"
            }
        }
    }
}
"#;
    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    // Don't define 'isUndefined' variable - it should short-circuit before reaching it

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Should render because true || X always evaluates to true (short-circuit)
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            // Should have at least one child (the conditional result wraps in div)
            assert!(children.len() > 0);
        }
        _ => panic!("Expected element node"),
    }
}
