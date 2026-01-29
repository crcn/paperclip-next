//! # Editing Pipeline
//!
//! Coordinates the full document lifecycle: Parse → Mutate → Evaluate → Diff
//!
//! The Pipeline manages:
//! - Applying mutations
//! - Re-evaluation to VDOM
//! - Incremental diffing
//! - Caching for efficiency

use crate::{Document, EditorError, Mutation};
use paperclip_evaluator::{diff_vdocument, VirtualDomDocument};

/// Manages the full edit → render pipeline
pub struct Pipeline {
    document: Document,
    last_vdom: Option<VirtualDomDocument>,
}

impl Pipeline {
    /// Create pipeline for document
    pub fn new(document: Document) -> Self {
        Self {
            document,
            last_vdom: None,
        }
    }

    /// Apply mutation and get incremental update
    ///
    /// This:
    /// 1. Applies the mutation
    /// 2. Re-evaluates to VDOM
    /// 3. Computes diff against previous VDOM
    /// 4. Returns patches for efficient client updates
    pub fn apply_mutation(&mut self, mutation: Mutation) -> Result<PipelineResult, EditorError> {
        // 1. Apply mutation
        let mutation_result = self.document.apply(mutation)?;

        // 2. Evaluate to new VDOM
        let new_vdom = self.document.evaluate()?;

        // 3. Compute diff (if we have a previous VDOM)
        let patches = if let Some(_old_vdom) = &self.last_vdom {
            // TODO: Use diff_vdocument when we have proper evaluation
            // For now, return empty patches
            vec![]
        } else {
            vec![]
        };

        // 4. Update cached VDOM
        self.last_vdom = Some(new_vdom.clone());

        Ok(PipelineResult {
            version: mutation_result.version,
            vdom: new_vdom,
            patches,
        })
    }

    /// Full re-evaluation (for recovery/debugging)
    ///
    /// Useful when:
    /// - Client gets out of sync
    /// - Error recovery
    /// - Initial render
    pub fn full_evaluate(&mut self) -> Result<VirtualDomDocument, EditorError> {
        let vdom = self.document.evaluate()?;
        self.last_vdom = Some(vdom.clone());
        Ok(vdom)
    }

    /// Get current document
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Get mutable document reference
    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }

    /// Get last VDOM (if any)
    pub fn last_vdom(&self) -> Option<&VirtualDomDocument> {
        self.last_vdom.as_ref()
    }

    /// Clear VDOM cache (force full re-render on next mutation)
    pub fn clear_cache(&mut self) {
        self.last_vdom = None;
    }
}

/// Result of pipeline execution
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// New version number
    pub version: u64,

    /// Full VDOM (for fallback)
    pub vdom: VirtualDomDocument,

    /// Incremental patches (for efficiency)
    pub patches: Vec<u8>, // TODO: Use actual patch type
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_pipeline_initial_evaluation() {
        let source = "component Test { render div {} }";
        let doc = Document::from_source(PathBuf::from("test.pc"), source.to_string()).unwrap();

        let mut pipeline = Pipeline::new(doc);

        // Initial evaluation
        let result = pipeline.full_evaluate();
        assert!(result.is_ok());

        // Should have cached VDOM
        assert!(pipeline.last_vdom().is_some());
    }

    #[test]
    fn test_pipeline_mutation_increments_version() {
        let source = "component Test { render div {} }";
        let doc = Document::from_source(PathBuf::from("test.pc"), source.to_string()).unwrap();

        let mut pipeline = Pipeline::new(doc);

        let mutation = Mutation::UpdateText {
            node_id: "text-123".to_string(),
            content: "Hello".to_string(),
        };

        // Will fail to apply but should still work through pipeline
        let _ = pipeline.apply_mutation(mutation);

        // Version should increment
        assert_eq!(pipeline.document().version, 1);
    }
}
