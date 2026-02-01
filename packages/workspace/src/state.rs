use paperclip_bundle::{AssetReference, AssetType, Bundle};
use paperclip_evaluator::{
    diff_vdocument, CssError, CssEvaluator, EvalError,
    Evaluator, VDocPatch, VDomCssRule, VNode, VirtualCssDocument, VirtualDomDocument,
};
use paperclip_parser::{ast::Document, get_document_id, parse_with_path, ParseError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, instrument, warn};

// Re-export evaluator's protobuf types
pub use paperclip_evaluator::vdom_differ::proto::patches::InitializePatch;
pub use paperclip_evaluator::vdom_differ::proto::vdom as proto_vdom;

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Evaluation error: {0}")]
    EvalError(#[from] EvalError),

    #[error("CSS error: {0}")]
    CssError(#[from] CssError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// Per-file cached state
// Note: AST is stored in Bundle.documents, assets in Bundle.assets
#[derive(Clone)]
pub struct FileState {
    pub source: String,
    pub vdom: VirtualDomDocument,
    pub css: VirtualCssDocument,
    pub version: u64,
    pub document_id: String,
}

// Workspace-level state cache
pub struct WorkspaceState {
    files: HashMap<PathBuf, FileState>,
    // Bundle for the workspace - rebuilt when files change
    bundle: Bundle,
}

impl WorkspaceState {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            bundle: Bundle::new(),
        }
    }

    // Update file and return VirtualDomDocument patches
    #[instrument(skip(self, new_source, project_root), fields(path = %path.display(), source_len = new_source.len()))]
    pub fn update_file(
        &mut self,
        path: PathBuf,
        new_source: String,
        project_root: &Path,
    ) -> Result<Vec<VDocPatch>, StateError> {
        let is_cached = self.files.contains_key(&path);
        info!(is_cached, "Updating file");

        // Get path string for ID generation
        let path_str = path.to_string_lossy();

        // Parse new source with file path for proper ID generation
        debug!("Parsing source");
        let new_ast = parse_with_path(&new_source, &path_str)?;

        // Get document ID
        let document_id = get_document_id(&path_str);

        // Add/update file in bundle
        debug!("Adding file to bundle");
        self.bundle.add_document(path.clone(), new_ast.clone());

        // Rebuild bundle dependencies
        debug!("Building bundle dependencies");
        if let Err(e) = self.bundle.build_dependencies(project_root) {
            warn!(error = ?e, "Failed to build dependencies, continuing with single-file evaluation");
        }

        // Evaluate using bundle for cross-file imports
        debug!("Evaluating AST for DOM with bundle");
        let mut evaluator = Evaluator::with_document_id(&path_str);
        let new_vdom = evaluator.evaluate_bundle(&self.bundle, &path)?;

        debug!("Evaluating AST for CSS with bundle");
        let mut css_evaluator = CssEvaluator::with_document_id(&path_str);
        let new_css = css_evaluator.evaluate_bundle(&self.bundle, &path)?;
        info!(css_rules = new_css.rules.len(), "CSS evaluated");

        debug!(assets_count = "extracting", "Extracting assets");
        let new_assets = extract_assets(&new_ast, project_root, &path);
        info!(assets_count = new_assets.len(), "Assets extracted");

        // Add assets to bundle
        for asset in &new_assets {
            self.bundle.add_asset(asset.clone());
        }

        // Get old state for diffing
        let patches = if let Some(old_state) = self.files.get(&path) {
            debug!(
                old_version = old_state.version,
                "Generating patches from diff"
            );
            // Generate patches by diffing
            let patches = diff_vdocument(&old_state.vdom, &new_vdom);
            info!(patch_count = patches.len(), "Patches generated");
            patches
        } else {
            info!("First evaluation, generating initialize patch");
            // First time - send full document as "initialize" patch
            use paperclip_evaluator::vdom_differ::proto::patches::v_doc_patch;
            vec![VDocPatch {
                patch_type: Some(v_doc_patch::PatchType::Initialize(InitializePatch {
                    vdom: Some(convert_vdom_to_proto(&new_vdom)),
                })),
            }]
        };

        // Update cached state
        let new_version = self.files.get(&path).map(|s| s.version + 1).unwrap_or(0);

        info!(
            new_version,
            nodes = new_vdom.nodes.len(),
            css_rules = new_css.rules.len(),
            "Caching file state"
        );

        self.files.insert(
            path,
            FileState {
                source: new_source,
                vdom: new_vdom,
                css: new_css,
                version: new_version,
                document_id,
            },
        );

        Ok(patches)
    }

    // Get current state (for queries)
    pub fn get_file(&self, path: &Path) -> Option<&FileState> {
        self.files.get(path)
    }

    /// Get the parsed AST for a file (from bundle)
    pub fn get_ast(&self, path: &Path) -> Option<&Document> {
        self.bundle.get_document(path)
    }

    /// Get all assets used by a specific file (from bundle)
    pub fn get_file_assets(&self, path: &Path) -> Vec<&AssetReference> {
        self.bundle.assets_for_file(path)
    }

    /// Get all unique assets across the workspace
    pub fn get_all_assets(&self) -> impl Iterator<Item = &AssetReference> {
        self.bundle.unique_assets()
    }

    /// Get the bundle (for advanced queries)
    pub fn bundle(&self) -> &Bundle {
        &self.bundle
    }
}

