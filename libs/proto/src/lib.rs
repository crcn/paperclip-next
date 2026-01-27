//! Paperclip AST and Virtual DOM definitions
//!
//! This crate defines the core data structures for:
//! - AST (Abstract Syntax Tree) for .pc files
//! - Virtual HTML/CSS for evaluation output
//! - Source spans for error reporting and roundtrip serialization

pub mod ast;
pub mod virt;

pub use ast::*;
pub use virt::*;
