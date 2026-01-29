pub mod ast;
mod debug_test;
pub mod error;
pub mod id_generator;
pub mod parser;
pub mod serializer;
pub mod tokenizer;

#[cfg(test)]
mod tests_comprehensive;

#[cfg(test)]
mod tests_new_features;

#[cfg(test)]
mod test_comprehensive_example;

#[cfg(test)]
mod tests_serializer;

pub use error::{ParseError, ParseResult};
pub use id_generator::{get_document_id, IDGenerator};
pub use parser::{parse, parse_with_path, Parser};
pub use serializer::{serialize, Serializer};
pub use tokenizer::{tokenize, Token};

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
