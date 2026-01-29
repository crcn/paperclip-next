//! Integration tests for editor crate

use paperclip_editor::{Document, EditSession, Pipeline, Mutation};
use std::path::PathBuf;

#[test]
fn test_document_lifecycle() {
    // Create document
    let source = r#"
        component Button {
            render div {
                text "Click me"
            }
        }
    "#;

    let mut doc = Document::from_source(
        PathBuf::from("button.pc"),
        source.to_string()
    ).unwrap();

    // Check initial state
    assert_eq!(doc.version, 0);
    assert!(!doc.is_dirty());

    // Evaluate
    let vdom = doc.evaluate();
    assert!(vdom.is_ok());
}

#[test]
fn test_edit_session_workflow() {
    let source = "component Test { render div {} }";
    let doc = Document::from_source(
        PathBuf::from("test.pc"),
        source.to_string()
    ).unwrap();

    let mut session = EditSession::new("test-client".to_string(), doc);

    // Apply optimistic mutation
    let mutation = Mutation::UpdateText {
        node_id: "text-1".to_string(),
        content: "Updated".to_string(),
    };

    // TODO: Will work once mutations are implemented
    let result = session.apply_optimistic(mutation);
    assert!(result.is_err(), "Mutations not yet implemented");
}

#[test]
fn test_pipeline_execution() {
    let source = "component Test { render div {} }";
    let doc = Document::from_source(
        PathBuf::from("test.pc"),
        source.to_string()
    ).unwrap();

    let mut pipeline = Pipeline::new(doc);

    // Initial evaluation
    let vdom = pipeline.full_evaluate();
    assert!(vdom.is_ok());

    // Apply mutation through pipeline
    let mutation = Mutation::UpdateText {
        node_id: "text-1".to_string(),
        content: "Test".to_string(),
    };

    let _ = pipeline.apply_mutation(mutation);
    assert_eq!(pipeline.document().version, 1);
}

#[test]
fn test_mutation_serialization() {
    let mutation = Mutation::MoveElement {
        node_id: "elem-1".to_string(),
        new_parent_id: "container-2".to_string(),
        index: 3,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&mutation).unwrap();

    // Deserialize back
    let deserialized: Mutation = serde_json::from_str(&json).unwrap();

    assert_eq!(mutation, deserialized);
}
