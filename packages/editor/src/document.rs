//! # Document Handle
//!
//! Core document abstraction for Paperclip editing.
//!
//! A Document represents a single .pc file and its editing state.
//! Documents can be:
//! - **Memory-backed**: Temporary, for testing or in-memory operations
//! - **File-backed**: Single-user editing with disk persistence
//! - **CRDT-backed**: Multi-user collaborative editing
//!
//! ## Lifecycle
//!
//! ```text
//! Load → Parse → Edit → Evaluate → Save
//!   ↓      ↓       ↓        ↓        ↓
//! File   AST   Mutations  VDOM    File
//! ```

use std::path::PathBuf;
use paperclip_parser::{parse, ast::Document as ASTDocument};
use paperclip_evaluator::{Evaluator, Bundle, VirtualDomDocument};
use crate::{Mutation, MutationResult, EditorError};

#[cfg(feature = "collaboration")]
use crate::crdt::CRDTDocument;

/// Editable Paperclip document
#[derive(Debug)]
pub struct Document {
    /// Path to source file (if any)
    pub path: PathBuf,

    /// Current version number (increments on each mutation)
    pub version: u64,

    /// Backing storage strategy
    storage: DocumentStorage,
}

/// Storage backend for document
#[derive(Debug)]
pub enum DocumentStorage {
    /// In-memory only (for testing, temp docs)
    Memory {
        source: String,
        ast: ASTDocument,
    },

    /// File-backed (single-user editing)
    File {
        source: String,
        ast: ASTDocument,
        dirty: bool,
    },

    /// CRDT-backed (for collaboration)
    #[cfg(feature = "collaboration")]
    CRDT {
        crdt: CRDTDocument,
        ast_cache: Option<ASTDocument>,
    },
}

impl Document {
    /// Create document from source text (memory-backed)
    pub fn from_source(path: PathBuf, source: String) -> Result<Self, EditorError> {
        let ast = parse(&source)?;

        Ok(Self {
            path,
            version: 0,
            storage: DocumentStorage::Memory { source, ast },
        })
    }

    /// Load document from file (file-backed)
    pub fn load(path: PathBuf) -> Result<Self, EditorError> {
        let source = std::fs::read_to_string(&path)?;
        let ast = parse(&source)?;

        Ok(Self {
            path,
            version: 0,
            storage: DocumentStorage::File {
                source,
                ast,
                dirty: false,
            },
        })
    }

    /// Create collaborative document (CRDT-backed)
    #[cfg(feature = "collaboration")]
    pub fn collaborative(path: PathBuf, source: String) -> Result<Self, EditorError> {
        let ast = parse(&source)?;
        let crdt = CRDTDocument::from_ast(&ast)?;

        Ok(Self {
            path,
            version: 0,
            storage: DocumentStorage::CRDT {
                crdt,
                ast_cache: Some(ast),
            },
        })
    }

    /// Get current AST (cheap if cached)
    pub fn ast(&mut self) -> &ASTDocument {
        match &mut self.storage {
            DocumentStorage::Memory { ast, .. } => ast,
            DocumentStorage::File { ast, .. } => ast,

            #[cfg(feature = "collaboration")]
            DocumentStorage::CRDT { crdt, ast_cache } => {
                if ast_cache.is_none() {
                    *ast_cache = Some(crdt.to_ast());
                }
                ast_cache.as_ref().unwrap()
            }
        }
    }

    /// Get mutable AST reference (invalidates caches)
    pub fn ast_mut(&mut self) -> &mut ASTDocument {
        match &mut self.storage {
            DocumentStorage::Memory { ast, .. } => ast,
            DocumentStorage::File { ast, dirty, .. } => {
                *dirty = true;
                ast
            },

            #[cfg(feature = "collaboration")]
            DocumentStorage::CRDT { ast_cache, .. } => {
                // For CRDT, we don't allow direct AST mutation
                // Must go through apply() which updates CRDT
                panic!("Cannot get mutable AST for CRDT-backed documents. Use apply() instead.");
            }
        }
    }

    /// Evaluate document to VDOM
    pub fn evaluate(&mut self) -> Result<VirtualDomDocument, EditorError> {
        let ast = self.ast();
        // Create a minimal bundle with just this document
        let mut bundle = Bundle::new();
        // For now, return empty VDOM - proper implementation needs bundle setup
        // TODO: Implement proper evaluation once we have bundle integration
        Ok(VirtualDomDocument {
            nodes: vec![],
            styles: vec![],
        })
    }

    /// Apply a mutation
    pub fn apply(&mut self, mutation: Mutation) -> Result<MutationResult, EditorError> {
        self.version += 1;

        match &mut self.storage {
            DocumentStorage::Memory { source, ast } |
            DocumentStorage::File { source, ast, .. } => {
                // Apply mutation to AST
                mutation.apply(ast)?;

                // TODO: Regenerate source from AST (for text editor sync)
                // This is optional - we could keep source and AST as separate views

                if let DocumentStorage::File { dirty, .. } = &mut self.storage {
                    *dirty = true;
                }

                Ok(MutationResult {
                    version: self.version,
                    vdom_patches: None,
                })
            }

            #[cfg(feature = "collaboration")]
            DocumentStorage::CRDT { crdt, ast_cache } => {
                // Apply to CRDT
                crdt.apply(&mutation)?;

                // Invalidate AST cache
                *ast_cache = None;

                Ok(MutationResult {
                    version: self.version,
                    vdom_patches: None,
                })
            }
        }
    }

    /// Check if document has unsaved changes
    pub fn is_dirty(&self) -> bool {
        match &self.storage {
            DocumentStorage::File { dirty, .. } => *dirty,
            _ => false,
        }
    }

    /// Save document to disk (if file-backed)
    pub fn save(&mut self) -> Result<(), EditorError> {
        match &mut self.storage {
            DocumentStorage::File { source, dirty, .. } => {
                std::fs::write(&self.path, source)?;
                *dirty = false;
                Ok(())
            }
            _ => Err(EditorError::NotFileBacked),
        }
    }

    /// Get source text
    pub fn source(&self) -> Option<&str> {
        match &self.storage {
            DocumentStorage::Memory { source, .. } => Some(source),
            DocumentStorage::File { source, .. } => Some(source),
            #[cfg(feature = "collaboration")]
            DocumentStorage::CRDT { .. } => None,  // Source derived from CRDT
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_create_memory_document() {
        let source = r#"
            component Button {
                render div {
                    text "Click me"
                }
            }
        "#;

        let doc = Document::from_source(
            PathBuf::from("test.pc"),
            source.to_string()
        );

        assert!(doc.is_ok());
        let mut doc = doc.unwrap();
        assert_eq!(doc.version, 0);
        assert!(!doc.is_dirty());

        // Can get AST
        let ast = doc.ast();
        assert_eq!(ast.components.len(), 1);
    }

    #[test]
    fn test_document_version_increments() {
        let source = "component Test { render div {} }";
        let mut doc = Document::from_source(
            PathBuf::from("test.pc"),
            source.to_string()
        ).unwrap();

        assert_eq!(doc.version, 0);

        // Apply mutation (will fail but still increment version)
        let mutation = Mutation::UpdateText {
            node_id: "test-123".to_string(),
            content: "Hello".to_string(),
        };

        let _ = doc.apply(mutation);
        assert_eq!(doc.version, 1);
    }
}
