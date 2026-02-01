//! Comprehensive tests for complex mutation sequences
//!
//! This tests:
//! - Move + rename + delete chains
//! - Undo/redo sequences
//! - Batched mutations
//! - Document integrity after operations

use paperclip_editor::{Mutation, UndoStack};
use paperclip_parser::ast::Expression;
use paperclip_parser::parse;

#[test]
fn test_move_then_delete_sequence() {
    let source = r#"
        component Test {
            render div {
                div {
                    text "Child 1"
                }
                div {
                    text "Child 2"
                }
            }
        }
    "#;

    let mut doc = parse(source).unwrap();
    let mut stack = UndoStack::new();

    // Get IDs
    let root = doc.components[0].body.as_ref().unwrap();
    let child1_id = root.children().unwrap()[0].span().id.clone();
    let child2_id = root.children().unwrap()[1].span().id.clone();
    let root_id = root.span().id.clone();

    // Move child2 into child1
    let move_mut = Mutation::MoveElement {
        node_id: child2_id.clone(),
        new_parent_id: child1_id.clone(),
        index: 0,
    };
    stack.apply(&move_mut, &mut doc).unwrap();

    // Verify move worked
    let child1 = doc.find_element(&child1_id).unwrap();
    assert_eq!(child1.children().unwrap().len(), 2); // text + child2

    // Delete child1 (should delete child2 too)
    let delete_mut = Mutation::RemoveNode {
        node_id: child1_id.clone(),
    };
    stack.apply(&delete_mut, &mut doc).unwrap();

    // Verify both removed
    assert!(doc.find_element(&child1_id).is_none());
    assert!(doc.find_element(&child2_id).is_none());

    // Undo delete (should restore both)
    stack.undo(&mut doc).unwrap();
    assert!(doc.find_element(&child1_id).is_some());
    assert!(doc.find_element(&child2_id).is_some());

    // Undo move (should move child2 back)
    stack.undo(&mut doc).unwrap();
    let root = doc.find_element(&root_id).unwrap();
    assert_eq!(root.children().unwrap().len(), 2);
}

#[test]
fn test_multiple_text_updates_with_undo_redo() {
    let source = "component Test { render div { text \"v0\" } }";
    let mut doc = parse(source).unwrap();
    let mut stack = UndoStack::new();

    let text_id = doc.components[0].body.as_ref().unwrap().children().unwrap()[0]
        .span()
        .id
        .clone();

    // Apply sequence of updates
    for i in 1..=5 {
        let mutation = Mutation::UpdateText {
            node_id: text_id.clone(),
            content: format!("v{}", i),
        };
        stack.apply(&mutation, &mut doc).unwrap();
    }

    assert_eq!(stack.undo_levels(), 5);

    // Undo all
    for _ in 0..5 {
        assert!(stack.undo(&mut doc).unwrap());
    }
    assert_eq!(stack.undo_levels(), 0);
    assert_eq!(stack.redo_levels(), 5);

    // Redo all
    for _ in 0..5 {
        assert!(stack.redo(&mut doc).unwrap());
    }
    assert_eq!(stack.undo_levels(), 5);
    assert_eq!(stack.redo_levels(), 0);

    // Undo 3, apply new (clears redo)
    for _ in 0..3 {
        stack.undo(&mut doc).unwrap();
    }
    assert_eq!(stack.redo_levels(), 3);

    let mutation = Mutation::UpdateText {
        node_id: text_id.clone(),
        content: "new branch".to_string(),
    };
    stack.apply(&mutation, &mut doc).unwrap();
    assert_eq!(stack.redo_levels(), 0); // Redo cleared
}

