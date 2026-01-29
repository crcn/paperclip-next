/// Convert byte offset to line and column number
///
/// # Arguments
/// * `source` - The source text
/// * `offset` - Byte offset in the source
///
/// # Returns
/// Tuple of (line, column) both 0-indexed
pub fn byte_offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0;
    let mut col = 0;
    let mut byte_pos = 0;

    for ch in source.chars() {
        if byte_pos >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        byte_pos += ch.len_utf8();
    }

    (line, col)
}

/// Convert line and column to byte offset
///
/// # Arguments
/// * `source` - The source text
/// * `line` - Line number (0-indexed)
/// * `col` - Column number (0-indexed)
///
/// # Returns
/// Byte offset in the source, or source.len() if out of bounds
pub fn line_col_to_byte_offset(source: &str, target_line: u32, target_col: u32) -> usize {
    let mut line = 0;
    let mut col = 0;
    let mut byte_pos = 0;

    for ch in source.chars() {
        if line == target_line && col == target_col {
            return byte_pos;
        }

        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        byte_pos += ch.len_utf8();
    }

    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_offset_to_line_col() {
        let source = "line 1\nline 2\nline 3";

        // Start of file
        assert_eq!(byte_offset_to_line_col(source, 0), (0, 0));

        // Start of second line
        assert_eq!(byte_offset_to_line_col(source, 7), (1, 0));

        // Middle of second line
        assert_eq!(byte_offset_to_line_col(source, 10), (1, 3));

        // Start of third line
        assert_eq!(byte_offset_to_line_col(source, 14), (2, 0));
    }

    #[test]
    fn test_line_col_to_byte_offset() {
        let source = "line 1\nline 2\nline 3";

        // Start of file
        assert_eq!(line_col_to_byte_offset(source, 0, 0), 0);

        // Start of second line
        assert_eq!(line_col_to_byte_offset(source, 1, 0), 7);

        // Middle of second line
        assert_eq!(line_col_to_byte_offset(source, 1, 3), 10);

        // Start of third line
        assert_eq!(line_col_to_byte_offset(source, 2, 0), 14);
    }

    #[test]
    fn test_roundtrip() {
        let source = "component Button {\n  render button\n}";
        let offset = 15;

        let (line, col) = byte_offset_to_line_col(source, offset);
        let back_to_offset = line_col_to_byte_offset(source, line, col);

        assert_eq!(offset, back_to_offset);
    }

    #[test]
    fn test_unicode_handling() {
        let source = "日本語\ntext";

        // Unicode characters should be handled correctly
        let (line, col) = byte_offset_to_line_col(source, 10); // After "日本語\n"
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_empty_source() {
        let source = "";
        assert_eq!(byte_offset_to_line_col(source, 0), (0, 0));
        assert_eq!(line_col_to_byte_offset(source, 0, 0), 0);
    }

    #[test]
    fn test_out_of_bounds() {
        let source = "short";

        // Out of bounds offset should return last position
        let (line, col) = byte_offset_to_line_col(source, 1000);
        assert_eq!(line, 0);
        assert_eq!(col, 5);

        // Out of bounds line/col should return source.len()
        assert_eq!(line_col_to_byte_offset(source, 10, 0), source.len());
    }
}
