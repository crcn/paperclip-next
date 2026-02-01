#[cfg(test)]
mod new_features_tests {
    use crate::parse;

    #[test]
    fn test_parse_script_directive() {
        let source = r#"
            component Button {
                script(src: "./button.tsx", target: "react", name: "MyButton")
                render button {
                    text "Click"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);

        let component = &doc.components[0];
        assert!(component.script.is_some());

        let script = component.script.as_ref().unwrap();
        assert_eq!(script.src, "./button.tsx");
        assert_eq!(script.target, "react");
        assert_eq!(script.name.as_ref().unwrap(), "MyButton");
    }

    #[test]
    fn test_parse_script_directive_minimal() {
        let source = r#"
            component Button {
                script(src: "./button.tsx", target: "react")
                render button
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        let script = doc.components[0].script.as_ref().unwrap();
        assert_eq!(script.src, "./button.tsx");
        assert_eq!(script.target, "react");
        assert!(script.name.is_none());
    }

    #[test]
    fn test_parse_insert_directive() {
        let source = r#"
            component Card {
                slot header
                slot body

                render div {
                    insert header {
                        text "Header Content"
                    }
                    insert body {
                        text "Body Content"
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_parse_element_with_name() {
        let source = r#"
            component Layout {
                render div container (class = "wrapper") {
                    text "Content"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        let component = &doc.components[0];

        if let Some(crate::ast::Element::Tag { tag_name, name, .. }) = &component.body {
            assert_eq!(tag_name, "div");
            assert_eq!(name.as_ref().unwrap(), "container");
        } else {
            panic!("Expected Tag element");
        }
    }

    #[test]
    fn test_parse_style_with_combination_variants() {
        let source = r#"
            component Button {
                variant hover
                variant active
                variant disabled

                render button {
                    style variant hover + active {
                        background: #FF0000
                    }

                    style variant disabled {
                        opacity: 0.5
                    }

                    text "Click"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        let component = &doc.components[0];

        if let Some(crate::ast::Element::Tag { styles, .. }) = &component.body {
            assert_eq!(styles.len(), 2);

            // First style block has combination variants
            assert_eq!(styles[0].variants.len(), 2);
            assert_eq!(styles[0].variants[0], "hover");
            assert_eq!(styles[0].variants[1], "active");

            // Second style block has single variant
            assert_eq!(styles[1].variants.len(), 1);
            assert_eq!(styles[1].variants[0], "disabled");
        } else {
            panic!("Expected Tag element with styles");
        }
    }

    #[test]
    fn test_parse_binary_operations() {
        let source = r#"
            component Math {
                render div {
                    text count + 5
                    text count * 2 + 10
                    text (a + b) * c
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_parse_comparison_operations() {
        let source = r#"
            component Conditional {
                render div {
                    if count > 5 {
                        text "Greater"
                    }
                    if count <= 10 {
                        text "Less or equal"
                    }
                    if count == 0 {
                        text "Zero"
                    }
                    if count != 0 {
                        text "Not zero"
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_parse_logical_operations() {
        let source = r#"
            component Logic {
                render div {
                    if isActive && isEnabled {
                        text "Active and enabled"
                    }
                    if isError || isWarning {
                        text "Has issues"
                    }
                    if isActive && (isEnabled || isForced) {
                        text "Complex condition"
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_parse_function_calls() {
        let source = r#"
            component Computed {
                render div {
                    text formatDate(timestamp)
                    text calculate(a, b, c)
                    text getUser().name
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_parse_member_access_chains() {
        let source = r#"
            component Data {
                render div {
                    text user.profile.name
                    text settings.theme.colors
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_parse_repeat_with_in_keyword() {
        let source = r#"
            component List {
                render div {
                    repeat item in items {
                        div {
                            text item.name
                        }
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_operator_precedence() {
        let source = r#"
            component Precedence {
                render div {
                    text a + b * c
                    text a * b + c
                    text a && b || c
                    text a || b && c
                    text a + b > c * d
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_complex_expression_combinations() {
        let source = r#"
            component Complex {
                render div {
                    if count > 0 && count < 10 || isSpecial {
                        text formatNumber(count + offset * 2)
                    }

                    repeat item in getItems(category, limit) {
                        div {
                            text item.name
                            if item.price > minPrice && item.stock > 0 {
                                text formatPrice(item.price * quantity)
                            }
                        }
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_multiple_style_extends() {
        let source = r#"
            component Button {
                render button {
                    style extends baseButton, primaryStyles {
                        padding: 16px
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        let component = &doc.components[0];

        if let Some(crate::ast::Element::Tag { styles, .. }) = &component.body {
            assert_eq!(styles[0].extends.len(), 2);
            assert_eq!(styles[0].extends[0], "baseButton");
            assert_eq!(styles[0].extends[1], "primaryStyles");
        } else {
            panic!("Expected Tag element with styles");
        }
    }

    #[test]
    fn test_trigger_declaration() {
        let source = r#"
            public trigger hover {
                ":hover",
                ":focus"
            }

            trigger mobile {
                "@media (max-width: 768px)"
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.triggers.len(), 2);

        assert!(doc.triggers[0].public);
        assert_eq!(doc.triggers[0].name, "hover");
        assert_eq!(doc.triggers[0].selectors.len(), 2);

        assert!(!doc.triggers[1].public);
        assert_eq!(doc.triggers[1].name, "mobile");
        assert_eq!(doc.triggers[1].selectors.len(), 1);
    }

    #[test]
    fn test_roundtrip_new_features() {
        let source = r#"component Advanced {
  script(src: "./advanced.tsx", target: "react")
  variant hover
  variant active
  slot header
  render div container (class = "wrapper") {
    style variant hover + active {
      background: #FF0000
    }
    insert header {
      text "Header"
    }
    if count > 0 && isEnabled {
      repeat item in items {
        div {
          text item.name
        }
      }
    }
  }
}
"#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        let serialized = crate::serializer::serialize(&doc);

        println!("=== SERIALIZED OUTPUT ===");
        println!("{}", serialized);
        println!("=== END OUTPUT ===");

        // Should be parseable again
        let reparsed = parse(&serialized);
        assert!(
            reparsed.is_ok(),
            "Reparse error: {:?}\n\nSerialized:\n{}",
            reparsed.err(),
            serialized
        );

        // Structure should match
        let reparsed_doc = reparsed.unwrap();
        assert_eq!(doc.components.len(), reparsed_doc.components.len());
    }

    #[test]
    fn test_template_string_interpolation() {
        let source = r#"
            component Greeting {
                render div {
                    text "Hello ${name}, you have ${count} messages"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        let component = &doc.components[0];

        if let Some(crate::ast::Element::Tag { children, .. }) = &component.body {
            if let crate::ast::Element::Text { content, .. } = &children[0] {
                if let crate::ast::Expression::Template { parts, .. } = content {
                    // "Hello ", ${name}, ", you have ", ${count}, " messages"
                    assert_eq!(parts.len(), 5);
                } else {
                    panic!("Expected Template expression");
                }
            } else {
                panic!("Expected Text element");
            }
        } else {
            panic!("Expected Tag element");
        }
    }

    #[test]
    fn test_nested_function_calls() {
        let source = r#"
            component Nested {
                render div {
                    text format(calculate(a, b), getUnit())
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_parenthesized_expressions() {
        let source = r#"
            component Parens {
                render div {
                    text (a + b) * (c + d)
                    if (count > 0) && (isActive) {
                        text "Active"
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    // ==========================================================
    // Top-level render tests
    // ==========================================================

    #[test]
    fn test_top_level_text() {
        let source = r#"text "Hello, world!""#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 0);
        assert_eq!(doc.renders.len(), 1);

        if let crate::ast::Element::Text { content, styles, .. } = &doc.renders[0] {
            assert!(styles.is_empty());
            if let crate::ast::Expression::Literal { value, .. } = content {
                assert_eq!(value, "Hello, world!");
            } else {
                panic!("Expected Literal expression");
            }
        } else {
            panic!("Expected Text element");
        }
    }

    #[test]
    fn test_top_level_text_with_styles() {
        let source = r#"
            text "Styled text" {
                style {
                    color: red
                    font-size: 16px
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.renders.len(), 1);

        if let crate::ast::Element::Text { content, styles, .. } = &doc.renders[0] {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].properties.get("color"), Some(&"red".to_string()));
            assert_eq!(styles[0].properties.get("font-size"), Some(&"16px".to_string()));

            if let crate::ast::Expression::Literal { value, .. } = content {
                assert_eq!(value, "Styled text");
            } else {
                panic!("Expected Literal expression");
            }
        } else {
            panic!("Expected Text element");
        }
    }

    #[test]
    fn test_top_level_text_with_multiple_style_blocks() {
        let source = r#"
            text "Multi-style" {
                style {
                    color: blue
                }
                style {
                    font-weight: bold
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.renders.len(), 1);

        if let crate::ast::Element::Text { styles, .. } = &doc.renders[0] {
            assert_eq!(styles.len(), 2);
            assert_eq!(styles[0].properties.get("color"), Some(&"blue".to_string()));
            assert_eq!(styles[1].properties.get("font-weight"), Some(&"bold".to_string()));
        } else {
            panic!("Expected Text element");
        }
    }

    #[test]
    fn test_top_level_div() {
        let source = r#"
            div {
                text "Inside div"
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.renders.len(), 1);

        if let crate::ast::Element::Tag { tag_name, children, .. } = &doc.renders[0] {
            assert_eq!(tag_name, "div");
            assert_eq!(children.len(), 1);
        } else {
            panic!("Expected Tag element");
        }
    }

    #[test]
    fn test_top_level_div_with_styles() {
        let source = r#"
            div {
                style {
                    padding: 16px
                    background: white
                }
                text "Styled div"
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.renders.len(), 1);

        if let crate::ast::Element::Tag { tag_name, styles, children, .. } = &doc.renders[0] {
            assert_eq!(tag_name, "div");
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].properties.get("padding"), Some(&"16px".to_string()));
            assert_eq!(styles[0].properties.get("background"), Some(&"white".to_string()));
            assert_eq!(children.len(), 1);
        } else {
            panic!("Expected Tag element");
        }
    }

    #[test]
    fn test_multiple_top_level_renders() {
        let source = r#"
            text "First"
            div {
                text "Second"
            }
            text "Third"
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.renders.len(), 3);
    }

    #[test]
    fn test_mixed_components_and_renders() {
        let source = r#"
            component Card {
                render div {
                    text "Card content"
                }
            }

            text "Standalone text"

            div {
                text "Standalone div"
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
        assert_eq!(doc.renders.len(), 2);
    }

    #[test]
    fn test_top_level_other_tags() {
        let source = r#"
            span {
                text "Span text"
            }
            button {
                text "Click me"
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.renders.len(), 2);

        if let crate::ast::Element::Tag { tag_name, .. } = &doc.renders[0] {
            assert_eq!(tag_name, "span");
        }
        if let crate::ast::Element::Tag { tag_name, .. } = &doc.renders[1] {
            assert_eq!(tag_name, "button");
        }
    }

    #[test]
    fn test_text_with_expression() {
        let source = r#"text someVariable"#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.renders.len(), 1);

        if let crate::ast::Element::Text { content, .. } = &doc.renders[0] {
            if let crate::ast::Expression::Variable { name, .. } = content {
                assert_eq!(name, "someVariable");
            } else {
                panic!("Expected Reference expression");
            }
        } else {
            panic!("Expected Text element");
        }
    }

    #[test]
    fn test_text_with_styled_expression() {
        let source = r#"
            text count {
                style {
                    color: green
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.renders.len(), 1);

        if let crate::ast::Element::Text { content, styles, .. } = &doc.renders[0] {
            assert_eq!(styles.len(), 1);
            if let crate::ast::Expression::Variable { name, .. } = content {
                assert_eq!(name, "count");
            } else {
                panic!("Expected Reference expression");
            }
        } else {
            panic!("Expected Text element");
        }
    }

    #[test]
    fn test_text_with_complex_expression_and_styles() {
        let source = r#"
            text (price * quantity) {
                style {
                    font-weight: bold
                    color: "{primary}"
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.renders.len(), 1);

        if let crate::ast::Element::Text { styles, .. } = &doc.renders[0] {
            assert_eq!(styles.len(), 1);
            assert_eq!(styles[0].properties.get("font-weight"), Some(&"bold".to_string()));
        } else {
            panic!("Expected Text element");
        }
    }
}
