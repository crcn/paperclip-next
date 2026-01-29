use crate::vdom::{VNode, VirtualDomDocument};
use paperclip_semantics::SemanticID;

// Include generated protobuf types
pub mod proto {
    pub mod patches {
        include!(concat!(env!("OUT_DIR"), "/paperclip.patches.rs"));
    }
    pub mod vdom {
        include!(concat!(env!("OUT_DIR"), "/paperclip.vdom.rs"));
    }
}

// Re-export for convenience
pub use proto::patches::*;
pub use proto::vdom as proto_vdom;

// Diff two VirtualDomDocuments and generate patches
pub fn diff_vdocument(old: &VirtualDomDocument, new: &VirtualDomDocument) -> Vec<VDocPatch> {
    let mut patches = Vec::new();

    // Match root nodes by semantic ID
    patches.extend(diff_children_by_semantic_id(&old.nodes, &new.nodes, vec![]));

    // Diff style rules
    patches.extend(diff_style_rules(&old.styles, &new.styles));

    patches
}

/// Diff children using semantic ID matching for stable patches
fn diff_children_by_semantic_id(
    old_children: &[VNode],
    new_children: &[VNode],
    parent_path: Vec<u32>,
) -> Vec<VDocPatch> {
    use std::collections::HashMap;

    let mut patches = Vec::new();

    // Separate elements (with semantic IDs) from text/comment nodes
    let mut old_elements: HashMap<String, (usize, &VNode)> = HashMap::new();
    let mut new_elements: HashMap<String, (usize, &VNode)> = HashMap::new();

    // For text/comment nodes without semantic IDs, use position-based matching
    let mut old_simple_nodes: Vec<(usize, &VNode)> = Vec::new();
    let mut new_simple_nodes: Vec<(usize, &VNode)> = Vec::new();

    for (i, node) in old_children.iter().enumerate() {
        if let Some(semantic_id) = get_node_semantic_id(node) {
            old_elements.insert(semantic_id.to_selector(), (i, node));
        } else {
            old_simple_nodes.push((i, node));
        }
    }

    for (i, node) in new_children.iter().enumerate() {
        if let Some(semantic_id) = get_node_semantic_id(node) {
            new_elements.insert(semantic_id.to_selector(), (i, node));
        } else {
            new_simple_nodes.push((i, node));
        }
    }

    // Diff elements by semantic ID
    // Find removed elements (in old but not in new)
    for (semantic_key, (old_idx, _)) in &old_elements {
        if !new_elements.contains_key(semantic_key) {
            let mut path = parent_path.clone();
            path.push(*old_idx as u32);
            patches.push(VDocPatch {
                patch_type: Some(v_doc_patch::PatchType::RemoveNode(RemoveNodePatch { path })),
            });
        }
    }

    // Find new elements (in new but not in old) and updated elements
    for (semantic_key, (new_idx, new_node)) in &new_elements {
        if let Some((_old_idx, old_node)) = old_elements.get(semantic_key) {
            // Node exists in both - diff it
            let mut path = parent_path.clone();
            path.push(*new_idx as u32);
            patches.extend(diff_vnodes_same_path(old_node, new_node, path));
        } else {
            // New node - create it
            let mut path = parent_path.clone();
            path.push(*new_idx as u32);
            patches.push(VDocPatch {
                patch_type: Some(v_doc_patch::PatchType::CreateNode(CreateNodePatch {
                    path: path.clone(),
                    node: Some(convert_vnode_to_proto(new_node)),
                    index: *new_idx as u32,
                })),
            });
        }
    }

    // Diff simple nodes (text/comment) by position
    let max_simple = old_simple_nodes.len().max(new_simple_nodes.len());
    for i in 0..max_simple {
        let old_node = old_simple_nodes.get(i);
        let new_node = new_simple_nodes.get(i);

        match (old_node, new_node) {
            (None, Some((new_idx, new_node))) => {
                // Create new simple node
                let mut path = parent_path.clone();
                path.push(*new_idx as u32);
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::CreateNode(CreateNodePatch {
                        path: path.clone(),
                        node: Some(convert_vnode_to_proto(new_node)),
                        index: *new_idx as u32,
                    })),
                });
            }
            (Some((old_idx, _)), None) => {
                // Remove simple node
                let mut path = parent_path.clone();
                path.push(*old_idx as u32);
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::RemoveNode(RemoveNodePatch { path })),
                });
            }
            (Some((_old_idx, old_node)), Some((new_idx, new_node))) => {
                // Use the new node's index for path (handles reordering)
                let mut path = parent_path.clone();
                path.push(*new_idx as u32);
                patches.extend(diff_vnodes_same_path(old_node, new_node, path));
            }
            (None, None) => {}
        }
    }

    patches
}

