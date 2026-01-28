/// Tests for semantic ID generation during evaluation
use crate::evaluator::Evaluator;
use crate::semantic_identity::SemanticSegment;
use crate::vdom::VNode;
use paperclip_parser::parse_with_path;

#[test]
fn test_simple_element_semantic_id() {
    let source = r#"
        public component Button {
            render button {
                text "Click"
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    assert_eq!(vdom.nodes.len(), 1);

    // Extract semantic ID from button element
    if let VNode::Element { semantic_id, tag, .. } = &vdom.nodes[0] {
        assert_eq!(tag, "button");

        // Semantic ID should have segments: Component("Button") -> Element("button")
        assert_eq!(semantic_id.segments.len(), 2);

        // First segment: Component
        match &semantic_id.segments[0] {
            SemanticSegment::Component { name, key } => {
                assert_eq!(name, "Button");
                assert!(key.is_some()); // Auto-generated key
            }
            _ => panic!("Expected Component segment"),
        }

        // Second segment: Element
        match &semantic_id.segments[1] {
            SemanticSegment::Element { tag, ast_id, .. } => {
                assert_eq!(tag, "button");
                assert!(!ast_id.is_empty()); // Should have AST ID
            }
            _ => panic!("Expected Element segment"),
        }

        println!("✓ Semantic ID: {}", semantic_id.to_selector());
    } else {
        panic!("Expected Element node");
    }
}

#[test]
fn test_nested_elements_semantic_id() {
    let source = r#"
        public component Card {
            render div {
                div {
                    text "Content"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Extract outer div
    if let VNode::Element {
        semantic_id: outer_id,
        children,
        ..
    } = &vdom.nodes[0]
    {
        println!("Outer div ID: {}", outer_id.to_selector());

        // Should have: Component -> Element(div)
        assert_eq!(outer_id.segments.len(), 2);

        // Extract inner div
        if let VNode::Element {
            semantic_id: inner_id,
            ..
        } = &children[0]
        {
            println!("Inner div ID: {}", inner_id.to_selector());

            // Should have: Component -> Element(div) -> Element(div)
            assert_eq!(inner_id.segments.len(), 3);

            // Verify parent relationship
            assert!(inner_id.is_descendant_of(outer_id));
        }
    }
}

#[test]
fn test_component_key_auto_generation() {
    let source = r#"
        component Button {
            render button { text "Click" }
        }

        public component App {
            render div {
                Button()
                Button()
                Button()
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Extract div
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        // Should have 3 Button instances
        assert_eq!(children.len(), 3);

        let mut keys = Vec::new();

        for (i, child) in children.iter().enumerate() {
            if let VNode::Element { semantic_id, .. } = child {
                // Each should have unique auto-generated key
                // Path: App -> div -> Button -> button (component renders to button element)
                assert_eq!(semantic_id.segments.len(), 4);

                // Extract Button component segment (at index 2)
                match &semantic_id.segments[2] {
                    SemanticSegment::Component { name, key } => {
                        assert_eq!(name, "Button");
                        assert!(key.is_some());

                        let key_val = key.as_ref().unwrap();
                        println!("Button {} key: {}", i, key_val);

                        // Keys should be unique
                        assert!(!keys.contains(key_val));
                        keys.push(key_val.clone());

                        // Should follow Button-0, Button-1, Button-2 pattern
                        assert!(key_val.starts_with("Button-"));
                    }
                    _ => panic!("Expected Component segment at index 2"),
                }

                // Verify the last segment is the button element
                match &semantic_id.segments[3] {
                    SemanticSegment::Element { tag, .. } => {
                        assert_eq!(tag, "button");
                    }
                    _ => panic!("Expected Element segment at index 3"),
                }
            }
        }

        // Verify we got 3 unique keys
        assert_eq!(keys.len(), 3);
    }
}

#[test]
fn test_element_with_role() {
    let source = r#"
        public component Card {
            render div {
                div(data-role="card-container") {
                    text "Content"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Navigate to outer div
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        // Get inner div with role
        if let VNode::Element { semantic_id, .. } = &children[0] {
            // Find the element segment with role
            let elem_segment = semantic_id
                .segments
                .iter()
                .find_map(|seg| match seg {
                    SemanticSegment::Element { role, .. } => role.as_ref(),
                    _ => None,
                })
                .expect("Should have element segment with role");

            assert_eq!(elem_segment, "card-container");

            println!("✓ Semantic ID with role: {}", semantic_id.to_selector());
        }
    }
}

#[test]
fn test_deterministic_semantic_ids() {
    let source = r#"
        public component Button {
            render button { text "Click" }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();

    // Evaluate twice
    let mut evaluator1 = Evaluator::with_document_id("/test.pc");
    let vdom1 = evaluator1.evaluate(&doc).unwrap();

    let mut evaluator2 = Evaluator::with_document_id("/test.pc");
    let vdom2 = evaluator2.evaluate(&doc).unwrap();

    // Extract semantic IDs
    let id1 = if let VNode::Element { semantic_id, .. } = &vdom1.nodes[0] {
        semantic_id.clone()
    } else {
        panic!("Expected element");
    };

    let id2 = if let VNode::Element { semantic_id, .. } = &vdom2.nodes[0] {
        semantic_id.clone()
    } else {
        panic!("Expected element");
    };

    // Should be identical
    assert_eq!(id1, id2);
    assert_eq!(id1.to_selector(), id2.to_selector());

    println!("✓ Deterministic ID: {}", id1.to_selector());
}