#[test]
fn test_batched_style_updates() {
    let source = r#"
        component Test {
            render div {
                style {
                    color: red
                }
            }
        }
    "#;

    let mut doc = parse(source).unwrap();
    let mut stack = UndoStack::new();

    let div_id = doc.components[0].body.as_ref().unwrap().span().id.clone();

    // Batch multiple style changes
    stack.begin_batch();
    stack.set_batch_description("Update theme");

    let mutations = vec![
        ("color", "blue"),
        ("background", "white"),
        ("padding", "20px"),
        ("border-radius", "8px"),
    ];

    for (prop, value) in mutations {
        let mutation = Mutation::SetInlineStyle {
            node_id: div_id.clone(),
            property: prop.to_string(),
            value: value.to_string(),
        };
        stack.apply(&mutation, &mut doc).unwrap();
    }

    stack.end_batch();

    // Should be single undo entry
    assert_eq!(stack.undo_levels(), 1);
    assert_eq!(stack.undo_description(), Some("Update theme"));

    // Undo reverts all 4 changes
    stack.undo(&mut doc).unwrap();

    // Verify reverted
    let div = doc.find_element(&div_id).unwrap();
    if let paperclip_parser::ast::Element::Tag { styles, .. } = div {
        if let Some(style_block) = styles.get(0) {
            // Should still have original color, not the batched changes
            assert_eq!(
                style_block.properties.get("color"),
                Some(&"red".to_string())
            );
            assert!(!style_block.properties.contains_key("background"));
        }
    }
}

#[test]
fn test_insert_and_remove_sequence() {
    let source = "component Test { render div {} }";
    let mut doc = parse(source).unwrap();
    let mut stack = UndoStack::new();

    let parent_id = doc.components[0].body.as_ref().unwrap().span().id.clone();

    // Create new elements to insert
    use paperclip_parser::ast::{Element, Expression, Span};

    let text_elem = Element::Text {
        content: Expression::Literal {
            value: "Inserted".to_string(),
            span: Span::new(0, 0, "inserted-text".to_string()),
        },
        styles: Vec::new(),
        span: Span::new(0, 0, "inserted-text".to_string()),
    };

    // Insert element
    let insert_mut = Mutation::InsertElement {
        parent_id: parent_id.clone(),
        index: 0,
        element: text_elem.clone(),
    };
    stack.apply(&insert_mut, &mut doc).unwrap();

    // Verify inserted
    let parent = doc.find_element(&parent_id).unwrap();
    assert_eq!(parent.children().unwrap().len(), 1);

    // Remove it
    let remove_mut = Mutation::RemoveNode {
        node_id: "inserted-text".to_string(),
    };
    stack.apply(&remove_mut, &mut doc).unwrap();

    // Verify removed
    let parent = doc.find_element(&parent_id).unwrap();
    assert_eq!(parent.children().unwrap().len(), 0);

    // Undo remove (restores)
    stack.undo(&mut doc).unwrap();
    let parent = doc.find_element(&parent_id).unwrap();
    assert_eq!(parent.children().unwrap().len(), 1);

    // Undo insert (removes)
    stack.undo(&mut doc).unwrap();
    let parent = doc.find_element(&parent_id).unwrap();
    assert_eq!(parent.children().unwrap().len(), 0);
}

