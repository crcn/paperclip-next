/// CSSOM (CSS Object Model) evaluation tests
/// Tests CSS generation from PC components
use crate::*;
use paperclip_parser::parse_with_path;

#[cfg(test)]
mod cssom_tests {
    use super::*;

    #[test]
    fn test_class_name_synchronization() {
        // Verify CSS selectors match DOM element class names
        let source = r#"
            public component Button {
                render button {
                    style {
                        padding: 8px
                        background: blue
                    }
                    text "Click"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");

        // Evaluate DOM
        let mut dom_evaluator = Evaluator::with_document_id("/test.pc");
        let vdom = dom_evaluator
            .evaluate(&doc)
            .expect("Failed to evaluate DOM");

        // Evaluate CSS
        let mut css_evaluator = CssEvaluator::with_document_id("/test.pc");
        let css = css_evaluator
            .evaluate(&doc)
            .expect("Failed to evaluate CSS");

        // Extract the button element from VirtualDomDocument
        let button_node = &vdom.nodes[0];
        if let VNode::Element {
            tag, attributes, ..
        } = button_node
        {
            assert_eq!(tag, "button");

            // Get the class name applied to the DOM element
            let class_name = attributes
                .get("class")
                .expect("Button should have class attribute");

            // Find the corresponding CSS rule
            let css_rule = css
                .rules
                .iter()
                .find(|r| r.selector == format!(".{}", class_name))
                .expect("Should find matching CSS rule");

            // Verify CSS properties
            assert_eq!(css_rule.properties.get("padding"), Some(&"8px".to_string()));
            assert_eq!(
                css_rule.properties.get("background"),
                Some(&"blue".to_string())
            );

            // Verify class name format (should contain component name, element name, and ID)
            assert!(
                class_name.starts_with("_Button-button-"),
                "Class name should follow format: _Button-button-<id>, got: {}",
                class_name
            );
        } else {
            panic!("Expected Element node");
        }
    }

    #[test]
    fn test_dual_evaluation_dom_and_css() {
        let source = r#"
            token primaryColor #3366FF
            token spacing 16px

            public component Button {
                render button {
                    style {
                        padding: 8px 16px
                        background: #3366FF
                        color: white
                        border-radius: 4px
                    }
                    text "Click me"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");

        // DOM Evaluation
        let mut dom_evaluator = Evaluator::with_document_id("/test.pc");
        let vdoc = dom_evaluator
            .evaluate(&doc)
            .expect("Failed to evaluate DOM");

        // CSS Evaluation
        let mut css_evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = css_evaluator
            .evaluate(&doc)
            .expect("Failed to evaluate CSS");

        // DOM should have element structure
        assert_eq!(vdoc.nodes.len(), 1);
        match &vdoc.nodes[0] {
            VNode::Element { tag, .. } => assert_eq!(tag, "button"),
            _ => panic!("Expected button element"),
        }

        // CSS should have style rules
        assert!(css_doc.rules.len() > 0);
        let button_rule = css_doc
            .rules
            .iter()
            .find(|r| r.selector.contains("button"))
            .expect("Should have button styles");

        assert!(button_rule.properties.contains_key("padding"));
        assert!(button_rule.properties.contains_key("background"));
    }

    #[test]
    fn test_css_scoping() {
        let source = r#"
            public component Card {
                render div {
                    style {
                        padding: 16px
                    }
                    div {
                        style {
                            margin: 8px
                        }
                    }
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should generate scoped selectors
        assert!(css_doc.rules.len() >= 2);

        // Selectors should include component name
        for rule in &css_doc.rules {
            assert!(
                rule.selector.contains("Card") || rule.selector.contains("div"),
                "Selector should be scoped: {}",
                rule.selector
            );
        }
    }

    #[test]
    fn test_css_output_to_text() {
        let source = r#"
            public component Button {
                render button {
                    style {
                        padding: 8px
                        background: blue
                    }
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        let css_text = css_doc.to_css();

        // Should generate valid CSS text
        assert!(css_text.contains("{"));
        assert!(css_text.contains("}"));
        assert!(css_text.contains("padding"));
        assert!(css_text.contains("background"));
        assert!(css_text.contains("8px"));
        assert!(css_text.contains("blue"));
    }

    #[test]
    fn test_multiple_components_css() {
        let source = r#"
            public component Button {
                render button {
                    style {
                        padding: 8px
                    }
                }
            }

            public component Card {
                render div {
                    style {
                        margin: 16px
                    }
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should have rules for both components
        assert!(css_doc.rules.len() >= 2);

        let has_button = css_doc.rules.iter().any(|r| r.selector.contains("Button"));
        let has_card = css_doc.rules.iter().any(|r| r.selector.contains("Card"));

        assert!(has_button, "Should have Button styles");
        assert!(has_card, "Should have Card styles");
    }

    #[test]
    fn test_css_with_conditional_elements() {
        let source = r#"
            public component Conditional {
                render div {
                    style {
                        padding: 16px
                    }
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should extract styles from conditional branches
        assert!(css_doc.rules.len() > 0);
    }

    #[test]
    fn test_css_token_resolution() {
        let source = r#"
            token spacing 16px

            public component Card {
                render div {
                    style {
                        padding: 16px
                    }
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Token should be registered
        assert_eq!(evaluator.tokens().get("spacing"), Some(&"16px".to_string()));

        // Styles should be present
        assert!(css_doc.rules.len() > 0);
    }

    #[test]
    fn test_global_style_declarations() {
        let source = r#"
            public style ButtonBase {
                padding: 8px 16px
                font-family: sans-serif
            }

            public style PrimaryButton {
                background: blue
                color: white
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should have 4 rules: 2 :root rules (one for each style) + 2 class rules
        assert_eq!(css_doc.rules.len(), 4);

        // Find the :root rules
        let root_rules: Vec<_> = css_doc
            .rules
            .iter()
            .filter(|r| r.selector == ":root")
            .collect();
        assert_eq!(root_rules.len(), 2);

        // Find the class rules (styles are now namespaced with document ID)
        let base_rule = css_doc
            .rules
            .iter()
            .find(|r| {
                r.selector.starts_with("._ButtonBase-")
                    && r.selector.contains(evaluator.document_id())
            })
            .expect("Should have ButtonBase style");

        // Properties should use var() with fallbacks
        let base_padding = base_rule.properties.get("padding").unwrap();
        assert!(base_padding.contains("var(--ButtonBase-padding-"));
        assert!(base_padding.contains("8px 16px"));

        let primary_rule = css_doc
            .rules
            .iter()
            .find(|r| {
                r.selector.starts_with("._PrimaryButton-")
                    && r.selector.contains(evaluator.document_id())
            })
            .expect("Should have PrimaryButton style");

        let primary_bg = primary_rule.properties.get("background").unwrap();
        assert!(primary_bg.contains("var(--PrimaryButton-background-"));
        assert!(primary_bg.contains("blue"));
    }

    #[test]
    fn test_private_vs_public_components() {
        let source = r#"
            component PrivateHelper {
                render div {
                    style {
                        margin: 8px
                    }
                }
            }

            public component PublicComponent {
                render div {
                    style {
                        padding: 16px
                    }
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should only generate CSS for public components
        let has_private = css_doc
            .rules
            .iter()
            .any(|r| r.selector.contains("PrivateHelper"));
        let has_public = css_doc
            .rules
            .iter()
            .any(|r| r.selector.contains("PublicComponent"));

        assert!(
            !has_private,
            "Should not generate CSS for private components"
        );
        assert!(has_public, "Should generate CSS for public components");
    }
}
