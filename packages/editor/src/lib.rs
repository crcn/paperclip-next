//! # Paperclip Editor
//!
//! Core document editing engine for Paperclip.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │ parser: .pc text → AST                      │
//! └─────────────────────────────────────────────┘
//!                     ↓
//! ┌─────────────────────────────────────────────┐
//! │ editor: Document lifecycle + mutations      │
//! │  - Load/save documents                      │
//! │  - Apply mutations with validation          │
//! │  - CRDT-backed convergence (optional)       │
//! │  - Coordinate parse → evaluate pipeline     │
//! └─────────────────────────────────────────────┘
//!                     ↓
//! ┌─────────────────────────────────────────────┐
//! │ evaluator: AST → VDOM                       │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Core Principles
//!
//! 1. **AST is source of truth**: VDOM and patches are derived views
//! 2. **CRDT for convergence**: Not for semantics - we define operation meaning
//! 3. **Structural collaboration**: Node-level operations, not text-level
//! 4. **Optimistic clients**: Local projection can be discarded and rebuilt
//! 5. **Server authority**: Client state always defers to server
//!
//! ## Usage
//!
//! ### Single-user editing
//!
//! ```rust,ignore
//! use paperclip_editor::{Document, Mutation};
//!
//! // Load document
//! let mut doc = Document::load("button.pc")?;
//!
//! // Apply mutation
//! let mutation = Mutation::UpdateText {
//!     node_id: "text-123".to_string(),
//!     content: "Click me!".to_string(),
//! };
//! doc.apply(mutation)?;
//!
//! // Evaluate to VDOM
//! let vdom = doc.evaluate()?;
//!
//! // Save
//! doc.save()?;
//! ```
//!
//! ### Collaborative editing
//!
//! ```rust,ignore
//! use paperclip_editor::{Document, EditSession};
//!
//! // Create collaborative document (requires "collaboration" feature)
//! #[cfg(feature = "collaboration")]
//! {
//!     let doc = Document::collaborative("button.pc")?;
//!     let mut session = EditSession::new("client-1", doc);
//!
//!     // Apply mutation optimistically
//!     session.apply_optimistic(mutation)?;
//!
//!     // Send to server, get updates, rebase as needed
//! }
//! ```

mod document;
mod errors;
mod mutations;
mod pipeline;
mod post_effects;
mod session;
mod undo_stack;

#[cfg(feature = "collaboration")]
mod crdt;

pub use document::{Document, DocumentStorage};
pub use errors::EditorError;
pub use mutations::{Mutation, MutationError, MutationResult};
pub use pipeline::{Pipeline, PipelineResult};
pub use post_effects::{PostEffect, PostEffectEngine};
pub use session::{EditSession, PendingMutation};
pub use undo_stack::{MutationBatch, UndoStack};

#[cfg(feature = "collaboration")]
pub use crdt::CRDTDocument;

// Re-export common types for convenience
pub use paperclip_evaluator::VirtualDomDocument;
pub use paperclip_parser::ast::Document as ASTDocument;
