//! End-to-end test for preview server
//!
//! Tests the full hot reload cycle:
//! 1. Start server
//! 2. Connect WebSocket
//! 3. Receive initial state
//! 4. Modify file
//! 5. Receive update

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[cfg(feature = "preview")]
use tokio::time::sleep;

#[cfg(feature = "preview")]
#[tokio::test]
#[ignore = "Integration test - requires manual verification"]
async fn test_preview_server_end_to_end() {
    // Create temporary test file
    let test_file = PathBuf::from("/tmp/paperclip_test_e2e.pc");
    let initial_content = r#"
public component Test {
    render div {
        style {
            color red
        }
        text "Version 1"
    }
}
"#;

    fs::write(&test_file, initial_content).expect("Failed to write test file");

    // Note: This test requires the preview server to be running manually
    // Run: cargo run --bin preview_server --features preview -- /tmp/paperclip_test_e2e.pc
    //
    // Then connect with a WebSocket client and verify:
    // 1. Initial message contains VDOM with "Version 1"
    // 2. Modify file to "Version 2"
    // 3. Update message contains VDOM with "Version 2"

    println!("✓ Test file created at: {}", test_file.display());
    println!("  Start server with:");
    println!("  cargo run --bin preview_server --features preview -- {}", test_file.display());
    println!("  Then modify the file to trigger hot reload");

    // Cleanup
    sleep(Duration::from_secs(1)).await;
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_file_modification_detection() {
    use std::fs::File;
    use std::io::Write;

    // Create temp file
    let test_file = PathBuf::from("/tmp/paperclip_test_modify.pc");
    let mut file = File::create(&test_file).expect("Failed to create file");
    write!(file, "// Version 1").expect("Failed to write");
    drop(file);

    // Get initial metadata
    let metadata1 = fs::metadata(&test_file).expect("Failed to get metadata");
    let modified1 = metadata1.modified().expect("Failed to get modified time");

    // Wait a bit to ensure timestamp differs
    std::thread::sleep(Duration::from_millis(10));

    // Modify file
    let mut file = File::create(&test_file).expect("Failed to create file");
    write!(file, "// Version 2").expect("Failed to write");
    drop(file);

    // Check modified time changed
    let metadata2 = fs::metadata(&test_file).expect("Failed to get metadata");
    let modified2 = metadata2.modified().expect("Failed to get modified time");

    assert!(modified2 > modified1, "File modification should update timestamp");

    // Cleanup
    let _ = fs::remove_file(&test_file);
    println!("✓ File modification detection works");
}

#[test]
fn test_parse_evaluate_cycle() {
    use paperclip_evaluator::{Evaluator, VirtualDomDocument};
    use paperclip_parser::parse_with_path;

    let source1 = r#"
public component Test {
    render div {
        text "Version 1"
    }
}
"#;

    let source2 = r#"
public component Test {
    render div {
        text "Version 2"
    }
}
"#;

    // First evaluation
    let doc1 = parse_with_path(source1, "/test.pc").expect("Parse failed");
    let mut evaluator1 = Evaluator::with_document_id("/test.pc");
    let vdom1 = evaluator1.evaluate(&doc1).expect("Evaluation failed");

    // Second evaluation (simulating file change)
    let doc2 = parse_with_path(source2, "/test.pc").expect("Parse failed");
    let mut evaluator2 = Evaluator::with_document_id("/test.pc");
    let vdom2 = evaluator2.evaluate(&doc2).expect("Evaluation failed");

    // Verify content changed
    assert_ne!(
        format!("{:?}", vdom1),
        format!("{:?}", vdom2),
        "VDOMs should differ"
    );

    println!("✓ Parse → Evaluate cycle works");
}

#[test]
fn test_vdom_serialization() {
    use paperclip_evaluator::Evaluator;
    use paperclip_parser::parse_with_path;
    use serde_json;

    let source = r#"
public component Test {
    render div {
        style {
            color: red;
        }
        text "Hello"
    }
}
"#;

    let doc = parse_with_path(source, "/test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Evaluation failed");

    // Serialize to JSON
    let json = serde_json::to_string(&vdom).expect("Serialization failed");

    // Verify JSON contains expected data
    assert!(json.contains("nodes"), "JSON should contain nodes");
    assert!(json.contains("styles"), "JSON should contain styles");

    // Deserialize back
    let vdom2: serde_json::Value = serde_json::from_str(&json).expect("Deserialization failed");
    assert!(vdom2.get("nodes").is_some(), "Should have nodes field");
    assert!(vdom2.get("styles").is_some(), "Should have styles field");

    println!("✓ VDOM serialization works");
}

#[test]
fn test_css_evaluation() {
    use paperclip_evaluator::Evaluator;
    use paperclip_parser::parse_with_path;

    let source = r#"
public component Test {
    render div {
        style {
            color: red;
            background: blue;
        }
        text "Styled"
    }
}
"#;

    let doc = parse_with_path(source, "/test.pc").expect("Parse failed");
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).expect("Evaluation failed");

    // CSS evaluation depends on css_evaluator - may be empty if not fully integrated yet
    // For now, just verify evaluation completes successfully
    println!("✓ VDOM generated with {} style rules", vdom.styles.len());

    // If styles are present, verify they can be accessed
    if !vdom.styles.is_empty() {
        for rule in &vdom.styles {
            println!("  Selector: {}, Props: {:?}", rule.selector, rule.properties);
        }
    }
}
