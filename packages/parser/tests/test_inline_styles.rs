use paperclip_parser::parse;

#[test]
fn test_inline_style_no_semicolons() {
    let source = r#"
        component Card {
            render div {
                style {
                    padding: 16px
                    background: #FF0000
                }
                text "Hello"
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_inline_style_with_semicolons() {
    let source = r#"
        component Card {
            render div {
                style {
                    padding: 16px;
                    background: #FF0000;
                }
                text "Hello"
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_nested_elements_with_styles() {
    let source = r#"
        component Card {
            render div {
                style {
                    border: 1px solid #ddd
                    padding: 16px
                }

                h1 {
                    style {
                        color: blue
                        margin: 0
                    }
                    text "Title"
                }

                p {
                    style {
                        color: #666
                    }
                    text "Content"
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_trigger_syntax() {
    let source = r#"
        trigger mobile {
            "@media screen and (max-width: 768px)"
        }

        component Navigation {
            variant isMobile trigger {
                mobile
            }

            render nav {
                style {
                    display: flex
                    padding: 16px
                }

                style variant isMobile {
                    flex-direction: column
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
