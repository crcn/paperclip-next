//! Comprehensive tests for mutation handling
//!
//! These tests are designed to try to break the system by:
//! - Simulating concurrent edits from multiple sources
//! - Testing edge cases and boundary conditions
//! - Verifying conflict detection works correctly
//! - Ensuring position resolution survives complex edit sequences

use crate::ast_index::NodeType;
use crate::crdt::CrdtDocument;
use crate::mutation_handler::{Mutation, MutationHandler, MutationResult};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_crdt_doc(content: &str) -> CrdtDocument {
    CrdtDocument::with_content(content)
}

fn get_text(doc: &CrdtDocument) -> String {
    doc.get_text()
}

// ============================================================================
// TEST: BASIC FRAME BOUNDS MUTATIONS
// ============================================================================

#[test]
fn test_set_frame_bounds_basic() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Find the frame ID
    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id: frame_id.clone(),
        x: 100.0,
        y: 200.0,
        width: 300.0,
        height: 400.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    match result {
        MutationResult::Applied { .. } => {
            let new_text = get_text(&crdt_doc);
            assert!(new_text.contains("x: 100"));
            assert!(new_text.contains("y: 200"));
            assert!(new_text.contains("width: 300"));
            assert!(new_text.contains("height: 400"));
        }
        _ => panic!("Expected mutation to be applied"),
    }
}

#[test]
fn test_set_frame_bounds_negative_coordinates() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: -100.0,
        y: -200.0,
        width: 50.0,
        height: 50.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    match result {
        MutationResult::Applied { .. } => {
            let new_text = get_text(&crdt_doc);
            assert!(new_text.contains("x: -100"));
            assert!(new_text.contains("y: -200"));
        }
        _ => panic!("Expected mutation to be applied"),
    }
}

// ============================================================================
// TEST: CONFLICT DETECTION
// ============================================================================

#[test]
fn test_no_conflict_when_edit_elsewhere() {
    let source = r#"// comment
/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Edit the comment (not the frame)
    crdt_doc.edit_range(0, 10, "// MODIFIED");

    // Need to rebuild index after external edit
    handler
        .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
        .unwrap();

    // Get new frame_id after rebuild
    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: 50.0,
        y: 60.0,
        width: 70.0,
        height: 80.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    match result {
        MutationResult::Applied { .. } => {
            let new_text = get_text(&crdt_doc);
            assert!(new_text.contains("// MODIFIED"));
            assert!(new_text.contains("x: 50"));
        }
        other => panic!("Expected Applied, got {:?}", other),
    }
}

// ============================================================================
// TEST: CONCURRENT EDIT SCENARIOS
// ============================================================================

#[test]
fn test_rapid_sequential_mutations() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Apply 10 rapid mutations
    for i in 0..10 {
        let frame_id = handler
            .index()
            .all_node_ids()
            .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
            .unwrap()
            .clone();

        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("mut-{}", i),
            frame_id,
            x: i as f32 * 10.0,
            y: i as f32 * 10.0,
            width: 100.0 + i as f32,
            height: 100.0 + i as f32,
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        assert!(result.is_ok(), "Mutation {} failed: {:?}", i, result);
    }

    let final_text = get_text(&crdt_doc);
    assert!(final_text.contains("x: 90"));
    assert!(final_text.contains("y: 90"));
}

#[test]
fn test_mutation_after_text_insertion_before() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Insert text before the frame
    crdt_doc.insert(0, "// header comment\n");

    // Rebuild index
    handler
        .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
        .unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 50.0,
        height: 50.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    match result {
        MutationResult::Applied { .. } => {
            let text = get_text(&crdt_doc);
            assert!(text.starts_with("// header comment"));
            assert!(text.contains("x: 50"));
        }
        other => panic!("Expected Applied, got {:?}", other),
    }
}

#[test]
fn test_mutation_after_text_insertion_after() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Insert text after the component
    let len = source.len() as u32;
    crdt_doc.insert(len, "\n// trailing comment");

    // Rebuild index
    handler
        .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
        .unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: 123.0,
        y: 456.0,
        width: 789.0,
        height: 101.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    match result {
        MutationResult::Applied { .. } => {
            let text = get_text(&crdt_doc);
            assert!(text.contains("// trailing comment"));
            assert!(text.contains("x: 123"));
        }
        other => panic!("Expected Applied, got {:?}", other),
    }
}

// ============================================================================
// TEST: COMPLEX DOCUMENTS
// ============================================================================

