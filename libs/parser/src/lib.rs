//! Paperclip Parser
//!
//! High-performance parser for .pc files using:
//! - `logos` for lexing (10-100x faster than hand-rolled)
//! - `chumsky` for parsing (excellent error recovery)
//! - `bumpalo` for arena allocation (zero-copy where possible)
//!
//! Performance targets:
//! - Parse 1000-line file in <10ms
//! - Use &str slices into source instead of String copies
//! - Preallocate based on file size heuristics

pub mod lexer;
pub mod parser;
pub mod error;

pub use error::{ParseError, ParseResult};
pub use parser::parse;

#[cfg(test)]
mod tests;