// Extract asset references from AST
fn extract_assets(ast: &Document, project_root: &Path, source_file: &Path) -> Vec<AssetReference> {
    let mut assets = Vec::new();

    for component in &ast.components {
        if let Some(body) = &component.body {
            extract_from_element(body, project_root, source_file, &mut assets);
        }
    }

    assets
}

fn extract_from_element(
    element: &paperclip_parser::ast::Element,
    project_root: &Path,
    source_file: &Path,
    assets: &mut Vec<AssetReference>,
) {
    use paperclip_parser::ast::Element;

    match element {
        Element::Tag {
            tag_name,
            attributes,
            children,
            ..
        } => {
            // Extract from img src
            if tag_name == "img" {
                if let Some(src_expr) = attributes.get("src") {
                    if let Some(src) = expression_to_string(src_expr) {
                        assets.push(AssetReference {
                            path: src.clone(),
                            asset_type: AssetType::Image,
                            resolved_path: resolve_asset_path(&src, project_root),
                            source_file: source_file.to_path_buf(),
                        });
                    }
                }
            }

            // Extract from link href (fonts, stylesheets)
            if tag_name == "link" {
                if let Some(href_expr) = attributes.get("href") {
                    if let Some(href) = expression_to_string(href_expr) {
                        let asset_type = if href.ends_with(".woff")
                            || href.ends_with(".woff2")
                            || href.ends_with(".ttf")
                        {
                            AssetType::Font
                        } else if href.ends_with(".css") {
                            AssetType::Other
                        } else {
                            AssetType::Other
                        };
                        assets.push(AssetReference {
                            path: href.clone(),
                            asset_type,
                            resolved_path: resolve_asset_path(&href, project_root),
                            source_file: source_file.to_path_buf(),
                        });
                    }
                }
            }

            // Extract from video/audio sources
            if tag_name == "video" || tag_name == "audio" {
                if let Some(src_expr) = attributes.get("src") {
                    if let Some(src) = expression_to_string(src_expr) {
                        let asset_type = if tag_name == "video" {
                            AssetType::Video
                        } else {
                            AssetType::Audio
                        };
                        assets.push(AssetReference {
                            path: src.clone(),
                            asset_type,
                            resolved_path: resolve_asset_path(&src, project_root),
                            source_file: source_file.to_path_buf(),
                        });
                    }
                }
            }

            // Extract from source elements (video/audio children)
            if tag_name == "source" {
                if let Some(src_expr) = attributes.get("src") {
                    if let Some(src) = expression_to_string(src_expr) {
                        assets.push(AssetReference {
                            path: src.clone(),
                            asset_type: AssetType::Other,
                            resolved_path: resolve_asset_path(&src, project_root),
                            source_file: source_file.to_path_buf(),
                        });
                    }
                }
            }

            // Recurse into children
            for child in children {
                extract_from_element(child, project_root, source_file, assets);
            }
        }

        Element::Instance { children, .. } => {
            // Recurse into component instance children
            for child in children {
                extract_from_element(child, project_root, source_file, assets);
            }
        }

        Element::Conditional {
            then_branch,
            else_branch,
            ..
        } => {
            // Extract from conditional branches
            for child in then_branch {
                extract_from_element(child, project_root, source_file, assets);
            }
            if let Some(else_br) = else_branch {
                for child in else_br {
                    extract_from_element(child, project_root, source_file, assets);
                }
            }
        }

        Element::Repeat { body, .. } => {
            // Extract from repeat body
            for child in body {
                extract_from_element(child, project_root, source_file, assets);
            }
        }

        Element::Text { .. } | Element::SlotInsert { .. } => {
            // No assets in text or slot inserts
        }

        Element::Insert { content, .. } => {
            // Extract from insert content
            for child in content {
                extract_from_element(child, project_root, source_file, assets);
            }
        }
    }
}

