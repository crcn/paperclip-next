use thiserror::Error;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Unexpected token at {pos}: expected {expected}, found {found}")]
    UnexpectedToken {
        pos: usize,
        expected: String,
        found: String,
    },

    #[error("Unexpected end of file at {pos}")]
    UnexpectedEof { pos: usize },

    #[error("Invalid syntax at {pos}: {message}")]
    InvalidSyntax { pos: usize, message: String },

    #[error("Lexer error at {pos}")]
    LexerError { pos: usize },
}

impl ParseError {
    pub fn unexpected_token(pos: usize, expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self::UnexpectedToken {
            pos,
            expected: expected.into(),
            found: found.into(),
        }
    }

    pub fn unexpected_eof(pos: usize) -> Self {
        Self::UnexpectedEof { pos }
    }

    pub fn invalid_syntax(pos: usize, message: impl Into<String>) -> Self {
        Self::InvalidSyntax {
            pos,
            message: message.into(),
        }
    }

    pub fn lexer_error(pos: usize) -> Self {
        Self::LexerError { pos }
    }
}
