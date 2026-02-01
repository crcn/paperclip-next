use crate::ast::{Element, Expression, TemplatePart};
/// Tests to verify serializer can round-trip all new expression types
use crate::*;

#[test]
fn test_roundtrip_binary_operations() {
    let sources = vec![
        "public component Test { render div { text {a + b} } }",
        "public component Test { render div { text {a - b} } }",
        "public component Test { render div { text {a * b} } }",
        "public component Test { render div { text {a / b} } }",
        "public component Test { render div { text {a == b} } }",
        "public component Test { render div { text {a != b} } }",
        "public component Test { render div { text {a < b} } }",
        "public component Test { render div { text {a <= b} } }",
        "public component Test { render div { text {a > b} } }",
        "public component Test { render div { text {a >= b} } }",
        "public component Test { render div { text {a && b} } }",
        "public component Test { render div { text {a || b} } }",
    ];

    for source in sources {
        let doc = parse(source).expect(&format!("Failed to parse: {}", source));
        let serialized = serialize(&doc);
        let reparsed = parse(&serialized).expect(&format!("Failed to reparse: {}", serialized));

        // Verify structure is preserved
        assert_eq!(doc.components.len(), reparsed.components.len());
        assert_eq!(doc.components[0].name, reparsed.components[0].name);
    }
}

#[test]
fn test_roundtrip_complex_binary_expressions() {
    let source = "public component Test { render div { text {a + b * c} } }";
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    assert_eq!(doc.components.len(), reparsed.components.len());
}

#[test]
fn test_roundtrip_function_calls() {
    let source =
        r#"public component Test { render div { text {formatDate(date, "YYYY-MM-DD")} } }"#;
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    assert_eq!(doc.components.len(), reparsed.components.len());

    // Verify function call structure
    if let Some(Element::Tag { children, .. }) = &reparsed.components[0].body {
        if let Some(Element::Text { content, .. }) = children.first() {
            if let Expression::Call {
                function,
                arguments,
                ..
            } = content
            {
                assert_eq!(function, "formatDate");
                assert_eq!(arguments.len(), 2);
            } else {
                panic!("Expected function call expression");
            }
        }
    }
}

#[test]
fn test_roundtrip_template_strings() {
    let source = r#"public component Test { render div { text "Hello ${name}!" } }"#;
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    assert_eq!(doc.components.len(), reparsed.components.len());

    // Verify template structure
    if let Some(Element::Tag { children, .. }) = &reparsed.components[0].body {
        if let Some(Element::Text { content, .. }) = children.first() {
            if let Expression::Template { parts, .. } = content {
                assert_eq!(parts.len(), 3); // "Hello ", ${name}, "!"
            } else {
                panic!("Expected template expression");
            }
        }
    }
}

#[test]
fn test_roundtrip_template_with_complex_expression() {
    let source = r#"public component Test { render div { text "Count: ${count + 1}" } }"#;
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    assert_eq!(doc.components.len(), reparsed.components.len());
}

#[test]
fn test_roundtrip_combination_variants() {
    let source = r#"
public component Button {
    render button {
        style variant primary + hover {
            background: blue
        }
    }
}
"#;
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    assert_eq!(doc.components.len(), reparsed.components.len());

    // Verify combination variants preserved
    if let Some(Element::Tag { styles, .. }) = &reparsed.components[0].body {
        assert_eq!(styles[0].variants.len(), 2);
        assert_eq!(styles[0].variants[0], "primary");
        assert_eq!(styles[0].variants[1], "hover");
    }
}

#[test]
fn test_roundtrip_trigger_declaration() {
    let source = r#"
trigger hover {
    ":hover",
    ":focus"
}

public component Button {
    render button {
        style variant hover {
            background: blue
        }
    }
}
"#;
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    assert_eq!(doc.triggers.len(), reparsed.triggers.len());
    assert_eq!(doc.triggers[0].name, "hover");
    assert_eq!(doc.triggers[0].selectors.len(), 2);
}

#[test]
fn test_roundtrip_script_directive() {
    let source = r#"
public component Button {
    script(src: "./Button.tsx", target: "react", name: "MyButton")
    render button {
        text "Click"
    }
}
"#;
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    assert!(reparsed.components[0].script.is_some());
    let script = reparsed.components[0].script.as_ref().unwrap();
    assert_eq!(script.src, "./Button.tsx");
    assert_eq!(script.target, "react");
    assert_eq!(script.name, Some("MyButton".to_string()));
}

