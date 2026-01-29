use sourcemap::{SourceMap as ExternalSourceMap, SourceMapBuilder as ExternalBuilder};

/// Builder for generating source maps during compilation
pub struct SourceMapBuilder {
    builder: ExternalBuilder,
    #[allow(dead_code)]
    source_file: String,
    current_line: u32,
    current_col: u32,
}

impl SourceMapBuilder {
    /// Create a new source map builder
    ///
    /// # Arguments
    /// * `source_file` - The original .pc file path
    /// * `source_content` - The original .pc file content
    pub fn new(source_file: &str, source_content: &str) -> Self {
        let mut builder = ExternalBuilder::new(None);

        // Add the source file and its content
        let source_id = builder.add_source(source_file);
        builder.set_source_contents(source_id, Some(source_content));

        Self {
            builder,
            source_file: source_file.to_string(),
            current_line: 0,
            current_col: 0,
        }
    }

    /// Add a mapping from generated position to source position
    ///
    /// # Arguments
    /// * `gen_line` - Line in generated file (0-indexed)
    /// * `gen_col` - Column in generated file (0-indexed)
    /// * `src_line` - Line in original .pc file (0-indexed)
    /// * `src_col` - Column in original .pc file (0-indexed)
    /// * `name` - Optional symbol name (e.g., component name)
    pub fn add_mapping(
        &mut self,
        gen_line: u32,
        gen_col: u32,
        src_line: u32,
        src_col: u32,
        name: Option<&str>,
    ) {
        let name_id = name.map(|n| self.builder.add_name(n));

        self.builder.add_raw(
            gen_line,
            gen_col,
            src_line,
            src_col,
            Some(0), // source index (always 0 since we have one source file)
            name_id,
            false, // is_range
        );
    }

    /// Track position advancement as we emit generated code
    ///
    /// Call this after appending text to the output buffer to keep
    /// track of the current position in the generated file.
    pub fn advance(&mut self, text: &str) {
        for ch in text.chars() {
            if ch == '\n' {
                self.current_line += 1;
                self.current_col = 0;
            } else {
                self.current_col += 1;
            }
        }
    }

    /// Get the current position in the generated output
    pub fn current_position(&self) -> (u32, u32) {
        (self.current_line, self.current_col)
    }

    /// Build the final source map
    pub fn build(self) -> ExternalSourceMap {
        self.builder.into_sourcemap()
    }

    /// Convert to JSON string
    pub fn to_json(self) -> Result<String, sourcemap::Error> {
        let map = self.build();
        let mut buf = Vec::new();
        map.to_writer(&mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let source = "component Button {}";
        let builder = SourceMapBuilder::new("button.pc", source);
        assert_eq!(builder.current_position(), (0, 0));
    }

    #[test]
    fn test_advance_tracking() {
        let source = "component Button {}";
        let mut builder = SourceMapBuilder::new("button.pc", source);

        builder.advance("const Button = () => {");
        let (line, col) = builder.current_position();
        assert_eq!(line, 0);
        assert_eq!(col, 22); // 22 characters in "const Button = () => {"

        builder.advance("\n");
        let (line, col) = builder.current_position();
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_mapping_generation() {
        let source = "component Button {}";
        let mut builder = SourceMapBuilder::new("button.pc", source);

        // Add a mapping: generated position (0,6) -> source position (0,10)
        builder.add_mapping(0, 6, 0, 10, Some("Button"));

        let map = builder.build();

        // Verify the source map has our source file
        assert_eq!(map.get_source(0), Some("button.pc"));
    }

    #[test]
    fn test_json_output() {
        let source = "component Button {}";
        let mut builder = SourceMapBuilder::new("button.pc", source);
        builder.add_mapping(0, 0, 0, 0, None);

        let json = builder.to_json().unwrap();

        // Basic validation - should be valid JSON with required fields
        assert!(json.contains("\"version\":3"));
        assert!(json.contains("\"sources\""));
        assert!(json.contains("button.pc"));
    }
}
