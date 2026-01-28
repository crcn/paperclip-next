use thiserror::Error;

/// Errors that can occur during type inference
#[derive(Error, Debug, Clone, PartialEq)]
pub enum InferenceError {
    #[error("Variable '{0}' not found in scope")]
    VariableNotFound(String),

    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Invalid operand types for operator '{operator}': {left} and {right}")]
    InvalidOperandTypes {
        operator: String,
        left: String,
        right: String,
    },

    #[error("Cannot infer type for expression")]
    CannotInfer,

    #[error("Type contradiction: variable '{variable}' used as both {type1} and {type2}")]
    TypeContradiction {
        variable: String,
        type1: String,
        type2: String,
    },

    #[error("Strict mode error: Unknown type remained after inference for '{0}'")]
    StrictModeUnknown(String),

    #[error("Invalid member access on non-object type: {0}")]
    InvalidMemberAccess(String),
}

pub type InferenceResult<T> = Result<T, InferenceError>;
