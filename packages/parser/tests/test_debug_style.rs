use paperclip_parser::parse;

#[test]
fn test_simple_nested_with_style() {
    let source = r#"
        component Card {
            render div {
                h1 {
                    style {
                        color: blue
                    }
                    text "Title"
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

#[test]
fn test_parent_and_child_styles() {
    let source = r#"
        component Card {
            render div {
                style {
                    padding: 16px
                }

                h1 {
                    style {
                        color: blue
                    }
                    text "Title"
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
