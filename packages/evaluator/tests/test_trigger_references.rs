use paperclip_evaluator::Evaluator;
use paperclip_parser::parse_with_path;

#[test]
fn test_trigger_reference() {
    let source = r#"
        trigger mobile {
            "@media screen and (max-width: 768px)"
        }

        trigger darkMode {
            "@media (prefers-color-scheme: dark)"
            ".dark"
        }

        public component Navigation {
            variant isMobile trigger {
                mobile
            }

            variant isDark trigger {
                darkMode
            }

            render nav {
                style {
                    display: flex
                    padding: 16px
                }

                style variant isMobile {
                    flex-direction: column
                }

                style variant isDark {
                    background: #1a1a1a
                }

                style variant isMobile + isDark {
                    border-bottom: 1px solid #333
                }

                text "Nav"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    println!("Triggers: {:#?}", doc.triggers);
    println!("Variants: {:#?}", doc.components[0].variants);

    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Eval failed");

    println!("\nVDOM styles ({} total):", vdom.styles.len());
    for (i, style) in vdom.styles.iter().enumerate() {
        println!("  [{}] selector: {}", i, style.selector);
        if let Some(ref mq) = style.media_query {
            println!("      media: {}", mq);
        }
        println!("      properties: {:?}", style.properties);
    }

    assert!(vdom.styles.len() >= 4, "Should generate CSS with trigger references");
}
