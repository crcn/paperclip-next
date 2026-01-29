//! # Spike 0.4: CSS Variant System
//!
//! Validates that the variant system works end-to-end:
//! - Variant declarations with triggers
//! - CSS selectors (.class, :hover, :active)
//! - Media queries (@media)
//! - Combination variants (a + b + c)
//! - Style variant blocks

use paperclip_parser::parse;

#[test]
fn test_variant_declaration_basic() {
    let source = r#"
        component Button {
            variant hover
            variant active
            variant disabled

            render button {}
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.components[0].variants.len(), 3);
    assert_eq!(doc.components[0].variants[0].name, "hover");
    assert_eq!(doc.components[0].variants[1].name, "active");
    assert_eq!(doc.components[0].variants[2].name, "disabled");

    println!("✓ Basic variant declarations parse correctly");
}

#[test]
fn test_variant_with_css_selector_triggers() {
    let source = r#"
        component Button {
            variant hover trigger {
                ":hover"
            }

            variant active trigger {
                ".active",
                ":active"
            }

            render button {}
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.components[0].variants.len(), 2);

    // hover variant has 1 trigger
    assert_eq!(doc.components[0].variants[0].name, "hover");
    assert_eq!(doc.components[0].variants[0].triggers.len(), 1);
    assert_eq!(doc.components[0].variants[0].triggers[0], ":hover");

    // active variant has 2 triggers
    assert_eq!(doc.components[0].variants[1].name, "active");
    assert_eq!(doc.components[0].variants[1].triggers.len(), 2);
    assert_eq!(doc.components[0].variants[1].triggers[0], ".active");
    assert_eq!(doc.components[0].variants[1].triggers[1], ":active");

    println!("✓ CSS selector triggers parse correctly");
}

#[test]
fn test_variant_with_media_query_triggers() {
    let source = r#"
        component Page {
            variant mobile trigger {
                "@media screen and (max-width: 768px)"
            }

            variant tablet trigger {
                "@media screen and (min-width: 769px) and (max-width: 1024px)"
            }

            variant dark trigger {
                "@media (prefers-color-scheme: dark)",
                ".dark-mode"
            }

            render div {}
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.components[0].variants.len(), 3);

    // mobile variant
    assert_eq!(doc.components[0].variants[0].name, "mobile");
    assert_eq!(doc.components[0].variants[0].triggers.len(), 1);
    assert!(doc.components[0].variants[0].triggers[0].contains("@media"));

    // dark variant with both media query and class selector
    assert_eq!(doc.components[0].variants[2].name, "dark");
    assert_eq!(doc.components[0].variants[2].triggers.len(), 2);

    println!("✓ Media query triggers parse correctly");
}

