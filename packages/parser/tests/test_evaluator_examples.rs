use paperclip_parser::parse_with_path;
use std::fs;

#[test]
fn test_parse_test_pc() {
    let source = fs::read_to_string("../evaluator/examples/test.pc").unwrap();
    let result = parse_with_path(&source, "test.pc");
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn test_parse_simple_test_pc() {
    let source = fs::read_to_string("../evaluator/examples/simple_test.pc").unwrap();
    let result = parse_with_path(&source, "simple_test.pc");
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn test_parse_styled_test_pc() {
    let source = fs::read_to_string("../evaluator/examples/styled_test.pc").unwrap();
    let result = parse_with_path(&source, "styled_test.pc");
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn test_parse_list_test_pc() {
    let source = fs::read_to_string("../evaluator/examples/list_test.pc").unwrap();
    let result = parse_with_path(&source, "list_test.pc");
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());
}