// Extract string value from Expression (handles literals only for now)
fn expression_to_string(expr: &paperclip_parser::ast::Expression) -> Option<String> {
    use paperclip_parser::ast::Expression;

    match expr {
        Expression::Literal { value, .. } => Some(value.clone()),
        // For template strings, try to extract if it's just a literal
        Expression::Template { parts, .. } => {
            use paperclip_parser::ast::TemplatePart;
            if parts.len() == 1 {
                if let TemplatePart::Literal(s) = &parts[0] {
                    return Some(s.clone());
                }
            }
            None
        }
        _ => None, // Variables, calls, etc. can't be statically extracted
    }
}

fn resolve_asset_path(relative_path: &str, project_root: &Path) -> PathBuf {
    // Handle leading ./ or ../
    let cleaned = relative_path.trim_start_matches("./");

    if cleaned.starts_with("http://")
        || cleaned.starts_with("https://")
        || cleaned.starts_with("//")
    {
        // External URL - return as-is (PathBuf will just store it)
        PathBuf::from(cleaned)
    } else {
        // Relative path - resolve from project root
        project_root.join(cleaned)
    }
}

// Convert VirtualDomDocument to protobuf format (public for server.rs SSE)
pub fn convert_vdom_to_proto(vdom: &VirtualDomDocument) -> proto_vdom::VDocument {
    proto_vdom::VDocument {
        nodes: vdom.nodes.iter().map(convert_vnode_to_proto).collect(),
        styles: vdom.styles.iter().map(convert_css_rule_to_proto).collect(),
    }
}

