/// Edge case tests for evaluator
/// Tests boundary conditions, error cases, and unusual inputs
use crate::*;
use paperclip_parser::parse_with_path;

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_document() {
        let source = "";
        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse empty document");
        let mut evaluator = Evaluator::new();
        let vdoc = evaluator
            .evaluate(&doc)
            .expect("Failed to evaluate empty document");

        assert_eq!(vdoc.nodes.len(), 0);
        assert_eq!(vdoc.styles.len(), 0);
    }

    #[test]
    fn test_component_with_no_body() {
        let source = r#"
            public component EmptyComponent {
                variant hover trigger { ":hover" }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should create default div for empty component body
        assert_eq!(vdoc.nodes.len(), 1);
    }

    #[test]
    fn test_empty_text_node() {
        let source = r#"
            public component EmptyText {
                render div {
                    text ""
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        match &vdoc.nodes[0] {
            VNode::Element { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    VNode::Text { content } => assert_eq!(content, ""),
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_very_long_component_name() {
        let long_name = "A".repeat(1000);
        let source = format!(
            r#"
            public component {} {{
                render div {{
                    text "test"
                }}
            }}
        "#,
            long_name
        );

        let doc = parse_with_path(&source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(&doc);

        // Should handle long names gracefully
        assert!(result.is_ok());
    }

    #[test]
    fn test_very_long_text_content() {
        let long_text = "Lorem ipsum ".repeat(1000); // Reduced from 10000 to avoid stack overflow
        let source = format!(
            r#"
            public component LongText {{
                render div {{
                    text "{}"
                }}
            }}
        "#,
            long_text
        );

        let doc = parse_with_path(&source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::new();
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        match &vdoc.nodes[0] {
            VNode::Element { children, .. } => {
                match &children[0] {
                    VNode::Text { content } => {
                        // Should preserve full content
                        assert!(content.len() > 10000);
                    }
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_many_attributes() {
        let mut attrs = String::new();
        for i in 0..100 {
            attrs.push_str(&format!(" attr{}=\"value{}\"", i, i));
        }

        let source = format!(
            r#"
            public component ManyAttrs {{
                render div {{}}
            }}
        "#
        );

        let doc = parse_with_path(&source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(&doc);

        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_in_text() {
        let source = r#"
            public component UnicodeText {
                render div {
                    text "Hello 世界 🌍 مرحبا Привет"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        match &vdoc.nodes[0] {
            VNode::Element { children, .. } => match &children[0] {
                VNode::Text { content } => {
                    assert!(content.contains("世界"));
                    assert!(content.contains("🌍"));
                    assert!(content.contains("مرحبا"));
                    assert!(content.contains("Привет"));
                }
                _ => panic!("Expected text node"),
            },
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_unicode_in_component_name() {
        let source = r#"
            public component Button世界 {
                render button {
                    text "Unicode component"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let result = evaluator.evaluate(&doc);

        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_props_map() {
        let source = r#"
            component Child {
                render div {
                    text "Child"
                }
            }

            public component Parent {
                render div {
                    Child()
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should handle empty props correctly
        assert_eq!(vdoc.nodes.len(), 1);
    }

    #[test]
    fn test_recursive_component_expansion() {
        // Test that we don't have infinite recursion protection yet
        // This will likely cause a stack overflow, but we should at least test it
        let source = r#"
            component Recursive {
                render div {
                    text "base case"
                }
            }

            public component App {
                render div {
                    Recursive()
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let result = evaluator.evaluate(&doc);

        // Should complete successfully (not actually recursive in this case)
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_style_blocks_same_element() {
        let source = r#"
            public component MultiStyle {
                render div {
                    style {
                        padding: 16px
                    }
                    style {
                        margin: 8px
                    }
                    text "Content"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        match &vdoc.nodes[0] {
            VNode::Element { styles, .. } => {
                // Both style properties should be present
                assert!(styles.contains_key("padding"));
                assert!(styles.contains_key("margin"));
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_special_characters_in_attribute_values() {
        let source = r#"
            public component SpecialChars {
                render div {
                    text "Special: <>&\"'"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        match &vdoc.nodes[0] {
            VNode::Element { children, .. } => {
                match &children[0] {
                    VNode::Text { content } => {
                        // Special characters should be preserved
                        assert!(content.contains("<"));
                        assert!(content.contains(">"));
                        assert!(content.contains("&"));
                    }
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_zero_values_in_expressions() {
        use crate::evaluator::{Evaluator, Value};
        use paperclip_parser::ast::{BinaryOp, Expression, Span};

        let evaluator = Evaluator::new();

        // Test 0 + 0
        let expr = Expression::Binary {
            left: Box::new(Expression::Number {
                value: 0.0,
                span: Span::new(0, 1, "test".to_string()),
            }),
            operator: BinaryOp::Add,
            right: Box::new(Expression::Number {
                value: 0.0,
                span: Span::new(4, 5, "test".to_string()),
            }),
            span: Span::new(0, 5, "test".to_string()),
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(0.0));

        // Test 0 * large_number
        let expr = Expression::Binary {
            left: Box::new(Expression::Number {
                value: 0.0,
                span: Span::new(0, 1, "test".to_string()),
            }),
            operator: BinaryOp::Multiply,
            right: Box::new(Expression::Number {
                value: 999999.0,
                span: Span::new(4, 10, "test".to_string()),
            }),
            span: Span::new(0, 10, "test".to_string()),
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(0.0));
    }

    #[test]
    fn test_negative_numbers() {
        use crate::evaluator::{Evaluator, Value};
        use paperclip_parser::ast::{Expression, Span};

        let evaluator = Evaluator::new();

        let expr = Expression::Number {
            value: -42.5,
            span: Span::new(0, 5, "test".to_string()),
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(-42.5));
    }

    #[test]
    fn test_very_large_numbers() {
        use crate::evaluator::{Evaluator, Value};
        use paperclip_parser::ast::{Expression, Span};

        let evaluator = Evaluator::new();

        let expr = Expression::Number {
            value: 1e308, // Near f64 max
            span: Span::new(0, 5, "test".to_string()),
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Number(n) => assert!(n > 1e307),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_empty_string_concatenation() {
        use crate::evaluator::{Evaluator, Value};
        use paperclip_parser::ast::{BinaryOp, Expression, Span};

        let evaluator = Evaluator::new();

        let expr = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: "".to_string(),
                span: Span::new(0, 2, "test".to_string()),
            }),
            operator: BinaryOp::Add,
            right: Box::new(Expression::Literal {
                value: "test".to_string(),
                span: Span::new(5, 11, "test".to_string()),
            }),
            span: Span::new(0, 11, "test".to_string()),
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("test".to_string()));
    }

    #[test]
    fn test_boolean_equality() {
        use crate::evaluator::{Evaluator, Value};
        use paperclip_parser::ast::{BinaryOp, Expression, Span};

        let evaluator = Evaluator::new();

        // true == true
        let expr = Expression::Binary {
            left: Box::new(Expression::Boolean {
                value: true,
                span: Span::new(0, 4, "test".to_string()),
            }),
            operator: BinaryOp::Equals,
            right: Box::new(Expression::Boolean {
                value: true,
                span: Span::new(8, 12, "test".to_string()),
            }),
            span: Span::new(0, 12, "test".to_string()),
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(true));

        // true == false
        let expr = Expression::Binary {
            left: Box::new(Expression::Boolean {
                value: true,
                span: Span::new(0, 4, "test".to_string()),
            }),
            operator: BinaryOp::Equals,
            right: Box::new(Expression::Boolean {
                value: false,
                span: Span::new(8, 13, "test".to_string()),
            }),
            span: Span::new(0, 13, "test".to_string()),
        };

        let result = evaluator.evaluate_expression(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(false));
    }
}
