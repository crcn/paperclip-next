use paperclip_evaluator::Evaluator;
use paperclip_parser::parse_with_path;
use std::fs;

#[test]
fn test_evaluate_test_pc() {
    let source = fs::read_to_string("examples/test.pc").unwrap();
    let doc = parse_with_path(&source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Failed to evaluate test.pc");

    let vdom = vdom.unwrap();
    println!("✓ test.pc: {} nodes, {} styles", vdom.nodes.len(), vdom.styles.len());
}

#[test]
fn test_evaluate_styled_test_pc() {
    let source = fs::read_to_string("examples/styled_test.pc").unwrap();
    let doc = parse_with_path(&source, "styled_test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("styled_test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Failed to evaluate styled_test.pc");

    let vdom = vdom.unwrap();
    println!("✓ styled_test.pc: {} nodes, {} styles", vdom.nodes.len(), vdom.styles.len());
}

#[test]
fn test_evaluate_list_test_pc() {
    let source = fs::read_to_string("examples/list_test.pc").unwrap();
    let doc = parse_with_path(&source, "list_test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("list_test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Failed to evaluate list_test.pc");

    let vdom = vdom.unwrap();
    println!("✓ list_test.pc: {} nodes, {} styles", vdom.nodes.len(), vdom.styles.len());
}

#[test]
fn test_evaluate_simple_test_pc() {
    let source = fs::read_to_string("examples/simple_test.pc").unwrap();
    let doc = parse_with_path(&source, "simple_test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("simple_test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Failed to evaluate simple_test.pc");

    let vdom = vdom.unwrap();
    println!("✓ simple_test.pc: {} nodes, {} styles", vdom.nodes.len(), vdom.styles.len());
}
