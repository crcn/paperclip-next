/// Determinism tests - ensure evaluation is deterministic and reproducible
///
/// These tests validate that:
/// - Same input produces identical output across multiple evaluations
/// - No HashMap iteration order leaks
/// - No non-deterministic ID generation
/// - Output is byte-for-byte identical

use paperclip_evaluator::{Evaluator, VirtualDomDocument};
use paperclip_parser::parse;

#[test]
fn test_evaluation_determinism_simple_component() {
    let source = r#"
        component Card {
            render div {
                text "Hello"
                span { text "World" }
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");

    // Evaluate 10 times
    let results: Vec<VirtualDomDocument> = (0..10)
        .map(|_| {
            let mut eval = Evaluator::new();
            eval.evaluate(&doc).expect("Evaluation failed")
        })
        .collect();

    // All results should be identical
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Evaluation {} differs from evaluation 0",
            i
        );
    }
}

#[test]
fn test_evaluation_determinism_with_variables() {
    let source = r#"
        component Card {
            render div {
                style {
                    color: #3B82F6;
                    background: #F3F4F6;
                }
                text title
                span { text subtitle }
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");

    // Evaluate 10 times
    let results: Vec<VirtualDomDocument> = (0..10)
        .map(|_| {
            let mut eval = Evaluator::new();
            eval.evaluate(&doc).expect("Evaluation failed")
        })
        .collect();

    // All results should be identical
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Evaluation {} differs from evaluation 0 (variables should resolve identically)",
            i
        );
    }
}

#[test]
fn test_evaluation_determinism_with_repeat() {
    let source = r#"
        component List {
            render ul {
                repeat item in items {
                    li { text item.name }
                }
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");

    // Evaluate 10 times
    let results: Vec<VirtualDomDocument> = (0..10)
        .map(|_| {
            let mut eval = Evaluator::new();
            eval.evaluate(&doc).expect("Evaluation failed")
        })
        .collect();

    // All results should be identical (repeat structure should be same)
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Evaluation {} differs from evaluation 0 (repeat blocks should be deterministic)",
            i
        );
    }
}

#[test]
fn test_evaluation_determinism_with_conditionals() {
    let source = r#"
        component Card {
            render div {
                if showHeader {
                    span { text "Header Shown" }
                }
                if showFooter {
                    span { text "Footer Shown" }
                }
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");

    // Evaluate 10 times
    let results: Vec<VirtualDomDocument> = (0..10)
        .map(|_| {
            let mut eval = Evaluator::new();
            eval.evaluate(&doc).expect("Evaluation failed")
        })
        .collect();

    // All results should be identical
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Evaluation {} differs from evaluation 0 (conditionals should be deterministic)",
            i
        );
    }
}

#[test]
fn test_semantic_id_stability_across_evaluations() {
    let source = r#"
        component Page {
            render div {
                Header()
                Main()
                Footer()
            }
        }

        component Header {
            render header { text "Header" }
        }

        component Main {
            render main { text "Content" }
        }

        component Footer {
            render footer { text "Footer" }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");

    // Evaluate multiple times and collect semantic IDs
    let semantic_ids_runs: Vec<Vec<String>> = (0..5)
        .map(|_| {
            let mut eval = Evaluator::new();
            let vdoc = eval.evaluate(&doc).expect("Evaluation failed");
            collect_semantic_ids(&vdoc.nodes)
        })
        .collect();

    // All runs should produce identical semantic IDs
    for i in 1..semantic_ids_runs.len() {
        assert_eq!(
            semantic_ids_runs[0], semantic_ids_runs[i],
            "Semantic IDs differ between evaluations (run 0 vs run {})",
            i
        );
    }
}

/// Helper to recursively collect all semantic IDs from VDOM
fn collect_semantic_ids(nodes: &[paperclip_evaluator::VNode]) -> Vec<String> {
    use paperclip_evaluator::VNode;

    let mut ids = Vec::new();
    for node in nodes {
        match node {
            VNode::Element {
                semantic_id,
                children,
                ..
            } => {
                ids.push(semantic_id.to_selector());
                ids.extend(collect_semantic_ids(children));
            }
            VNode::Error { semantic_id, .. } => {
                ids.push(semantic_id.to_selector());
            }
            _ => {}
        }
    }
    ids
}

#[test]
fn test_css_evaluation_determinism() {
    let source = r#"
        component Card {
            render div {
                style {
                    color: red;
                    background: blue;
                    padding: 10px;
                    margin: 5px;
                }
            }
        }
    "#;

    let doc = parse(source).expect("Failed to parse");

    // Evaluate 10 times
    let results: Vec<VirtualDomDocument> = (0..10)
        .map(|_| {
            let mut eval = Evaluator::new();
            eval.evaluate(&doc).expect("Evaluation failed")
        })
        .collect();

    // CSS rules should be in same order every time
    for i in 1..results.len() {
        assert_eq!(
            results[0].styles, results[i].styles,
            "CSS evaluation differs between runs (run 0 vs run {})",
            i
        );
    }
}
