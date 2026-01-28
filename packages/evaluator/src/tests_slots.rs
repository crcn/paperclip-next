/// Tests for slot implementation with semantic IDs
use crate::evaluator::Evaluator;
use crate::semantic_identity::{SemanticSegment, SlotVariant};
use crate::vdom::VNode;
use paperclip_parser::parse_with_path;

#[test]
fn test_slot_with_default_content() {
    let source = r#"
        public component Card {
            slot children {
                text "Default content"
            }

            render div {
                children
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render default slot content
    assert_eq!(vdom.nodes.len(), 1);

    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        assert_eq!(children.len(), 1);

        // Should be text with default content
        if let VNode::Text { content } = &children[0] {
            assert_eq!(content, "Default content");
        } else {
            panic!("Expected Text node");
        }
    }
}

#[test]
fn test_slot_with_inserted_content() {
    let source = r#"
        component Card {
            slot children {
                text "Default content"
            }

            render div {
                children
            }
        }

        public component App {
            render Card {
                text "Inserted content"
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // App renders Card with inserted content
    assert_eq!(vdom.nodes.len(), 1);

    // Navigate to Card's div
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        assert_eq!(children.len(), 1);

        // Should be text with INSERTED content (not default)
        if let VNode::Text { content } = &children[0] {
            assert_eq!(content, "Inserted content");
        } else {
            panic!("Expected Text node with inserted content");
        }
    }
}

#[test]
fn test_slot_semantic_id_default() {
    let source = r#"
        public component Card {
            slot children {
                div {
                    text "Default"
                }
            }

            render div {
                children
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Navigate to the slot content
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        if let VNode::Element { semantic_id, .. } = &children[0] {
            println!("Semantic ID: {}", semantic_id.to_selector());

            // Should have Slot segment with Default variant
            let has_default_slot = semantic_id.segments.iter().any(|seg| {
                matches!(
                    seg,
                    SemanticSegment::Slot {
                        name,
                        variant: SlotVariant::Default
                    } if name == "children"
                )
            });

            assert!(
                has_default_slot,
                "Semantic ID should contain Slot segment with Default variant"
            );
        }
    }
}

#[test]
fn test_slot_semantic_id_inserted() {
    let source = r#"
        component Card {
            slot children {
                div {
                    text "Default"
                }
            }

            render div {
                children
            }
        }

        public component App {
            render Card {
                div {
                    text "Inserted"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Navigate to Card's content
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        if let VNode::Element { semantic_id, .. } = &children[0] {
            println!("Semantic ID: {}", semantic_id.to_selector());

            // Should have Slot segment with Inserted variant
            let has_inserted_slot = semantic_id.segments.iter().any(|seg| {
                matches!(
                    seg,
                    SemanticSegment::Slot {
                        name,
                        variant: SlotVariant::Inserted
                    } if name == "children"
                )
            });

            assert!(
                has_inserted_slot,
                "Semantic ID should contain Slot segment with Inserted variant"
            );
        }
    }
}

#[test]
fn test_named_slot() {
    let source = r#"
        public component Card {
            slot header {
                text "Default header"
            }

            slot footer {
                text "Default footer"
            }

            render div {
                header
                div {
                    text "Body"
                }
                footer
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render with default header and footer
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        assert_eq!(children.len(), 3);

        // Header
        if let VNode::Text { content } = &children[0] {
            assert_eq!(content, "Default header");
        } else {
            panic!("Expected header text");
        }

        // Body
        if let VNode::Element { children: body_children, .. } = &children[1] {
            if let VNode::Text { content } = &body_children[0] {
                assert_eq!(content, "Body");
            }
        }

        // Footer
        if let VNode::Text { content } = &children[2] {
            assert_eq!(content, "Default footer");
        } else {
            panic!("Expected footer text");
        }
    }
}

#[test]
fn test_empty_slot() {
    let source = r#"
        public component Card {
            slot children {
            }

            render div {
                children
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render with comment for empty slot
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        assert_eq!(children.len(), 1);

        if let VNode::Comment { content } = &children[0] {
            assert!(content.contains("empty slot"));
        } else {
            panic!("Expected comment node for empty slot");
        }
    }
}

#[test]
fn test_component_instance_syntax_variants() {
    // Test that both Card() and Card work for instances with children
    let source = r#"
        component Card {
            slot children {
                text "Default"
            }

            render div {
                children
            }
        }

        public component App {
            render div {
                Card {
                    text "Without parens"
                }
                Card() {
                    text "With parens"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // App renders a div with two Card instances
    assert_eq!(vdom.nodes.len(), 1);

    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        assert_eq!(children.len(), 2);

        // First Card (without parens)
        if let VNode::Element { children: card1_children, .. } = &children[0] {
            if let VNode::Text { content } = &card1_children[0] {
                assert_eq!(content, "Without parens");
            } else {
                panic!("Expected text in first Card");
            }
        }

        // Second Card (with parens)
        if let VNode::Element { children: card2_children, .. } = &children[1] {
            if let VNode::Text { content } = &card2_children[0] {
                assert_eq!(content, "With parens");
            } else {
                panic!("Expected text in second Card");
            }
        }
    }
}