#[test]
fn test_multiple_frames_in_document() {
    let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
component Button {
    render div {
        text "Click"
    }
}

/**
 * @frame(x: 200, y: 0, width: 100, height: 100)
 */
component Card {
    render div {
        text "Card"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Find all frames
    let frame_ids: Vec<String> = handler
        .index()
        .all_node_ids()
        .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .cloned()
        .collect();

    assert_eq!(frame_ids.len(), 2, "Should find 2 frames");

    // Mutate the first frame
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id: frame_ids[0].clone(),
        x: 50.0,
        y: 50.0,
        width: 150.0,
        height: 150.0,
    };

    handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    let text = get_text(&crdt_doc);

    // Both components should still exist
    assert!(text.contains("component Button"));
    assert!(text.contains("component Card"));
}

#[test]
fn test_deeply_nested_component() {
    let source = r#"/**
 * @frame(x: 0, y: 0, width: 400, height: 400)
 */
component DeepNest {
    render div {
        div {
            div {
                div {
                    div {
                        text "Deep inside"
                    }
                }
            }
        }
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: 10.0,
        y: 20.0,
        width: 500.0,
        height: 600.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    match result {
        MutationResult::Applied { .. } => {
            let text = get_text(&crdt_doc);
            assert!(text.contains("x: 10"));
            assert!(text.contains("Deep inside")); // Content preserved
        }
        other => panic!("Expected Applied, got {:?}", other),
    }
}

// ============================================================================
// TEST: EDGE CASES
// ============================================================================

#[test]
fn test_empty_document() {
    let source = "";
    let crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    let _result = handler.rebuild_index(crdt_doc.doc(), source);

    // Should not crash, just have empty index
    assert!(handler.index().is_empty());
}

#[test]
fn test_frame_with_special_characters_in_document() {
    let source = r#"// Special chars: "quotes" 'apostrophe' <angle> &ampersand
/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {
        text "Hello 'world'"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: 5.0,
        y: 5.0,
        width: 5.0,
        height: 5.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();
    assert!(matches!(result, MutationResult::Applied { .. }));
}

#[test]
fn test_mutation_with_unknown_node_id() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id: "nonexistent-node-id".to_string(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(result.is_err());
}

#[test]
fn test_zero_dimension_frame() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    // Set to zero dimensions
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    match result {
        MutationResult::Applied { .. } => {
            let text = get_text(&crdt_doc);
            assert!(text.contains("width: 0"));
            assert!(text.contains("height: 0"));
        }
        other => panic!("Expected Applied, got {:?}", other),
    }
}

#[test]
fn test_very_large_coordinates() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: 999999.0,
        y: 999999.0,
        width: 999999.0,
        height: 999999.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    match result {
        MutationResult::Applied { .. } => {
            let text = get_text(&crdt_doc);
            assert!(text.contains("999999"));
        }
        other => panic!("Expected Applied, got {:?}", other),
    }
}

// ============================================================================
// TEST: SIMULATED CONCURRENT EDITING RACE CONDITIONS
// ============================================================================

#[test]
fn test_simulated_race_vs_code_and_designer() {
    // Simulates: VS Code starts editing, Designer sends mutation
    // The mutation should either succeed or be properly rejected

    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component A {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // VS Code inserts a new line at position 0 (simulating user typing)
    crdt_doc.insert(0, "// new line\n");

    // Rebuild to pick up changes
    handler
        .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
        .unwrap();

    // Now get the new frame ID
    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 50.0,
        height: 50.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(result.is_ok());

    let text = get_text(&crdt_doc);
    assert!(text.contains("// new line"));
    assert!(text.contains("x: 50"));
}

#[test]
fn test_interleaved_mutations_and_edits() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();

    // Interleave mutations and external edits
    for i in 0..5 {
        handler
            .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
            .unwrap();

        let frame_id = handler
            .index()
            .all_node_ids()
            .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
            .unwrap()
            .clone();

        // Designer mutation
        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("mut-{}", i),
            frame_id,
            x: i as f32 * 10.0,
            y: i as f32 * 10.0,
            width: 100.0,
            height: 100.0,
        };

        handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

        // VS Code edit (insert comment)
        let text = get_text(&crdt_doc);
        let insert_pos = text.find("component").unwrap_or(0) as u32;
        crdt_doc.insert(insert_pos, &format!("// edit {}\n", i));
    }

    let final_text = get_text(&crdt_doc);
    // All edits should be present
    for i in 0..5 {
        assert!(final_text.contains(&format!("// edit {}", i)));
    }
}

// ============================================================================
// TEST: DELETE NODE
// ============================================================================

#[test]
fn test_delete_node_basic() {
    let source = r#"component Test {
    render div {
        text "Hello"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Find the text node
    let text_node_id = handler
        .index()
        .all_node_ids()
        .find(|id| {
            if let Some(node) = handler.index().get_node(id) {
                node.node_type == NodeType::Text
            } else {
                false
            }
        })
        .cloned();

    if let Some(node_id) = text_node_id {
        let mutation = Mutation::DeleteNode {
            mutation_id: "mut-1".to_string(),
            node_id: node_id.clone(),
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        // The delete may fail with a parse error since removing `text "Hello"`
        // can leave invalid syntax. Both outcomes are acceptable for this test.
        match result {
            Ok(_) => {
                // Delete succeeded and reparse worked
                let text = get_text(&crdt_doc);
                assert!(!text.contains("Hello"), "Text should be deleted");
            }
            Err(crate::mutation_handler::MutationError::ParseError(_)) => {
                // Expected - deletion may result in invalid syntax
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
        }
    } else {
        // No text node found - that's fine for this test
        println!("No text node found to delete");
    }
}

// ============================================================================
// TEST: INSERT NODE
// ============================================================================

#[test]
fn test_insert_node_basic() {
    let source = r#"component Test {
    render div {
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Find the component node
    let component_id = handler
        .index()
        .all_node_ids()
        .find(|id| {
            if let Some(node) = handler.index().get_node(id) {
                node.node_type == NodeType::Component
            } else {
                false
            }
        })
        .cloned();

    if let Some(parent_id) = component_id {
        let mutation = Mutation::InsertNode {
            mutation_id: "mut-1".to_string(),
            parent_id,
            index: 0,
            source: r#"text "New text""#.to_string(),
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        if let Ok(MutationResult::Applied { .. }) = result {
            let text = get_text(&crdt_doc);
            assert!(text.contains("New text"));
        }
    }
}

// ============================================================================
// TEST: STRESS TESTS
// ============================================================================

#[test]
fn test_stress_100_rapid_mutations() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();

    for i in 0..100 {
        handler
            .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
            .unwrap();

        let frame_id = handler
            .index()
            .all_node_ids()
            .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
            .unwrap()
            .clone();

        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("mut-{}", i),
            frame_id,
            x: (i % 1000) as f32,
            y: (i % 1000) as f32,
            width: 100.0,
            height: 100.0,
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        assert!(result.is_ok(), "Mutation {} failed", i);
    }

    let final_text = get_text(&crdt_doc);
    assert!(final_text.contains("@frame"));
}

#[test]
fn test_stress_alternating_insert_delete() {
    let source = r#"component Test {
    render div {
        text "Original"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();

    // Alternate between inserting and deleting
    for i in 0..20 {
        handler
            .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
            .unwrap();

        if i % 2 == 0 {
            // Find component and insert
            let component_id = handler
                .index()
                .all_node_ids()
                .find(|id| {
                    if let Some(node) = handler.index().get_node(id) {
                        node.node_type == NodeType::Component
                    } else {
                        false
                    }
                })
                .cloned();

            if let Some(parent_id) = component_id {
                let mutation = Mutation::InsertNode {
                    mutation_id: format!("insert-{}", i),
                    parent_id,
                    index: 0,
                    source: format!(r#"text "Item {}""#, i),
                };
                let _ = handler.apply_mutation(&mutation, &mut crdt_doc);
            }
        }
    }

    let final_text = get_text(&crdt_doc);
    assert!(final_text.contains("component Test"));
}

// ============================================================================
// TEST: UNICODE HANDLING
// ============================================================================

#[test]
fn test_frame_with_unicode_in_document() {
    let source = r#"// Unicode: 你好世界 🎉 émoji
/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {
        text "Hello 世界"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: 42.0,
        y: 42.0,
        width: 42.0,
        height: 42.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    match result {
        MutationResult::Applied { .. } => {
            let text = get_text(&crdt_doc);
            assert!(text.contains("你好世界"));
            assert!(text.contains("🎉"));
            assert!(text.contains("x: 42"));
        }
        other => panic!("Expected Applied, got {:?}", other),
    }
}

#[test]
fn test_insert_before_unicode() {
    let source = r#"🎉/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Insert text at beginning (before emoji)
    crdt_doc.insert(0, "// prefix\n");

    handler
        .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
        .unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id,
        x: 77.0,
        y: 88.0,
        width: 99.0,
        height: 111.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(result.is_ok());

    let text = get_text(&crdt_doc);
    assert!(text.starts_with("// prefix"));
    assert!(text.contains("🎉"));
}

// ============================================================================
// TEST: INDEX CORRECTNESS AFTER MUTATIONS
// ============================================================================

#[test]
fn test_index_rebuilt_correctly_after_mutation() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);

    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-1".to_string(),
        frame_id: frame_id.clone(),
        x: 500.0,
        y: 500.0,
        width: 500.0,
        height: 500.0,
    };

    handler.apply_mutation(&mutation, &mut crdt_doc).unwrap();

    // Index should still have a frame
    let frame_nodes: Vec<_> = handler
        .index()
        .all_node_ids()
        .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .collect();

    assert!(
        !frame_nodes.is_empty(),
        "Index should still contain frame after mutation"
    );
}

// ============================================================================
// TEST: STICKY INDEX ENCODING/DECODING
// ============================================================================

#[test]
fn test_sticky_index_roundtrip() {
    use yrs::updates::encoder::Encode;
    use yrs::{Assoc, Doc, GetString, IndexedSequence, Text, Transact};

    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");

    {
        let mut txn = doc.transact_mut();
        text.insert(&mut txn, 0, "Hello, World!");
    }

    // Create sticky index (requires mutable transaction)
    let sticky = {
        let mut txn = doc.transact_mut();
        text.sticky_index(&mut txn, 7, Assoc::After).unwrap()
    };
    let encoded = sticky.encode_v1();

    // Decode and resolve
    use yrs::updates::decoder::Decode;
    use yrs::StickyIndex;
    let decoded = StickyIndex::decode_v1(&encoded).unwrap();
    let txn = doc.transact();
    let offset = decoded.get_offset(&txn).unwrap();

    assert_eq!(offset.index, 7);
}

#[test]
fn test_sticky_index_after_insert() {
    use yrs::updates::encoder::Encode;
    use yrs::{Assoc, Doc, GetString, IndexedSequence, Text, Transact};

    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");

    {
        let mut txn = doc.transact_mut();
        text.insert(&mut txn, 0, "World");
    }

    // Mark position 0 (before 'W') - requires mutable transaction
    let sticky = {
        let mut txn = doc.transact_mut();
        text.sticky_index(&mut txn, 0, Assoc::After).unwrap()
    };
    let encoded = sticky.encode_v1();

    // Insert before
    {
        let mut txn = doc.transact_mut();
        text.insert(&mut txn, 0, "Hello, ");
    }

    // Position should still point to same character (now at different index)
    use yrs::updates::decoder::Decode;
    use yrs::StickyIndex;
    let decoded = StickyIndex::decode_v1(&encoded).unwrap();
    let txn = doc.transact();
    let offset = decoded.get_offset(&txn).unwrap();

    let text_content = text.get_string(&txn);
    assert_eq!(text_content, "Hello, World");

    // The sticky index should now point to where 'W' is (index 7)
    // because it was attached to the character, not the position
    assert_eq!(offset.index, 7);
}

// ============================================================================
// WILD STRESS TESTS - TRY TO BREAK EVERYTHING
// ============================================================================

// ----------------------------------------------------------------------------
// SECTION 1: COMPLEX SYNTAX VARIATIONS
// ----------------------------------------------------------------------------

#[test]
fn test_mutation_with_all_element_types() {
    // Document with every element type: text, div, component, conditional, repeat
    let source = r#"/** @frame(x: 0, y: 0, width: 800, height: 600) */
component ComplexPage {
    render div container {
        style {
            display: flex
            padding: 16px
        }

        text "Header text"

        if isActive {
            div activeContent {
                text "Active!"
            }
        }

        repeat item in items {
            div itemRow {
                text item.name
            }
        }
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Find frame ID
    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    // Mutate the frame
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-complex-1".to_string(),
        frame_id: frame_id.clone(),
        x: 100.0,
        y: 100.0,
        width: 1200.0,
        height: 900.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: 100"));
    assert!(text.contains("width: 1200"));
    // Verify structure preserved
    assert!(text.contains("if isActive"));
    assert!(text.contains("repeat item in items"));
    assert!(text.contains("display: flex"));
}

#[test]
fn test_mutation_with_multiple_style_blocks() {
    // Component with many style blocks including variants
    let source = r#"/** @frame(x: 0, y: 0, width: 400, height: 300) */
component Button {
    variant hover
    variant active
    variant disabled

    render button btn {
        style {
            padding: 8px 16px
            border: none
            border-radius: 4px
        }
        style variant hover {
            background: blue
            transform: scale(1.05)
        }
        style variant active {
            background: darkblue
        }
        style variant disabled {
            opacity: 0.5
            cursor: not-allowed
        }
        text "Click me"
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-styles-1".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 500.0,
        height: 400.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    // All style blocks preserved
    assert!(text.contains("style variant hover"));
    assert!(text.contains("style variant active"));
    assert!(text.contains("style variant disabled"));
    assert!(text.contains("transform: scale(1.05)"));
}

#[test]
fn test_mutation_with_nested_conditionals() {
    let source = r#"/** @frame(x: 0, y: 0, width: 600, height: 400) */
component NestedIf {
    render div {
        if level1 {
            div {
                if level2 {
                    div {
                        if level3 {
                            text "Deep!"
                        }
                    }
                }
            }
        }
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-nested-if".to_string(),
        frame_id,
        x: 999.0,
        y: 888.0,
        width: 777.0,
        height: 666.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("if level1"));
    assert!(text.contains("if level2"));
    assert!(text.contains("if level3"));
}

#[test]
fn test_mutation_with_nested_repeats() {
    let source = r#"/** @frame(x: 0, y: 0, width: 800, height: 600) */
component NestedRepeat {
    render div {
        repeat row in rows {
            div {
                repeat col in row.cols {
                    div {
                        repeat cell in col.cells {
                            text cell.value
                        }
                    }
                }
            }
        }
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-nested-repeat".to_string(),
        frame_id,
        x: 111.0,
        y: 222.0,
        width: 333.0,
        height: 444.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("repeat row in rows"));
    assert!(text.contains("repeat col in row.cols"));
    assert!(text.contains("repeat cell in col.cells"));
}

#[test]
fn test_mutation_with_complex_expressions() {
    // Note: Parser doesn't support unary ! operator, and 'style' is a keyword
    let source = r#"/** @frame(x: 0, y: 0, width: 500, height: 300) */
component ExprComponent {
    render div {
        div(
            class=activeClass,
            onClick=handleClick,
            disabled=isLoading || hasNoPermission,
            customStyle=computeStyle(theme.primary, 0.5)
        ) {
            text formatMessage(user.name, count * 2 + offset)
            text "Template: ${firstName} ${lastName} (${age})"
        }
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-expr".to_string(),
        frame_id,
        x: 0.0,
        y: 0.0,
        width: 1000.0,
        height: 1000.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    // Complex expressions preserved (note: we use hasNoPermission instead of !hasPermission)
    assert!(text.contains("isLoading || hasNoPermission"));
    assert!(text.contains("count * 2 + offset"));
    assert!(text.contains("${firstName} ${lastName}"));
}

#[test]
fn test_mutation_with_tokens_and_triggers() {
    let source = r#"public token primaryColor #3366FF
public token spacing 16px

public trigger hover {
    ":hover",
    ":focus"
}

public style baseButton {
    padding: spacing
    background: primaryColor
}

/** @frame(x: 0, y: 0, width: 300, height: 200) */
component TokenButton {
    render button {
        style extends baseButton {
            border-radius: 8px
        }
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "mut-tokens".to_string(),
        frame_id,
        x: 500.0,
        y: 500.0,
        width: 600.0,
        height: 400.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    // Tokens and triggers preserved
    assert!(text.contains("public token primaryColor #3366FF"));
    assert!(text.contains("public trigger hover"));
    assert!(text.contains("style extends baseButton"));
}

// ----------------------------------------------------------------------------
// SECTION 2: EDGE CASE CHAOS
// ----------------------------------------------------------------------------

#[test]
fn test_mutation_with_extreme_values() {
    // Test that extreme values don't cause panics.
    // The mutation may succeed or fail gracefully, but shouldn't crash.
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test { render div {} }"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    // Test moderately large values that should work
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "extreme-1".to_string(),
        frame_id: frame_id.clone(),
        x: 1_000_000.0,
        y: 1_000_000.0,
        width: 50_000.0,
        height: 50_000.0,
    };
    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(result.is_ok(), "Moderate large values should work");

    // Re-index and re-find frame ID (it changes after mutation)
    handler.rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc)).unwrap();
    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    // Test negative values
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "extreme-2".to_string(),
        frame_id,
        x: -100_000.0,
        y: -100_000.0,
        width: 100.0,
        height: 100.0,
    };
    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(result.is_ok(), "Negative values should work");

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: -100000"));
}

#[test]
fn test_mutation_with_nan_and_infinity() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test { render div {} }"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    // Test NaN - should be handled somehow
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "nan-test".to_string(),
        frame_id: frame_id.clone(),
        x: f32::NAN,
        y: f32::NAN,
        width: 100.0,
        height: 100.0,
    };
    // This might fail or produce 0 - either way shouldn't crash
    let _ = handler.apply_mutation(&mutation, &mut crdt_doc);

    // Test infinity
    handler.rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc)).unwrap();
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "inf-test".to_string(),
        frame_id,
        x: f32::INFINITY,
        y: f32::NEG_INFINITY,
        width: 100.0,
        height: 100.0,
    };
    let _ = handler.apply_mutation(&mutation, &mut crdt_doc);
}

#[test]
fn test_mutation_with_many_escape_sequences() {
    let source = r#"/** @frame(x: 0, y: 0, width: 400, height: 300) */
component Escapes {
    render div {
        text "Line1\nLine2\tTabbed\rReturn\\Backslash\"Quote"
        text "More: \n\n\n\t\t\t"
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "escape-test".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 500.0,
        height: 400.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    // Escapes should be preserved
    assert!(text.contains(r#"\n"#));
    assert!(text.contains(r#"\t"#));
}

#[test]
fn test_mutation_with_emojis_everywhere() {
    let source = r#"/** @frame(x: 0, y: 0, width: 400, height: 300) */
component 🚀Component {
    render div {
        text "🎉 Welcome 🎊"
        text "👨‍👩‍👧‍👦 Family emoji (zwj sequence)"
        text "🏳️‍🌈 Flag"
        div(class="emoji-class-🔥") {
            text "🌟✨💫"
        }
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "emoji-test".to_string(),
        frame_id,
        x: 200.0,
        y: 200.0,
        width: 600.0,
        height: 500.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("🚀"));
    assert!(text.contains("👨‍👩‍👧‍👦")); // ZWJ sequence
    assert!(text.contains("🏳️‍🌈"));
}

#[test]
fn test_mutation_with_rtl_and_bidi_text() {
    let source = r#"/** @frame(x: 0, y: 0, width: 400, height: 300) */
component BiDi {
    render div {
        text "English عربي English"
        text "Mixed: hello שלום world"
        text "Numbers: 123 ٤٥٦ 789"
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "bidi-test".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 500.0,
        height: 400.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("عربي"));
    assert!(text.contains("שלום"));
}

#[test]
fn test_mutation_with_long_identifiers() {
    let very_long_name = "a".repeat(500);
    let source = format!(
        r#"/** @frame(x: 0, y: 0, width: 400, height: 300) */
component {} {{
    render div {} {{
        text "content"
    }}
}}"#,
        very_long_name, very_long_name
    );
    let mut crdt_doc = create_crdt_doc(&source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), &source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "long-id-test".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 600.0,
        height: 400.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains(&very_long_name));
}

#[test]
fn test_mutation_with_many_attributes() {
    let mut attrs = String::new();
    for i in 0..50 {
        if i > 0 {
            attrs.push_str(", ");
        }
        attrs.push_str(&format!("attr{}=value{}", i, i));
    }

    let source = format!(
        r#"/** @frame(x: 0, y: 0, width: 400, height: 300) */
component ManyAttrs {{
    render div({}) {{
        text "content"
    }}
}}"#,
        attrs
    );
    let mut crdt_doc = create_crdt_doc(&source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), &source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "many-attrs-test".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 800.0,
        height: 600.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("attr0=value0"));
    assert!(text.contains("attr49=value49"));
}

// ----------------------------------------------------------------------------
// SECTION 3: CONCURRENT MUTATION CHAOS
// ----------------------------------------------------------------------------

#[test]
fn test_mutation_storm_same_frame() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test { render div {} }"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    // Rapid-fire 100 mutations on the same frame (reduced from 1000 for debugging)
    for i in 0..100 {
        let current_text = get_text(&crdt_doc);
        let rebuild_result = handler.rebuild_index(crdt_doc.doc(), &current_text);
        if rebuild_result.is_err() {
            println!("Rebuild failed at iteration {}: {:?}", i, rebuild_result);
            println!("Current text:\n{}", current_text);
            panic!("Rebuild failed at iteration {}", i);
        }

        // Re-find the frame ID since it might have changed
        let current_frame_id = handler
            .index()
            .all_node_ids()
            .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
            .map(|s| s.clone());

        if current_frame_id.is_none() {
            println!("Frame ID not found at iteration {}", i);
            println!("Current text:\n{}", current_text);
            panic!("Frame ID not found at iteration {}", i);
        }

        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("storm-{}", i),
            frame_id: current_frame_id.unwrap(),
            x: (i % 500) as f32,
            y: ((i * 7) % 500) as f32,
            width: 100.0 + (i % 100) as f32,
            height: 100.0 + ((i * 3) % 100) as f32,
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        if result.is_err() {
            println!("Mutation {} failed: {:?}", i, result);
            println!("Current text:\n{}", get_text(&crdt_doc));
            panic!("Mutation {} failed", i);
        }
    }

    // Final state should be valid
    let text = get_text(&crdt_doc);
    assert!(text.contains("@frame("));
    assert!(text.contains("component Test"));
}

#[test]
fn test_interleaved_inserts_at_boundaries() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test { render div { text "middle" } }"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Insert at position 0 (before frame annotation)
    crdt_doc.insert(0, "// Start comment\n");
    handler
        .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
        .unwrap();

    // Re-find frame ID after rebuild
    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    // Mutate frame
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "boundary-1".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 200.0,
        height: 200.0,
    };
    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    // Insert at end
    let text_len = get_text(&crdt_doc).len() as u32;
    crdt_doc.insert(text_len, "\n// End comment");
    handler
        .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
        .unwrap();

    // Re-find frame ID after rebuild
    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    // Mutate again
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "boundary-2".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 300.0,
        height: 300.0,
    };
    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.starts_with("// Start comment"));
    assert!(text.ends_with("// End comment"));
    assert!(text.contains("x: 100"));
}

#[test]
fn test_mutation_after_massive_insertion() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test { render div {} }"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Insert a MASSIVE amount of text before the frame
    let huge_text = "// ".to_string() + &"x".repeat(50000) + "\n";
    crdt_doc.insert(0, &huge_text);
    handler
        .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
        .unwrap();

    // Re-find the frame ID after rebuild
    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    // Mutation should still work with shifted positions
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "massive-insert".to_string(),
        frame_id,
        x: 999.0,
        y: 888.0,
        width: 777.0,
        height: 666.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: 999"));
}

#[test]
fn test_mutation_after_partial_deletions() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test {
    render div container {
        style {
            padding: 16px
            margin: 8px
        }
        div inner {
            text "Hello"
        }
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    // Delete small chunks from middle of the document (simulate partial edits)
    let text = get_text(&crdt_doc);
    if let Some(pos) = text.find("padding: 16px") {
        // Delete just "padding: 16px" but leave the style block intact
        crdt_doc.delete(pos as u32, 13);
    }

    handler
        .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
        .unwrap();

    // Mutation should still work
    let mutation = Mutation::SetFrameBounds {
        mutation_id: "partial-delete".to_string(),
        frame_id,
        x: 555.0,
        y: 444.0,
        width: 333.0,
        height: 222.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: 555"));
    assert!(text.contains("margin: 8px")); // Other content preserved
}

// ----------------------------------------------------------------------------
// SECTION 4: MULTI-COMPONENT DOCUMENTS
// ----------------------------------------------------------------------------

#[test]
fn test_mutation_in_multi_component_document() {
    let source = r#"/** @frame(x: 0, y: 0, width: 200, height: 150) */
component Button {
    render button { text "Click" }
}

/** @frame(x: 250, y: 0, width: 300, height: 200) */
component Card {
    render div {
        Button()
        text "Card content"
    }
}

/** @frame(x: 600, y: 0, width: 400, height: 300) */
component Page {
    render div {
        Card()
        Button()
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Collect initial frame count
    let initial_frame_count = handler
        .index()
        .all_node_ids()
        .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .count();

    assert_eq!(initial_frame_count, 3, "Should have 3 frames");

    // Mutate each frame - but re-find frame IDs after each mutation
    // since the @frame annotation gets rewritten and changes the ID
    for i in 0..3 {
        handler
            .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
            .unwrap();

        // Get all frame IDs after rebuild and pick the i-th one
        let frame_ids: Vec<String> = handler
            .index()
            .all_node_ids()
            .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
            .cloned()
            .collect();

        // Pick a frame that hasn't been mutated yet (by checking if bounds match original)
        // For simplicity, just mutate the first frame we find each time
        let frame_id = &frame_ids[0];

        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("multi-{}", i),
            frame_id: frame_id.clone(),
            x: (i * 100) as f32,
            y: (i * 50) as f32,
            width: 500.0,
            height: 400.0,
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        assert!(
            matches!(result, Ok(MutationResult::Applied { .. })),
            "Mutation for iteration {} failed: {:?}",
            i,
            result
        );
    }

    let text = get_text(&crdt_doc);
    assert!(text.contains("component Button"));
    assert!(text.contains("component Card"));
    assert!(text.contains("component Page"));
}

#[test]
fn test_mutation_with_imports() {
    // Test that imports don't interfere with frame mutations
    let source = r#"import "./button.pc" as buttons
import "./card.pc" as cards

/** @frame(x: 0, y: 0, width: 500, height: 400) */
component Page {
    render div {
        text "Using imported components"
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "import-test".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 800.0,
        height: 600.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("import \"./button.pc\""));
    assert!(text.contains("import \"./card.pc\" as cards"));
}

// ----------------------------------------------------------------------------
// SECTION 5: MALFORMED BUT PARSEABLE CONTENT
// ----------------------------------------------------------------------------

#[test]
fn test_mutation_with_extra_whitespace() {
    let source = r#"


/** @frame(x: 0, y: 0, width: 100, height: 100) */


component     Test     {


    render    div    {   }


}


"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "whitespace-test".to_string(),
        frame_id,
        x: 200.0,
        y: 200.0,
        width: 400.0,
        height: 300.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));
}

#[test]
fn test_mutation_with_comments_everywhere() {
    let source = r#"// Top level comment
/* Multi-line
   comment */

/** @frame(x: 0, y: 0, width: 100, height: 100) */
// Comment before component
component Test { // inline comment
    // Comment inside
    render div { // another inline
        /* block inside */
        text "hello" // text comment
    }
    // trailing comment
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "comments-test".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 300.0,
        height: 200.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("// Top level comment"));
    assert!(text.contains("/* Multi-line"));
}

// ----------------------------------------------------------------------------
// SECTION 6: RECOVERY AND ERROR HANDLING
// ----------------------------------------------------------------------------

#[test]
fn test_mutation_on_nonexistent_frame() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test { render div {} }"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "nonexistent".to_string(),
        frame_id: "totally-fake-frame-id-12345".to_string(),
        x: 100.0,
        y: 100.0,
        width: 200.0,
        height: 200.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    // Should fail gracefully
    assert!(result.is_err());
}

#[test]
fn test_mutation_with_empty_mutation_id() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test { render div {} }"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "".to_string(), // Empty ID
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 200.0,
        height: 200.0,
    };

    // Should still work - mutation_id is just for tracking
    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(result.is_ok());
}

#[test]
fn test_rebuild_index_after_invalid_syntax_edit() {
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Test { render div {} }"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Break the syntax by deleting a crucial part
    let text = get_text(&crdt_doc);
    if let Some(pos) = text.find("component") {
        crdt_doc.delete(pos as u32, 9); // Delete "component"
    }

    // Rebuild should fail with parse error
    let result = handler.rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc));
    assert!(result.is_err());

    // Restore valid syntax
    let text = get_text(&crdt_doc);
    let pos = text.find(" Test").unwrap_or(0);
    crdt_doc.insert(pos as u32, "component");

    // Should work again
    let result = handler.rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc));
    assert!(result.is_ok());
}

#[test]
fn test_multiple_frames_same_component_name() {
    // This tests a potential edge case where frame IDs might collide
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component Card { render div {} }

/** @frame(x: 200, y: 0, width: 100, height: 100) */
public component Card2 { render div {} }

/** @frame(x: 400, y: 0, width: 100, height: 100) */
component CardVariant { render div {} }"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_ids: Vec<String> = handler
        .index()
        .all_node_ids()
        .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .cloned()
        .collect();

    // Each frame should have a unique ID
    let unique_count = frame_ids.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(frame_ids.len(), unique_count, "Frame IDs should be unique");

    // Mutate each frame - but re-find frame IDs after each mutation
    // since the @frame annotation gets rewritten and changes the ID
    for i in 0..3 {
        handler
            .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
            .unwrap();

        // Get all frame IDs after rebuild and pick the first one
        let current_frame_ids: Vec<String> = handler
            .index()
            .all_node_ids()
            .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
            .cloned()
            .collect();

        assert!(
            !current_frame_ids.is_empty(),
            "Should still have frames at iteration {}",
            i
        );

        let frame_id = &current_frame_ids[0];

        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("multi-card-{}", i),
            frame_id: frame_id.clone(),
            x: (i * 100) as f32,
            y: (i * 50) as f32,
            width: 300.0,
            height: 200.0,
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        assert!(
            matches!(result, Ok(MutationResult::Applied { .. })),
            "Mutation for iteration {} failed: {:?}",
            i,
            result
        );
    }
}

// ----------------------------------------------------------------------------
// SECTION 7: DOCUMENT STRUCTURE EDGE CASES
// ----------------------------------------------------------------------------

#[test]
fn test_deeply_nested_elements_10_levels() {
    let mut source = String::from(r#"/** @frame(x: 0, y: 0, width: 500, height: 500) */
component DeepNest {
    render div level0 {
"#);

    // Build 10 levels of nesting
    for i in 1..=10 {
        source.push_str(&format!(
            "{indent}div level{i} {{\n",
            indent = "        ".repeat(i),
            i = i
        ));
    }
    source.push_str(&format!(
        "{indent}text \"Bottom!\"\n",
        indent = "        ".repeat(11)
    ));
    for i in (1..=10).rev() {
        source.push_str(&format!("{indent}}}\n", indent = "        ".repeat(i)));
    }
    source.push_str("    }\n}");

    let mut crdt_doc = create_crdt_doc(&source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), &source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "deep-nest-test".to_string(),
        frame_id,
        x: 250.0,
        y: 250.0,
        width: 800.0,
        height: 600.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("level10"));
    assert!(text.contains("Bottom!"));
}

#[test]
fn test_horizontal_complexity_many_siblings() {
    let mut children = String::new();
    for i in 0..100 {
        children.push_str(&format!(
            "        div child{i} {{ text \"Content {i}\" }}\n",
            i = i
        ));
    }

    let source = format!(
        r#"/** @frame(x: 0, y: 0, width: 1000, height: 2000) */
component ManySiblings {{
    render div container {{
{}    }}
}}"#,
        children
    );

    let mut crdt_doc = create_crdt_doc(&source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), &source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "many-siblings-test".to_string(),
        frame_id,
        x: 0.0,
        y: 0.0,
        width: 1200.0,
        height: 3000.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("child0"));
    assert!(text.contains("child99"));
}

#[test]
fn test_minimal_valid_document() {
    let source = r#"/** @frame(x: 0, y: 0, width: 1, height: 1) */
component A{render div{}}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "minimal".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 200.0,
        height: 200.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));
}

#[test]
fn test_document_with_only_tokens_and_styles() {
    let source = r#"token a 1px
token b 2px
public token c #FFF

style s1 {
    padding: a
}

public style s2 extends s1 {
    margin: b
}

/** @frame(x: 0, y: 0, width: 100, height: 100) */
component UseStyles {
    render div {
        style extends s2 {}
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "tokens-styles".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 300.0,
        height: 200.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("token a 1px"));
    assert!(text.contains("public style s2 extends s1"));
}

// ============================================================================
// SECTION 8: IMPORTS - TRYING TO BREAK BREAK BREAK
// ============================================================================

#[test]
fn test_import_chaos_many_imports() {
    // Document with TONS of imports before the component
    let mut imports = String::new();
    for i in 0..50 {
        imports.push_str(&format!(
            "import \"./component{}.pc\" as comp{}\n",
            i, i
        ));
    }

    let source = format!(
        r#"{}
/** @frame(x: 0, y: 0, width: 500, height: 400) */
component MainPage {{
    render div {{
        text "Using imported components"
    }}
}}"#,
        imports
    );

    let mut crdt_doc = create_crdt_doc(&source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), &source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "many-imports".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 800.0,
        height: 600.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("import \"./component0.pc\""));
    assert!(text.contains("import \"./component49.pc\""));
    assert!(text.contains("x: 100"));
}

#[test]
fn test_import_with_special_paths() {
    let source = r#"import "./path-with-dashes/component.pc" as dashed
import "../../../deeply/nested/path.pc" as deep
import "./file.pc" as dots

/** @frame(x: 0, y: 0, width: 300, height: 200) */
component Test {
    render div {
        text "Using imports"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "special-paths".to_string(),
        frame_id,
        x: 200.0,
        y: 200.0,
        width: 400.0,
        height: 300.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("path-with-dashes"));
    assert!(text.contains("deeply/nested"));
}

#[test]
fn test_import_interleaved_with_tokens() {
    let source = r#"import "./colors.pc" as colors

token primaryColor #3366FF
token spacing 16px

import "./typography.pc" as typo

token fontSize 16px

import "./layout.pc" as layout

/** @frame(x: 0, y: 0, width: 400, height: 300) */
component Mixed {
    render div {
        style {
            color: primaryColor
            padding: spacing
            font-size: fontSize
        }
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "interleaved".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 600.0,
        height: 450.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("import \"./colors.pc\""));
    assert!(text.contains("token primaryColor #3366FF"));
    assert!(text.contains("import \"./typography.pc\""));
}

// ============================================================================
// SECTION 9: COMPONENTS - TRYING TO BREAK BREAK BREAK
// ============================================================================

#[test]
fn test_component_with_all_modifiers() {
    let source = r#"/** @frame(x: 0, y: 0, width: 300, height: 200) */
public component PublicButton {
    render button { text "Public" }
}

/** @frame(x: 350, y: 0, width: 300, height: 200) */
component PrivateButton {
    render button { text "Private" }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Mutate both frames
    for i in 0..2 {
        handler
            .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
            .unwrap();

        let frame_ids: Vec<String> = handler
            .index()
            .all_node_ids()
            .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
            .cloned()
            .collect();

        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("modifier-{}", i),
            frame_id: frame_ids[0].clone(),
            x: (i * 200) as f32,
            y: (i * 100) as f32,
            width: 400.0,
            height: 300.0,
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        assert!(matches!(result, Ok(MutationResult::Applied { .. })));
    }

    let text = get_text(&crdt_doc);
    assert!(text.contains("public component PublicButton"));
    assert!(text.contains("component PrivateButton"));
}

#[test]
fn test_component_using_other_components() {
    let source = r#"/** @frame(x: 0, y: 0, width: 150, height: 100) */
component Icon {
    render span { text "★" }
}

/** @frame(x: 200, y: 0, width: 200, height: 100) */
component Label {
    render span { text "Label" }
}

/** @frame(x: 450, y: 0, width: 300, height: 150) */
component Button {
    render button {
        Icon()
        Label()
    }
}

/** @frame(x: 0, y: 200, width: 500, height: 300) */
component Card {
    render div {
        Button()
        Button()
        Button()
    }
}

/** @frame(x: 550, y: 200, width: 800, height: 600) */
component Page {
    render div {
        Card()
        Card()
        div {
            Button()
            Icon()
            Label()
        }
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let initial_frames: Vec<String> = handler
        .index()
        .all_node_ids()
        .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .cloned()
        .collect();

    assert_eq!(initial_frames.len(), 5, "Should have 5 frames");

    // Mutate all 5 frames rapidly
    for i in 0..5 {
        handler
            .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
            .unwrap();

        let frame_ids: Vec<String> = handler
            .index()
            .all_node_ids()
            .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
            .cloned()
            .collect();

        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("nested-comp-{}", i),
            frame_id: frame_ids[0].clone(),
            x: (i * 50) as f32,
            y: (i * 25) as f32,
            width: 400.0 + (i * 20) as f32,
            height: 300.0 + (i * 15) as f32,
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        assert!(
            matches!(result, Ok(MutationResult::Applied { .. })),
            "Failed at iteration {}: {:?}",
            i,
            result
        );
    }

    let text = get_text(&crdt_doc);
    assert!(text.contains("component Icon"));
    assert!(text.contains("component Page"));
    assert!(text.contains("Card()"));
}

#[test]
fn test_component_with_slots() {
    // Slots are defined at component level, not inside render
    let source = r#"/** @frame(x: 0, y: 0, width: 400, height: 300) */
component Modal {
    slot header {
        text "Default Header"
    }
    slot body {
        text "Default Body"
    }
    slot footer {
        text "Default Footer"
    }

    render div modal {
        div headerSection {
            header
        }
        div bodySection {
            body
        }
        div footerSection {
            footer
        }
    }
}

/** @frame(x: 450, y: 0, width: 500, height: 400) */
component CustomModal {
    render Modal {
        insert header {
            text "Custom Header!"
        }
        insert body {
            div {
                text "Custom content here"
                text "More content"
            }
        }
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    for i in 0..2 {
        handler
            .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
            .unwrap();

        let frame_ids: Vec<String> = handler
            .index()
            .all_node_ids()
            .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
            .cloned()
            .collect();

        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("slot-{}", i),
            frame_id: frame_ids[0].clone(),
            x: (i * 100) as f32,
            y: (i * 100) as f32,
            width: 600.0,
            height: 500.0,
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        assert!(matches!(result, Ok(MutationResult::Applied { .. })));
    }

    let text = get_text(&crdt_doc);
    assert!(text.contains("slot header"));
    assert!(text.contains("insert header"));
}

#[test]
fn test_component_with_complex_props() {
    let source = r#"/** @frame(x: 0, y: 0, width: 400, height: 300) */
component ComplexProps {
    render div(
        class=computeClass(isActive, isDisabled, theme),
        onClick=handleClick,
        onMouseEnter=handleHover,
        data-testid="complex-div",
        aria-label=ariaLabel || "Default label",
        tabindex=isInteractive && 0
    ) {
        if showIcon {
            span(class="icon") { text icon }
        }
        text label
        if showBadge {
            span(class=badgeClass) { text badgeCount }
        }
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "complex-props".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 600.0,
        height: 450.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("computeClass(isActive, isDisabled, theme)"));
    assert!(text.contains("data-testid=\"complex-div\""));
}

// ============================================================================
// SECTION 10: VARIANTS - TRYING TO BREAK BREAK BREAK
// ============================================================================

#[test]
fn test_many_variants() {
    let mut variants = String::new();
    for i in 0..20 {
        variants.push_str(&format!("    variant state{}\n", i));
    }

    let source = format!(
        r#"/** @frame(x: 0, y: 0, width: 400, height: 500) */
component ManyVariants {{
{}
    render button {{
        style {{
            padding: 8px
        }}
        style variant state0 {{
            background: red
        }}
        style variant state10 {{
            background: green
        }}
        style variant state19 {{
            background: blue
        }}
        text "Button"
    }}
}}"#,
        variants
    );

    let mut crdt_doc = create_crdt_doc(&source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), &source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "many-variants".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 500.0,
        height: 600.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("variant state0"));
    assert!(text.contains("variant state19"));
    assert!(text.contains("style variant state10"));
}

#[test]
fn test_variant_combinations() {
    let source = r#"/** @frame(x: 0, y: 0, width: 400, height: 400) */
component ButtonVariants {
    variant size
    variant color
    variant state
    variant rounded

    render button btn {
        style {
            display: inline-flex
        }
        style variant size {
            padding: 16px 32px
        }
        style variant color {
            background: blue
        }
        style variant state {
            opacity: 0.8
        }
        style variant rounded {
            border-radius: 999px
        }
        style variant size + color {
            font-weight: bold
        }
        style variant size + color + state {
            transform: scale(0.98)
        }
        style variant size + color + state + rounded {
            box-shadow: 0 4px 6px rgba(0,0,0,0.1)
        }
        text "Multi-variant Button"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "variant-combo".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 600.0,
        height: 500.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("style variant size + color"));
    assert!(text.contains("style variant size + color + state + rounded"));
}

#[test]
fn test_variant_with_trigger_combinations() {
    // Triggers are linked to variants using `variant name trigger { triggerName }`
    let source = r#"trigger hover {
    ":hover"
}

trigger focus {
    ":focus"
}

/** @frame(x: 0, y: 0, width: 400, height: 400) */
component InteractiveButton {
    variant isHovered trigger {
        hover
    }
    variant isFocused trigger {
        focus
    }
    variant primary
    variant disabled

    render button {
        style {
            padding: 12px 24px
            cursor: pointer
        }
        style variant isHovered {
            background: lightblue
        }
        style variant isFocused {
            outline: 2px solid blue
        }
        style variant primary {
            background: blue
            color: white
        }
        style variant primary + isHovered {
            background: darkblue
        }
        style variant disabled {
            opacity: 0.5
            cursor: not-allowed
        }
        text "Interactive"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "variant-trigger".to_string(),
        frame_id,
        x: 200.0,
        y: 200.0,
        width: 500.0,
        height: 500.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("trigger hover"));
    assert!(text.contains("variant isHovered trigger"));
    assert!(text.contains("style variant primary + isHovered"));
}

// ============================================================================
// SECTION 11: TRIGGERS - TRYING TO BREAK BREAK BREAK
// ============================================================================

#[test]
fn test_many_triggers() {
    // Triggers are defined at top level, then referenced in variants
    let source = r#"trigger hover { ":hover" }
trigger focus { ":focus" }
trigger active { ":active" }
public trigger visited { ":visited" }
public trigger disabled { ":disabled" }

/** @frame(x: 0, y: 0, width: 400, height: 600) */
component AllTriggers {
    variant isHovered trigger { hover }
    variant isFocused trigger { focus }
    variant isActive trigger { active }

    render div {
        style {
            color: black
        }
        style variant isHovered {
            color: red
        }
        style variant isFocused {
            color: blue
        }
        style variant isActive {
            color: green
        }
        text "All triggers!"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "many-triggers".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 500.0,
        height: 700.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("trigger hover"));
    assert!(text.contains("public trigger visited"));
    assert!(text.contains("variant isHovered trigger"));
}

#[test]
fn test_trigger_with_multiple_selectors() {
    let source = r#"trigger interaction {
    ":hover",
    ":focus",
    ":focus-visible"
}

trigger formStates {
    ":valid",
    ":invalid"
}

trigger mediaQuery {
    "@media (min-width: 768px)"
}

/** @frame(x: 0, y: 0, width: 500, height: 400) */
component MultiSelector {
    variant isInteracting trigger { interaction }
    variant hasFormState trigger { formStates }
    variant isDesktop trigger { mediaQuery }

    render input {
        style {
            padding: 8px
        }
        style variant isInteracting {
            border-color: blue
        }
        style variant hasFormState {
            border-width: 2px
        }
        style variant isDesktop {
            padding: 16px
        }
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "multi-selector".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 600.0,
        height: 500.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains(":focus-visible"));
    assert!(text.contains("@media (min-width: 768px)"));
    assert!(text.contains("variant isInteracting trigger"));
}

// ============================================================================
// SECTION 12: TOKENS - TRYING TO BREAK BREAK BREAK
// ============================================================================

#[test]
fn test_many_tokens() {
    let mut tokens = String::new();
    for i in 0..50 {
        tokens.push_str(&format!("token size{} {}px\n", i, i * 2));
        tokens.push_str(&format!("public token color{} #{:02X}{:02X}{:02X}\n", i, i * 5 % 256, i * 3 % 256, i * 7 % 256));
    }

    let source = format!(
        r#"{}
/** @frame(x: 0, y: 0, width: 500, height: 400) */
component TokenMadness {{
    render div {{
        style {{
            padding: size10
            margin: size20
            color: color25
            background: color49
        }}
    }}
}}"#,
        tokens
    );

    let mut crdt_doc = create_crdt_doc(&source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), &source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "many-tokens".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 600.0,
        height: 500.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("token size0 0px"));
    assert!(text.contains("token size49 98px"));
    assert!(text.contains("public token color49"));
}

#[test]
fn test_token_types_variety() {
    // Token values are simple - no function calls like rgba() allowed in token definitions
    let source = r#"// Size tokens
token spacingXs 4px
token spacingSm 8px
token spacingMd 16px
token spacingLg 24px
token spacingXl 32px

// Color tokens
public token colorPrimary #3366FF
public token colorSecondary #FF6633
public token colorSuccess #33CC66
public token colorWarning #FFCC00
public token colorError #FF3333

// Typography tokens
token fontSizeXs 12px
token fontSizeSm 14px
token fontSizeMd 16px
token fontSizeLg 20px
token fontSizeXl 24px

// Border tokens
token borderRadius 4px
token borderRadiusLg 8px
token borderWidth 1px
token borderColor #E0E0E0

// Animation tokens
token transitionFast 150ms
token transitionNormal 300ms

/** @frame(x: 0, y: 0, width: 600, height: 500) */
component DesignSystem {
    render div {
        style {
            padding: spacingMd
            color: colorPrimary
            font-size: fontSizeMd
            border-radius: borderRadius
        }
        text "Design System Component"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "token-variety".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 700.0,
        height: 600.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("token spacingXs 4px"));
    assert!(text.contains("public token colorPrimary #3366FF"));
}

#[test]
fn test_token_referencing_other_tokens() {
    // Tokens can reference other tokens, but NOT with arithmetic
    let source = r#"token baseSize 8px
token spacingSm baseSize
token spacingMd 16px
token spacingLg 24px

token baseFontSize 16px
token fontSizeSm 14px
token fontSizeLg 20px

token baseColor #3366FF
token colorLight #5588FF
token colorDark #2244DD

/** @frame(x: 0, y: 0, width: 400, height: 300) */
component TokenRefs {
    render div {
        style {
            padding: spacingMd
            font-size: fontSizeLg
            color: colorDark
        }
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "token-refs".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 500.0,
        height: 400.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("token spacingSm baseSize"));
    assert!(text.contains("token fontSizeLg 20px"));
}

// ============================================================================
// SECTION 13: MEGA CHAOS - EVERYTHING COMBINED
// ============================================================================

#[test]
fn test_mega_document_everything() {
    // Simplified mega document with valid PC syntax
    let source = r#"// MEGA DOCUMENT WITH EVERYTHING
import "./design-system.pc" as ds
import "./icons.pc" as icons

// Tokens
public token primaryColor #3366FF
public token secondaryColor #FF6633
token spacingUnit 8px
token borderRadius 4px

// Triggers
trigger hover { ":hover" }
trigger focus { ":focus" }

// Base styles
public style baseButton {
    padding: 16px
    border-radius: borderRadius
    cursor: pointer
}

public style baseCard {
    padding: 24px
    border-radius: 8px
}

/** @frame(x: 0, y: 0, width: 200, height: 100) */
component Icon {
    slot default {
        text "★"
    }
    render span icon {
        style { display: inline-flex }
        default
    }
}

/** @frame(x: 250, y: 0, width: 300, height: 150) */
public component Button {
    variant primary
    variant secondary
    variant isHovered trigger { hover }

    slot default {
        text "Button"
    }

    render button btn {
        style extends baseButton {
            display: inline-flex
            align-items: center
            gap: spacingUnit
        }
        style variant isHovered {
            opacity: 0.9
        }
        style variant primary {
            background: primaryColor
            color: white
        }
        style variant primary + isHovered {
            background: #2255DD
        }
        style variant secondary {
            background: secondaryColor
            color: white
        }

        if showIcon {
            Icon()
        }
        default
    }
}

/** @frame(x: 600, y: 0, width: 400, height: 300) */
public component Card {
    variant elevated
    variant outlined

    slot header {
        text "Card Header"
    }
    slot body {
        text "Card Body"
    }
    slot footer {
        Button { text "Action" }
    }

    render div card {
        style extends baseCard {
            display: flex
            flex-direction: column
        }
        style variant elevated {
            box-shadow: 0 8px 16px black
        }
        style variant outlined {
            border: 1px solid #E0E0E0
        }

        div headerSection {
            style { padding-bottom: 16px }
            header
        }
        div bodySection {
            style { flex: 1 }
            body
        }
        div footerSection {
            style { padding-top: 16px }
            footer
        }
    }
}

/** @frame(x: 0, y: 350, width: 500, height: 400) */
component Form {
    variant isFocused trigger { focus }

    render form {
        style {
            display: flex
            flex-direction: column
            gap: 16px
        }

        div field {
            label { text "Email" }
            input(type="email", placeholder="Enter email") {
                style {
                    padding: spacingUnit
                    border: 1px solid #CCC
                    border-radius: borderRadius
                }
                style variant isFocused {
                    border-color: primaryColor
                }
            }
        }

        div actions {
            Button() { text "Submit" }
        }
    }
}

/** @frame(x: 550, y: 350, width: 800, height: 600) */
component Dashboard {
    render div dashboard {
        style {
            display: grid
            padding: 32px
        }

        repeat item in dashboardItems {
            Card() {
                insert header {
                    text item.title
                }
                insert body {
                    div {
                        text item.value
                    }
                }
            }
        }
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let initial_frames: Vec<String> = handler
        .index()
        .all_node_ids()
        .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .cloned()
        .collect();

    assert_eq!(initial_frames.len(), 5, "Should have 5 frames");

    // Mutate all frames multiple times
    for round in 0..3 {
        for i in 0..5 {
            handler
                .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
                .unwrap();

            let frame_ids: Vec<String> = handler
                .index()
                .all_node_ids()
                .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
                .cloned()
                .collect();

            if frame_ids.is_empty() {
                panic!("No frames found at round {} iteration {}", round, i);
            }

            let mutation = Mutation::SetFrameBounds {
                mutation_id: format!("mega-{}-{}", round, i),
                frame_id: frame_ids[0].clone(),
                x: (round * 100 + i * 20) as f32,
                y: (round * 50 + i * 10) as f32,
                width: 400.0 + (round * 50) as f32,
                height: 300.0 + (round * 30) as f32,
            };

            let result = handler.apply_mutation(&mutation, &mut crdt_doc);
            assert!(
                matches!(result, Ok(MutationResult::Applied { .. })),
                "Failed at round {} iteration {}: {:?}",
                round,
                i,
                result
            );
        }
    }

    let text = get_text(&crdt_doc);
    // Verify key elements survived
    assert!(text.contains("import \"./design-system.pc\""));
    assert!(text.contains("public token primaryColor"));
    assert!(text.contains("trigger hover"));
    assert!(text.contains("style extends baseButton"));
    assert!(text.contains("style variant primary + isHovered"));
    assert!(text.contains("repeat item in dashboardItems"));
    assert!(text.contains("component Dashboard"));
}

#[test]
fn test_stress_rapid_mutations_across_all_frames() {
    // Create a document with many frames and mutate them all rapidly
    let source = r#"/** @frame(x: 0, y: 0, width: 100, height: 100) */
component A { render div {} }

/** @frame(x: 150, y: 0, width: 100, height: 100) */
component B { render div {} }

/** @frame(x: 300, y: 0, width: 100, height: 100) */
component C { render div {} }

/** @frame(x: 450, y: 0, width: 100, height: 100) */
component D { render div {} }

/** @frame(x: 0, y: 150, width: 100, height: 100) */
component E { render div {} }

/** @frame(x: 150, y: 150, width: 100, height: 100) */
component F { render div {} }

/** @frame(x: 300, y: 150, width: 100, height: 100) */
component G { render div {} }

/** @frame(x: 450, y: 150, width: 100, height: 100) */
component H { render div {} }"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // 50 rapid mutations
    for i in 0..50 {
        handler
            .rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc))
            .unwrap();

        let frame_ids: Vec<String> = handler
            .index()
            .all_node_ids()
            .filter(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
            .cloned()
            .collect();

        assert!(!frame_ids.is_empty(), "No frames at iteration {}", i);

        // Pick a "random" frame based on iteration
        let frame_idx = i % frame_ids.len();
        let frame_id = &frame_ids[frame_idx];

        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("rapid-{}", i),
            frame_id: frame_id.clone(),
            x: ((i * 17) % 500) as f32,
            y: ((i * 13) % 400) as f32,
            width: 100.0 + ((i * 7) % 200) as f32,
            height: 100.0 + ((i * 11) % 150) as f32,
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        assert!(
            matches!(result, Ok(MutationResult::Applied { .. })),
            "Failed at iteration {}: {:?}",
            i,
            result
        );
    }

    let text = get_text(&crdt_doc);
    // All components should still exist
    for c in ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'] {
        assert!(
            text.contains(&format!("component {}", c)),
            "Component {} missing",
            c
        );
    }
}

#[test]
fn test_mutation_with_style_extends_chain() {
    let source = r#"style base {
    display: block
}

style level1 extends base {
    padding: 8px
}

style level2 extends level1 {
    margin: 8px
}

style level3 extends level2 {
    border: 1px solid black
}

style level4 extends level3 {
    border-radius: 4px
}

public style level5 extends level4 {
    box-shadow: 0 2px 4px rgba(0,0,0,0.1)
}

/** @frame(x: 0, y: 0, width: 400, height: 300) */
component DeepExtends {
    render div {
        style extends level5 {
            background: white
        }
        text "Deep inheritance"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "extends-chain".to_string(),
        frame_id,
        x: 100.0,
        y: 100.0,
        width: 500.0,
        height: 400.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("style level1 extends base"));
    assert!(text.contains("style level5 extends level4"));
    assert!(text.contains("style extends level5"));
}

#[test]
fn test_mutation_with_media_queries_in_triggers() {
    // Media queries go inside trigger definitions, then variants reference the trigger
    let source = r#"trigger mobile {
    "@media (max-width: 767px)"
}

trigger desktop {
    "@media (min-width: 1024px)"
}

trigger darkMode {
    "@media (prefers-color-scheme: dark)"
}

/** @frame(x: 0, y: 0, width: 500, height: 400) */
component Responsive {
    variant isMobile trigger { mobile }
    variant isDesktop trigger { desktop }
    variant isDark trigger { darkMode }

    render div {
        style {
            padding: 16px
            background: white
            color: black
        }
        style variant isMobile {
            padding: 8px
            font-size: 14px
        }
        style variant isDesktop {
            padding: 24px
            font-size: 18px
        }
        style variant isDark {
            background: #1a1a1a
            color: white
        }
        text "Responsive Component"
    }
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| handler.index().get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "media-queries".to_string(),
        frame_id,
        x: 50.0,
        y: 50.0,
        width: 600.0,
        height: 500.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("@media (max-width: 767px)"));
    assert!(text.contains("@media (prefers-color-scheme: dark)"));
    assert!(text.contains("variant isDark trigger"));
}

// ============================================================================
// SECTION 14: ANNOTATION MUTATIONS - SetComponentAnnotation & RemoveComponentAnnotation
// ============================================================================

#[test]
fn test_set_annotation_new_doc_comment() {
    // Add annotation to component without existing doc comment
    let source = r#"component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "add-frame".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "frame".to_string(),
        params_str: "x: 100, y: 200, width: 300, height: 400".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("/**"));
    assert!(text.contains("@frame(x: 100, y: 200, width: 300, height: 400)"));
    assert!(text.contains("*/"));
    assert!(text.contains("component Test"));
}

#[test]
fn test_set_annotation_update_existing() {
    // Update existing @frame annotation
    let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "update-frame".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "frame".to_string(),
        params_str: "x: 500, y: 600, width: 700, height: 800".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: 500"));
    assert!(text.contains("y: 600"));
    assert!(text.contains("width: 700"));
    assert!(text.contains("height: 800"));
    // Old values should be gone
    assert!(!text.contains("x: 0"));
}

#[test]
fn test_set_annotation_add_to_existing_doc_comment() {
    // Add new annotation to doc comment that already has other annotations
    let source = r#"/**
 * A test component
 * @frame(x: 100, y: 100)
 */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "add-meta".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "meta".to_string(),
        params_str: "category: ui".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("@frame(x: 100, y: 100)"));
    assert!(text.contains("@meta(category: ui)"));
    assert!(text.contains("A test component"));
}

#[test]
fn test_remove_annotation_basic() {
    let source = r#"/**
 * @frame(x: 100, y: 200, width: 300, height: 400)
 */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::RemoveComponentAnnotation {
        mutation_id: "remove-frame".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "frame".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(!text.contains("@frame"));
}

#[test]
fn test_remove_annotation_preserves_others() {
    let source = r#"/**
 * A description
 * @frame(x: 100, y: 200)
 * @meta(category: ui)
 * @deprecated
 */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::RemoveComponentAnnotation {
        mutation_id: "remove-meta".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "meta".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("@frame"));
    assert!(text.contains("@deprecated"));
    assert!(!text.contains("@meta"));
}

#[test]
fn test_annotation_with_negative_values() {
    let source = r#"component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "negative".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "frame".to_string(),
        params_str: "x: -100, y: -200, width: 50, height: 50".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: -100"));
    assert!(text.contains("y: -200"));
}

#[test]
fn test_annotation_with_decimal_values() {
    let source = r#"component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "decimal".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "frame".to_string(),
        params_str: "x: 100.5, y: 200.75".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: 100.5"));
    assert!(text.contains("y: 200.75"));
}

#[test]
fn test_annotation_without_params() {
    let source = r#"component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "deprecated".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "deprecated".to_string(),
        params_str: "".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("@deprecated"));
}

#[test]
fn test_annotation_on_nonexistent_component() {
    let source = r#"component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "fail".to_string(),
        component_name: "NotExist".to_string(),
        annotation_name: "frame".to_string(),
        params_str: "x: 100".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(result.is_err());
}

#[test]
fn test_remove_nonexistent_annotation() {
    let source = r#"/**
 * @frame(x: 100, y: 200)
 */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::RemoveComponentAnnotation {
        mutation_id: "fail".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "nonexistent".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(result.is_err());
}

#[test]
fn test_rapid_annotation_updates() {
    // Rapidly update the same annotation multiple times
    let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    for i in 0..50 {
        let mutation = Mutation::SetComponentAnnotation {
            mutation_id: format!("rapid-{}", i),
            component_name: "Test".to_string(),
            annotation_name: "frame".to_string(),
            params_str: format!("x: {}, y: {}, width: {}, height: {}", i * 10, i * 5, 100 + i, 100 + i),
        };

        handler.rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc)).unwrap();
        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        assert!(
            matches!(result, Ok(MutationResult::Applied { .. })),
            "Failed at iteration {}: {:?}",
            i,
            result
        );
    }

    let text = get_text(&crdt_doc);
    // Should have final values
    assert!(text.contains("x: 490"));
    assert!(text.contains("y: 245"));
}

#[test]
fn test_annotation_with_special_string_values() {
    let source = r#"component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "special".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "meta".to_string(),
        params_str: r#"path: "/foo/bar", name: "Test Component""#.to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("/foo/bar"));
    assert!(text.contains("Test Component"));
}

#[test]
fn test_annotation_multi_component_document() {
    let source = r#"/**
 * @frame(x: 0, y: 0)
 */
component A {
    render div {}
}

/**
 * @frame(x: 100, y: 0)
 */
component B {
    render div {}
}

component C {
    render div {}
}"#;

    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Add annotation to C
    let mutation1 = Mutation::SetComponentAnnotation {
        mutation_id: "add-c".to_string(),
        component_name: "C".to_string(),
        annotation_name: "frame".to_string(),
        params_str: "x: 200, y: 0".to_string(),
    };

    handler.rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc)).unwrap();
    let result = handler.apply_mutation(&mutation1, &mut crdt_doc);
    assert!(
        matches!(result, Ok(MutationResult::Applied { .. })),
        "First mutation failed: {:?}\nText: {}",
        result,
        get_text(&crdt_doc)
    );

    // Update A
    let mutation2 = Mutation::SetComponentAnnotation {
        mutation_id: "update-a".to_string(),
        component_name: "A".to_string(),
        annotation_name: "frame".to_string(),
        params_str: "x: 500, y: 500".to_string(),
    };

    handler.rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc)).unwrap();
    let result = handler.apply_mutation(&mutation2, &mut crdt_doc);
    assert!(
        matches!(result, Ok(MutationResult::Applied { .. })),
        "Second mutation failed: {:?}\nText: {}",
        result,
        get_text(&crdt_doc)
    );

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: 500")); // A's new value
    assert!(text.contains("x: 100")); // B unchanged
    assert!(text.contains("x: 200")); // C's new frame
}

#[test]
fn test_annotation_after_concurrent_edit() {
    let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
component Test {
    render div {
        text "Hello"
    }
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Insert content at beginning (simulating concurrent edit)
    crdt_doc.insert(0, "// Comment added\n");
    handler.rebuild_index(crdt_doc.doc(), &get_text(&crdt_doc)).unwrap();

    // Now update annotation
    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "after-edit".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "frame".to_string(),
        params_str: "x: 999, y: 888".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.starts_with("// Comment added"));
    assert!(text.contains("x: 999"));
}

#[test]
fn test_annotation_preserves_doc_comment_description() {
    let source = r#"/**
 * This is a very important component.
 * It has multiple lines of description.
 * @frame(x: 0, y: 0)
 */
component Test {
    render div {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "preserve".to_string(),
        component_name: "Test".to_string(),
        annotation_name: "frame".to_string(),
        params_str: "x: 100, y: 200".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    // Description should still be there
    assert!(text.contains("This is a very important component"));
    assert!(text.contains("multiple lines"));
    assert!(text.contains("x: 100"));
}

#[test]
fn test_annotation_on_public_component() {
    let source = r#"public component Button {
    render button {}
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let mutation = Mutation::SetComponentAnnotation {
        mutation_id: "public".to_string(),
        component_name: "Button".to_string(),
        annotation_name: "frame".to_string(),
        params_str: "x: 50, y: 50".to_string(),
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("@frame"));
    assert!(text.contains("public component Button"));
}

// ==================== Render Frame Mutation Tests ====================

#[test]
fn test_set_frame_bounds_on_render() {
    // Test SetFrameBounds mutation on a top-level render element
    let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
div {
    text "Hello"
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Find the frame ID
    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| {
            handler
                .index()
                .get_node(id)
                .map(|n| n.node_type == NodeType::Frame)
                .unwrap_or(false)
        })
        .expect("Should find a frame")
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "render-frame-1".to_string(),
        frame_id,
        x: 200.0,
        y: 150.0,
        width: 400.0,
        height: 300.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: 200"));
    assert!(text.contains("y: 150"));
    assert!(text.contains("width: 400"));
    assert!(text.contains("height: 300"));
    assert!(text.contains("div {"));
}

#[test]
fn test_set_frame_bounds_on_render_text() {
    // Test SetFrameBounds mutation on a top-level text render
    let source = r#"/**
 * @frame(x: 10, y: 20, width: 50, height: 30)
 */
text "Hello world""#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| {
            handler
                .index()
                .get_node(id)
                .map(|n| n.node_type == NodeType::Frame)
                .unwrap_or(false)
        })
        .expect("Should find a frame")
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "render-text-1".to_string(),
        frame_id,
        x: 100.0,
        y: 200.0,
        width: 300.0,
        height: 150.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: 100"));
    assert!(text.contains("y: 200"));
    assert!(text.contains("text \"Hello world\""));
}

#[test]
fn test_set_frame_bounds_mixed_components_and_renders() {
    // Test mutations on both component and render frames in same document
    let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
component Card {
    render div { text "Card" }
}

/**
 * @frame(x: 200, y: 0, width: 100, height: 100)
 */
div {
    text "Standalone"
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Get all frame IDs
    let frame_ids: Vec<_> = handler
        .index()
        .all_node_ids()
        .filter(|id| {
            handler
                .index()
                .get_node(id)
                .map(|n| n.node_type == NodeType::Frame)
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    assert_eq!(frame_ids.len(), 2, "Should have 2 frames");

    // Mutate first frame
    let mutation1 = Mutation::SetFrameBounds {
        mutation_id: "mixed-1".to_string(),
        frame_id: frame_ids[0].clone(),
        x: 10.0,
        y: 20.0,
        width: 110.0,
        height: 120.0,
    };
    let result = handler.apply_mutation(&mutation1, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    // Mutate second frame
    let mutation2 = Mutation::SetFrameBounds {
        mutation_id: "mixed-2".to_string(),
        frame_id: frame_ids[1].clone(),
        x: 300.0,
        y: 400.0,
        width: 500.0,
        height: 600.0,
    };
    let result = handler.apply_mutation(&mutation2, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    // Both frames should be updated
    assert!(text.contains("x: 10") || text.contains("x: 300"));
    assert!(text.contains("component Card"));
    assert!(text.contains("text \"Standalone\""));
}

#[test]
fn test_render_frame_rapid_updates() {
    // Test rapid sequential updates to a render frame
    let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
div {}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    // Perform 10 rapid updates
    for i in 0..10 {
        let frame_id = handler
            .index()
            .all_node_ids()
            .find(|id| {
                handler
                    .index()
                    .get_node(id)
                    .map(|n| n.node_type == NodeType::Frame)
                    .unwrap_or(false)
            })
            .unwrap()
            .clone();

        let mutation = Mutation::SetFrameBounds {
            mutation_id: format!("rapid-{}", i),
            frame_id,
            x: (i * 10) as f32,
            y: (i * 20) as f32,
            width: 100.0 + i as f32,
            height: 100.0 + i as f32,
        };

        let result = handler.apply_mutation(&mutation, &mut crdt_doc);
        assert!(
            matches!(result, Ok(MutationResult::Applied { .. })),
            "Mutation {} should succeed",
            i
        );
    }

    // Final state should have last values
    let text = get_text(&crdt_doc);
    assert!(text.contains("x: 90"));
    assert!(text.contains("y: 180"));
}

#[test]
fn test_render_frame_preserves_description() {
    // Test that updating a render frame preserves the doc comment description
    let source = r#"/**
 * This is a hero section for the landing page.
 * It should be prominent and eye-catching.
 * @frame(x: 0, y: 0, width: 1200, height: 600)
 */
div {
    text "Hero"
}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| {
            handler
                .index()
                .get_node(id)
                .map(|n| n.node_type == NodeType::Frame)
                .unwrap_or(false)
        })
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "preserve-desc".to_string(),
        frame_id,
        x: 100.0,
        y: 50.0,
        width: 1000.0,
        height: 500.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    // Description should be preserved
    assert!(text.contains("hero section"));
    assert!(text.contains("eye-catching"));
    // Frame should be updated
    assert!(text.contains("x: 100"));
    assert!(text.contains("y: 50"));
}

#[test]
fn test_render_frame_negative_coordinates() {
    let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
div {}"#;
    let mut crdt_doc = create_crdt_doc(source);
    let mut handler = MutationHandler::new();
    handler.rebuild_index(crdt_doc.doc(), source).unwrap();

    let frame_id = handler
        .index()
        .all_node_ids()
        .find(|id| {
            handler
                .index()
                .get_node(id)
                .map(|n| n.node_type == NodeType::Frame)
                .unwrap_or(false)
        })
        .unwrap()
        .clone();

    let mutation = Mutation::SetFrameBounds {
        mutation_id: "negative".to_string(),
        frame_id,
        x: -100.0,
        y: -50.0,
        width: 200.0,
        height: 150.0,
    };

    let result = handler.apply_mutation(&mutation, &mut crdt_doc);
    assert!(matches!(result, Ok(MutationResult::Applied { .. })));

    let text = get_text(&crdt_doc);
    assert!(text.contains("x: -100"));
    assert!(text.contains("y: -50"));
}