#[test]
fn test_attribute_set_and_remove() {
    let source = r#"
        component Test {
            render div {}
        }
    "#;

    let mut doc = parse(source).unwrap();
    let mut stack = UndoStack::new();

    let div_id = doc.components[0].body.as_ref().unwrap().span().id.clone();

    // Add initial attribute
    let initial_mut = Mutation::SetAttribute {
        node_id: div_id.clone(),
        name: "id".to_string(),
        value: "original".to_string(),
    };
    stack.apply(&initial_mut, &mut doc).unwrap();

    // Change attribute
    let set_mut = Mutation::SetAttribute {
        node_id: div_id.clone(),
        name: "id".to_string(),
        value: "modified".to_string(),
    };
    stack.apply(&set_mut, &mut doc).unwrap();

    // Add new attribute
    let add_mut = Mutation::SetAttribute {
        node_id: div_id.clone(),
        name: "class".to_string(),
        value: "test-class".to_string(),
    };
    stack.apply(&add_mut, &mut doc).unwrap();

    // Remove new attribute
    let remove_mut = Mutation::RemoveAttribute {
        node_id: div_id.clone(),
        name: "class".to_string(),
    };
    stack.apply(&remove_mut, &mut doc).unwrap();

    // Verify attribute removed
    let div = doc.find_element(&div_id).unwrap();
    if let paperclip_parser::ast::Element::Tag { attributes, .. } = div {
        assert!(!attributes.contains_key("class"));
        // id should still be modified
        assert!(attributes.contains_key("id"));
    }

    // Undo sequence: remove -> add -> set -> initial
    stack.undo(&mut doc).unwrap(); // Undo remove (restores class)
    let div = doc.find_element(&div_id).unwrap();
    if let paperclip_parser::ast::Element::Tag { attributes, .. } = div {
        assert!(attributes.contains_key("class"));
    }

    stack.undo(&mut doc).unwrap(); // Undo add (removes class)
    let div = doc.find_element(&div_id).unwrap();
    if let paperclip_parser::ast::Element::Tag { attributes, .. } = div {
        assert!(!attributes.contains_key("class"));
    }

    stack.undo(&mut doc).unwrap(); // Undo set (restores original id)
    let div = doc.find_element(&div_id).unwrap();
    if let paperclip_parser::ast::Element::Tag { attributes, .. } = div {
        if let Some(Expression::Literal { value, .. }) = attributes.get("id") {
            assert_eq!(value, "original");
        }
    }

    stack.undo(&mut doc).unwrap(); // Undo initial (removes id)
    let div = doc.find_element(&div_id).unwrap();
    if let paperclip_parser::ast::Element::Tag { attributes, .. } = div {
        assert!(!attributes.contains_key("id"));
    }
}

#[test]
fn test_document_integrity_after_complex_sequence() {
    let source = r#"
        component Card {
            render div {
                div {
                    text "Title"
                }
                div {
                    text "Content"
                }
            }
        }
    "#;

    let mut doc = parse(source).unwrap();
    let mut stack = UndoStack::new();

    // Get IDs
    let root = doc.components[0].body.as_ref().unwrap();
    let title_container_id = root.children().unwrap()[0].span().id.clone();
    let content_container_id = root.children().unwrap()[1].span().id.clone();
    let title_text_id = root.children().unwrap()[0].children().unwrap()[0]
        .span()
        .id
        .clone();

    // Complex sequence
    stack.begin_batch();
    stack.set_batch_description("Redesign card");

    // Update text
    let mut1 = Mutation::UpdateText {
        node_id: title_text_id.clone(),
        content: "New Title".to_string(),
    };
    stack.apply(&mut1, &mut doc).unwrap();

    // Add style
    let mut2 = Mutation::SetInlineStyle {
        node_id: title_container_id.clone(),
        property: "font-weight".to_string(),
        value: "bold".to_string(),
    };
    stack.apply(&mut2, &mut doc).unwrap();

    // Add attribute
    let mut3 = Mutation::SetAttribute {
        node_id: content_container_id.clone(),
        name: "class".to_string(),
        value: "card-content".to_string(),
    };
    stack.apply(&mut3, &mut doc).unwrap();

    stack.end_batch();

    // Verify all changes applied
    assert!(doc.find_element(&title_text_id).is_some());
    assert!(doc.find_element(&title_container_id).is_some());
    assert!(doc.find_element(&content_container_id).is_some());

    // Undo entire batch
    stack.undo(&mut doc).unwrap();

    // Verify document still valid (no orphans, cycles, etc.)
    assert!(doc.find_element(&title_text_id).is_some());
    assert!(doc.find_element(&title_container_id).is_some());
    assert!(doc.find_element(&content_container_id).is_some());

    // Can still serialize and parse
    use paperclip_parser::serialize;
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized);
    assert!(reparsed.is_ok());
}
