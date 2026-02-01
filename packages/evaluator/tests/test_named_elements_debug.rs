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
    // All components render for preview (public keyword only affects exports)
    assert_eq!(vdom.nodes.len(), 1, "Non-public component should render for preview");
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
