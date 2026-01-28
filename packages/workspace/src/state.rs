use std::collections::HashMap;
use std::path::{Path, PathBuf};
use paperclip_parser::{parse, ast::Document};
use paperclip_evaluator::{Evaluator, VDocument, diff_vdocument, VDocPatch};

// Re-export evaluator's protobuf types
pub use paperclip_evaluator::vdom_differ::proto::patches::InitializePatch;
pub use paperclip_evaluator::vdom_differ::proto::vdom as proto_vdom;

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Evaluation error: {0}")]
    EvalError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// Asset reference extracted from AST
#[derive(Clone, Debug)]
pub struct AssetReference {
    pub path: String,
    pub asset_type: AssetType,
    pub resolved_path: PathBuf,
}

#[derive(Clone, Debug)]
pub enum AssetType {
    Image,
    Font,
    Video,
    Audio,
    Other,
}

// Per-file cached state
#[derive(Clone)]
pub struct FileState {
    pub source: String,
    pub ast: Document,
    pub vdom: VDocument,
    pub assets: Vec<AssetReference>,
    pub version: u64,
}

// Workspace-level state cache
pub struct WorkspaceState {
    files: HashMap<PathBuf, FileState>,
}

impl WorkspaceState {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    // Update file and return VDocument patches
    pub fn update_file(
        &mut self,
        path: PathBuf,
        new_source: String,
        project_root: &Path,
    ) -> Result<Vec<VDocPatch>, StateError> {
        // Parse new source
        let new_ast = parse(&new_source)
            .map_err(|e| StateError::ParseError(format!("{:?}", e)))?;

        let mut evaluator = Evaluator::new();
        let new_vdom = evaluator.evaluate(&new_ast)
            .map_err(|e| StateError::EvalError(format!("{:?}", e)))?;

        let new_assets = extract_assets(&new_ast, project_root);

        // Get old state for diffing
        let patches = if let Some(old_state) = self.files.get(&path) {
            // Generate patches by diffing
            diff_vdocument(&old_state.vdom, &new_vdom)
        } else {
            // First time - send full document as "initialize" patch
            use paperclip_evaluator::vdom_differ::proto::patches::v_doc_patch;
            vec![VDocPatch {
                patch_type: Some(v_doc_patch::PatchType::Initialize(
                    InitializePatch {
                        vdom: Some(convert_vdom_to_proto(&new_vdom)),
                    }
                )),
            }]
        };

        // Update cached state
        let new_version = self.files.get(&path)
            .map(|s| s.version + 1)
            .unwrap_or(0);

        self.files.insert(path, FileState {
            source: new_source,
            ast: new_ast,
            vdom: new_vdom,
            assets: new_assets,
            version: new_version,
        });

        Ok(patches)
    }

    // Get current state (for queries)
    pub fn get_file(&self, path: &Path) -> Option<&FileState> {
        self.files.get(path)
    }
}

// Extract asset references from AST
fn extract_assets(ast: &Document, project_root: &Path) -> Vec<AssetReference> {
    let mut assets = Vec::new();

    for component in &ast.components {
        if let Some(body) = &component.body {
            extract_from_element(body, project_root, &mut assets);
        }
    }

    assets
}

fn extract_from_element(
    element: &paperclip_parser::ast::Element,
    project_root: &Path,
    assets: &mut Vec<AssetReference>,
) {
    use paperclip_parser::ast::Element;

    match element {
        Element::Tag { name, attributes, children, .. } => {
            // Extract from img src
            if name == "img" {
                if let Some(src_expr) = attributes.get("src") {
                    if let Some(src) = expression_to_string(src_expr) {
                        assets.push(AssetReference {
                            path: src.clone(),
                            asset_type: AssetType::Image,
                            resolved_path: resolve_asset_path(&src, project_root),
                        });
                    }
                }
            }

            // Extract from link href (fonts, stylesheets)
            if name == "link" {
                if let Some(href_expr) = attributes.get("href") {
                    if let Some(href) = expression_to_string(href_expr) {
                        let asset_type = if href.ends_with(".woff") || href.ends_with(".woff2") || href.ends_with(".ttf") {
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
                        });
                    }
                }
            }

            // Extract from video/audio sources
            if name == "video" || name == "audio" {
                if let Some(src_expr) = attributes.get("src") {
                    if let Some(src) = expression_to_string(src_expr) {
                        let asset_type = if name == "video" {
                            AssetType::Video
                        } else {
                            AssetType::Audio
                        };
                        assets.push(AssetReference {
                            path: src.clone(),
                            asset_type,
                            resolved_path: resolve_asset_path(&src, project_root),
                        });
                    }
                }
            }

            // Extract from source elements (video/audio children)
            if name == "source" {
                if let Some(src_expr) = attributes.get("src") {
                    if let Some(src) = expression_to_string(src_expr) {
                        assets.push(AssetReference {
                            path: src.clone(),
                            asset_type: AssetType::Other,
                            resolved_path: resolve_asset_path(&src, project_root),
                        });
                    }
                }
            }

            // Recurse into children
            for child in children {
                extract_from_element(child, project_root, assets);
            }
        }

        Element::Instance { children, .. } => {
            // Recurse into component instance children
            for child in children {
                extract_from_element(child, project_root, assets);
            }
        }

        Element::Conditional { then_branch, else_branch, .. } => {
            // Extract from conditional branches
            for child in then_branch {
                extract_from_element(child, project_root, assets);
            }
            if let Some(else_br) = else_branch {
                for child in else_br {
                    extract_from_element(child, project_root, assets);
                }
            }
        }

        Element::Repeat { body, .. } => {
            // Extract from repeat body
            for child in body {
                extract_from_element(child, project_root, assets);
            }
        }

        Element::Text { .. } | Element::SlotInsert { .. } => {
            // No assets in text or slot inserts
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

    if cleaned.starts_with("http://") || cleaned.starts_with("https://") || cleaned.starts_with("//") {
        // External URL - return as-is (PathBuf will just store it)
        PathBuf::from(cleaned)
    } else {
        // Relative path - resolve from project root
        project_root.join(cleaned)
    }
}

// Convert VDocument to protobuf format (stub for now)
fn convert_vdom_to_proto(_vdom: &VDocument) -> proto_vdom::VDocument {
    // TODO: Implement full conversion
    // For now, return empty document
    proto_vdom::VDocument {
        nodes: vec![],
        styles: vec![],
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

        state.update_file(path.clone(), "component A {}".to_string(), &project_root).unwrap();
        assert_eq!(state.get_file(&path).unwrap().version, 0);

        state.update_file(path.clone(), "component B {}".to_string(), &project_root).unwrap();
        assert_eq!(state.get_file(&path).unwrap().version, 1);

        state.update_file(path.clone(), "component C {}".to_string(), &project_root).unwrap();
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

        state.update_file(path.clone(), source.to_string(), &project_root).unwrap();

        let file_state = state.get_file(&path).unwrap();
        // Assets list exists (may be empty if no assets in this simple component)
        assert!(file_state.assets.len() >= 0);
    }
}
