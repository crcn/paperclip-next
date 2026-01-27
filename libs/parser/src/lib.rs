pub mod tokenizer;
pub mod parser;
pub mod ast;
pub mod error;
mod debug_test;

pub use tokenizer::{Token, tokenize};
pub use parser::{Parser, parse};
pub use error::{ParseError, ParseResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_basic() {
        let source = "component Button";
        let tokens = tokenize(source);
        assert_eq!(tokens.len(), 2);
    }
}
