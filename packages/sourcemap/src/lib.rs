//! Source map utilities for Paperclip compilers
//!
//! This crate provides shared utilities for generating source maps
//! during compilation from .pc files to various targets (React, CSS, HTML).

pub mod builder;
pub mod utils;

pub use builder::SourceMapBuilder;
pub use utils::{byte_offset_to_line_col, line_col_to_byte_offset};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let source = "line 1\nline 2\nline 3";
        let (line, col) = byte_offset_to_line_col(source, 7);
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }
}
