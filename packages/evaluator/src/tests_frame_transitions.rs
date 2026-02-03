/// Frame transition tests using macro-based approach
///
/// Tests the pattern: source -> evaluate -> check frames
/// Also tests transitions: before -> after -> frames persist
///
/// This follows the testing pattern from the original paperclip codebase.

use crate::*;
use paperclip_bundle::Bundle;
use paperclip_parser::parse_with_path;
use std::path::PathBuf;

/// Helper to evaluate a source and extract frame attributes
fn evaluate_and_get_frames(source: &str) -> Vec<Option<(f64, f64, Option<f64>, Option<f64>)>> {
    let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
    let mut bundle = Bundle::new();
    bundle.add_document(PathBuf::from("/test.pc"), doc);

    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdoc = evaluator
        .evaluate_bundle(&bundle, std::path::Path::new("/test.pc"))
        .expect("Failed to evaluate");

    vdoc.nodes
        .iter()
        .map(|node| {
            if let VNode::Element { attributes, .. } = node {
                let x = attributes.get("data-frame-x").and_then(|v| v.parse().ok());
                let y = attributes.get("data-frame-y").and_then(|v| v.parse().ok());
                let w = attributes.get("data-frame-width").and_then(|v| v.parse().ok());
                let h = attributes.get("data-frame-height").and_then(|v| v.parse().ok());
                if x.is_some() || y.is_some() {
                    Some((x.unwrap_or(0.0), y.unwrap_or(0.0), w, h))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Macro for testing source -> frame output
macro_rules! frame_case {
    ($name:ident, $source:expr, $expected_frames:expr) => {
        #[test]
        fn $name() {
            let frames = evaluate_and_get_frames($source);
            let expected: Vec<Option<(f64, f64, Option<f64>, Option<f64>)>> = $expected_frames;
            assert_eq!(
                frames.len(),
                expected.len(),
                "Frame count mismatch: got {} frames, expected {}",
                frames.len(),
                expected.len()
            );
            for (i, (got, exp)) in frames.iter().zip(expected.iter()).enumerate() {
                assert_eq!(
                    got, exp,
                    "Frame {} mismatch: got {:?}, expected {:?}",
                    i, got, exp
                );
            }
        }
    };
}

/// Macro for testing transitions - both before and after should have frames
macro_rules! transition_case {
    ($name:ident, $before:expr, $after:expr) => {
        #[test]
        fn $name() {
            let before_frames = evaluate_and_get_frames($before);
            let after_frames = evaluate_and_get_frames($after);

            // Both should have the same number of frames
            assert_eq!(
                before_frames.len(),
                after_frames.len(),
                "Frame count changed: before={}, after={}",
                before_frames.len(),
                after_frames.len()
            );

            // Frame coordinates should be preserved
            for (i, (before, after)) in before_frames.iter().zip(after_frames.iter()).enumerate() {
                if let (Some(b), Some(a)) = (before, after) {
                    assert_eq!(
                        b.0, a.0,
                        "Frame {} x changed: before={}, after={}",
                        i, b.0, a.0
                    );
                    assert_eq!(
                        b.1, a.1,
                        "Frame {} y changed: before={}, after={}",
                        i, b.1, a.1
                    );
                }
                // Both should have frame or both should not
                assert_eq!(
                    before.is_some(),
                    after.is_some(),
                    "Frame {} presence changed: before={:?}, after={:?}",
                    i, before, after
                );
            }
        }
    };
}

// ============================================================================
// Basic Frame Tests
// ============================================================================

frame_case! {
    test_simple_div_with_frame,
    r#"/**
 * @frame(x: 100, y: 200, width: 400, height: 300)
 */
div {
    text "hello"
}"#,
    vec![Some((100.0, 200.0, Some(400.0), Some(300.0)))]
}

frame_case! {
    test_component_with_frame,
    r#"/**
 * @frame(x: 50, y: 75)
 */
public component Button {
    render button {
        text "Click"
    }
}"#,
    vec![Some((50.0, 75.0, None, None))]
}

frame_case! {
    test_multiple_frames,
    r#"/**
 * @frame(x: 0, y: 0, width: 200, height: 100)
 */
public component A {
    render div { text "A" }
}

/**
 * @frame(x: 300, y: 0, width: 200, height: 100)
 */
public component B {
    render div { text "B" }
}

/**
 * @frame(x: 600, y: 0)
 */
div {
    text "Render"
}"#,
    vec![
        Some((0.0, 0.0, Some(200.0), Some(100.0))),
        Some((300.0, 0.0, Some(200.0), Some(100.0))),
        Some((600.0, 0.0, None, None))
    ]
}

// ============================================================================
// Transition Tests - Frame Persistence
// ============================================================================

transition_case! {
    test_adding_text_content_preserves_frame,
    // Before
    r#"/**
 * @frame(x: 100, y: 100)
 */
div {
    text "hello"
}"#,
    // After
    r#"/**
 * @frame(x: 100, y: 100)
 */
div {
    text "hello world"
}"#
}

transition_case! {
    test_adding_style_block_preserves_frame,
    // Before: div with text, no style
    r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    text "hello"
}"#,
    // After: div with text and style
    r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    style {
        color: red
    }
    text "hello"
}"#
}

transition_case! {
    test_adding_style_to_text_element_preserves_frame,
    // Before: text with empty braces (user bug report)
    r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    style {
        padding: 32px
        color: orange
        font-size: 32px
        font-weight: bold
        text-decoration: underline
    }
    text "hello wodrlddddd" {
    }
}"#,
    // After: text with style block (user bug report - this caused frame to disappear)
    r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    style {
        padding: 32px
        color: orange
        font-size: 32px
        font-weight: bold
        text-decoration: underline
    }
    text "hello wodrlddddd" {
        style {
            color: red
        }
    }
}"#
}

transition_case! {
    test_adding_nested_element_preserves_frame,
    // Before
    r#"/**
 * @frame(x: 50, y: 50)
 */
div {
    text "header"
}"#,
    // After
    r#"/**
 * @frame(x: 50, y: 50)
 */
div {
    text "header"
    div {
        text "content"
    }
}"#
}

transition_case! {
    test_changing_component_body_preserves_frame,
    // Before
    r#"/**
 * @frame(x: 200, y: 100)
 */
public component Card {
    render div {
        text "title"
    }
}"#,
    // After
    r#"/**
 * @frame(x: 200, y: 100)
 */
public component Card {
    render div {
        text "title"
        div {
            text "content"
        }
    }
}"#
}

// ============================================================================
// Edge Case Tests
// ============================================================================

frame_case! {
    test_text_with_style_block_has_frame,
    r#"/**
 * @frame(x: 0, y: 0, width: 500, height: 400)
 */
div {
    text "styled text" {
        style {
            color: blue
            font-size: 24px
        }
    }
}"#,
    vec![Some((0.0, 0.0, Some(500.0), Some(400.0)))]
}

frame_case! {
    test_multiple_text_with_styles,
    r#"/**
 * @frame(x: 100, y: 200)
 */
div {
    text "first" {
        style { color: red }
    }
    text "second" {
        style { color: blue }
    }
    text "third" {
        style { color: green }
    }
}"#,
    vec![Some((100.0, 200.0, None, None))]
}

frame_case! {
    test_deeply_nested_with_frame,
    r#"/**
 * @frame(x: 0, y: 0, width: 1000, height: 800)
 */
div {
    div {
        div {
            div {
                text "deep" {
                    style { color: purple }
                }
            }
        }
    }
}"#,
    vec![Some((0.0, 0.0, Some(1000.0), Some(800.0)))]
}

// ============================================================================
// Sequence-Based Transition Tests
// ============================================================================

/// Expectation type for transition tests
#[derive(Debug, Clone)]
enum TransitionExpectation {
    /// Expect successful evaluation with specific frames
    Ok(Vec<Option<(f64, f64, Option<f64>, Option<f64>)>>),
    /// Expect a parse or evaluation error (syntax error)
    Error,
    /// Expect frames to recover to previous valid state
    Recovers,
}

/// Helper to evaluate source and return Result (for error handling)
fn try_evaluate_frames(source: &str) -> Result<Vec<Option<(f64, f64, Option<f64>, Option<f64>)>>, String> {
    let doc = match parse_with_path(source, "/test.pc") {
        Ok(doc) => doc,
        Err(e) => return Err(format!("Parse error: {:?}", e)),
    };

    let mut bundle = Bundle::new();
    bundle.add_document(PathBuf::from("/test.pc"), doc);

    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdoc = match evaluator.evaluate_bundle(&bundle, std::path::Path::new("/test.pc")) {
        Ok(vdoc) => vdoc,
        Err(e) => return Err(format!("Eval error: {:?}", e)),
    };

    // Check for error nodes
    for node in &vdoc.nodes {
        if let VNode::Error { message, .. } = node {
            return Err(format!("Error node: {}", message));
        }
    }

    Ok(vdoc.nodes
        .iter()
        .map(|node| {
            if let VNode::Element { attributes, .. } = node {
                let x = attributes.get("data-frame-x").and_then(|v| v.parse().ok());
                let y = attributes.get("data-frame-y").and_then(|v| v.parse().ok());
                let w = attributes.get("data-frame-width").and_then(|v| v.parse().ok());
                let h = attributes.get("data-frame-height").and_then(|v| v.parse().ok());
                if x.is_some() || y.is_some() {
                    Some((x.unwrap_or(0.0), y.unwrap_or(0.0), w, h))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect())
}

/// Macro for testing transition sequences with mutations, errors, and recovery
///
/// Each step provides a source and an expectation:
/// - `ok [ frame1, frame2, ... ]` - expect successful evaluation with these frames
/// - `error` - expect parse/evaluation error (syntax error)
/// - `recovers` - expect frames to match previous valid state
///
/// Usage:
/// ```ignore
/// test_transition! {
///     test_name,
///     "source1" => ok [ Some((100.0, 100.0, None, None)) ],
///     "source2" => ok [ Some((100.0, 100.0, None, None)) ],
///     "source3_with_syntax_error" => error,
///     "source4_fixed" => recovers,
/// }
/// ```
macro_rules! test_transition {
    ($name:ident, $( $source:expr => $expectation:tt $( [ $($frame:expr),* $(,)? ] )? ),+ $(,)?) => {
        #[test]
        fn $name() {
            let mut last_valid_frames: Option<Vec<Option<(f64, f64, Option<f64>, Option<f64>)>>> = None;
            let mut step_idx = 0;

            $(
                step_idx += 1;
                let source = $source;
                let expectation = test_transition!(@parse_expectation $expectation $( [ $($frame),* ] )? );

                match expectation {
                    TransitionExpectation::Ok(ref expected_frames) => {
                        let result = try_evaluate_frames(source);
                        match result {
                            Ok(frames) => {
                                assert_eq!(
                                    &frames, expected_frames,
                                    "Step {} - Frame mismatch:\n  Source: {:?}\n  Expected: {:?}\n  Got: {:?}",
                                    step_idx, source.chars().take(50).collect::<String>(), expected_frames, frames
                                );
                                last_valid_frames = Some(frames);
                            }
                            Err(e) => {
                                panic!(
                                    "Step {} - Expected OK but got error:\n  Source: {:?}\n  Error: {}",
                                    step_idx, source.chars().take(50).collect::<String>(), e
                                );
                            }
                        }
                    }
                    TransitionExpectation::Error => {
                        let result = try_evaluate_frames(source);
                        assert!(
                            result.is_err(),
                            "Step {} - Expected error but evaluation succeeded:\n  Source: {:?}\n  Result: {:?}",
                            step_idx, source.chars().take(50).collect::<String>(), result
                        );
                    }
                    TransitionExpectation::Recovers => {
                        let result = try_evaluate_frames(source);
                        match result {
                            Ok(frames) => {
                                if let Some(ref last) = last_valid_frames {
                                    assert_eq!(
                                        &frames, last,
                                        "Step {} - Recovery failed:\n  Source: {:?}\n  Expected (last valid): {:?}\n  Got: {:?}",
                                        step_idx, source.chars().take(50).collect::<String>(), last, frames
                                    );
                                }
                                last_valid_frames = Some(frames);
                            }
                            Err(e) => {
                                panic!(
                                    "Step {} - Expected recovery but got error:\n  Source: {:?}\n  Error: {}",
                                    step_idx, source.chars().take(50).collect::<String>(), e
                                );
                            }
                        }
                    }
                }
            )+
        }
    };

    // Helper to parse expectation
    (@parse_expectation ok [ $($frame:expr),* $(,)? ]) => {
        TransitionExpectation::Ok(vec![$($frame),*])
    };
    (@parse_expectation error) => {
        TransitionExpectation::Error
    };
    (@parse_expectation recovers) => {
        TransitionExpectation::Recovers
    };
}

// ============================================================================
// Sequence Transition Tests
// ============================================================================

test_transition! {
    test_sequence_typing_text,
    // Initial state
    r#"/**
 * @frame(x: 100, y: 100)
 */
div {
    text "h"
}"# => ok [ Some((100.0, 100.0, None, None)) ],
    // Type more characters
    r#"/**
 * @frame(x: 100, y: 100)
 */
div {
    text "he"
}"# => ok [ Some((100.0, 100.0, None, None)) ],
    // Complete word
    r#"/**
 * @frame(x: 100, y: 100)
 */
div {
    text "hello"
}"# => ok [ Some((100.0, 100.0, None, None)) ],
    // Add space and more
    r#"/**
 * @frame(x: 100, y: 100)
 */
div {
    text "hello world"
}"# => ok [ Some((100.0, 100.0, None, None)) ],
}

test_transition! {
    test_sequence_adding_style_block,
    // Initial state - text with empty braces
    r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    text "hello" {
    }
}"# => ok [ Some((100.0, 100.0, Some(400.0), Some(300.0))) ],
    // Add style block
    r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    text "hello" {
        style {
        }
    }
}"# => ok [ Some((100.0, 100.0, Some(400.0), Some(300.0))) ],
    // Add style property
    r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    text "hello" {
        style {
            color: red
        }
    }
}"# => ok [ Some((100.0, 100.0, Some(400.0), Some(300.0))) ],
}

test_transition! {
    test_sequence_nested_elements,
    // Initial
    r#"/**
 * @frame(x: 50, y: 50)
 */
div {
}"# => ok [ Some((50.0, 50.0, None, None)) ],
    // Add text
    r#"/**
 * @frame(x: 50, y: 50)
 */
div {
    text "header"
}"# => ok [ Some((50.0, 50.0, None, None)) ],
    // Add nested div
    r#"/**
 * @frame(x: 50, y: 50)
 */
div {
    text "header"
    div {
    }
}"# => ok [ Some((50.0, 50.0, None, None)) ],
    // Add content to nested div
    r#"/**
 * @frame(x: 50, y: 50)
 */
div {
    text "header"
    div {
        text "content"
    }
}"# => ok [ Some((50.0, 50.0, None, None)) ],
    // Add another nested level
    r#"/**
 * @frame(x: 50, y: 50)
 */
div {
    text "header"
    div {
        text "content"
        div {
            text "deep"
        }
    }
}"# => ok [ Some((50.0, 50.0, None, None)) ],
}

test_transition! {
    test_sequence_style_modifications,
    // Initial with one style
    r#"/**
 * @frame(x: 0, y: 0, width: 200, height: 200)
 */
div {
    style {
        color: red
    }
}"# => ok [ Some((0.0, 0.0, Some(200.0), Some(200.0))) ],
    // Add another property
    r#"/**
 * @frame(x: 0, y: 0, width: 200, height: 200)
 */
div {
    style {
        color: red
        font-size: 16px
    }
}"# => ok [ Some((0.0, 0.0, Some(200.0), Some(200.0))) ],
    // Change existing property
    r#"/**
 * @frame(x: 0, y: 0, width: 200, height: 200)
 */
div {
    style {
        color: blue
        font-size: 16px
    }
}"# => ok [ Some((0.0, 0.0, Some(200.0), Some(200.0))) ],
    // Add more properties
    r#"/**
 * @frame(x: 0, y: 0, width: 200, height: 200)
 */
div {
    style {
        color: blue
        font-size: 16px
        padding: 10px
        margin: 5px
    }
}"# => ok [ Some((0.0, 0.0, Some(200.0), Some(200.0))) ],
}

test_transition! {
    test_sequence_multiple_frames,
    // Start with one component
    r#"/**
 * @frame(x: 0, y: 0)
 */
public component A {
    render div { text "A" }
}"# => ok [ Some((0.0, 0.0, None, None)) ],
    // Add second component
    r#"/**
 * @frame(x: 0, y: 0)
 */
public component A {
    render div { text "A" }
}

/**
 * @frame(x: 200, y: 0)
 */
public component B {
    render div { text "B" }
}"# => ok [ Some((0.0, 0.0, None, None)), Some((200.0, 0.0, None, None)) ],
    // Modify first component content
    r#"/**
 * @frame(x: 0, y: 0)
 */
public component A {
    render div {
        text "A"
        text "more content"
    }
}

/**
 * @frame(x: 200, y: 0)
 */
public component B {
    render div { text "B" }
}"# => ok [ Some((0.0, 0.0, None, None)), Some((200.0, 0.0, None, None)) ],
}

test_transition! {
    test_sequence_error_and_recovery,
    // Initial valid state
    r#"/**
 * @frame(x: 100, y: 100)
 */
div {
    text "hello"
}"# => ok [ Some((100.0, 100.0, None, None)) ],
    // Syntax error - unclosed brace
    r#"/**
 * @frame(x: 100, y: 100)
 */
div {
    text "hello"
"# => error,
    // Another syntax error
    r#"/**
 * @frame(x: 100, y: 100)
 */
div {
    text "hello" {
        style {
"# => error,
    // Recovery - valid syntax again, should have same frames
    r#"/**
 * @frame(x: 100, y: 100)
 */
div {
    text "hello"
}"# => recovers,
}

test_transition! {
    test_sequence_incomplete_typing_recovery,
    // Start typing a component
    r#"/**
 * @frame(x: 50, y: 50, width: 300, height: 200)
 */
public component Button {
    render button {
        text "Click"
    }
}"# => ok [ Some((50.0, 50.0, Some(300.0), Some(200.0))) ],
    // User starts typing a new style block but hasn't closed it yet
    r#"/**
 * @frame(x: 50, y: 50, width: 300, height: 200)
 */
public component Button {
    render button {
        style {
            color: red
        text "Click"
    }
}"# => error,
    // User finishes typing
    r#"/**
 * @frame(x: 50, y: 50, width: 300, height: 200)
 */
public component Button {
    render button {
        style {
            color: red
        }
        text "Click"
    }
}"# => recovers,
}

// ============================================================================
// Patch Generation Tests - verify patches preserve frames
// ============================================================================

#[test]
fn test_patch_generation_adding_style_to_text() {
    use crate::diff_vdocument;

    let before = r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    style {
        padding: 32px
    }
    text "hello" {
    }
}"#;

    let after = r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    style {
        padding: 32px
    }
    text "hello" {
        style {
            color: red
        }
    }
}"#;

    // Evaluate before
    let doc_before = parse_with_path(before, "/test.pc").unwrap();
    let mut bundle_before = Bundle::new();
    bundle_before.add_document(PathBuf::from("/test.pc"), doc_before);
    let mut eval_before = Evaluator::with_document_id("/test.pc");
    let vdom_before = eval_before
        .evaluate_bundle(&bundle_before, std::path::Path::new("/test.pc"))
        .unwrap();

    // Evaluate after
    let doc_after = parse_with_path(after, "/test.pc").unwrap();
    let mut bundle_after = Bundle::new();
    bundle_after.add_document(PathBuf::from("/test.pc"), doc_after);
    let mut eval_after = Evaluator::with_document_id("/test.pc");
    let vdom_after = eval_after
        .evaluate_bundle(&bundle_after, std::path::Path::new("/test.pc"))
        .unwrap();

    // Check both have frames
    let before_semantic = vdom_before.nodes.first().and_then(|n| {
        if let VNode::Element { semantic_id, attributes, .. } = n {
            println!("Before semantic_id: {:?}", semantic_id);
            println!("Before attributes: {:?}", attributes);
            Some(semantic_id.clone())
        } else {
            None
        }
    });
    let after_semantic = vdom_after.nodes.first().and_then(|n| {
        if let VNode::Element { semantic_id, attributes, .. } = n {
            println!("After semantic_id: {:?}", semantic_id);
            println!("After attributes: {:?}", attributes);
            Some(semantic_id.clone())
        } else {
            None
        }
    });

    println!("Before semantic: {:?}", before_semantic);
    println!("After semantic: {:?}", after_semantic);

    let before_has_frame = vdom_before.nodes.first().and_then(|n| {
        if let VNode::Element { attributes, .. } = n {
            attributes.get("data-frame-x")
        } else {
            None
        }
    });
    let after_has_frame = vdom_after.nodes.first().and_then(|n| {
        if let VNode::Element { attributes, .. } = n {
            attributes.get("data-frame-x")
        } else {
            None
        }
    });

    assert!(before_has_frame.is_some(), "Before should have frame");
    assert!(after_has_frame.is_some(), "After should have frame");

    // Generate patches
    let patches = diff_vdocument(&vdom_before, &vdom_after);

    println!("Patches generated: {}", patches.len());
    for (i, patch) in patches.iter().enumerate() {
        println!("  Patch {}: {:?}", i, patch.patch_type);
    }

    // If there's a ReplaceNode patch, the new node should have frame attributes
    use crate::vdom_differ::proto::vdom::v_node::NodeType;
    for patch in &patches {
        if let Some(crate::vdom_differ::proto::patches::v_doc_patch::PatchType::ReplaceNode(
            replace,
        )) = &patch.patch_type
        {
            if let Some(new_node) = &replace.new_node {
                println!("ReplaceNode new_node node_type: {:?}", new_node.node_type.is_some());
                // Check if new node has frame attributes
                if let Some(NodeType::Element(elem)) = &new_node.node_type {
                    println!("ReplaceNode element attributes: {:?}", elem.attributes.keys().collect::<Vec<_>>());
                    assert!(
                        elem.attributes.contains_key("data-frame-x"),
                        "ReplaceNode new_node should have data-frame-x attribute, got: {:?}",
                        elem.attributes.keys().collect::<Vec<_>>()
                    );
                }
            }
        }
    }
}
