//! Comprehensive mutation tests

use paperclip_editor::{Document, Mutation};
use std::path::PathBuf;

#[test]
fn test_update_text_mutation() {
    let source = r#"
        component Test {
            render div {
                text "Hello"
            }
        }
    "#;

    let mut doc = Document::from_source(PathBuf::from("test.pc"), source.to_string()).unwrap();

    // Get the text node ID by evaluating
    let ast = doc.ast();
    let text_id = if let Some(comp) = ast.components.first() {
        if let Some(body) = &comp.body {
            if let paperclip_parser::ast::Element::Tag { children, .. } = body {
                if let Some(paperclip_parser::ast::Element::Text { span, .. }) = children.first() {
                    span.id.clone()
                } else {
                    panic!("Expected text node");
                }
            } else {
                panic!("Expected tag");
            }
        } else {
            panic!("Expected body");
        }
    } else {
        panic!("Expected component");
    };

    let mutation = Mutation::UpdateText {
        node_id: text_id,
        content: "World".to_string(),
    };

    let result = doc.apply(mutation);
    assert!(result.is_ok(), "Mutation should succeed");
}

#[test]
fn test_remove_node_mutation() {
    let source = r#"
        component Test {
            render div {
                div {}
                div {}
            }
        }
    "#;

    let mut doc = Document::from_source(PathBuf::from("test.pc"), source.to_string()).unwrap();

    // Get a child div ID
    let ast = doc.ast();
    let child_id = if let Some(comp) = ast.components.first() {
        if let Some(body) = &comp.body {
            if let paperclip_parser::ast::Element::Tag { children, .. } = body {
                if let Some(paperclip_parser::ast::Element::Tag { span, .. }) = children.first() {
                    span.id.clone()
                } else {
                    panic!("Expected child div");
                }
            } else {
                panic!("Expected tag");
            }
        } else {
            panic!("Expected body");
        }
    } else {
        panic!("Expected component");
    };

    let mutation = Mutation::RemoveNode { node_id: child_id };

    let result = doc.apply(mutation);
    assert!(result.is_ok(), "Remove should succeed");
}

#[test]
fn test_cycle_detection() {
    let source = r#"
        component Test {
            render div {
                div {}
            }
        }
    "#;

    let mut doc = Document::from_source(PathBuf::from("test.pc"), source.to_string()).unwrap();

    let ast = doc.ast();
    let (parent_id, child_id) = if let Some(comp) = ast.components.first() {
        if let Some(body) = &comp.body {
            if let paperclip_parser::ast::Element::Tag { span, children, .. } = body {
                let parent = span.id.clone();
                let child = if let Some(paperclip_parser::ast::Element::Tag { span, .. }) =
                    children.first()
                {
                    span.id.clone()
                } else {
                    panic!("Expected child");
                };
                (parent, child)
            } else {
                panic!("Expected tag");
            }
        } else {
            panic!("Expected body");
        }
    } else {
        panic!("Expected component");
    };

    // Try to move parent into child (would create cycle)
    let mutation = Mutation::MoveElement {
        node_id: parent_id,
        new_parent_id: child_id,
        index: 0,
    };

    let result = doc.apply(mutation);
    assert!(result.is_err(), "Should detect cycle");
}

#[test]
fn test_cannot_edit_repeat_instance() {
    let source = r#"
        component Test {
            render repeat item in items {
                div {}
            }
        }
    "#;

    // Parse to get IDs
    let ast = paperclip_parser::parse(source).unwrap();
    let div_id = if let Some(comp) = ast.components.first() {
        if let Some(body) = &comp.body {
            if let paperclip_parser::ast::Element::Repeat { body, .. } = body {
                if let Some(paperclip_parser::ast::Element::Tag { span, .. }) = body.first() {
                    span.id.clone()
                } else {
                    panic!("Expected div in repeat");
                }
            } else {
                panic!("Expected repeat");
            }
        } else {
            panic!("Expected body");
        }
    } else {
        panic!("Expected component");
    };

    let mutation = Mutation::MoveElement {
        node_id: div_id,
        new_parent_id: "fake-parent".to_string(),
        index: 0,
    };

    let result = mutation.validate(&ast);
    assert!(result.is_err(), "Should reject editing repeat instance");
}

#[test]
fn test_set_inline_style() {
    let source = r#"
        component Test {
            render div {}
        }
    "#;

    // Parse to get IDs
    let ast = paperclip_parser::parse(source).unwrap();
    let div_id = if let Some(comp) = ast.components.first() {
        if let Some(body) = &comp.body {
            body.span().id.clone()
        } else {
            panic!("Expected body");
        }
    } else {
        panic!("Expected component");
    };

    let mut doc = Document::from_source(PathBuf::from("test.pc"), source.to_string()).unwrap();

    let mutation = Mutation::SetInlineStyle {
        node_id: div_id,
        property: "color".to_string(),
        value: "red".to_string(),
    };

    let result = doc.apply(mutation);
    assert!(result.is_ok(), "SetInlineStyle should succeed");
}

#[test]
fn test_set_attribute() {
    let source = r#"
        component Test {
            render div {}
        }
    "#;

    // Parse to get IDs
    let ast = paperclip_parser::parse(source).unwrap();
    let div_id = if let Some(comp) = ast.components.first() {
        if let Some(body) = &comp.body {
            body.span().id.clone()
        } else {
            panic!("Expected body");
        }
    } else {
        panic!("Expected component");
    };

    let mut doc = Document::from_source(PathBuf::from("test.pc"), source.to_string()).unwrap();

    let mutation = Mutation::SetAttribute {
        node_id: div_id,
        name: "class".to_string(),
        value: "container".to_string(),
    };

    let result = doc.apply(mutation);
    assert!(result.is_ok(), "SetAttribute should succeed");
}
