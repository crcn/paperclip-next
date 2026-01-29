//! Tests for the new attribute syntax with parentheses
//!
//! Validates:
//! - Attributes in parentheses: div(id="btn", class="card")
//! - Expressions: div(width=100 + 20, active=isActive)
//! - Comma separation required
//! - Old syntax rejected: div { id "btn" } is invalid

use paperclip_parser::parse;

#[test]
fn test_attributes_in_parens_string_literals() {
    let source = r#"
        component Test {
            render div(id="container", class="card") {
                text "Hello"
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];
    let root = component.body.as_ref().unwrap();

    if let paperclip_parser::ast::Element::Tag { attributes, .. } = root {
        assert_eq!(attributes.len(), 2);
        assert!(attributes.contains_key("id"));
        assert!(attributes.contains_key("class"));

        // Check that values are string literals
        if let paperclip_parser::ast::Expression::Literal { value, .. } = &attributes["id"] {
            assert_eq!(value, "container");
        } else {
            panic!("Expected string literal for id");
        }
    }
}

#[test]
fn test_attributes_with_expressions() {
    let source = r#"
        component Test {
            render div(width=100 + 20, height=cardHeight) {
                text "Hello"
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];
    let root = component.body.as_ref().unwrap();

    if let paperclip_parser::ast::Element::Tag { attributes, .. } = root {
        assert_eq!(attributes.len(), 2);
        assert!(attributes.contains_key("width"));
        assert!(attributes.contains_key("height"));

        // width should be a binary expression (100 + 20)
        if let paperclip_parser::ast::Expression::Binary { .. } = &attributes["width"] {
            // Success
        } else {
            panic!("Expected binary expression for width");
        }

        // height should be a variable reference
        if let paperclip_parser::ast::Expression::Variable { name, .. } = &attributes["height"] {
            assert_eq!(name, "cardHeight");
        } else {
            panic!("Expected variable for height");
        }
    }
}

#[test]
fn test_attributes_mixed_literals_and_expressions() {
    let source = r#"
        component Test {
            render button(type="submit", disabled=isDisabled, width=100) {
                text "Submit"
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];
    let root = component.body.as_ref().unwrap();

    if let paperclip_parser::ast::Element::Tag { attributes, .. } = root {
        assert_eq!(attributes.len(), 3);
        assert!(attributes.contains_key("type"));
        assert!(attributes.contains_key("disabled"));
        assert!(attributes.contains_key("width"));
    }
}

#[test]
fn test_attributes_only_no_children() {
    let source = r#"
        component Test {
            render img(src="/logo.png", alt="Logo")
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];
    let root = component.body.as_ref().unwrap();

    if let paperclip_parser::ast::Element::Tag {
        attributes,
        children,
        ..
    } = root
    {
        assert_eq!(attributes.len(), 2);
        assert_eq!(children.len(), 0);
    }
}

#[test]
fn test_attributes_with_styles_and_children() {
    let source = r#"
        component Test {
            render div(id="card", class="container") {
                style {
                    padding: 20px
                    background: white
                }
                text "Content"
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];
    let root = component.body.as_ref().unwrap();

    if let paperclip_parser::ast::Element::Tag {
        attributes,
        styles,
        children,
        ..
    } = root
    {
        assert_eq!(attributes.len(), 2);
        assert_eq!(styles.len(), 1);
        assert_eq!(children.len(), 1);
    }
}

#[test]
fn test_component_instances_use_parens() {
    let source = r#"
        component Test {
            render Button(buttonStyle="primary", size="large")
        }

        component Button {
            render button {}
        }
    "#;

    let result = parse(source);
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];
    let root = component.body.as_ref().unwrap();

    if let paperclip_parser::ast::Element::Instance { props, .. } = root {
        assert_eq!(props.len(), 2);
        assert!(props.contains_key("buttonStyle"));
        assert!(props.contains_key("size"));
    }
}

#[test]
fn test_no_comma_between_attributes_fails() {
    let source = r#"
        component Test {
            render div(id="a" class="b") {
                text "Hello"
            }
        }
    "#;

    let result = parse(source);
    // Should fail because comma is required between attributes
    assert!(result.is_err());
}

#[test]
fn test_trailing_comma_allowed() {
    let source = r#"
        component Test {
            render div(id="a", class="b",) {
                text "Hello"
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());
}

#[test]
fn test_empty_parens_allowed() {
    let source = r#"
        component Test {
            render div() {
                text "Hello"
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];
    let root = component.body.as_ref().unwrap();

    if let paperclip_parser::ast::Element::Tag { attributes, .. } = root {
        assert_eq!(attributes.len(), 0);
    }
}

#[test]
fn test_old_syntax_attributes_in_braces_rejected() {
    let source = r#"
        component Test {
            render div {
                id "btn"
                class "primary"
                text "Hello"
            }
        }
    "#;

    let result = parse(source);
    // Old syntax should be rejected - attributes not allowed in braces
    assert!(result.is_err());
}

#[test]
fn test_complex_expressions_in_attributes() {
    let source = r#"
        component Test {
            render div(
                x=index * spacing + offset,
                visible=isActive && isShown
            ) {
                text "Content"
            }
        }
    "#;

    let result = parse(source);
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];
    let root = component.body.as_ref().unwrap();

    if let paperclip_parser::ast::Element::Tag { attributes, .. } = root {
        assert_eq!(attributes.len(), 2);
        assert!(attributes.contains_key("x"));
        assert!(attributes.contains_key("visible"));
    }
}
