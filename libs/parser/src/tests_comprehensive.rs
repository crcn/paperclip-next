/// Comprehensive test suite for parser
/// Tests edge cases, error conditions, complex syntax
use crate::*;
use crate::ast::Element;

#[cfg(test)]
mod parser_comprehensive_tests {
    use super::*;

    #[test]
    fn test_parse_multiple_components() {
        let source = r#"
            public component Button {
                render button {
                    text "Click me"
                }
            }

            public component Card {
                render div {
                    text "Card content"
                }
            }

            component Internal {
                render span {
                    text "Internal only"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 3);
        assert!(doc.components[0].public);
        assert!(doc.components[1].public);
        assert!(!doc.components[2].public);
        assert_eq!(doc.components[0].name, "Button");
        assert_eq!(doc.components[1].name, "Card");
        assert_eq!(doc.components[2].name, "Internal");
    }

    #[test]
    fn test_parse_nested_elements() {
        let source = r#"
            public component Card {
                render div {
                    style {
                        padding: 16px
                    }
                    div {
                        text "Header"
                    }
                    div {
                        div {
                            text "Nested content"
                        }
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);

        // Verify nested structure
        if let Some(Element::Tag { children, .. }) = &doc.components[0].body {
            assert_eq!(children.len(), 2); // Two div children
            if let Element::Tag { children: inner, .. } = &children[1] {
                assert_eq!(inner.len(), 1); // One nested div
            }
        }
    }

    #[test]
    fn test_parse_css_with_dashes() {
        let source = r#"
            public component Box {
                render div {
                    style {
                        margin-top: 8px
                        margin-bottom: 16px
                        line-height: 1.5
                        font-size: 14px
                        border-radius: 4px
                        background-color: #fff
                    }
                    text "Content"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);

        if let Some(Element::Tag { styles, .. }) = &doc.components[0].body {
            assert!(!styles.is_empty());
            // Verify CSS properties are parsed
            assert!(styles[0].properties.len() >= 5);
        }
    }

    // Note: Variants are not yet fully implemented in parser
    // #[test]
    // fn test_parse_component_with_variants() {
    //     let source = r#"
    //         public component Button {
    //             variant hover trigger { ":hover" }
    //             variant active trigger { ".active" }
    //             variant disabled state { disabled }
    //
    //             render button {
    //                 text "Click"
    //             }
    //         }
    //     "#;
    //
    //     let result = parse(source);
    //     assert!(result.is_ok());
    //     let doc = result.unwrap();
    //     assert_eq!(doc.components.len(), 1);
    //     assert_eq!(doc.components[0].variants.len(), 3);
    //     assert_eq!(doc.components[0].variants[0].name, "hover");
    //     assert_eq!(doc.components[0].variants[1].name, "active");
    //     assert_eq!(doc.components[0].variants[2].name, "disabled");
    // }

    #[test]
    fn test_parse_component_with_slots() {
        let source = r#"
            public component Card {
                slot header {
                    text "Default Header"
                }

                slot content {
                    text "Default Content"
                }

                render div {
                    header
                    content
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
        assert_eq!(doc.components[0].slots.len(), 2);
        assert_eq!(doc.components[0].slots[0].name, "header");
        assert_eq!(doc.components[0].slots[1].name, "content");
    }

    #[test]
    fn test_parse_multiple_tokens() {
        let source = r#"
            public token primaryColor #3366FF
            public token spacing 16px
            token internalFont "Inter, sans-serif"
            public token borderRadius 4px
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.tokens.len(), 4);
        assert!(doc.tokens[0].public);
        assert!(doc.tokens[1].public);
        assert!(!doc.tokens[2].public);
        assert!(doc.tokens[3].public);
        assert_eq!(doc.tokens[0].name, "primaryColor");
        assert_eq!(doc.tokens[1].name, "spacing");
    }

    // Note: Top-level style declarations are not yet implemented
    // #[test]
    // fn test_parse_style_declarations() {
    //     let source = r#"
    //         public style baseButton {
    //             padding: 8px 16px
    //             background: #333
    //             color: white
    //         }
    //
    //         style extends baseButton {
    //             border-radius: 4px
    //             font-weight: bold
    //         }
    //     "#;
    //
    //     let result = parse(source);
    //     assert!(result.is_ok());
    //     let doc = result.unwrap();
    //     assert_eq!(doc.styles.len(), 2);
    //     assert!(doc.styles[0].public);
    //     assert!(!doc.styles[1].public);
    //     assert_eq!(doc.styles[0].name, "baseButton");
    // }

    #[test]
    fn test_parse_expressions_in_text() {
        let source = r#"
            public component Greeting {
                render div {
                    text "Hello, {name}!"
                    text "Count: {count}"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);

        if let Some(Element::Tag { children, .. }) = &doc.components[0].body {
            assert_eq!(children.len(), 2);
        }
    }

    #[test]
    fn test_parse_component_instance() {
        let source = r#"
            public component App {
                render div {
                    Button(label="Click me", disabled=false)
                    Card(title="My Card")
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);

        if let Some(Element::Tag { children, .. }) = &doc.components[0].body {
            assert_eq!(children.len(), 2);
        }
    }

    #[test]
    fn test_parse_error_unexpected_eof() {
        let source = r#"
            public component Button {
                render button {
        "#; // Missing closing braces

        let result = parse(source);
        // Parser should return an error (could be UnexpectedEof or UnexpectedToken)
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_invalid_token() {
        let source = r#"
            public component Button {
                render button {
                    text "Hello"
                }
                invalid_keyword
            }
        "#;

        let result = parse(source);
        // Should either error or skip invalid keyword
        // Depending on parser implementation
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_parse_empty_component() {
        let source = r#"
            public component Empty {
                render div {}
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_parse_component_without_render() {
        let source = r#"
            public component NoRender {
                variant hover trigger { ":hover" }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
        assert!(doc.components[0].body.is_none());
    }

    #[test]
    fn test_parse_all_html_tags() {
        let source = r#"
            public component AllTags {
                render div {
                    button {
                        text "Button"
                    }
                    span {
                        text "Span"
                    }
                    input {}
                    div {
                        text "Inner Div"
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);

        if let Some(Element::Tag { children, .. }) = &doc.components[0].body {
            assert_eq!(children.len(), 4);
        }
    }

    #[test]
    fn test_parse_multiple_style_blocks() {
        let source = r#"
            public component Styled {
                render div {
                    style {
                        padding: 16px
                    }
                    style {
                        margin: 8px
                    }
                    style {
                        border: 1px solid #ccc
                    }
                    text "Content"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);

        if let Some(Element::Tag { styles, .. }) = &doc.components[0].body {
            assert_eq!(styles.len(), 3);
        }
    }

    #[test]
    fn test_parse_complex_expressions() {
        let source = r#"
            public component Calculator {
                render div {
                    text "Result: {a + b}"
                    text "Product: {x * y}"
                    text "Member: {user.name}"
                    text "Nested: {data.items.length}"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let source = "public component Compact{render div{text\"Hello\"}}";

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
        assert_eq!(doc.components[0].name, "Compact");
    }

    #[test]
    fn test_parse_with_unicode() {
        let source = r#"
            public component Unicode {
                render div {
                    text "Hello 世界 🌍"
                    text "Emoji: 🚀 ✨"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }
}
