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

// Extract asset references from AST (simplified for spike - just return empty)
fn extract_assets(_ast: &Document, _project_root: &Path) -> Vec<AssetReference> {
    // TODO: Implement full asset extraction
    // For spike, we skip this as AST structure is complex
    vec![]
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
}