#[test]
fn test_roundtrip_element_names() {
    let source = r#"
public component Card {
    render div container {
        div header {
            text "Title"
        }
    }
}
"#;
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    // Verify element names preserved
    if let Some(Element::Tag {
        tag_name,
        name,
        children,
        ..
    }) = &reparsed.components[0].body
    {
        assert_eq!(tag_name, "div");
        assert_eq!(name, &Some("container".to_string()));

        if let Some(Element::Tag {
            tag_name: child_tag,
            name: child_name,
            ..
        }) = children.first()
        {
            assert_eq!(child_tag, "div");
            assert_eq!(child_name, &Some("header".to_string()));
        }
    }
}

#[test]
fn test_roundtrip_insert_directive() {
    let source = r#"
public component Card {
    render div {
        insert icon {
            div {
                text "Icon"
            }
        }
    }
}
"#;
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    // Verify insert directive preserved
    if let Some(Element::Tag { children, .. }) = &reparsed.components[0].body {
        if let Some(Element::Insert {
            slot_name, content, ..
        }) = children.first()
        {
            assert_eq!(slot_name, "icon");
            assert_eq!(content.len(), 1);
        } else {
            panic!("Expected Insert element");
        }
    }
}

#[test]
fn test_roundtrip_complex_expression_with_precedence() {
    let source = r#"public component Test { render div { text {a + b * c > d && e || f} } }"#;
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    // Should parse and serialize without errors
    assert_eq!(doc.components.len(), reparsed.components.len());
}

#[test]
fn test_roundtrip_all_features_combined() {
    let source = r#"
trigger hover {
    ":hover"
}

public component Button {
    script(src: "./Button.tsx", target: "react")
    variant primary
    variant disabled
    slot icon

    render button container {
        style variant primary + hover {
            background: blue
            color: white
        }

        insert icon {
            div {
                text "Icon"
            }
        }

        text "Click ${label}: ${count + 1}"

        if active && enabled {
            div {
                text {formatMessage("Active")}
            }
        }
    }
}
"#;
    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    // Verify major structures preserved
    assert_eq!(doc.triggers.len(), reparsed.triggers.len());
    assert_eq!(doc.components.len(), reparsed.components.len());
    assert_eq!(
        doc.components[0].variants.len(),
        reparsed.components[0].variants.len()
    );
    assert_eq!(
        doc.components[0].slots.len(),
        reparsed.components[0].slots.len()
    );
    assert!(reparsed.components[0].script.is_some());
}

#[test]
fn test_roundtrip_top_level_text() {
    let source = r#"text "Hello, world!""#;

    let doc = parse(source).expect("Failed to parse");
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).expect("Failed to reparse");

    assert_eq!(doc.renders.len(), reparsed.renders.len());
    assert_eq!(doc.renders.len(), 1);
}

#[test]
fn test_roundtrip_top_level_text_with_styles() {
    let source = r#"
text "Styled text" {
    style {
        color: red
        font-size: 16px
    }
}
"#;

    let doc = parse(source).expect("Failed to parse");
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).expect(&format!("Failed to reparse: {}", serialized));

    assert_eq!(doc.renders.len(), reparsed.renders.len());

    if let (Element::Text { styles: styles1, .. }, Element::Text { styles: styles2, .. }) =
        (&doc.renders[0], &reparsed.renders[0])
    {
        assert_eq!(styles1.len(), styles2.len());
        assert_eq!(styles1[0].properties.len(), styles2[0].properties.len());
    } else {
        panic!("Expected Text elements");
    }
}

#[test]
fn test_roundtrip_top_level_div() {
    let source = r#"
div {
    style {
        padding: 16px
    }
    text "Inside div"
}
"#;

    let doc = parse(source).expect("Failed to parse");
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).expect(&format!("Failed to reparse: {}", serialized));

    assert_eq!(doc.renders.len(), reparsed.renders.len());
}

#[test]
fn test_roundtrip_mixed_components_and_renders() {
    let source = r#"
component Card {
    render div {
        text "Card"
    }
}

text "Standalone"

div {
    text "Div content"
}
"#;

    let doc = parse(source).expect("Failed to parse");
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).expect(&format!("Failed to reparse: {}", serialized));

    assert_eq!(doc.components.len(), reparsed.components.len());
    assert_eq!(doc.renders.len(), reparsed.renders.len());
    assert_eq!(doc.renders.len(), 2);
}
