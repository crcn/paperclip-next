use paperclip_parser::parse;

#[test]
fn test_card_minimal() {
    let source = r#"
        component Card {
            slot title {
                text "Default Title"
            }

            render div container {
                style {
                    padding: 16px
                }

                title
            }
        }
    "#;

    let result = parse(source);
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn test_card_with_named_elements() {
    let source = r#"
        component Card {
            slot title {
                text "Default Title"
            }

            slot children {
                text "Default content"
            }

            render div container {
                style {
                    padding: 16px
                }

                div header {
                    title
                }

                div body {
                    children
                }
            }
        }
    "#;

    let result = parse(source);
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());
}
