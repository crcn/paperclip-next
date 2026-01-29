use paperclip_evaluator::Evaluator;
use paperclip_parser::parse_with_path;

#[test]
fn test_simple_variant() {
    let source = r#"
        public component Button {
            variant isActive trigger {
                ".active"
            }

            render button {
                style {
                    background: blue
                }

                style variant isActive {
                    background: red
                }

                text "Click"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    println!("Parsed variants: {:#?}", doc.components[0].variants);

    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Eval failed");

    println!("\nVDOM styles ({} total):", vdom.styles.len());
    for (i, style) in vdom.styles.iter().enumerate() {
        println!("  [{}] selector: {}", i, style.selector);
        println!("      properties: {:?}", style.properties);
    }

    assert!(vdom.styles.len() > 0, "Should generate CSS rules");
}

#[test]
fn test_inline_selector_variant() {
    let source = r#"
        public component Button {
            variant hover trigger {
                ":hover"
            }

            render button {
                style {
                    background: blue
                }

                style variant hover {
                    background: red
                }

                text "Click"
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

    assert!(vdom.styles.len() > 0, "Should generate CSS with inline selector");
}
