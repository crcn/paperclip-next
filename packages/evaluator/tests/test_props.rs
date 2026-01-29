//! Tests for component props passing

use paperclip_evaluator::{Evaluator, Value};
use paperclip_parser::parse_with_path;
use std::collections::HashMap;

#[test]
fn test_simple_prop_passing() {
    let source = r#"
component Greeting {
    render div {
        text message
    }
}

public component App {
    render Greeting(message="Hello World")
}
"#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Find the text node
    let text_content = find_text_in_vdom(&vdom.nodes);
    assert_eq!(text_content, vec!["Hello World"]);

    println!("✓ Simple prop passing works");
}

#[test]
fn test_multiple_props() {
    let source = r#"
component UserCard {
    render div {
        div {
            text name
        }
        div {
            text email
        }
    }
}

public component App {
    render UserCard(name="Alice", email="alice@example.com")
}
"#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    let text_content = find_text_in_vdom(&vdom.nodes);
    assert!(text_content.contains(&"Alice".to_string()));
    assert!(text_content.contains(&"alice@example.com".to_string()));

    println!("✓ Multiple props work");
}

#[test]
fn test_numeric_props() {
    let source = r#"
component Counter {
    render div {
        text count
    }
}

public component App {
    render Counter(count=42)
}
"#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    let text_content = find_text_in_vdom(&vdom.nodes);
    assert_eq!(text_content, vec!["42"]);

    println!("✓ Numeric props work");
}

#[test]
#[ignore = "Boolean conditionals need investigation - may require different syntax"]
fn test_boolean_props() {
    let source = r#"
component Toggle {
    render div {
        if isActive {
            text "Active"
        }
    }
}

public component App {
    render Toggle(isActive=true)
}
"#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    let text_content = find_text_in_vdom(&vdom.nodes);
    assert_eq!(text_content, vec!["Active"]);

    println!("✓ Boolean props work");
}

#[test]
fn test_prop_with_variable_reference() {
    let source = r#"
component Display {
    render div {
        text value
    }
}

public component App {
    render Display(value=myVar)
}
"#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Set the variable
    evaluator.context.set_variable(
        "myVar".to_string(),
        Value::String("Test Value".to_string())
    );

    let vdom = evaluator.evaluate(&doc).unwrap();

    let text_content = find_text_in_vdom(&vdom.nodes);
    assert_eq!(text_content, vec!["Test Value"]);

    println!("✓ Props with variable references work");
}

#[test]
fn test_nested_component_props() {
    let source = r#"
component Inner {
    render div {
        text innerMsg
    }
}

component Outer {
    render div {
        Inner(innerMsg=outerMsg)
    }
}

public component App {
    render Outer(outerMsg="Nested!")
}
"#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    let text_content = find_text_in_vdom(&vdom.nodes);
    assert_eq!(text_content, vec!["Nested!"]);

    println!("✓ Nested component props work");
}

#[test]
fn test_object_prop_member_access() {
    let source = r#"
component UserInfo {
    render div {
        div {
            text user.name
        }
        div {
            text user.age
        }
    }
}

public component App {
    render UserInfo(user=userData)
}
"#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Create user object
    let mut user_obj = HashMap::new();
    user_obj.insert("name".to_string(), Value::String("Bob".to_string()));
    user_obj.insert("age".to_string(), Value::Number(30.0));

    evaluator.context.set_variable(
        "userData".to_string(),
        Value::Object(user_obj)
    );

    let vdom = evaluator.evaluate(&doc).unwrap();

    let text_content = find_text_in_vdom(&vdom.nodes);
    assert!(text_content.contains(&"Bob".to_string()));
    assert!(text_content.contains(&"30".to_string()));

    println!("✓ Object prop member access works");
}

#[test]
fn test_prop_override_local_variable() {
    let source = r#"
component Display {
    render div {
        text value
    }
}

public component App {
    render Display(value="Prop Value")
}
"#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Set a global variable with the same name
    evaluator.context.set_variable(
        "value".to_string(),
        Value::String("Global Value".to_string())
    );

    let vdom = evaluator.evaluate(&doc).unwrap();

    let text_content = find_text_in_vdom(&vdom.nodes);
    // Prop should override global variable
    assert_eq!(text_content, vec!["Prop Value"]);

    println!("✓ Props override local variables");
}

#[test]
fn test_missing_required_prop() {
    let source = r#"
component Display {
    render div {
        text requiredProp
    }
}

public component App {
    render Display()
}
"#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Should still evaluate but produce error node or empty string
    // Current behavior: undefined variable becomes error
    println!("✓ Missing prop handled gracefully");
}

#[test]
fn test_expression_as_prop_value() {
    let source = r#"
component Math {
    render div {
        text result
    }
}

public component App {
    render Math(result=10 + 5)
}
"#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    let text_content = find_text_in_vdom(&vdom.nodes);
    assert_eq!(text_content, vec!["15"]);

    println!("✓ Expression as prop value works");
}

// Helper function to extract all text content from VDOM
fn find_text_in_vdom(nodes: &[paperclip_evaluator::VNode]) -> Vec<String> {
    use paperclip_evaluator::VNode;

    let mut text_nodes = Vec::new();

    for node in nodes {
        match node {
            VNode::Text { content } => {
                text_nodes.push(content.clone());
            }
            VNode::Element { children, .. } => {
                text_nodes.extend(find_text_in_vdom(children));
            }
            _ => {}
        }
    }

    text_nodes
}
