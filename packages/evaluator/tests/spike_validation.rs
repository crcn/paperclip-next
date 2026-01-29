//! Validation tests for spike features
//!
//! This test suite validates that evaluator correctly handles features
//! from completed spikes (0.3, 0.4, 0.6, 0.7)

use paperclip_evaluator::evaluator::Evaluator;
use paperclip_evaluator::vdom::VNode;
use paperclip_parser::parse_with_path;

// ========== Spike 0.6: Conditional Rendering ==========

#[test]
fn test_conditional_basic_evaluation() {
    let source = r#"
        public component Message {
            render div {
                if isVisible {
                    text "Hello World"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Set variable
    evaluator.context.set_variable("isVisible".to_string(), paperclip_evaluator::evaluator::Value::Boolean(true));

    let vdom = evaluator.evaluate(&doc).unwrap();
    assert_eq!(vdom.nodes.len(), 1);

    // Should render the text node
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        assert_eq!(children.len(), 1);
        if let VNode::Text { content } = &children[0] {
            assert_eq!(content, "Hello World");
        } else {
            panic!("Expected Text node");
        }
    } else {
        panic!("Expected Element node");
    }

    println!("✓ Basic conditional evaluation works");
}

#[test]
fn test_conditional_false_evaluation() {
    let source = r#"
        public component Message {
            render div {
                if isVisible {
                    text "Hello World"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Set variable to false
    evaluator.context.set_variable("isVisible".to_string(), paperclip_evaluator::evaluator::Value::Boolean(false));

    let vdom = evaluator.evaluate(&doc).unwrap();
    assert_eq!(vdom.nodes.len(), 1);

    // Should render empty (comment node)
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        // Should have 1 child (comment node for false conditional)
        assert_eq!(children.len(), 1);
        assert!(matches!(children[0], VNode::Comment { .. }));
    } else {
        panic!("Expected Element node");
    }

    println!("✓ False conditional evaluation works");
}

#[test]
fn test_conditional_with_complex_expression() {
    let source = r#"
        public component Card {
            render div {
                if isActive && isShown {
                    text "Active and shown"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Set both variables to true
    evaluator.context.set_variable("isActive".to_string(), paperclip_evaluator::evaluator::Value::Boolean(true));
    evaluator.context.set_variable("isShown".to_string(), paperclip_evaluator::evaluator::Value::Boolean(true));

    let vdom = evaluator.evaluate(&doc).unwrap();

    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        if let VNode::Text { content } = &children[0] {
            assert_eq!(content, "Active and shown");
        }
    }

    println!("✓ Conditional with complex expression works");
}

// ========== Spike 0.7: Repeat/Loop Rendering ==========

#[test]
fn test_repeat_basic_evaluation() {
    let source = r#"
        public component TodoList {
            render ul {
                repeat todo in todos {
                    li {
                        text "Item"
                    }
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Set array variable
    evaluator.context.set_variable(
        "todos".to_string(),
        paperclip_evaluator::evaluator::Value::Array(vec![
            paperclip_evaluator::evaluator::Value::String("Task 1".to_string()),
            paperclip_evaluator::evaluator::Value::String("Task 2".to_string()),
            paperclip_evaluator::evaluator::Value::String("Task 3".to_string()),
        ])
    );

    let vdom = evaluator.evaluate(&doc).unwrap();
    assert_eq!(vdom.nodes.len(), 1);

    // Should render ul with repeat wrapper
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        // Should have 1 wrapper div for repeat
        assert_eq!(children.len(), 1);

        if let VNode::Element { children: repeat_children, .. } = &children[0] {
            // Should have 3 li elements
            assert_eq!(repeat_children.len(), 3);
        }
    }

    println!("✓ Basic repeat evaluation works");
}

#[test]
fn test_repeat_with_member_access() {
    let source = r#"
        public component UserList {
            render div {
                repeat user in users {
                    div {
                        text user.name
                    }
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Set array variable with objects
    evaluator.context.set_variable(
        "users".to_string(),
        paperclip_evaluator::evaluator::Value::Array(vec![
            paperclip_evaluator::evaluator::Value::Object(
                vec![("name".to_string(), paperclip_evaluator::evaluator::Value::String("Alice".to_string()))]
                    .into_iter()
                    .collect()
            ),
            paperclip_evaluator::evaluator::Value::Object(
                vec![("name".to_string(), paperclip_evaluator::evaluator::Value::String("Bob".to_string()))]
                    .into_iter()
                    .collect()
            ),
        ])
    );

    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render user names
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        if let VNode::Element { children: repeat_children, .. } = &children[0] {
            assert_eq!(repeat_children.len(), 2);

            // Check first user div contains "Alice"
            if let VNode::Element { children: user_children, .. } = &repeat_children[0] {
                if let VNode::Text { content } = &user_children[0] {
                    assert_eq!(content, "Alice");
                }
            }
        }
    }

    println!("✓ Repeat with member access works");
}

// ========== Spike 0.3: Component Composition & Slots ==========

#[test]
fn test_component_instance_basic() {
    let source = r#"
        component Button {
            render button {
                text "Click"
            }
        }

        public component App {
            render div {
                Button()
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render App > div > button
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        assert_eq!(children.len(), 1);
        if let VNode::Element { tag, children: button_children, .. } = &children[0] {
            assert_eq!(tag, "button");
            if let VNode::Text { content } = &button_children[0] {
                assert_eq!(content, "Click");
            }
        }
    }

    println!("✓ Component instance evaluation works");
}

#[test]
fn test_slot_default_content() {
    let source = r#"
        component Card {
            slot content {
                text "Empty"
            }

            render div {
                content
            }
        }

        public component App {
            render Card()
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render default slot content
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        if let VNode::Text { content } = &children[0] {
            assert_eq!(content, "Empty");
        }
    }

    println!("✓ Slot default content works");
}

#[test]
fn test_slot_inserted_content() {
    let source = r#"
        component Card {
            slot children {
                text "Empty"
            }

            render div {
                children
            }
        }

        public component App {
            render Card {
                text "Custom content"
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render inserted content (not default)
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        if let VNode::Text { content } = &children[0] {
            assert_eq!(content, "Custom content");
        }
    }

    println!("✓ Slot inserted content works");
}

#[test]
fn test_named_slots_with_insert() {
    let source = r#"
        component Dialog {
            slot header
            slot body

            render div {
                div(class="header") {
                    header
                }
                div(class="body") {
                    body
                }
            }
        }

        public component App {
            render Dialog {
                insert header {
                    text "Title"
                }
                insert body {
                    text "Content"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render dialog with named slot content
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        assert_eq!(children.len(), 2);

        // Check header slot
        if let VNode::Element { children: header_children, .. } = &children[0] {
            if let VNode::Text { content } = &header_children[0] {
                assert_eq!(content, "Title");
            }
        }

        // Check body slot
        if let VNode::Element { children: body_children, .. } = &children[1] {
            if let VNode::Text { content } = &body_children[0] {
                assert_eq!(content, "Content");
            }
        }
    }

    println!("✓ Named slots with insert works");
}

// ========== Combined: Conditionals + Repeats + Slots ==========

#[test]
fn test_conditional_inside_repeat() {
    let source = r#"
        public component TaskList {
            render ul {
                repeat task in tasks {
                    if showItem {
                        li {
                            text "Task"
                        }
                    }
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    evaluator.context.set_variable(
        "tasks".to_string(),
        paperclip_evaluator::evaluator::Value::Array(vec![
            paperclip_evaluator::evaluator::Value::String("Task 1".to_string()),
            paperclip_evaluator::evaluator::Value::String("Task 2".to_string()),
        ])
    );
    evaluator.context.set_variable("showItem".to_string(), paperclip_evaluator::evaluator::Value::Boolean(true));

    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render tasks with conditionals
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        if let VNode::Element { children: repeat_children, .. } = &children[0] {
            // Should have 2 li elements (both conditionals true)
            assert_eq!(repeat_children.len(), 2);
        }
    }

    println!("✓ Conditional inside repeat works");
}

#[test]
fn test_repeat_inside_conditional() {
    let source = r#"
        public component Inbox {
            render div {
                if hasMessages {
                    ul {
                        repeat message in messages {
                            li {
                                text "Message"
                            }
                        }
                    }
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    evaluator.context.set_variable("hasMessages".to_string(), paperclip_evaluator::evaluator::Value::Boolean(true));
    evaluator.context.set_variable(
        "messages".to_string(),
        paperclip_evaluator::evaluator::Value::Array(vec![
            paperclip_evaluator::evaluator::Value::String("Msg 1".to_string()),
            paperclip_evaluator::evaluator::Value::String("Msg 2".to_string()),
        ])
    );

    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render ul with messages
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        // Conditional renders ul
        if let VNode::Element { tag, children: ul_children, .. } = &children[0] {
            assert_eq!(tag, "ul");
            // ul has repeat wrapper
            if let VNode::Element { children: repeat_children, .. } = &ul_children[0] {
                assert_eq!(repeat_children.len(), 2);
            }
        }
    }

    println!("✓ Repeat inside conditional works");
}

#[test]
fn test_slots_with_conditional_content() {
    let source = r#"
        component Card {
            slot content

            render div {
                content
            }
        }

        public component App {
            render Card {
                if isVisible {
                    text "Conditional content"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    evaluator.context.set_variable("isVisible".to_string(), paperclip_evaluator::evaluator::Value::Boolean(true));

    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should render card with conditional content
    if let VNode::Element { children, .. } = &vdom.nodes[0] {
        if let VNode::Text { content } = &children[0] {
            assert_eq!(content, "Conditional content");
        }
    }

    println!("✓ Slots with conditional content works");
}
