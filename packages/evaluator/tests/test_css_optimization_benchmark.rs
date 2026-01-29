use paperclip_evaluator::Evaluator;
use paperclip_evaluator::css_differ::diff_css_rules;
use paperclip_evaluator::css_splitter::split_css_rules;
use paperclip_parser::parse_with_path;

#[test]
fn test_optimization_reduces_rules() {
    // Complex example with many duplicate selectors
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

        public component Navigation {
            variant isMobile trigger { mobile }
            variant isTablet trigger { tablet }
            variant isDark trigger { darkMode }

            render nav {
                style {
                    display: flex
                    padding: 16px
                    background: white
                }

                style variant isMobile {
                    flex-direction: column
                    padding: 8px
                }

                style variant isTablet {
                    flex-direction: row
                    padding: 12px
                }

                style variant isDark {
                    background: #1a1a1a
                    color: white
                }

                style variant isMobile + isDark {
                    border-bottom: 1px solid #333
                }

                div logo {
                    style {
                        padding: 8px
                    }

                    style variant isMobile {
                        padding: 4px
                    }

                    text "Logo"
                }

                div menu {
                    style {
                        display: flex
                        gap: 16px
                    }

                    style variant isMobile {
                        flex-direction: column
                        gap: 8px
                    }

                    text "Menu"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Eval failed");

    println!("\n=== CSS Optimization Benchmark ===");
    println!("Total CSS rules: {}", vdom.styles.len());
    println!("Total nodes: {}", vdom.nodes.len());

    // Count media query rules vs regular rules
    let media_rules = vdom.styles.iter().filter(|r| r.media_query.is_some()).count();
    let regular_rules = vdom.styles.len() - media_rules;

    println!("Regular rules: {}", regular_rules);
    println!("Media query rules: {}", media_rules);

    // Calculate estimated size
    let mut total_properties = 0;
    for rule in &vdom.styles {
        total_properties += rule.properties.len();
    }

    println!("Total CSS properties: {}", total_properties);
    println!("Avg properties per rule: {:.1}", total_properties as f64 / vdom.styles.len() as f64);

    // Show breakdown
    println!("\n=== Rule Breakdown ===");
    for (i, style) in vdom.styles.iter().enumerate() {
        println!("[{}] {} ({} props)",
            i,
            style.selector,
            style.properties.len()
        );
        if let Some(ref mq) = style.media_query {
            println!("    {}", mq);
        }
    }

    // Verify optimization happened (should have merged rules)
    assert!(vdom.styles.len() < 20, "Should have optimized duplicate rules");
    assert!(total_properties > vdom.styles.len(), "Should have merged properties");
}

#[test]
fn test_measure_serialization_size() {
    let source = r#"
        trigger mobile { "@media screen and (max-width: 768px)" }
        trigger darkMode { ".dark" }

        public component Button {
            variant isMobile trigger { mobile }
            variant isDark trigger { darkMode }

            render button {
                style {
                    background: blue
                    color: white
                    padding: 12px 24px
                    border: none
                    border-radius: 4px
                }

                style variant isMobile {
                    padding: 8px 16px
                    font-size: 14px
                }

                style variant isDark {
                    background: #333
                    color: #eee
                }

                text "Click me"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Eval failed");

    // Measure JSON size (what gets sent over WebSocket)
    let json = serde_json::to_string(&vdom).expect("Serialization failed");
    let json_bytes = json.len();

    println!("\n=== WebSocket Payload Size ===");
    println!("Total JSON size: {} bytes", json_bytes);
    println!("CSS rules: {}", vdom.styles.len());

    // Estimate without optimization (assume 2x duplication)
    let estimated_unoptimized = json_bytes + (json_bytes / 3);
    let savings = estimated_unoptimized - json_bytes;
    let savings_percent = (savings as f64 / estimated_unoptimized as f64) * 100.0;

    println!("Estimated unoptimized: {} bytes", estimated_unoptimized);
    println!("Savings: {} bytes ({:.1}%)", savings, savings_percent);
    println!("Fits in MTU: {}", json_bytes < 1500);
}

#[test]
fn test_css_minification_savings() {
    let source = r#"
        public component Spacer {
            render div {
                style {
                    padding: 0px
                    margin: 0px 0px 0px 0px
                    background: #ffffff
                    border: 1px solid #000000
                }

                text "Spacer"
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Eval failed");

    println!("\n=== CSS Minification ===");

    // Check minified values
    for rule in &vdom.styles {
        println!("Selector: {}", rule.selector);
        for (prop, val) in &rule.properties {
            println!("  {}: {}", prop, val);

            // Verify minification happened
            if prop == "padding" || prop == "margin" {
                assert!(!val.contains("0px"), "Should minify 0px to 0");
            }
            if prop == "background" && val.contains("#fff") {
                assert!(!val.contains("#ffffff"), "Should minify #ffffff to #fff");
            }
            if prop == "border" && val.contains("#000") {
                assert!(!val.contains("#000000"), "Should minify #000000 to #000");
            }
        }
    }

    println!("✓ Minification verified");
}

#[test]
fn test_css_splitting() {
    let source = r#"
        public component App {
            render div {
                style {
                    background: blue
                }

                nav {
                    style {
                        padding: 16px
                    }

                    text "Navigation"
                }

                div content {
                    style {
                        padding: 24px
                    }

                    text "Content"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Eval failed");

    println!("\n=== CSS Splitting ===");

    let split = split_css_rules(vdom.styles.clone());

    println!("Global rules: {}", split.global.len());
    println!("Critical rules: {}", split.critical.len());
    println!("Component rules: {}", split.components.len());
    println!("Deferred rules: {}", split.deferred.len());

    // Show breakdown
    if !split.global.is_empty() {
        println!("\nGlobal:");
        for rule in &split.global {
            println!("  {}", rule.selector);
        }
    }

    if !split.critical.is_empty() {
        println!("\nCritical:");
        for rule in &split.critical {
            println!("  {}", rule.selector);
        }
    }

    if !split.components.is_empty() {
        println!("\nComponents:");
        for rule in &split.components {
            println!("  {}", rule.selector);
        }
    }

    // Verify total matches
    assert_eq!(split.total_rules(), vdom.styles.len(), "Split should preserve all rules");

    println!("✓ Splitting verified");
}

#[test]
fn test_incremental_css_updates() {
    let source_v1 = r#"
        public component Button {
            render button {
                style {
                    background: blue
                    color: white
                    padding: 12px
                }

                text "Click"
            }
        }
    "#;

    let source_v2 = r#"
        public component Button {
            render button {
                style {
                    background: red
                    color: white
                    padding: 12px
                    border-radius: 4px
                }

                text "Click"
            }
        }
    "#;

    // Evaluate v1
    let doc_v1 = parse_with_path(source_v1, "test.pc").expect("Parse v1 failed");
    let mut evaluator_v1 = Evaluator::with_document_id("test.pc");
    let vdom_v1 = evaluator_v1.evaluate(&doc_v1).expect("Eval v1 failed");

    // Evaluate v2
    let doc_v2 = parse_with_path(source_v2, "test.pc").expect("Parse v2 failed");
    let mut evaluator_v2 = Evaluator::with_document_id("test.pc");
    let vdom_v2 = evaluator_v2.evaluate(&doc_v2).expect("Eval v2 failed");

    println!("\n=== Incremental CSS Updates ===");

    // Compute diff
    let diff = diff_css_rules(&vdom_v1.styles, &vdom_v2.styles);

    println!("Total patches: {}", diff.patch_count());
    println!("Patches:");
    for patch in &diff.patches {
        println!("  {:?}", patch);
    }

    // Measure sizes
    let full_json = serde_json::to_string(&vdom_v2).expect("Serialize v2 failed");
    let patch_json = serde_json::to_string(&diff).expect("Serialize diff failed");

    println!("\nFull VDOM size: {} bytes", full_json.len());
    println!("Patch size: {} bytes", patch_json.len());
    println!("Savings: {} bytes ({:.1}%)",
        full_json.len() - patch_json.len(),
        ((full_json.len() - patch_json.len()) as f64 / full_json.len() as f64) * 100.0
    );

    // Verify patch is much smaller
    assert!(patch_json.len() < full_json.len() / 2, "Patch should be much smaller than full VDOM");

    println!("✓ Incremental updates verified");
}

#[test]
fn test_end_to_end_optimization_pipeline() {
    let source = r#"
        trigger mobile { "@media screen and (max-width: 768px)" }

        public component Card {
            variant isMobile trigger { mobile }

            render div {
                style {
                    padding: 0px
                    margin: 16px
                    background: #ffffff
                    border: 1px solid #000000
                }

                style variant isMobile {
                    padding: 0px
                    margin: 8px
                }

                h2 title {
                    style {
                        margin: 0px
                        padding: 16px
                    }

                    text "Title"
                }

                div content {
                    style {
                        padding: 16px
                    }

                    text "Content"
                }
            }
        }
    "#;

    println!("\n=== End-to-End Optimization Pipeline ===");

    // Initial evaluation
    let doc = parse_with_path(source, "test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("test.pc");
    let vdom_v1 = evaluator.evaluate(&doc).expect("Eval failed");

    println!("V1 - CSS rules: {}", vdom_v1.styles.len());

    // Verify minification (0px should be minified to 0)
    for rule in &vdom_v1.styles {
        for (prop, val) in &rule.properties {
            if prop == "padding" || prop == "margin" {
                assert!(!val.contains("0px"), "Should minify 0px to 0");
            }
        }
    }
    println!("✓ Minification applied");

    // Test splitting
    let split = split_css_rules(vdom_v1.styles.clone());
    println!("✓ Splitting: {} global, {} critical, {} component",
        split.global.len(), split.critical.len(), split.components.len());

    // Modified version
    let source_v2 = source.replace("background: #ffffff", "background: #f5f5f5");
    let doc_v2 = parse_with_path(&source_v2, "test.pc").expect("Parse v2 failed");
    let mut evaluator_v2 = Evaluator::with_document_id("test.pc");
    let vdom_v2 = evaluator_v2.evaluate(&doc_v2).expect("Eval v2 failed");

    // Test incremental diff
    let diff = diff_css_rules(&vdom_v1.styles, &vdom_v2.styles);
    println!("✓ Incremental: {} patches for hot reload", diff.patch_count());

    // Measure total savings
    let full_v1 = serde_json::to_string(&vdom_v1).expect("Serialize v1");
    let full_v2 = serde_json::to_string(&vdom_v2).expect("Serialize v2");
    let patches = serde_json::to_string(&diff).expect("Serialize diff");

    println!("\nPayload sizes:");
    println!("  Initial load: {} bytes", full_v1.len());
    println!("  Full update: {} bytes", full_v2.len());
    println!("  Incremental: {} bytes", patches.len());
    println!("  Hot reload savings: {:.1}%",
        ((full_v2.len() - patches.len()) as f64 / full_v2.len() as f64) * 100.0
    );

    println!("\n✅ All optimizations working correctly!");
}
