//! # CRDT-backed Document Storage
//!
//! Wraps Yjs to provide convergent document editing.
//!
//! ## Key Principles
//!
//! 1. **CRDT is source of truth**: AST is derived from CRDT state
//! 2. **AST is a view**: Can be rebuilt from CRDT at any time
//! 3. **Mutations map to CRDT ops**: High-level mutations → low-level CRDT operations
//! 4. **Convergence guaranteed**: CRDT handles merging, we define semantics

use crate::{Mutation, MutationError};
use paperclip_parser::ast::Document as ASTDocument;
use yrs::{Doc as YDoc, ReadTxn, Transact};

/// CRDT-backed document
///
/// NOTE: This is a simplified implementation for Phase 3.
/// Full CRDT ↔ AST serialization is a complex task that requires
/// careful design of the CRDT schema. This provides the foundation
/// for collaboration while allowing the AST mutation logic to work.
#[derive(Debug)]
pub struct CRDTDocument {
    /// Yjs document (source of truth)
    doc: YDoc,
}

impl CRDTDocument {
    /// Create new CRDT document
    pub fn new() -> Self {
        Self { doc: YDoc::new() }
    }

    /// Create from existing AST
    pub fn from_ast(_ast: &ASTDocument) -> Result<Self, crate::EditorError> {
        let crdt = Self::new();
        // TODO: Serialize AST into CRDT structure
        // This requires defining a stable schema for CRDT representation
        Ok(crdt)
    }

    /// Apply mutation to CRDT
    pub fn apply(&mut self, _mutation: &Mutation) -> Result<(), MutationError> {
        // TODO: Implement CRDT mutations
        //
        // Full implementation would:
        // 1. Find the target node in CRDT structure
        // 2. Apply the mutation using Yjs operations
        // 3. Let CRDT handle convergence
        //
        // For now, this is a placeholder that allows the architecture
        // to compile and the collaboration model to be tested.
        Ok(())
    }

    /// Reconstruct AST from CRDT state
    ///
    /// This is the "derived view" - AST is downstream of CRDT.
    /// Can be called at any time to get current state.
    pub fn to_ast(&self) -> ASTDocument {
        // TODO: Deserialize CRDT structure back to AST
        //
        // This requires walking the CRDT and rebuilding the AST structure.
        // For now, return empty document.
        ASTDocument {
            imports: vec![],
            tokens: vec![],
            triggers: vec![],
            components: vec![],
            styles: vec![],
        }
    }

    /// Get CRDT update to send to other clients
    pub fn get_update(&self) -> Vec<u8> {
        // For now, return empty update
        // Full implementation would encode the document state
        vec![]
    }

    /// Apply update from another client
    pub fn apply_update(&mut self, _update: &[u8]) -> Result<(), crate::EditorError> {
        // For now, accept updates without processing
        // Full implementation would decode and apply Yjs updates
        Ok(())
    }
}

impl Default for CRDTDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_creation() {
        let crdt = CRDTDocument::new();
        let ast = crdt.to_ast();

        // Should create empty document
        assert_eq!(ast.components.len(), 0);
        assert_eq!(ast.styles.len(), 0);
    }

    #[test]
    fn test_crdt_from_ast() {
        let source = "component Test { render div {} }";
        let ast = paperclip_parser::parse(source).unwrap();

        let result = CRDTDocument::from_ast(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_crdt_update_sync() {
        let mut crdt1 = CRDTDocument::new();
        let mut crdt2 = CRDTDocument::new();

        // Get update from crdt1
        let update = crdt1.get_update();

        // Apply to crdt2
        let result = crdt2.apply_update(&update);
        assert!(result.is_ok());
    }
}
