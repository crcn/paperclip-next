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

// ==================== Annotation Roundtrip Tests ====================

#[test]
fn test_roundtrip_doc_comment_with_frame() {
    let source = r#"/**
 * @frame(x: 100, y: 200, width: 300, height: 400)
 */
component Card {
    render div {
        text "Card"
    }
}"#;

    let doc = parse(source).expect("Failed to parse");
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).expect(&format!("Failed to reparse: {}", serialized));

    assert_eq!(doc.components.len(), reparsed.components.len());
    assert!(reparsed.components[0].doc_comment.is_some());
    assert!(reparsed.components[0].frame.is_some());

    let frame = reparsed.components[0].frame.as_ref().unwrap();
    assert_eq!(frame.x, 100.0);
    assert_eq!(frame.y, 200.0);
    assert_eq!(frame.width, Some(300.0));
    assert_eq!(frame.height, Some(400.0));
}

#[test]
fn test_roundtrip_doc_comment_with_description() {
    let source = r#"/**
 * This is a card component for displaying content.
 * @frame(x: 50, y: 100)
 */
component Card {
    render div {
        text "Card"
    }
}"#;

    let doc = parse(source).expect("Failed to parse");
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).expect(&format!("Failed to reparse: {}", serialized));

    let doc_comment = reparsed.components[0].doc_comment.as_ref().unwrap();
    assert!(doc_comment.description.contains("card component"));
    assert_eq!(doc_comment.annotations.len(), 1);
    assert_eq!(doc_comment.annotations[0].name, "frame");
}

#[test]
fn test_roundtrip_multiple_annotations() {
    let source = r#"/**
 * @frame(x: 100, y: 200)
 * @meta(category: ui, priority: 5)
 * @deprecated
 */
component Button {
    render button {
        text "Click"
    }
}"#;

    let doc = parse(source).expect("Failed to parse");
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).expect(&format!("Failed to reparse: {}", serialized));

    let doc_comment = reparsed.components[0].doc_comment.as_ref().unwrap();
    assert_eq!(doc_comment.annotations.len(), 3);

    let annotation_names: Vec<_> = doc_comment
        .annotations
        .iter()
        .map(|a| a.name.as_str())
        .collect();
    assert!(annotation_names.contains(&"frame"));
    assert!(annotation_names.contains(&"meta"));
    assert!(annotation_names.contains(&"deprecated"));
}

#[test]
fn test_roundtrip_annotation_with_array() {
    let source = r#"/**
 * @config(items: [1, 2, 3], tags: [foo, bar])
 */
component List {
    render ul {
        text "List"
    }
}"#;

    let doc = parse(source).expect("Failed to parse");
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).expect(&format!("Failed to reparse: {}", serialized));

    let doc_comment = reparsed.components[0].doc_comment.as_ref().unwrap();
    assert_eq!(doc_comment.annotations.len(), 1);
    assert_eq!(doc_comment.annotations[0].name, "config");

    let items = doc_comment.annotations[0]
        .params
        .iter()
        .find(|(k, _)| k == "items");
    assert!(items.is_some());
}

#[test]
fn test_roundtrip_annotation_with_boolean() {
    let source = r#"/**
 * @config(locked: true, visible: false)
 */
component Panel {
    render div {
        text "Panel"
    }
}"#;

    let doc = parse(source).expect("Failed to parse");
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).expect(&format!("Failed to reparse: {}", serialized));

    let doc_comment = reparsed.components[0].doc_comment.as_ref().unwrap();
    let config = &doc_comment.annotations[0];

    let locked = config.params.iter().find(|(k, _)| k == "locked").unwrap();
    assert_eq!(locked.1, crate::ast::AnnotationValue::Boolean(true));

    let visible = config.params.iter().find(|(k, _)| k == "visible").unwrap();
    assert_eq!(visible.1, crate::ast::AnnotationValue::Boolean(false));
}

#[test]
fn test_roundtrip_annotation_preserves_frame_backward_compat() {
    let source = r#"/**
 * @frame(x: -50, y: 100.5)
 */
component Card {
    render div {}
}"#;

    let doc = parse(source).expect("Failed to parse");

    // Verify frame field is populated for backward compatibility
    let frame = doc.components[0].frame.as_ref().unwrap();
    assert_eq!(frame.x, -50.0);
    assert_eq!(frame.y, 100.5);

    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    // Frame should still be accessible after roundtrip
    let frame2 = reparsed.components[0].frame.as_ref().unwrap();
    assert_eq!(frame2.x, -50.0);
    assert_eq!(frame2.y, 100.5);
}
