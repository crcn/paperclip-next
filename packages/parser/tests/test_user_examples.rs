use paperclip_parser::parse;

#[test]
fn test_navigation_with_variants_and_triggers() {
    let source = r#"
        trigger mobile {
            "@media screen and (max-width: 768px)"
        }

        trigger tablet {
            "@media screen and (min-width: 769px) and (max-width: 1024px)"
        }

        trigger darkMode {
            "@media (prefers-color-scheme: dark)"
            ".dark"
        }

        component Navigation {
            variant isMobile trigger {
                mobile
            }

            variant isTablet trigger {
                tablet
            }

            variant isDark trigger {
                darkMode
            }

            render nav {
                style {
                    display: flex
                    background: white
                    padding: 16px
                }

                style variant isMobile {
                    flex-direction: column
                    padding: 8px
                }

                style variant isDark {
                    background: #1a1a1a
                    color: white
                }

                style variant isMobile + isDark {
                    border-bottom: 1px solid #333
                }
            }
        }
    "#;

    let result = parse(source);
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.triggers.len(), 3);
    assert_eq!(doc.components.len(), 1);

    let component = &doc.components[0];
    assert_eq!(component.variants.len(), 3);
}

#[test]
fn test_card_with_slots_and_nested_styles() {
    let source = r#"
        component Card {
            slot title {
                text "Default Title"
            }

            slot children {
                text "Default content"
            }

            slot actions

            render div container {
                style {
                    border: 1px solid #ddd
                    border-radius: 8px
                    padding: 16px
                }

                div header {
                    style {
                        border-bottom: 1px solid #eee
                        padding-bottom: 12px
                        margin-bottom: 12px
                    }
                    title
                }

                div body {
                    children
                }

                div footer {
                    style {
                        margin-top: 12px
                        padding-top: 12px
                        border-top: 1px solid #eee
                    }
                    actions
                }
            }
        }

        public component App {
            render div {
                Card {
                    insert title {
                        h2 {
                            style {
                                margin: 0
                                font-size: 20px
                            }
                            text "My Card"
                        }
                    }

                    div {
                        style {
                            color: #666
                        }
                        text "This is the card content"
                    }

                    insert actions {
                        button {
                            style {
                                background: blue
                                color: white
                                border: none
                                padding: 8px 16px
                                border-radius: 4px
                            }
                            text "Action"
                        }
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    if let Err(e) = &result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.components.len(), 2, "Expected Card and App components");
}
