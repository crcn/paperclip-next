//! Error types for the Paperclip parser

use crate::lexer::TokenSpan;
use thiserror::Error;

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Parse error with location and context
#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("Unexpected token at {span:?}: expected {expected}, found {found}")]
    UnexpectedToken {
        span: TokenSpan,
        expected: String,
        found: String,
    },

    #[error("Unexpected end of input: expected {expected}")]
    UnexpectedEof { expected: String },

    #[error("Invalid syntax at {span:?}: {message}")]
    InvalidSyntax { span: TokenSpan, message: String },

    #[error("Lexer error at {span:?}: {message}")]
    LexError { span: TokenSpan, message: String },
}

impl ParseError {
    pub fn span(&self) -> Option<TokenSpan> {
        match self {
            ParseError::UnexpectedToken { span, .. } => Some(*span),
            ParseError::UnexpectedEof { .. } => None,
            ParseError::InvalidSyntax { span, .. } => Some(*span),
            ParseError::LexError { span, .. } => Some(*span),
        }
    }
}

/// Collection of errors from a parse attempt
#[derive(Debug, Default)]
pub struct ParseErrors {
    pub errors: Vec<ParseError>,
}

impl ParseErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn push(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

/// Pretty-print errors with source context using ariadne
pub fn format_errors(source: &str, filename: &str, errors: &ParseErrors) -> String {
    use ariadne::{Color, Label, Report, ReportKind, Source};

    let mut output = Vec::new();

    for error in &errors.errors {
        let span = error.span().unwrap_or(TokenSpan {
            start: source.len().saturating_sub(1),
            end: source.len(),
        });

        let report = Report::build(ReportKind::Error, filename, span.start)
            .with_message(error.to_string())
            .with_label(
                Label::new((filename, span.start..span.end))
                    .with_color(Color::Red)
                    .with_message(match error {
                        ParseError::UnexpectedToken { expected, .. } => {
                            format!("expected {}", expected)
                        }
                        ParseError::UnexpectedEof { expected } => {
                            format!("expected {}", expected)
                        }
                        ParseError::InvalidSyntax { message, .. } => message.clone(),
                        ParseError::LexError { message, .. } => message.clone(),
                    }),
            )
            .finish();

        report
            .write((filename, Source::from(source)), &mut output)
            .unwrap();
    }

    String::from_utf8(output).unwrap_or_else(|_| "Error formatting failed".to_string())
}
