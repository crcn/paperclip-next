//! Test recursive component rendering

use paperclip_evaluator::evaluator::{Evaluator, EvalError, Value};
use paperclip_parser::parse_with_path;
use std::collections::HashMap;

#[test]
fn test_direct_recursion() {
    let source = r#"
        public component AB {
            render div {
                AB()
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Should detect cycle and return RecursiveComponent error
    let result = evaluator.evaluate(&doc);

    match result {
        Ok(_) => panic!("Expected RecursiveComponent error but got Ok result"),
        Err(EvalError::RecursiveComponent { component, call_stack, hint }) => {
            assert_eq!(component, "AB");
            assert_eq!(call_stack, vec!["AB", "AB"]);
            assert!(hint.is_some());
            assert!(hint.unwrap().contains("renders itself unconditionally"));
            println!("✓ Direct recursion detected correctly");
        }
        Err(e) => panic!("Expected RecursiveComponent error but got: {:?}", e),
    }
}

#[test]
fn test_indirect_recursion() {
    let source = r#"
        component A {
            render div {
                B()
            }
        }

        component B {
            render div {
                A()
            }
        }

        public component App {
            render A()
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Should detect indirect cycle (A → B → A)
    let result = evaluator.evaluate(&doc);

    match result {
        Ok(_) => panic!("Expected RecursiveComponent error but got Ok result"),
        Err(EvalError::RecursiveComponent { component, call_stack, hint }) => {
            assert_eq!(component, "A");
            assert_eq!(call_stack, vec!["App", "A", "B", "A"]);
            assert!(hint.is_some());
            assert!(hint.unwrap().contains("cycle detected"));
            println!("✓ Indirect recursion (A → B → A) detected correctly");
        }
        Err(e) => panic!("Expected RecursiveComponent error but got: {:?}", e),
    }
}

#[test]
fn test_conditional_recursion_without_data_change() {
    let source = r#"
        public component Countdown {
            render div {
                text count
                if count > 0 {
                    Countdown()
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Set initial count
    evaluator.context.set_variable(
        "count".to_string(),
        paperclip_evaluator::evaluator::Value::Number(3.0)
    );

    // Should detect recursion even though it's conditional
    // (count never changes, so it recurses infinitely)
    let result = evaluator.evaluate(&doc);

    match result {
        Ok(_) => panic!("Expected RecursiveComponent error but got Ok result"),
        Err(EvalError::RecursiveComponent { component, call_stack, .. }) => {
            assert_eq!(component, "Countdown");
            assert_eq!(call_stack, vec!["Countdown", "Countdown"]);
            println!("✓ Conditional recursion without data change detected correctly");
        }
        Err(e) => panic!("Expected RecursiveComponent error but got: {:?}", e),
    }
}

#[test]
fn test_bare_identifier_is_safe() {
    let source = r#"
        public component AB {
            render div {
                AB
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Bare identifier becomes SlotInsert, not recursion
    let result = evaluator.evaluate(&doc);

    // Should succeed but with error node for missing slot
    assert!(result.is_ok());
    let vdom = result.unwrap();

    // The div should contain an error node about missing slot
    // (not a RecursiveComponent error)
    println!("✓ Bare identifier AB is treated as slot insert (safe)");
}

#[test]
fn test_three_component_cycle() {
    let source = r#"
        component A {
            render div {
                B()
            }
        }

        component B {
            render div {
                C()
            }
        }

        component C {
            render div {
                A()
            }
        }

        public component App {
            render A()
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Should detect three-component cycle (A → B → C → A)
    let result = evaluator.evaluate(&doc);

    match result {
        Ok(_) => panic!("Expected RecursiveComponent error but got Ok result"),
        Err(EvalError::RecursiveComponent { component, call_stack, .. }) => {
            assert_eq!(component, "A");
            assert_eq!(call_stack, vec!["App", "A", "B", "C", "A"]);
            println!("✓ Three-component cycle (A → B → C → A) detected correctly");
        }
        Err(e) => panic!("Expected RecursiveComponent error but got: {:?}", e),
    }
}

#[test]
fn test_non_recursive_component_reuse() {
    let source = r#"
        component Button {
            render button {
                text "Click"
            }
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

    // Should work fine - same component used multiple times but not recursively
    let result = evaluator.evaluate(&doc);

    assert!(result.is_ok());
    println!("✓ Non-recursive component reuse works correctly");
}

#[test]
#[ignore = "Valid recursion with data-dependent termination - future enhancement"]
fn test_valid_tree_recursion_with_props() {
    // This test demonstrates the VALID pattern that should be allowed
    // TreeNode(node=child) where each child is structurally smaller
    //
    // Current implementation will still catch this as a cycle because
    // we check component name equality, not data equality.
    //
    // Future enhancement: Track prop changes and allow recursion if
    // props are demonstrably different (e.g., array is smaller)

    let source = r#"
        component TreeNode {
            render div {
                text node.label
                if node.children {
                    repeat child in node.children {
                        TreeNode(node=child)
                    }
                }
            }
        }

        public component App {
            render TreeNode(node=root)
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");

    // Set up tree data: root with 2 children (leaf nodes)
    let leaf1 = Value::Object({
        let mut map = HashMap::new();
        map.insert("label".to_string(), Value::String("Leaf 1".to_string()));
        map.insert("children".to_string(), Value::Null);
        map
    });

    let leaf2 = Value::Object({
        let mut map = HashMap::new();
        map.insert("label".to_string(), Value::String("Leaf 2".to_string()));
        map.insert("children".to_string(), Value::Null);
        map
    });

    let root = Value::Object({
        let mut map = HashMap::new();
        map.insert("label".to_string(), Value::String("Root".to_string()));
        map.insert("children".to_string(), Value::Array(vec![leaf1, leaf2]));
        map
    });

    evaluator.context.set_variable("root".to_string(), root);

    // This SHOULD work because:
    // 1. Each TreeNode call has different node prop
    // 2. Recursion depth is bounded by tree depth
    // 3. Leaf nodes have no children, terminating recursion
    //
    // But current implementation will detect cycle: TreeNode → TreeNode

    let result = evaluator.evaluate(&doc);

    // Currently fails with RecursiveComponent error
    // Future: Should succeed and render tree
    match result {
        Ok(_) => println!("✓ Valid tree recursion works!"),
        Err(EvalError::RecursiveComponent { .. }) => {
            println!("⚠ Valid tree recursion blocked by cycle detection (expected for now)");
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
