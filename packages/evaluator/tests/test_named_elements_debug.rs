use paperclip_evaluator::Evaluator;
use paperclip_parser::parse_with_path;

#[test]
fn test_simple_component() {
    let source = r#"
        component Card {
            render div {
                text "Hello"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    println!("Parsed document: {:#?}", doc);

    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Eval failed");

    println!("VDOM nodes: {:#?}", vdom.nodes);
    assert_eq!(vdom.nodes.len(), 0, "Non-public component should produce 0 nodes");
}

#[test]
fn test_public_component() {
    let source = r#"
        public component Card {
            render div {
                text "Hello"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");

    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Eval failed");

    println!("VDOM nodes: {:#?}", vdom.nodes);
    assert_eq!(vdom.nodes.len(), 1, "Public component should produce 1 node");
}

#[test]
fn test_named_element_component() {
    let source = r#"
        public component Card {
            render div container {
                text "Hello"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");

    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Eval failed");

    println!("VDOM nodes: {:#?}", vdom.nodes);
    assert_eq!(vdom.nodes.len(), 1, "Named element should produce 1 node");
}