/// Extract semantic ID from a VNode
fn get_node_semantic_id(node: &VNode) -> Option<&SemanticID> {
    match node {
        VNode::Element { semantic_id, .. } => Some(semantic_id),
        _ => None,
    }
}

fn diff_vnodes_same_path(old: &VNode, new: &VNode, path: Vec<u32>) -> Vec<VDocPatch> {
    let mut patches = Vec::new();

    // Check if node types match
    if !vnodes_same_type(old, new) {
        // Different types - replace entire node
        patches.push(VDocPatch {
            patch_type: Some(v_doc_patch::PatchType::ReplaceNode(ReplaceNodePatch {
                path,
                new_node: Some(convert_vnode_to_proto(new)),
            })),
        });
        return patches;
    }

    // Same type - check for updates
    match (old, new) {
        (
            VNode::Element {
                tag: old_tag,
                attributes: old_attrs,
                styles: old_styles,
                children: old_children,
                ..
            },
            VNode::Element {
                tag: new_tag,
                attributes: new_attrs,
                styles: new_styles,
                children: new_children,
                ..
            },
        ) => {
            if old_tag != new_tag {
                // Tag changed - replace entire node
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::ReplaceNode(ReplaceNodePatch {
                        path: path.clone(),
                        new_node: Some(convert_vnode_to_proto(new)),
                    })),
                });
                return patches;
            }

            // Check attributes
            if old_attrs != new_attrs {
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::UpdateAttributes(
                        UpdateAttributesPatch {
                            path: path.clone(),
                            attributes: new_attrs.clone(),
                        },
                    )),
                });
            }

            // Check styles
            if old_styles != new_styles {
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::UpdateStyles(UpdateStylesPatch {
                        path: path.clone(),
                        styles: new_styles.clone(),
                    })),
                });
            }

            // Diff children using semantic ID matching
            patches.extend(diff_children_by_semantic_id(
                old_children,
                new_children,
                path,
            ));
        }
        (VNode::Text { content: old_text }, VNode::Text { content: new_text }) => {
            if old_text != new_text {
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::UpdateText(UpdateTextPatch {
                        path,
                        content: new_text.clone(),
                    })),
                });
            }
        }
        (VNode::Comment { content: old_text }, VNode::Comment { content: new_text }) => {
            if old_text != new_text {
                // Comments changing - replace node
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::ReplaceNode(ReplaceNodePatch {
                        path,
                        new_node: Some(convert_vnode_to_proto(new)),
                    })),
                });
            }
        }
        _ => {}
    }

    patches
}

fn vnodes_same_type(a: &VNode, b: &VNode) -> bool {
    matches!(
        (a, b),
        (VNode::Element { .. }, VNode::Element { .. })
            | (VNode::Text { .. }, VNode::Text { .. })
            | (VNode::Comment { .. }, VNode::Comment { .. })
    )
}

fn diff_style_rules(old: &[crate::vdom::CssRule], new: &[crate::vdom::CssRule]) -> Vec<VDocPatch> {
    let mut patches = Vec::new();

    // Build maps by selector for efficient matching
    use std::collections::HashMap;
    let mut old_map: HashMap<&str, &crate::vdom::CssRule> = HashMap::new();
    let mut new_map: HashMap<&str, &crate::vdom::CssRule> = HashMap::new();

    for rule in old {
        old_map.insert(&rule.selector, rule);
    }

    for rule in new {
        new_map.insert(&rule.selector, rule);
    }

    // Find removed rules (in old but not in new)
    for (i, old_rule) in old.iter().enumerate() {
        if !new_map.contains_key(old_rule.selector.as_str()) {
            patches.push(VDocPatch {
                patch_type: Some(v_doc_patch::PatchType::RemoveStyleRule(
                    RemoveStyleRulePatch { index: i as u32 },
                )),
            });
        }
    }

    // Find new rules (in new but not in old) or updated rules
    for new_rule in new {
        if let Some(old_rule) = old_map.get(new_rule.selector.as_str()) {
            // Rule exists - check if properties changed
            if old_rule.properties != new_rule.properties {
                // Properties changed - for now, remove and add
                // In future, we could add an UpdateStyleRule patch
                if let Some(old_index) = old.iter().position(|r| r.selector == new_rule.selector) {
                    patches.push(VDocPatch {
                        patch_type: Some(v_doc_patch::PatchType::RemoveStyleRule(
                            RemoveStyleRulePatch {
                                index: old_index as u32,
                            },
                        )),
                    });
                }
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::AddStyleRule(AddStyleRulePatch {
                        rule: Some(proto_vdom::CssRule {
                            selector: new_rule.selector.clone(),
                            properties: new_rule.properties.clone(),
                        }),
                    })),
                });
            }
            // else: no change, no patch needed
        } else {
            // New rule - add it
            patches.push(VDocPatch {
                patch_type: Some(v_doc_patch::PatchType::AddStyleRule(AddStyleRulePatch {
                    rule: Some(proto_vdom::CssRule {
                        selector: new_rule.selector.clone(),
                        properties: new_rule.properties.clone(),
                    }),
                })),
            });
        }
    }

    patches
}

