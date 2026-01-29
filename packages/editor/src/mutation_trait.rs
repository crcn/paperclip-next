use paperclip_parser::ast::Document;
use crate::mutations::MutationError;

/// Trait for mutation operations
///
/// Each mutation type implements this trait to provide:
/// - Validation logic
/// - Apply logic
/// - Inverse operation for undo
pub trait MutationOp: Send + Sync {
    /// Validate that this mutation can be applied to the document
    fn validate(&self, doc: &Document) -> Result<(), MutationError>;

    /// Apply this mutation to the document
    fn apply(&self, doc: &mut Document) -> Result<(), MutationError>;

    /// Create the inverse mutation for undo
    fn to_inverse(&self, doc: &Document) -> Result<Box<dyn MutationOp>, MutationError>;

    /// Get a debug name for this mutation
    fn name(&self) -> &'static str;
}
