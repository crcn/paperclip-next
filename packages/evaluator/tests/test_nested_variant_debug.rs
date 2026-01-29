use paperclip_evaluator::Evaluator;
use paperclip_parser::parse_with_path;

#[test]
fn test_nested_element_with_variant() {
    let source = r#"
        public component Card {
            variant isActive trigger {
                ".active"
            }

            render div {
                style {
                    background: white
                }

                style variant isActive {
                    background: lightgray
                }

                div header {
                    style {
                        padding: 8px
                    }

                    style variant isActive {
                        background: lightblue
                    }

                    text "Header"
                }

                text "Body"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");

    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Eval failed");

    println!("\nVDOM styles ({} total):", vdom.styles.len());
    for (i, style) in vdom.styles.iter().enumerate() {
        println!("  [{}] selector: {}", i, style.selector);
        println!("      properties: {:?}", style.properties);
    }

    assert!(vdom.styles.len() >= 4, "Should generate CSS for nested elements with variants");
}