// Convert internal VNode to protobuf VNode
fn convert_vnode_to_proto(vnode: &VNode) -> proto_vdom::VNode {
    match vnode {
        VNode::Element {
            tag,
            attributes,
            styles,
            children,
            ..
        } => proto_vdom::VNode {
            node_type: Some(proto_vdom::v_node::NodeType::Element(
                proto_vdom::ElementNode {
                    tag: tag.clone(),
                    attributes: attributes.clone(),
                    styles: styles.clone(),
                    children: children.iter().map(convert_vnode_to_proto).collect(),
                    id: None,
                },
            )),
        },
        VNode::Text { content } => proto_vdom::VNode {
            node_type: Some(proto_vdom::v_node::NodeType::Text(proto_vdom::TextNode {
                content: content.clone(),
            })),
        },
        VNode::Comment { content } => proto_vdom::VNode {
            node_type: Some(proto_vdom::v_node::NodeType::Comment(
                proto_vdom::CommentNode {
                    content: content.clone(),
                },
            )),
        },
        VNode::Error { message, .. } => {
            // Render errors as red text nodes for visual feedback
            proto_vdom::VNode {
                node_type: Some(proto_vdom::v_node::NodeType::Element(
                    proto_vdom::ElementNode {
                        tag: "span".to_string(),
                        attributes: [
                            ("class".to_string(), "paperclip-error".to_string()),
                            ("title".to_string(), message.clone()),
                        ]
                        .into_iter()
                        .collect(),
                        styles: [
                            ("color".to_string(), "red".to_string()),
                            ("font-weight".to_string(), "bold".to_string()),
                            ("background".to_string(), "#fee".to_string()),
                            ("padding".to_string(), "2px 4px".to_string()),
                            ("border-radius".to_string(), "2px".to_string()),
                            ("border".to_string(), "1px solid red".to_string()),
                        ]
                        .into_iter()
                        .collect(),
                        children: vec![proto_vdom::VNode {
                            node_type: Some(proto_vdom::v_node::NodeType::Text(
                                proto_vdom::TextNode {
                                    content: format!("\u{26A0} {}", message),
                                },
                            )),
                        }],
                        id: None,
                    },
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_diff_create_node() {
        let old = VirtualDomDocument {
            nodes: vec![],
            styles: vec![],
        };

        let new = VirtualDomDocument {
            nodes: vec![VNode::Element {
                tag: "div".to_string(),
                attributes: HashMap::new(),
                styles: HashMap::new(),
                children: vec![],
                semantic_id: SemanticID::root(),
                key: None,
                id: None,
            }],
            styles: vec![],
        };

        let patches = diff_vdocument(&old, &new);
        assert_eq!(patches.len(), 1);

        match &patches[0].patch_type {
            Some(v_doc_patch::PatchType::CreateNode(patch)) => {
                assert_eq!(patch.path, vec![0]);
            }
            _ => panic!("Expected CreateNode patch"),
        }
    }

    #[test]
    fn test_diff_remove_node() {
        let old = VirtualDomDocument {
            nodes: vec![VNode::Element {
                tag: "div".to_string(),
                attributes: HashMap::new(),
                styles: HashMap::new(),
                children: vec![],
                semantic_id: SemanticID::root(),
                key: None,
                id: None,
            }],
            styles: vec![],
        };

        let new = VirtualDomDocument {
            nodes: vec![],
            styles: vec![],
        };

        let patches = diff_vdocument(&old, &new);
        assert_eq!(patches.len(), 1);

        match &patches[0].patch_type {
            Some(v_doc_patch::PatchType::RemoveNode(patch)) => {
                assert_eq!(patch.path, vec![0]);
            }
            _ => panic!("Expected RemoveNode patch"),
        }
    }

    #[test]
    fn test_diff_update_text() {
        let old = VirtualDomDocument {
            nodes: vec![VNode::Text {
                content: "old".to_string(),
            }],
            styles: vec![],
        };

        let new = VirtualDomDocument {
            nodes: vec![VNode::Text {
                content: "new".to_string(),
            }],
            styles: vec![],
        };

        let patches = diff_vdocument(&old, &new);
        assert_eq!(patches.len(), 1);

        match &patches[0].patch_type {
            Some(v_doc_patch::PatchType::UpdateText(patch)) => {
                assert_eq!(patch.path, vec![0]);
                assert_eq!(patch.content, "new");
            }
            _ => panic!("Expected UpdateText patch"),
        }
    }

    #[test]
    fn test_diff_with_semantic_id_reordering() {
        use paperclip_semantics::{SemanticID, SemanticSegment};

        // Create two elements with different semantic IDs
        let elem1_id = SemanticID::new(vec![SemanticSegment::Element {
            tag: "div".to_string(),
            role: None,
            ast_id: "elem1".to_string(),
        }]);

        let elem2_id = SemanticID::new(vec![SemanticSegment::Element {
            tag: "div".to_string(),
            role: None,
            ast_id: "elem2".to_string(),
        }]);

        // Old: [elem1, elem2]
        let old = VirtualDomDocument {
            nodes: vec![
                VNode::Element {
                    tag: "div".to_string(),
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("id".to_string(), "first".to_string());
                        attrs
                    },
                    styles: HashMap::new(),
                    children: vec![],
                    semantic_id: elem1_id.clone(),
                    key: None,
                    id: None,
                },
                VNode::Element {
                    tag: "div".to_string(),
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("id".to_string(), "second".to_string());
                        attrs
                    },
                    styles: HashMap::new(),
                    children: vec![],
                    semantic_id: elem2_id.clone(),
                    key: None,
                    id: None,
                },
            ],
            styles: vec![],
        };

        // New: [elem2, elem1] - reordered!
        let new = VirtualDomDocument {
            nodes: vec![
                VNode::Element {
                    tag: "div".to_string(),
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("id".to_string(), "second".to_string());
                        attrs
                    },
                    styles: HashMap::new(),
                    children: vec![],
                    semantic_id: elem2_id.clone(),
                    key: None,
                    id: None,
                },
                VNode::Element {
                    tag: "div".to_string(),
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("id".to_string(), "first".to_string());
                        attrs
                    },
                    styles: HashMap::new(),
                    children: vec![],
                    semantic_id: elem1_id.clone(),
                    key: None,
                    id: None,
                },
            ],
            styles: vec![],
        };

        let patches = diff_vdocument(&old, &new);

        // Should have NO patches - nodes matched by semantic ID, not position
        // Even though positions changed, the nodes themselves are identical
        assert_eq!(
            patches.len(),
            0,
            "Reordering nodes with same semantic IDs should produce no patches"
        );
    }

    #[test]
    fn test_diff_update_attributes() {
        let mut old_attrs = HashMap::new();
        old_attrs.insert("class".to_string(), "old-class".to_string());

        let mut new_attrs = HashMap::new();
        new_attrs.insert("class".to_string(), "new-class".to_string());

        let old = VirtualDomDocument {
            nodes: vec![VNode::Element {
                tag: "div".to_string(),
                attributes: old_attrs,
                styles: HashMap::new(),
                children: vec![],
                semantic_id: SemanticID::root(),
                key: None,
                id: None,
            }],
            styles: vec![],
        };

        let new = VirtualDomDocument {
            nodes: vec![VNode::Element {
                tag: "div".to_string(),
                attributes: new_attrs,
                styles: HashMap::new(),
                children: vec![],
                semantic_id: SemanticID::root(),
                key: None,
                id: None,
            }],
            styles: vec![],
        };

        let patches = diff_vdocument(&old, &new);
        assert_eq!(patches.len(), 1);

        match &patches[0].patch_type {
            Some(v_doc_patch::PatchType::UpdateAttributes(patch)) => {
                assert_eq!(patch.path, vec![0]);
                assert_eq!(patch.attributes.get("class").unwrap(), "new-class");
            }
            _ => panic!("Expected UpdateAttributes patch"),
        }
    }
}
