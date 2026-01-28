use crate::vdom::{VDocument, VNode};

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

// Diff two VDocuments and generate patches
pub fn diff_vdocument(old: &VDocument, new: &VDocument) -> Vec<VDocPatch> {
    let mut patches = Vec::new();

    // Diff root nodes
    let max_len = old.nodes.len().max(new.nodes.len());
    for i in 0..max_len {
        let old_node = old.nodes.get(i);
        let new_node = new.nodes.get(i);

        patches.extend(diff_vnode(old_node, new_node, vec![i as u32]));
    }

    // Diff style rules
    patches.extend(diff_style_rules(&old.styles, &new.styles));

    patches
}

fn diff_vnode(
    old: Option<&VNode>,
    new: Option<&VNode>,
    path: Vec<u32>
) -> Vec<VDocPatch> {
    let mut patches = Vec::new();

    match (old, new) {
        (None, Some(node)) => {
            // Create new node
            patches.push(VDocPatch {
                patch_type: Some(v_doc_patch::PatchType::CreateNode(
                    CreateNodePatch {
                        path: path.clone(),
                        node: Some(convert_vnode_to_proto(node)),
                        index: path.last().copied().unwrap_or(0),
                    }
                )),
            });
        }
        (Some(_), None) => {
            // Remove node
            patches.push(VDocPatch {
                patch_type: Some(v_doc_patch::PatchType::RemoveNode(
                    RemoveNodePatch { path }
                )),
            });
        }
        (Some(old_node), Some(new_node)) => {
            // Compare nodes
            patches.extend(diff_vnodes_same_path(old_node, new_node, path));
        }
        (None, None) => {}
    }

    patches
}

fn diff_vnodes_same_path(
    old: &VNode,
    new: &VNode,
    path: Vec<u32>
) -> Vec<VDocPatch> {
    let mut patches = Vec::new();

    // Check if node types match
    if !vnodes_same_type(old, new) {
        // Different types - replace entire node
        patches.push(VDocPatch {
            patch_type: Some(v_doc_patch::PatchType::ReplaceNode(
                ReplaceNodePatch {
                    path,
                    new_node: Some(convert_vnode_to_proto(new)),
                }
            )),
        });
        return patches;
    }

    // Same type - check for updates
    match (old, new) {
        (VNode::Element { tag: old_tag, attributes: old_attrs, styles: old_styles, children: old_children, .. },
         VNode::Element { tag: new_tag, attributes: new_attrs, styles: new_styles, children: new_children, .. }) => {

            if old_tag != new_tag {
                // Tag changed - replace entire node
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::ReplaceNode(
                        ReplaceNodePatch {
                            path: path.clone(),
                            new_node: Some(convert_vnode_to_proto(new)),
                        }
                    )),
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
                        }
                    )),
                });
            }

            // Check styles
            if old_styles != new_styles {
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::UpdateStyles(
                        UpdateStylesPatch {
                            path: path.clone(),
                            styles: new_styles.clone(),
                        }
                    )),
                });
            }

            // Diff children
            let max_children = old_children.len().max(new_children.len());
            for i in 0..max_children {
                let mut child_path = path.clone();
                child_path.push(i as u32);

                let old_child = old_children.get(i);
                let new_child = new_children.get(i);

                patches.extend(diff_vnode(old_child, new_child, child_path));
            }
        }
        (VNode::Text { content: old_text }, VNode::Text { content: new_text }) => {
            if old_text != new_text {
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::UpdateText(
                        UpdateTextPatch {
                            path,
                            content: new_text.clone(),
                        }
                    )),
                });
            }
        }
        (VNode::Comment { content: old_text }, VNode::Comment { content: new_text }) => {
            if old_text != new_text {
                // Comments changing - replace node
                patches.push(VDocPatch {
                    patch_type: Some(v_doc_patch::PatchType::ReplaceNode(
                        ReplaceNodePatch {
                            path,
                            new_node: Some(convert_vnode_to_proto(new)),
                        }
                    )),
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

fn diff_style_rules(
    old: &[crate::vdom::CssRule],
    new: &[crate::vdom::CssRule]
) -> Vec<VDocPatch> {
    let mut patches = Vec::new();

    // Simple approach: if styles changed, send add/remove patches
    // More sophisticated diffing could match rules by selector

    // Remove old rules not in new
    for (i, old_rule) in old.iter().enumerate() {
        if !new.contains(old_rule) {
            patches.push(VDocPatch {
                patch_type: Some(v_doc_patch::PatchType::RemoveStyleRule(
                    RemoveStyleRulePatch {
                        index: i as u32,
                    }
                )),
            });
        }
    }

    // Add new rules not in old
    for new_rule in new {
        if !old.contains(new_rule) {
            patches.push(VDocPatch {
                patch_type: Some(v_doc_patch::PatchType::AddStyleRule(
                    AddStyleRulePatch {
                        rule: Some(proto_vdom::CssRule {
                            selector: new_rule.selector.clone(),
                            properties: new_rule.properties.clone(),
                        }),
                    }
                )),
            });
        }
    }

    patches
}

// Convert internal VNode to protobuf VNode
fn convert_vnode_to_proto(vnode: &VNode) -> proto_vdom::VNode {
    match vnode {
        VNode::Element { tag, attributes, styles, children, .. } => {
            proto_vdom::VNode {
                node_type: Some(proto_vdom::v_node::NodeType::Element(
                    proto_vdom::ElementNode {
                        tag: tag.clone(),
                        attributes: attributes.clone(),
                        styles: styles.clone(),
                        children: children.iter().map(convert_vnode_to_proto).collect(),
                        id: None,
                    }
                )),
            }
        }
        VNode::Text { content } => {
            proto_vdom::VNode {
                node_type: Some(proto_vdom::v_node::NodeType::Text(
                    proto_vdom::TextNode {
                        content: content.clone(),
                    }
                )),
            }
        }
        VNode::Comment { content } => {
            proto_vdom::VNode {
                node_type: Some(proto_vdom::v_node::NodeType::Comment(
                    proto_vdom::CommentNode {
                        content: content.clone(),
                    }
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
        let old = VDocument {
            nodes: vec![],
            styles: vec![],
        };

        let new = VDocument {
            nodes: vec![VNode::Element {
                tag: "div".to_string(),
                attributes: HashMap::new(),
                styles: HashMap::new(),
                children: vec![],
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
        let old = VDocument {
            nodes: vec![VNode::Element {
                tag: "div".to_string(),
                attributes: HashMap::new(),
                styles: HashMap::new(),
                children: vec![],
                id: None,
            }],
            styles: vec![],
        };

        let new = VDocument {
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
        let old = VDocument {
            nodes: vec![VNode::Text { content: "old".to_string() }],
            styles: vec![],
        };

        let new = VDocument {
            nodes: vec![VNode::Text { content: "new".to_string() }],
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
    fn test_diff_update_attributes() {
        let mut old_attrs = HashMap::new();
        old_attrs.insert("class".to_string(), "old-class".to_string());

        let mut new_attrs = HashMap::new();
        new_attrs.insert("class".to_string(), "new-class".to_string());

        let old = VDocument {
            nodes: vec![VNode::Element {
                tag: "div".to_string(),
                attributes: old_attrs,
                styles: HashMap::new(),
                children: vec![],
                id: None,
            }],
            styles: vec![],
        };

        let new = VDocument {
            nodes: vec![VNode::Element {
                tag: "div".to_string(),
                attributes: new_attrs,
                styles: HashMap::new(),
                children: vec![],
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
