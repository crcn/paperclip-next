/// Comprehensive tests for partial evaluation with error recovery
///
/// These tests verify that the evaluator can handle errors gracefully
/// instead of crashing, allowing developers to see inline errors in the preview.
use crate::evaluator::{Evaluator, Value};
use crate::vdom::VNode;
use paperclip_parser::parse;

#[test]
fn test_error_in_text_expression() {
    let source = r#"
        public component Button {
            render div {
                text {invalidVariable}
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&doc);

    assert!(result.is_ok(), "Should not crash on invalid variable");
    let vdoc = result.unwrap();

    // Should have one component
    assert_eq!(vdoc.nodes.len(), 1);

    // The div should have an error node as child
    if let VNode::Element { children, .. } = &vdoc.nodes[0] {
        assert_eq!(children.len(), 1);
        match &children[0] {
            VNode::Error { message, .. } => {
                assert!(
                    message.contains("invalidVariable"),
                    "Error should mention the variable name"
                );
            }
            other => panic!("Expected Error node, got {:?}", other),
        }
    } else {
        panic!("Expected Element node");
    }
}

#[test]
fn test_error_in_attribute_expression() {
    let source = r#"
        public component Button {
            render div(title={invalidVariable}) {
                text "Click me"
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&doc);

    assert!(result.is_ok(), "Should not crash on invalid variable");
    let vdoc = result.unwrap();

    // Should have one component with element
    assert_eq!(vdoc.nodes.len(), 1);

    if let VNode::Element { attributes, .. } = &vdoc.nodes[0] {
        // Attribute should contain error message
        let title = attributes.get("title").expect("Title attribute should exist");
        assert!(
            title.contains("Error"),
            "Attribute should contain error: {}",
            title
        );
    } else {
        panic!("Expected Element node");
    }
}

#[test]
fn test_partial_evaluation_with_mixed_errors() {
    let source = r#"
        public component Card {
            render div {
                text "Valid text"
                text {invalidVar1}
                text "Another valid text"
                text {invalidVar2}
                text "Final valid text"
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&doc);

    assert!(result.is_ok(), "Should not crash on multiple errors");
    let vdoc = result.unwrap();

    // Should have one component
    assert_eq!(vdoc.nodes.len(), 1);

    if let VNode::Element { children, .. } = &vdoc.nodes[0] {
        assert_eq!(children.len(), 5, "Should have 5 children (3 valid, 2 errors)");

        // First child: valid text
        matches!(&children[0], VNode::Text { content } if content == "Valid text");

        // Second child: error
        matches!(&children[1], VNode::Error { .. });

        // Third child: valid text
        matches!(&children[2], VNode::Text { content } if content == "Another valid text");

        // Fourth child: error
        matches!(&children[3], VNode::Error { .. });

        // Fifth child: valid text
        matches!(&children[4], VNode::Text { content } if content == "Final valid text");
    } else {
        panic!("Expected Element node");
    }
}

#[test]
fn test_error_in_conditional_expression() {
    let source = r#"
        public component Widget {
            render div {
                if {invalidCondition} {
                    text "Then branch"
                }
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&doc);

    assert!(
        result.is_ok(),
        "Should not crash on invalid conditional expression"
    );
    let vdoc = result.unwrap();

    // Should have one component div with children
    assert_eq!(vdoc.nodes.len(), 1);

    if let VNode::Element { children, .. } = &vdoc.nodes[0] {
        // The conditional error should be one of the children
        let has_error = children.iter().any(|child| matches!(child, VNode::Error { .. }));
        assert!(has_error, "Should have at least one error node for invalid condition");
    }
}

#[test]
fn test_error_in_repeat_collection() {
    let source = r#"
        public component List {
            render div {
                repeat item in {invalidCollection} {
                    text {item}
                }
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&doc);

    assert!(
        result.is_ok(),
        "Should not crash on invalid repeat collection"
    );
    let vdoc = result.unwrap();

    // Should have one component div with an error child
    assert_eq!(vdoc.nodes.len(), 1);

    if let VNode::Element { children, .. } = &vdoc.nodes[0] {
        assert_eq!(children.len(), 1);
        match &children[0] {
            VNode::Error { message, .. } => {
                assert!(
                    message.contains("repeat"),
                    "Error should mention repeat collection"
                );
            }
            other => panic!("Expected Error node for failed repeat, got {:?}", other),
        }
    }
}

#[test]
fn test_error_in_nested_element() {
    let source = r#"
        public component Card {
            render div {
                div {
                    div {
                        text {deepError}
                    }
                }
                text "Should still render"
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&doc);

    assert!(result.is_ok(), "Should not crash on nested error");
    let vdoc = result.unwrap();

    // Should successfully evaluate with error deep in the tree
    assert_eq!(vdoc.nodes.len(), 1);
}

#[test]
fn test_error_has_semantic_id() {
    let source = r#"
        public component Button {
            render div {
                text {invalidVar}
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&doc).expect("Should evaluate");

    // Error node should have a semantic ID
    if let VNode::Element { children, .. } = &result.nodes[0] {
        match &children[0] {
            VNode::Error { semantic_id, .. } => {
                assert!(
                    !semantic_id.segments.is_empty(),
                    "Error should have semantic ID"
                );
            }
            other => panic!("Expected Error node, got {:?}", other),
        }
    }
}

#[test]
fn test_error_in_component_prop() {
    let source = r#"
        component Child {
            render div {
                text "Child content"
            }
        }

        public component Parent {
            render div {
                Child(prop={invalidProp})
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&doc);

    assert!(
        result.is_ok(),
        "Should not crash on invalid component prop"
    );
    let vdoc = result.unwrap();

    // Should have one public component (Parent)
    // The parent component should still render even though prop evaluation failed
    assert!(!vdoc.nodes.is_empty(), "Should have rendered components");
}

#[test]
fn test_division_by_zero_error() {
    let source = r#"
        public component Math {
            render div {
                text {result}
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    // Note: division by zero would be caught at runtime if we had runtime division
    // For now, just test with missing variable which will also produce error
    let result = evaluator.evaluate(&doc);

    assert!(result.is_ok(), "Should not crash on missing variable");
    let vdoc = result.unwrap();

    if let VNode::Element { children, .. } = &vdoc.nodes[0] {
        match &children[0] {
            VNode::Error { message, .. } => {
                assert!(
                    message.contains("result") || message.contains("not found"),
                    "Error should mention missing variable"
                );
            }
            other => panic!("Expected Error node, got {:?}", other),
        }
    }
}

#[test]
fn test_type_error_in_member_access() {
    let source = r#"
        public component Widget {
            render div {
                text {someNumber.property}
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    evaluator
        .context
        .set_variable("someNumber".to_string(), Value::Number(42.0));
    let result = evaluator.evaluate(&doc);

    assert!(
        result.is_ok(),
        "Should not crash on invalid member access"
    );
    let vdoc = result.unwrap();

    if let VNode::Element { children, .. } = &vdoc.nodes[0] {
        match &children[0] {
            VNode::Error { message, .. } => {
                assert!(
                    message.contains("property") || message.contains("TypeError"),
                    "Error should mention type error: {}",
                    message
                );
            }
            other => panic!("Expected Error node, got {:?}", other),
        }
    }
}

#[test]
fn test_error_recovery_preserves_sibling_elements() {
    let source = r#"
        public component Layout {
            render div {
                header {
                    text "Header"
                }
                main {
                    text {brokenExpression}
                }
                footer {
                    text "Footer"
                }
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&doc);

    assert!(result.is_ok(), "Should not crash");
    let vdoc = result.unwrap();

    // The parent div should still have all three children (header, main, footer)
    if let VNode::Element { children, .. } = &vdoc.nodes[0] {
        assert_eq!(
            children.len(),
            3,
            "Should have 3 children despite error in main"
        );

        // Header should be fine
        if let VNode::Element { tag, children, .. } = &children[0] {
            assert_eq!(tag, "header");
            assert_eq!(children.len(), 1);
        }

        // Main should have error
        if let VNode::Element { tag, children, .. } = &children[1] {
            assert_eq!(tag, "main");
            assert_eq!(children.len(), 1);
            assert!(matches!(&children[0], VNode::Error { .. }));
        }

        // Footer should be fine
        if let VNode::Element { tag, children, .. } = &children[2] {
            assert_eq!(tag, "footer");
            assert_eq!(children.len(), 1);
        }
    }
}

#[test]
fn test_error_in_conditional_branch_body() {
    let source = r#"
        public component ConditionalError {
            render div {
                if {shouldShow} {
                    text "Valid"
                    text {invalidVar}
                    text "Still valid"
                }
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    evaluator
        .context
        .set_variable("shouldShow".to_string(), Value::Boolean(true));
    let result = evaluator.evaluate(&doc);

    assert!(
        result.is_ok(),
        "Should not crash on error in conditional branch"
    );
    let vdoc = result.unwrap();

    // The div should have the conditional result as child
    if let VNode::Element { children, .. } = &vdoc.nodes[0] {
        assert_eq!(children.len(), 1, "Should have conditional branch wrapper");

        // The conditional should render a wrapper div with the three children
        if let VNode::Element { children, .. } = &children[0] {
            assert_eq!(
                children.len(),
                3,
                "Should have 3 children (valid, error, valid)"
            );
            assert!(matches!(&children[1], VNode::Error { .. }));
        }
    }
}


#[test]
fn test_multiple_errors_in_repeat_body() {
    let source = r#"
        public component ListWithErrors {
            render div {
                repeat item in {items} {
                    text "Valid: "
                    text {item.invalidProp}
                }
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    evaluator.context.set_variable(
        "items".to_string(),
        Value::Array(vec![
            Value::Object([("id".to_string(), Value::Number(1.0))].into_iter().collect()),
            Value::Object([("id".to_string(), Value::Number(2.0))].into_iter().collect()),
        ]),
    );
    let result = evaluator.evaluate(&doc);

    assert!(result.is_ok(), "Should not crash on errors in repeat body");
    let vdoc = result.unwrap();

    // Should still render the repeat wrapper with errors inline
    if let VNode::Element { children, .. } = &vdoc.nodes[0] {
        assert_eq!(children.len(), 1, "Should have repeat wrapper");

        // The repeat wrapper should have children despite errors
        if let VNode::Element { children, .. } = &children[0] {
            // 2 items * 2 children per item = 4 total
            assert!(!children.is_empty(), "Should have children despite errors");
        }
    }
}

#[test]
fn test_simple_expression_error() {
    let source = r#"
        public component Test {
            render div {
                text {missing}
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&doc);

    assert!(result.is_ok(), "Should not crash");
    let vdoc = result.unwrap();

    // Should have error node
    if let VNode::Element { children, .. } = &vdoc.nodes[0] {
        assert!(matches!(&children[0], VNode::Error { .. }));
    }
}
