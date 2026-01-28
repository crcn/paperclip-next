pub mod tokenizer;
pub mod parser;
pub mod ast;
pub mod error;
pub mod serializer;
mod debug_test;

#[cfg(test)]
mod tests_comprehensive;

pub use tokenizer::{Token, tokenize};
pub use parser::{Parser, parse};
pub use serializer::{Serializer, serialize};
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