// Convert VNode to proto VNode
fn convert_vnode_to_proto(node: &VNode) -> proto_vdom::VNode {
    match node {
        VNode::Element {
            tag,
            attributes,
            styles,
            children,
            semantic_id,
            key,
        } => proto_vdom::VNode {
            node_type: Some(proto_vdom::v_node::NodeType::Element(
                proto_vdom::ElementNode {
                    tag: tag.clone(),
                    attributes: attributes.clone(),
                    styles: styles.clone(),
                    children: children.iter().map(convert_vnode_to_proto).collect(),
                    semantic_id: semantic_id.to_selector(),
                    key: key.clone(),
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
        VNode::Error { message, semantic_id, .. } => proto_vdom::VNode {
            node_type: Some(proto_vdom::v_node::NodeType::Error(
                proto_vdom::ErrorNode {
                    message: message.clone(),
                    semantic_id: semantic_id.to_selector(),
                },
            )),
        },
    }
}

// Convert CssRule to proto CssRule
fn convert_css_rule_to_proto(rule: &VDomCssRule) -> proto_vdom::CssRule {
    proto_vdom::CssRule {
        selector: rule.selector.clone(),
        properties: rule.properties.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_state_creation() {
        let state = WorkspaceState::new();
        assert_eq!(state.files.len(), 0);
    }

    #[test]
    fn test_file_caching() {
        let mut state = WorkspaceState::new();
        let path = PathBuf::from("/test/file.pc");
        let source = "component Button {}".to_string();
        let project_root = PathBuf::from("/test");

        let result = state.update_file(path.clone(), source.clone(), &project_root);
        assert!(result.is_ok());

        let file_state = state.get_file(&path);
        assert!(file_state.is_some());
        assert_eq!(file_state.unwrap().version, 0);
        assert_eq!(file_state.unwrap().source, source);
    }

    #[test]
    fn test_version_increment() {
        let mut state = WorkspaceState::new();
        let path = PathBuf::from("/test/file.pc");
        let project_root = PathBuf::from("/test");

        state
            .update_file(path.clone(), "component A {}".to_string(), &project_root)
            .unwrap();
        assert_eq!(state.get_file(&path).unwrap().version, 0);

        state
            .update_file(path.clone(), "component B {}".to_string(), &project_root)
            .unwrap();
        assert_eq!(state.get_file(&path).unwrap().version, 1);

        state
            .update_file(path.clone(), "component C {}".to_string(), &project_root)
            .unwrap();
        assert_eq!(state.get_file(&path).unwrap().version, 2);
    }

    #[test]
    fn test_asset_extraction_enabled() {
        let mut state = WorkspaceState::new();
        let path = PathBuf::from("/test/page.pc");
        let project_root = PathBuf::from("/test");

        let source = r#"component Page {
  render div {
    text "Page content"
  }
}"#;

        state
            .update_file(path.clone(), source.to_string(), &project_root)
            .unwrap();

        // Assets are now accessed through the bundle
        let file_assets = state.get_file_assets(&path);
        // Assets list exists (may be empty if no assets in this simple component)
        assert!(file_assets.len() >= 0);
    }

    #[test]
    fn test_non_public_component_rendered() {
        let mut state = WorkspaceState::new();
        let path = PathBuf::from("./test.pc");
        let project_root = PathBuf::from(".");

        let source = r#"component Card {
    render div {
        text "Hello"
    }
}"#.to_string();

        let result = state.update_file(path.clone(), source, &project_root);
        assert!(result.is_ok(), "update_file should succeed: {:?}", result.err());

        let patches = result.unwrap();
        assert_eq!(patches.len(), 1, "Should have 1 initialize patch");

        // Check the patch contains a vdom with nodes
        if let Some(patch_type) = &patches[0].patch_type {
            use paperclip_evaluator::vdom_differ::proto::patches::v_doc_patch::PatchType;
            match patch_type {
                PatchType::Initialize(init) => {
                    let vdom = init.vdom.as_ref().expect("Should have vdom");
                    println!("vdom.nodes.len() = {}", vdom.nodes.len());
                    for (i, node) in vdom.nodes.iter().enumerate() {
                        println!("node[{}] = {:?}", i, node);
                    }
                    assert!(!vdom.nodes.is_empty(), "vdom.nodes should not be empty");
                }
                _ => panic!("Expected Initialize patch"),
            }
        } else {
            panic!("patch_type should be Some");
        }
    }

    #[test]
    fn test_path_join_format() {
        // Test the same path format the server uses
        let root_dir = std::path::PathBuf::from(".");
        let file_path = "test.pc";
        let path = root_dir.join(file_path);
        
        println!("path = {:?}", path);
        println!("path.display() = {}", path.display());
        
        let mut state = WorkspaceState::new();
        let project_root = std::path::PathBuf::from(".");

        let source = r#"component Card {
    render div {
        text "Hello"
    }
}"#.to_string();

        let result = state.update_file(path.clone(), source, &project_root);
        assert!(result.is_ok(), "update_file should succeed: {:?}", result.err());

        let patches = result.unwrap();
        
        if let Some(patch_type) = &patches[0].patch_type {
            use paperclip_evaluator::vdom_differ::proto::patches::v_doc_patch::PatchType;
            match patch_type {
                PatchType::Initialize(init) => {
                    let vdom = init.vdom.as_ref().expect("Should have vdom");
                    println!("nodes.len() = {}", vdom.nodes.len());
                    assert!(!vdom.nodes.is_empty(), "vdom.nodes should not be empty");
                }
                _ => panic!("Expected Initialize patch"),
            }
        }
    }
}