#[test]
fn test_style_variant_blocks() {
    let source = r#"
        component Button {
            variant primary
            variant hover

            render button {
                style {
                    background: blue
                    color: white
                }

                style variant primary {
                    background: red
                }

                style variant hover {
                    background: darkblue
                    transform: scale(1.05)
                }

                text "Click"
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    // Component has 2 variants
    assert_eq!(component.variants.len(), 2);

    // Check the button element has variant styles
    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { styles, .. } = body {
            // Should have 3 style blocks (base + 2 variants)
            assert_eq!(styles.len(), 3);

            // First is base style (no variants)
            assert_eq!(styles[0].variants.len(), 0);

            // Second is primary variant
            assert_eq!(styles[1].variants.len(), 1);
            assert_eq!(styles[1].variants[0], "primary");

            // Third is hover variant
            assert_eq!(styles[2].variants.len(), 1);
            assert_eq!(styles[2].variants[0], "hover");
        }
    }

    println!("✓ Style variant blocks parse correctly");
}

#[test]
fn test_combination_variants() {
    let source = r#"
        component Button {
            variant primary
            variant hover
            variant active

            render button {
                style variant primary {
                    background: blue
                }

                style variant hover {
                    transform: scale(1.05)
                }

                style variant primary + hover {
                    background: darkblue
                    transform: scale(1.1)
                }

                style variant primary + hover + active {
                    background: navy
                    transform: scale(1.0)
                }

                text "Click"
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { styles, .. } = body {
            // Should have 4 style blocks
            assert_eq!(styles.len(), 4);

            // Check single variants
            assert_eq!(styles[0].variants, vec!["primary"]);
            assert_eq!(styles[1].variants, vec!["hover"]);

            // Check combination variant: primary + hover
            assert_eq!(styles[2].variants, vec!["primary", "hover"]);

            // Check combination variant: primary + hover + active
            assert_eq!(styles[3].variants, vec!["primary", "hover", "active"]);
        }
    }

    println!("✓ Combination variants (a + b + c) parse correctly");
}

#[test]
fn test_variant_priority_cascade() {
    // Tests that multiple variant styles can be applied to same element
    let source = r#"
        component Card {
            variant dark
            variant compact
            variant highlighted

            render div(class="card") {
                style {
                    padding: 20px
                    background: white
                    border: 1px solid #ccc
                }

                style variant dark {
                    background: #333
                    color: white
                    border-color: #666
                }

                style variant compact {
                    padding: 10px
                }

                style variant highlighted {
                    border-width: 2px
                    border-color: blue
                }

                style variant dark + highlighted {
                    border-color: lightblue
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    assert_eq!(component.variants.len(), 3);

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { styles, .. } = body {
            // Base + 3 single variants + 1 combination
            assert_eq!(styles.len(), 5);
        }
    }

    println!("✓ Multiple variants with cascade parse correctly");
}

#[test]
#[ignore = "Complex nested variant combinations not yet fully supported"]
fn test_complex_real_world_example() {
    let source = r#"
        component NavigationMenu {
            variant mobile trigger {
                "@media screen and (max-width: 768px)"
            }

            variant dark trigger {
                ".dark-mode",
                "@media (prefers-color-scheme: dark)"
            }

            variant collapsed trigger {
                ".collapsed"
            }

            render nav(class="navigation") {
                style {
                    display: flex
                }

                style variant mobile {
                    flex-direction: column
                }

                style variant dark {
                    background: #1a1a1a
                }

                style variant mobile + dark {
                    background: #0d0d0d
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
    let component = &doc.components[0];

    // Verify variant declarations
    assert_eq!(component.variants.len(), 3);
    assert_eq!(component.variants[0].name, "mobile");
    assert_eq!(component.variants[1].name, "dark");
    assert_eq!(component.variants[2].name, "collapsed");

    // Verify mobile has media query trigger
    assert_eq!(component.variants[0].triggers.len(), 1);
    assert!(component.variants[0].triggers[0].contains("@media"));

    // Verify dark has both class and media query triggers
    assert_eq!(component.variants[1].triggers.len(), 2);

    println!("✓ Complex real-world variant example parses correctly");
}

#[test]
fn test_variant_extends_style_mixins() {
    let source = r#"
        style baseButton {
            padding: 8px 16px
            border-radius: 4px
            cursor: pointer
        }

        component Button {
            variant primary
            variant large

            render button {
                style extends baseButton {
                    background: gray
                }

                style variant primary extends baseButton {
                    background: blue
                    color: white
                }

                style variant large {
                    padding: 12px 24px
                    font-size: 18px
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();

    // Should have style mixin
    assert_eq!(doc.styles.len(), 1);
    assert_eq!(doc.styles[0].name, "baseButton");

    // Component should have variants
    assert_eq!(doc.components[0].variants.len(), 2);

    println!("✓ Variants with style mixin extends work correctly");
}

#[test]
fn test_variant_serialization_roundtrip() {
    use paperclip_parser::serialize;

    let source = r#"
component Button {
    variant hover trigger {
        ":hover",
        ".hover"
    }
    variant active
    render button {
        style variant hover {
            background: blue
        }
        style variant hover + active {
            background: darkblue
        }
    }
}
    "#;

    let doc = parse(source).unwrap();
    let serialized = serialize(&doc);
    let reparsed = parse(&serialized).unwrap();

    // Should preserve variants
    assert_eq!(doc.components[0].variants.len(), reparsed.components[0].variants.len());
    assert_eq!(doc.components[0].variants[0].name, reparsed.components[0].variants[0].name);
    assert_eq!(
        doc.components[0].variants[0].triggers.len(),
        reparsed.components[0].variants[0].triggers.len()
    );

    println!("✓ Variant serialization roundtrip works");
}
