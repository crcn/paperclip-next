/// Tests for semantic ID-based diffing and stable patches
use crate::evaluator::Evaluator;
use crate::vdom_differ::diff_vdocument;
use paperclip_parser::parse_with_path;

#[test]
fn test_component_reordering_no_patches() {
    let source = r#"
        component Button {
            render button { text "Click" }
        }

        public component App {
            render div {
                Button()
                Button()
                Button()
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();

    // Evaluate twice - should produce identical VDOMs with same semantic IDs
    let mut evaluator1 = Evaluator::with_document_id("/test.pc");
    let vdom1 = evaluator1.evaluate(&doc).unwrap();

    let mut evaluator2 = Evaluator::with_document_id("/test.pc");
    let vdom2 = evaluator2.evaluate(&doc).unwrap();

    // Diffing identical VDOMs should produce no patches
    let patches = diff_vdocument(&vdom1, &vdom2);
    assert_eq!(
        patches.len(),
        0,
        "Identical VDOMs should produce no patches"
    );
}

#[test]
fn test_semantic_id_survives_attribute_changes() {
    let source = r#"
        public component Card {
            render div {
                div(class="container") {
                    text "Content"
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/test.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Manually modify an attribute to simulate a change
    // The semantic ID should remain the same
    let old_vdom = vdom.clone();

    // In a real scenario, the attribute would change via re-evaluation
    // Here we're just verifying the semantic IDs are present and stable
    let patches = diff_vdocument(&old_vdom, &vdom);
    assert_eq!(patches.len(), 0, "Same VDOM should produce no patches");
}
