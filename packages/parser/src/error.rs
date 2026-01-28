use std::ops::Range;
use thiserror::Error;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Unexpected token at {}: expected {expected}, found {found}", span.start)]
    UnexpectedToken {
        span: Box<Range<usize>>,
        expected: String,
        found: String,
    },

    #[error("Unexpected end of file at {}", span.start)]
    UnexpectedEof { span: Box<Range<usize>> },

    #[error("Invalid syntax at {}: {message}", span.start)]
    InvalidSyntax { span: Box<Range<usize>>, message: String },

    #[error("Lexer error at {}", span.start)]
    LexerError { span: Box<Range<usize>> },
}

impl ParseError {
    // New span-based constructors (preferred)
    pub fn unexpected_token_span(
        span: Range<usize>,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        Self::UnexpectedToken {
            span: Box::new(span),
            expected: expected.into(),
            found: found.into(),
        }
    }

    pub fn unexpected_eof_span(span: Range<usize>) -> Self {
        Self::UnexpectedEof { span: Box::new(span) }
    }

    pub fn invalid_syntax_span(span: Range<usize>, message: impl Into<String>) -> Self {
        Self::InvalidSyntax {
            span: Box::new(span),
            message: message.into(),
        }
    }

    pub fn lexer_error_span(span: Range<usize>) -> Self {
        Self::LexerError { span: Box::new(span) }
    }

    // Backward-compatible position-based constructors (convert pos to pos..pos+1)
    pub fn unexpected_token(
        pos: usize,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        Self::UnexpectedToken {
            span: Box::new(pos..pos + 1),
            expected: expected.into(),
            found: found.into(),
        }
    }

    pub fn unexpected_eof(pos: usize) -> Self {
        Self::UnexpectedEof { span: Box::new(pos..pos + 1) }
    }

    pub fn invalid_syntax(pos: usize, message: impl Into<String>) -> Self {
        Self::InvalidSyntax {
            span: Box::new(pos..pos + 1),
            message: message.into(),
        }
    }

    pub fn lexer_error(pos: usize) -> Self {
        Self::LexerError { span: Box::new(pos..pos + 1) }
    }

    // Utility accessors
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::UnexpectedToken { span, .. } => *span.clone(),
            Self::UnexpectedEof { span } => *span.clone(),
            Self::InvalidSyntax { span, .. } => *span.clone(),
            Self::LexerError { span } => *span.clone(),
        }
    }

    pub fn position(&self) -> usize {
        self.span().start
    }
}

/// Pretty error formatting using ariadne
#[cfg(feature = "pretty-errors")]
pub mod pretty {
    use super::ParseError;
    use ariadne::{Color, Label, Report, ReportKind, Source};

    /// Format an error with beautiful output showing source context
    pub fn format_error(error: &ParseError, file_path: &str, source: &str) -> String {
        let span = error.span();

        // Build report inline to avoid lifetime issues
        let report = match error {
            ParseError::UnexpectedToken { expected, found, .. } => {
                Report::build(ReportKind::Error, file_path, span.start)
                    .with_message("Unexpected token")
                    .with_label(
                        Label::new((file_path, span.clone()))
                            .with_message(format!("expected {}, found {}", expected, found))
                            .with_color(Color::Red),
                    )
                    .with_help(get_context_help(expected, found))
                    .finish()
            }
            ParseError::UnexpectedEof { .. } => {
                Report::build(ReportKind::Error, file_path, span.start)
                    .with_message("Unexpected end of file")
                    .with_label(
                        Label::new((file_path, span.clone()))
                            .with_message("file ended unexpectedly")
                            .with_color(Color::Red),
                    )
                    .with_help("Check for missing closing braces or brackets")
                    .finish()
            }
            ParseError::InvalidSyntax { message, .. } => {
                Report::build(ReportKind::Error, file_path, span.start)
                    .with_message("Invalid syntax")
                    .with_label(
                        Label::new((file_path, span.clone()))
                            .with_message(message.clone())
                            .with_color(Color::Red),
                    )
                    .finish()
            }
            ParseError::LexerError { .. } => {
                Report::build(ReportKind::Error, file_path, span.start)
                    .with_message("Lexer error")
                    .with_label(
                        Label::new((file_path, span.clone()))
                            .with_message("invalid token")
                            .with_color(Color::Red),
                    )
                    .finish()
            }
        };

        let mut output = Vec::new();
        report
            .write((file_path, Source::from(source)), &mut output)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to format error with ariadne: {}", e);
            });

        String::from_utf8(output).unwrap_or_else(|_| error.to_string())
    }

    /// Emit an error directly to stderr with formatting
    pub fn emit_error(error: &ParseError, file_path: &str, source: &str) {
        let formatted = format_error(error, file_path, source);
        eprintln!("{}", formatted);
    }

    fn get_context_help(expected: &str, found: &str) -> String {
        // Provide context-aware help messages
        if expected.contains("string") && found.contains("}") {
            "Text elements require a content expression".to_string()
        } else if expected.contains("}") {
            "Check for missing closing brace".to_string()
        } else if expected.contains("{") {
            "Expected opening brace to start block".to_string()
        } else if expected.contains("identifier") {
            "Expected a name or identifier here".to_string()
        } else {
            format!("Expected {} but got {}", expected, found)
        }
    }
}
