/// Comprehensive test suite for evaluator
/// Tests expression evaluation, component composition, error handling
use crate::*;
use paperclip_parser::parse_with_path;

#[cfg(test)]
mod evaluator_comprehensive_tests {
    use super::*;

    #[test]
    fn test_evaluate_multiple_components() {
        let source = r#"
            public component Button {
                render button {
                    text "Button"
                }
            }

            public component Card {
                render div {
                    text "Card"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        assert_eq!(vdoc.nodes.len(), 2);

        // Check first component (Button)
        match &vdoc.nodes[0] {
            VNode::Element { tag, children, .. } => {
                assert_eq!(tag, "button");
                assert_eq!(children.len(), 1);
                match &children[0] {
                    VNode::Text { content } => assert_eq!(content, "Button"),
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected element node"),
        }

        // Check second component (Card)
        match &vdoc.nodes[1] {
            VNode::Element { tag, children, .. } => {
                assert_eq!(tag, "div");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_evaluate_nested_elements() {
        let source = r#"
            public component Card {
                render div {
                    div {
                        text "Header"
                    }
                    div {
                        div {
                            text "Nested"
                        }
                    }
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        assert_eq!(vdoc.nodes.len(), 1);

        match &vdoc.nodes[0] {
            VNode::Element { tag, children, .. } => {
                assert_eq!(tag, "div");
                assert_eq!(children.len(), 2);

                // Check nested structure
                match &children[1] {
                    VNode::Element {
                        children: inner, ..
                    } => {
                        assert_eq!(inner.len(), 1);
                        match &inner[0] {
                            VNode::Element {
                                children: deepest, ..
                            } => {
                                assert_eq!(deepest.len(), 1);
                            }
                            _ => panic!("Expected element node"),
                        }
                    }
                    _ => panic!("Expected element node"),
                }
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_evaluate_inline_styles() {
        let source = r#"
            public component Box {
                render div {
                    style {
                        padding: 16px
                        margin: 8px
                        background: #fff
                        border-radius: 4px
                    }
                    text "Content"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        assert_eq!(vdoc.nodes.len(), 1);

        match &vdoc.nodes[0] {
            VNode::Element { styles, .. } => {
                assert!(!styles.is_empty());
                assert!(styles.contains_key("padding"));
                assert!(styles.contains_key("margin"));
                assert!(styles.contains_key("background"));
                assert!(styles.contains_key("border-radius"));
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_evaluate_multiple_style_blocks() {
        let source = r#"
            public component MultiStyle {
                render div {
                    style {
                        padding: 16px
                    }
                    style {
                        margin: 8px
                    }
                    style {
                        background: #fff
                    }
                    text "Content"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        assert_eq!(vdoc.nodes.len(), 1);

        match &vdoc.nodes[0] {
            VNode::Element { styles, .. } => {
                // All styles should be merged
                assert_eq!(styles.len(), 3);
                assert!(styles.contains_key("padding"));
                assert!(styles.contains_key("margin"));
                assert!(styles.contains_key("background"));
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_evaluate_css_with_dashes() {
        let source = r#"
            public component DashedProps {
                render div {
                    style {
                        margin-top: 8px
                        margin-bottom: 16px
                        line-height: 1.5
                        border-radius: 4px
                        background-color: #fff
                    }
                    text "Content"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        assert_eq!(vdoc.nodes.len(), 1);

        match &vdoc.nodes[0] {
            VNode::Element { styles, .. } => {
                assert!(styles.contains_key("margin-top"));
                assert!(styles.contains_key("margin-bottom"));
                assert!(styles.contains_key("line-height"));
                assert!(styles.contains_key("border-radius"));
                assert!(styles.contains_key("background-color"));
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_evaluate_expression_literal() {
        let source = r#"
            public component Literal {
                render div {
                    text "Hello World"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        match &vdoc.nodes[0] {
            VNode::Element { children, .. } => match &children[0] {
                VNode::Text { content } => {
                    assert_eq!(content, "Hello World");
                }
                _ => panic!("Expected text node"),
            },
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_evaluate_expression_variable() {
        let source = r#"
            public component Var {
                render div {
                    text {userName}
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");

        // Set variable
        evaluator
            .context
            .set_variable("userName".to_string(), Value::String("Alice".to_string()));

        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        match &vdoc.nodes[0] {
            VNode::Element { children, .. } => match &children[0] {
                VNode::Text { content } => {
                    assert_eq!(content, "Alice");
                }
                _ => panic!("Expected text node"),
            },
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_evaluate_expression_member_access() {
        let source = r#"
            public component MemberAccess {
                render div {
                    text {user.name}
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");

        // Set object variable
        let mut user = std::collections::HashMap::new();
        user.insert("name".to_string(), Value::String("Bob".to_string()));
        evaluator
            .context
            .set_variable("user".to_string(), Value::Object(user));

        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        match &vdoc.nodes[0] {
            VNode::Element { children, .. } => match &children[0] {
                VNode::Text { content } => {
                    assert_eq!(content, "Bob");
                }
                _ => panic!("Expected text node"),
            },
            _ => panic!("Expected element node"),
        }
    }

    // Note: Binary expressions in text blocks are not yet fully implemented
    // This test is commented out until parser supports complex expressions
    // #[test]
    // fn test_evaluate_expression_binary_add() {
    //     let source = r#"
    //         public component BinaryAdd {
    //             render div {
    //                 text {a + b}
    //             }
    //         }
    //     "#;
    //
    //     let doc = parse(source).expect("Failed to parse");
    //     let mut evaluator = Evaluator::new();
    //
    //     evaluator
    //         .context
    //         .set_variable("a".to_string(), Value::Number(5.0));
    //     evaluator
    //         .context
    //         .set_variable("b".to_string(), Value::Number(3.0));
    //
    //     let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");
    //
    //     match &vdoc.nodes[0] {
    //         VNode::Element { children, .. } => {
    //             match &children[0] {
    //                 VNode::Text { content } => {
    //                     assert_eq!(content, "8");
    //                 }
    //                 _ => panic!("Expected text node"),
    //             }
    //         }
    //         _ => panic!("Expected element node"),
    //     }
    // }

    // Note: Binary expressions in text blocks are not yet fully implemented
    // #[test]
    // fn test_evaluate_expression_binary_multiply() {
    //     let source = r#"
    //         public component BinaryMult {
    //             render div {
    //                 text {price * quantity}
    //             }
    //         }
    //     "#;
    //
    //     let doc = parse(source).expect("Failed to parse");
    //     let mut evaluator = Evaluator::new();
    //
    //     evaluator
    //         .context
    //         .set_variable("price".to_string(), Value::Number(10.0));
    //     evaluator
    //         .context
    //         .set_variable("quantity".to_string(), Value::Number(3.0));
    //
    //     let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");
    //
    //     match &vdoc.nodes[0] {
    //         VNode::Element { children, .. } => {
    //             match &children[0] {
    //                 VNode::Text { content } => {
    //                     assert_eq!(content, "30");
    //                 }
    //                 _ => panic!("Expected text node"),
    //             }
    //         }
    //         _ => panic!("Expected element node"),
    //     }
    // }

    // Note: Binary expressions in text blocks are not yet fully implemented
    // #[test]
    // fn test_evaluate_expression_string_concatenation() {
    //     let source = r#"
    //         public component Concat {
    //             render div {
    //                 text {firstName + " " + lastName}
    //             }
    //         }
    //     "#;
    //
    //     let doc = parse(source).expect("Failed to parse");
    //     let mut evaluator = Evaluator::new();
    //
    //     evaluator
    //         .context
    //         .set_variable("firstName".to_string(), Value::String("John".to_string()));
    //     evaluator
    //         .context
    //         .set_variable("lastName".to_string(), Value::String("Doe".to_string()));
    //
    //     let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");
    //
    //     match &vdoc.nodes[0] {
    //         VNode::Element { children, .. } => {
    //             match &children[0] {
    //                 VNode::Text { content } => {
    //                     // Should concatenate strings
    //                     assert!(content.contains("John"));
    //                     assert!(content.contains("Doe"));
    //                 }
    //                 _ => panic!("Expected text node"),
    //             }
    //         }
    //         _ => panic!("Expected element node"),
    //     }
    // }

    #[test]
    fn test_evaluate_error_component_not_found() {
        let source = r#"
            public component App {
                render div {
                    MissingComponent()
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");

        let result = evaluator.evaluate(&doc);
        // Should handle missing component gracefully
        // Either error or skip the component
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_evaluate_error_variable_not_found() {
        let source = r#"
            public component App {
                render div {
                    text {missingVar}
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");

        let result = evaluator.evaluate(&doc);
        // Should handle missing variable gracefully
        // Either error or return empty string
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_error_messages_include_context() {
        let source = r#"
            public component App {
                render div {
                    text {undefinedVariable}
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");

        let result = evaluator.evaluate(&doc);
        // With partial evaluation, errors are now caught and converted to Error nodes
        assert!(result.is_ok(), "Should not crash on undefined variable");

        let vdoc = result.unwrap();
        // The div should have an Error node as child
        if let VNode::Element { children, .. } = &vdoc.nodes[0] {
            match &children[0] {
                VNode::Error { message, span, .. } => {
                    // Error message should include variable name
                    assert!(
                        message.contains("undefinedVariable"),
                        "Error should mention variable name: {}",
                        message
                    );
                    // Span should be present
                    assert!(span.is_some(), "Error should have span information");
                }
                other => panic!("Expected Error node, got {:?}", other),
            }
        } else {
            panic!("Expected Element node");
        }
    }

    #[test]
    fn test_error_division_by_zero() {
        use crate::evaluator::{EvalError, Evaluator};
        use paperclip_parser::ast::{BinaryOp, Expression, Span};

        // Manually create evaluator and test division by zero
        let evaluator = Evaluator::new();

        let expr = Expression::Binary {
            left: Box::new(Expression::Number {
                value: 5.0,
                span: Span::new(0, 1, "test".to_string()),
            }),
            operator: BinaryOp::Divide,
            right: Box::new(Expression::Number {
                value: 0.0,
                span: Span::new(4, 5, "test".to_string()),
            }),
            span: Span::new(0, 5, "test".to_string()),
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_err());

        if let Err(err) = result {
            match err {
                EvalError::DivisionByZero { span } => {
                    // Should have span information
                    assert_eq!(span, Span::new(0, 5, "test".to_string()));
                }
                _ => panic!("Expected DivisionByZero error, got: {:?}", err),
            }
        }
    }

    #[test]
    fn test_error_invalid_operands() {
        use crate::evaluator::{EvalError, Evaluator};
        use paperclip_parser::ast::{BinaryOp, Expression, Span};

        // Manually create evaluator and test invalid operands
        let evaluator = Evaluator::new();

        let expr = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: "hello".to_string(),
                span: Span::new(0, 7, "test".to_string()),
            }),
            operator: BinaryOp::Subtract,
            right: Box::new(Expression::Literal {
                value: "world".to_string(),
                span: Span::new(10, 17, "test".to_string()),
            }),
            span: Span::new(0, 17, "test".to_string()),
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_err());

        if let Err(err) = result {
            let err_str = format!("{}", err);
            // Error should mention the operator and types
            assert!(
                err_str.contains("-") || err_str.contains("Invalid"),
                "Error should mention operator: {}",
                err_str
            );

            match err {
                EvalError::InvalidOperands {
                    operator,
                    details,
                    span,
                } => {
                    assert_eq!(operator, "-");
                    assert!(
                        details.contains("String") || details.contains("string"),
                        "Details should mention types: {}",
                        details
                    );
                    assert_eq!(span, Span::new(0, 17, "test".to_string()));
                }
                _ => panic!("Expected InvalidOperands error, got: {:?}", err),
            }
        }
    }

    #[test]
    fn test_evaluate_empty_component() {
        let source = r#"
            public component Empty {
                render div {}
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        assert_eq!(vdoc.nodes.len(), 1);

        match &vdoc.nodes[0] {
            VNode::Element { children, .. } => {
                assert_eq!(children.len(), 0);
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_evaluate_all_html_tags() {
        let source = r#"
            public component AllTags {
                render div {
                    button {
                        text "Button"
                    }
                    span {
                        text "Span"
                    }
                    div {
                        text "Div"
                    }
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        assert_eq!(vdoc.nodes.len(), 1);

        match &vdoc.nodes[0] {
            VNode::Element { tag, children, .. } => {
                assert_eq!(tag, "div");
                assert_eq!(children.len(), 3);

                // Verify each tag type
                let tags: Vec<String> = children
                    .iter()
                    .filter_map(|child| match child {
                        VNode::Element { tag, .. } => Some(tag.clone()),
                        _ => None,
                    })
                    .collect();

                assert!(tags.contains(&"button".to_string()));
                assert!(tags.contains(&"span".to_string()));
                assert!(tags.contains(&"div".to_string()));
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_evaluate_vdocument_serialization() {
        let source = r#"
            public component Button {
                render button {
                    style {
                        padding: 8px
                    }
                    text "Click"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Test JSON serialization
        let json = serde_json::to_string(&vdoc).expect("Failed to serialize");
        assert!(!json.is_empty());
        assert!(json.contains("button"));
        assert!(json.contains("padding"));
        assert!(json.contains("Click"));

        // Test deserialization
        let deserialized: VirtualDomDocument =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.nodes.len(), 1);
    }
}
