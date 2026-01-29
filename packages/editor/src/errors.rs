//! Error types for the editor

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("Parse error: {0}")]
    Parse(#[from] paperclip_parser::error::ParseError),

    #[error("Evaluation error: {0}")]
    Evaluation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Mutation error: {0}")]
    Mutation(#[from] crate::mutations::MutationError),

    #[error("Document is not file-backed")]
    NotFileBacked,

    #[error("Document is read-only")]
    ReadOnly,

    #[cfg(feature = "collaboration")]
    #[error("CRDT error: {0}")]
    CRDT(String),
}

impl From<paperclip_evaluator::EvalError> for EditorError {
    fn from(e: paperclip_evaluator::EvalError) -> Self {
        EditorError::Evaluation(format!("{:?}", e))
    }
}
