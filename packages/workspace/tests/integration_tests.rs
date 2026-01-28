/// Integration tests for the complete pipeline
/// Tests parser → evaluator → Virtual DOM flow
use paperclip_evaluator::{Evaluator, VNode};
use paperclip_parser::parse;

#[test]
fn test_integration_simple_button() {
    let source = r#"
        public component Button {
            render button {
                style {
                    padding: 8px 16px
                    background: #3366FF
                    color: white
                }
                text "Click me"
            }
        }
    "#;

    // Parse
    let doc = parse(source).expect("Failed to parse");
    assert_eq!(doc.components.len(), 1);

    // Evaluate
    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");
    assert_eq!(vdoc.nodes.len(), 1);

    // Verify Virtual DOM structure
    match &vdoc.nodes[0] {
        VNode::Element {
            tag,
            styles,
            children,
            ..
        } => {
            assert_eq!(tag, "button");
            assert!(styles.contains_key("padding"));
            assert!(styles.contains_key("background"));
            assert!(styles.contains_key("color"));
            assert_eq!(children.len(), 1);

            match &children[0] {
                VNode::Text { content } => {
                    assert_eq!(content, "Click me");
                }
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }

    // Verify JSON serialization
    let json = serde_json::to_string(&vdoc).expect("Failed to serialize");
    assert!(json.contains("button"));
    assert!(json.contains("padding"));
    assert!(json.contains("Click me"));
}

#[test]
fn test_integration_multiple_components() {
    let source = r#"
        public component Button {
            render button {
                text "Button"
            }
        }

        public component Card {
            render div {
                style {
                    padding: 16px
                }
                text "Card"
            }
        }

        public component Input {
            render input {}
        }
    "#;

    // Parse
    let doc = parse(source).expect("Failed to parse");
    assert_eq!(doc.components.len(), 3);

    // Evaluate
    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");
    assert_eq!(vdoc.nodes.len(), 3);

    // Verify each component rendered correctly
    let tags: Vec<String> = vdoc
        .nodes
        .iter()
        .filter_map(|node| match node {
            VNode::Element { tag, .. } => Some(tag.clone()),
            _ => None,
        })
        .collect();

    assert!(tags.contains(&"button".to_string()));
    assert!(tags.contains(&"div".to_string()));
    assert!(tags.contains(&"input".to_string()));
}

#[test]
fn test_integration_nested_structure() {
    let source = r#"
        public component Card {
            render div {
                style {
                    padding: 16px
                    background: white
                }
                div {
                    style {
                        margin-bottom: 8px
                    }
                    text "Header"
                }
                div {
                    text "Content"
                }
                div {
                    div {
                        text "Nested"
                    }
                }
            }
        }
    "#;

    // Parse
    let doc = parse(source).expect("Failed to parse");

    // Evaluate
    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

    // Verify nested structure
    match &vdoc.nodes[0] {
        VNode::Element { tag, children, .. } => {
            assert_eq!(tag, "div");
            assert_eq!(children.len(), 3);

            // Check first child
            match &children[0] {
                VNode::Element { styles, children, .. } => {
                    assert!(styles.contains_key("margin-bottom"));
                    assert_eq!(children.len(), 1);
                }
                _ => panic!("Expected element"),
            }

            // Check third child (nested)
            match &children[2] {
                VNode::Element { children, .. } => {
                    assert_eq!(children.len(), 1);
                    match &children[0] {
                        VNode::Element { children, .. } => {
                            assert_eq!(children.len(), 1);
                        }
                        _ => panic!("Expected nested element"),
                    }
                }
                _ => panic!("Expected element"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_css_properties_with_dashes() {
    let source = r#"
        public component Box {
            render div {
                style {
                    margin-top: 8px
                    margin-bottom: 16px
                    margin-left: 4px
                    margin-right: 4px
                    padding-top: 12px
                    padding-bottom: 12px
                    line-height: 1.5
                    font-size: 14px
                    font-weight: bold
                    border-radius: 4px
                    border-width: 1px
                    border-style: solid
                    border-color: #ccc
                    background-color: #fff
                    text-align: center
                }
                text "Styled Content"
            }
        }
    "#;

    // Parse
    let doc = parse(source).expect("Failed to parse");

    // Evaluate
    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

    // Verify all CSS properties with dashes are preserved
    match &vdoc.nodes[0] {
        VNode::Element { styles, .. } => {
            assert!(styles.contains_key("margin-top"));
            assert!(styles.contains_key("margin-bottom"));
            assert!(styles.contains_key("margin-left"));
            assert!(styles.contains_key("margin-right"));
            assert!(styles.contains_key("padding-top"));
            assert!(styles.contains_key("padding-bottom"));
            assert!(styles.contains_key("line-height"));
            assert!(styles.contains_key("font-size"));
            assert!(styles.contains_key("font-weight"));
            assert!(styles.contains_key("border-radius"));
            assert!(styles.contains_key("border-width"));
            assert!(styles.contains_key("border-style"));
            assert!(styles.contains_key("border-color"));
            assert!(styles.contains_key("background-color"));
            assert!(styles.contains_key("text-align"));
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_expressions_with_variables() {
    let source = r#"
        public component Greeting {
            render div {
                text {userName}
            }
        }
    "#;

    // Parse
    let doc = parse(source).expect("Failed to parse");

    // Evaluate with context
    let mut evaluator = Evaluator::new();
    evaluator.context.set_variable(
        "userName".to_string(),
        paperclip_evaluator::Value::String("Alice".to_string()),
    );
    evaluator.context.set_variable(
        "count".to_string(),
        paperclip_evaluator::Value::Number(42.0),
    );

    let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

    // Verify expressions were evaluated
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            assert_eq!(children.len(), 1);

            match &children[0] {
                VNode::Text { content } => {
                    assert_eq!(content, "Alice");
                }
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

// Note: Binary expressions in text blocks are not yet fully implemented
// This test is commented out until parser supports complex expressions
// #[test]
// fn test_integration_binary_expressions() {
//     let source = r#"
//         public component Calculator {
//             render div {
//                 text "Sum: {a + b}"
//                 text "Product: {x * y}"
//                 text "Concat: {first + " " + last}"
//             }
//         }
//     "#;
//
//     // Parse
//     let doc = parse(source).expect("Failed to parse");
//
//     // Evaluate with context
//     let mut evaluator = Evaluator::new();
//     evaluator.context.set_variable(
//         "a".to_string(),
//         paperclip_evaluator::Value::Number(5.0),
//     );
//     evaluator.context.set_variable(
//         "b".to_string(),
//         paperclip_evaluator::Value::Number(3.0),
//     );
//     evaluator.context.set_variable(
//         "x".to_string(),
//         paperclip_evaluator::Value::Number(4.0),
//     );
//     evaluator.context.set_variable(
//         "y".to_string(),
//         paperclip_evaluator::Value::Number(7.0),
//     );
//     evaluator.context.set_variable(
//         "first".to_string(),
//         paperclip_evaluator::Value::String("John".to_string()),
//     );
//     evaluator.context.set_variable(
//         "last".to_string(),
//         paperclip_evaluator::Value::String("Doe".to_string()),
//     );
//
//     let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");
//
//     // Verify binary expressions were evaluated
//     match &vdoc.nodes[0] {
//         VNode::Element { children, .. } => {
//             assert_eq!(children.len(), 3);
//
//             // Check sum (5 + 3 = 8)
//             match &children[0] {
//                 VNode::Text { content } => {
//                     assert!(content.contains("8"));
//                 }
//                 _ => panic!("Expected text node"),
//             }
//
//             // Check product (4 * 7 = 28)
//             match &children[1] {
//                 VNode::Text { content } => {
//                     assert!(content.contains("28"));
//                 }
//                 _ => panic!("Expected text node"),
//             }
//
//             // Check concatenation
//             match &children[2] {
//                 VNode::Text { content } => {
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
fn test_integration_member_access() {
    let source = r#"
        public component UserCard {
            render div {
                text {user.name}
                text {user.email}
            }
        }
    "#;

    // Parse
    let doc = parse(source).expect("Failed to parse");

    // Evaluate with object
    let mut evaluator = Evaluator::new();
    let mut user = std::collections::HashMap::new();
    user.insert(
        "name".to_string(),
        paperclip_evaluator::Value::String("Alice".to_string()),
    );
    user.insert(
        "email".to_string(),
        paperclip_evaluator::Value::String("alice@example.com".to_string()),
    );
    evaluator.context.set_variable(
        "user".to_string(),
        paperclip_evaluator::Value::Object(user),
    );

    let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

    // Verify member access worked
    match &vdoc.nodes[0] {
        VNode::Element { children, .. } => {
            assert_eq!(children.len(), 2);

            match &children[0] {
                VNode::Text { content } => {
                    assert_eq!(content, "Alice");
                }
                _ => panic!("Expected text node"),
            }

            match &children[1] {
                VNode::Text { content } => {
                    assert_eq!(content, "alice@example.com");
                }
                _ => panic!("Expected text node"),
            }
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_integration_json_roundtrip() {
    let source = r#"
        public component App {
            render div {
                style {
                    padding: 16px
                }
                button {
                    text "Click"
                }
                span {
                    text "Label"
                }
            }
        }
    "#;

    // Parse
    let doc = parse(source).expect("Failed to parse");

    // Evaluate
    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

    // Serialize to JSON
    let json = serde_json::to_string(&vdoc).expect("Failed to serialize");

    // Deserialize from JSON
    let deserialized: paperclip_evaluator::VDocument =
        serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify roundtrip preserved structure
    assert_eq!(deserialized.nodes.len(), vdoc.nodes.len());

    match (&vdoc.nodes[0], &deserialized.nodes[0]) {
        (
            VNode::Element {
                tag: tag1,
                children: children1,
                ..
            },
            VNode::Element {
                tag: tag2,
                children: children2,
                ..
            },
        ) => {
            assert_eq!(tag1, tag2);
            assert_eq!(children1.len(), children2.len());
        }
        _ => panic!("Expected matching element nodes"),
    }
}

#[test]
fn test_integration_error_recovery() {
    // Invalid syntax should be caught by parser
    let invalid_source = r#"
        public component Invalid {
            render button {
    "#;

    let result = parse(invalid_source);
    assert!(result.is_err());

    // Valid syntax should work
    let valid_source = r#"
        public component Valid {
            render button {
                text "Valid"
            }
        }
    "#;

    let result = parse(valid_source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc);
    assert!(vdoc.is_ok());
}

#[test]
fn test_integration_performance_batch() {
    // Test processing multiple components efficiently
    let source = r#"
        public component C1 { render div { text "1" } }
        public component C2 { render div { text "2" } }
        public component C3 { render div { text "3" } }
        public component C4 { render div { text "4" } }
        public component C5 { render div { text "5" } }
    "#;

    // Parse
    let doc = parse(source).expect("Failed to parse");
    assert_eq!(doc.components.len(), 5);

    // Evaluate
    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");
    assert_eq!(vdoc.nodes.len(), 5);

    // Verify all components processed
    for (i, node) in vdoc.nodes.iter().enumerate() {
        match node {
            VNode::Element { children, .. } => {
                match &children[0] {
                    VNode::Text { content } => {
                        assert_eq!(content, &format!("{}", i + 1));
                    }
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected element node"),
        }
    }
}
