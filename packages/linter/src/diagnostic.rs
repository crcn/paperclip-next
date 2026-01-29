use paperclip_parser::ast::Span;
use serde::{Deserialize, Serialize};

/// Severity level of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

/// A diagnostic message from the linter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// The severity level
    pub level: DiagnosticLevel,

    /// The rule that generated this diagnostic
    pub rule: String,

    /// Human-readable message
    pub message: String,

    /// Source location where the issue was found
    pub span: Span,

    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn error(rule: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            rule: rule.into(),
            message: message.into(),
            span,
            suggestion: None,
        }
    }

    pub fn warning(rule: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            rule: rule.into(),
            message: message.into(),
            span,
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}
