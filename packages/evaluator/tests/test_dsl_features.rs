use paperclip_evaluator::Evaluator;
use paperclip_parser::parse_with_path;

#[test]
fn test_trigger_with_multiple_selectors() {
    let source = r#"
        trigger darkMode {
            "@media (prefers-color-scheme: dark)"
            ".dark"
        }

        public component Test {
            variant isDark trigger {
                darkMode
            }

            render div {
                style {
                    background: white
                }

                style variant isDark {
                    background: black
                }

                text "Hello"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Should evaluate trigger with multiple selectors");

    let vdom = vdom.unwrap();
    println!("✓ Trigger with multiple selectors: {} styles", vdom.styles.len());
}

#[test]
fn test_variant_combination() {
    let source = r#"
        trigger mobile {
            "@media screen and (max-width: 768px)"
        }

        public component Navigation {
            variant isMobile trigger {
                mobile
            }

            variant isDark trigger {
                ".dark"
            }

            render nav {
                style {
                    background: white
                    padding: 16px
                }

                style variant isMobile {
                    padding: 8px
                }

                style variant isDark {
                    background: black
                }

                style variant isMobile + isDark {
                    border-bottom: 1px solid #333
                }

                text "Nav"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Should evaluate variant combinations");

    let vdom = vdom.unwrap();
    println!("✓ Variant combination (isMobile + isDark): {} styles", vdom.styles.len());
}

#[test]
fn test_named_elements() {
    let source = r#"
        public component Card {
            render div container {
                style {
                    padding: 16px
                }

                div header {
                    style {
                        border-bottom: 1px solid #eee
                    }
                    text "Header"
                }

                div body {
                    text "Body"
                }

                div footer {
                    text "Footer"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Should evaluate named elements");

    let vdom = vdom.unwrap();
    println!("✓ Named elements: {} nodes, {} styles", vdom.nodes.len(), vdom.styles.len());
}

#[test]
fn test_style_extends() {
    let source = r#"
        public style baseCard {
            border: 1px solid #ddd
            border-radius: 8px
            padding: 16px
        }

        public component Card {
            render div {
                style extends baseCard {
                    background: white
                }
                text "Card"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Should evaluate style extends");

    let vdom = vdom.unwrap();
    println!("✓ Style extends: {} styles", vdom.styles.len());
}

#[test]
fn test_tokens() {
    let source = r#"
        public token primaryColor #3366FF
        public token spacing 16px

        public component Button {
            render button {
                style {
                    background: primaryColor
                    padding: spacing
                }
                text "Click me"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Should evaluate tokens");

    let vdom = vdom.unwrap();
    println!("✓ Tokens: {} styles", vdom.styles.len());
}

#[test]
fn test_component_with_insert() {
    let source = r#"
        public component Card {
            slot title {
                text "Default Title"
            }

            slot content

            render div {
                style {
                    padding: 16px
                }

                h2 {
                    title
                }

                div {
                    content
                }
            }
        }

        public component App {
            render div {
                Card {
                    insert title {
                        text "Custom Title"
                    }

                    insert content {
                        p {
                            text "Custom content"
                        }
                    }
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Should evaluate component with inserts");

    let vdom = vdom.unwrap();
    println!("✓ Component with inserts: {} nodes", vdom.nodes.len());
}

#[test]
fn test_nested_variant_styles() {
    let source = r#"
        public component Card {
            variant isActive trigger {
                ".active"
            }

            render div {
                style {
                    background: white
                    border: 1px solid #ddd
                }

                style variant isActive {
                    border-color: blue
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

                div body {
                    text "Body"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Should evaluate nested variant styles");

    let vdom = vdom.unwrap();
    println!("✓ Nested variant styles: {} styles", vdom.styles.len());
}

#[test]
fn test_override_syntax() {
    let source = r#"
        public component Button {
            render button {
                style {
                    background: blue
                    color: white
                }
                text "Click"
            }
        }

        public component App {
            override Button {
                style {
                    background: red
                }
            }

            render div {
                Button()
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc);

    if let Err(e) = &vdom {
        eprintln!("Evaluation error: {:?}", e);
    }
    assert!(vdom.is_ok(), "Should evaluate overrides");

    let vdom = vdom.unwrap();
    println!("✓ Overrides: {} styles", vdom.styles.len());
}
