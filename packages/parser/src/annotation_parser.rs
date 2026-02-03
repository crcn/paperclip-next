//! Parser for doc comment annotations
//!
//! This module handles parsing of annotations within doc comments, such as:
//! ```text
//! /**
//!  * This is a card component for displaying content.
//!  * @frame(x: 100, y: 200, width: 300, height: 400)
//!  * Additional notes about usage go here.
//!  * @prop(variant: primary, size: large)
//!  */
//! ```

use crate::ast::{Annotation, AnnotationValue, DocComment, Span};
use crate::id_generator::IDGenerator;

/// Parse doc comment content - handles mixed text and annotations
/// Input: "/** Description here @frame(x: 100) more text @prop(a: 1) */"
/// Output: DocComment { description: "Description here more text", annotations: [...] }
pub fn parse_doc_comment(content: &str, span: Span, id_gen: &mut IDGenerator) -> DocComment {
    // Strip /** and */ delimiters
    let inner = strip_delimiters(content);

    // Strip leading * from each line and normalize
    let cleaned = clean_doc_lines(&inner);

    // Scan for @name(...) patterns and extract annotations
    let (description, annotations) = extract_annotations(&cleaned, &span, id_gen);

    DocComment {
        description: description.trim().to_string(),
        annotations,
        span,
    }
}

/// Strip the /** and */ delimiters from a doc comment
fn strip_delimiters(content: &str) -> String {
    let trimmed = content.trim();

    // Remove leading /**
    let without_start = if trimmed.starts_with("/**") {
        &trimmed[3..]
    } else {
        trimmed
    };

    // Remove trailing */
    let without_end = if without_start.ends_with("*/") {
        &without_start[..without_start.len() - 2]
    } else {
        without_start
    };

    without_end.to_string()
}

/// Clean doc comment lines by removing leading * and normalizing whitespace
fn clean_doc_lines(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            // Remove leading * if present (common doc comment style)
            if trimmed.starts_with('*') {
                trimmed[1..].trim_start()
            } else {
                trimmed
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract annotations from cleaned doc comment content
/// Returns (description_text, annotations)
fn extract_annotations(
    content: &str,
    doc_span: &Span,
    id_gen: &mut IDGenerator,
) -> (String, Vec<Annotation>) {
    let mut annotations = Vec::new();
    let mut description = String::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Look for @ that starts an annotation
        if chars[i] == '@' {
            // Check if this @ is inside a quoted string by scanning backwards
            // For simplicity, we'll assume @ at the start or after whitespace is an annotation
            let is_start_of_word =
                i == 0 || chars[i - 1].is_whitespace() || chars[i - 1] == '(' || chars[i - 1] == ',';

            if is_start_of_word {
                // Try to parse annotation name
                let mut j = i + 1;
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                    j += 1;
                }

                if j > i + 1 {
                    let name: String = chars[i + 1..j].iter().collect();

                    // Check for optional params in parentheses
                    if j < chars.len() && chars[j] == '(' {
                        // Find matching closing paren
                        if let Some((params_str, end_pos)) = find_matching_paren(&chars, j) {
                            // Parse the annotation with params
                            if let Some(annotation) =
                                parse_annotation(&name, &params_str, doc_span, id_gen)
                            {
                                annotations.push(annotation);
                            }
                            i = end_pos + 1;
                            continue;
                        }
                    } else {
                        // Annotation without params (like @deprecated)
                        annotations.push(Annotation {
                            name,
                            params: Vec::new(),
                            span: Span::new(doc_span.start, doc_span.end, id_gen.new_id()),
                        });
                        i = j;
                        continue;
                    }
                }
            }
        }

        // Not an annotation, add to description
        description.push(chars[i]);
        i += 1;
    }

    (description, annotations)
}

/// Find matching closing paren, tracking nested parens and brackets
/// Returns (content_between_parens, position_of_closing_paren)
fn find_matching_paren(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars[start] != '(' {
        return None;
    }

    let mut depth = 1;
    let mut bracket_depth = 0;
    let mut in_string = false;
    let mut string_char = '"';
    let mut content = String::new();
    let mut i = start + 1;

    while i < chars.len() && depth > 0 {
        let c = chars[i];

        // Handle string literals (protect @ inside strings)
        if !in_string && (c == '"' || c == '\'') {
            in_string = true;
            string_char = c;
            content.push(c);
        } else if in_string && c == string_char && (i == 0 || chars[i - 1] != '\\') {
            in_string = false;
            content.push(c);
        } else if in_string {
            content.push(c);
        } else {
            // Not in string, track parens and brackets
            match c {
                '(' => {
                    depth += 1;
                    content.push(c);
                }
                ')' => {
                    depth -= 1;
                    if depth > 0 {
                        content.push(c);
                    }
                }
                '[' => {
                    bracket_depth += 1;
                    content.push(c);
                }
                ']' => {
                    bracket_depth -= 1;
                    content.push(c);
                }
                _ => content.push(c),
            }
        }

        i += 1;
    }

    // Check if we have unbalanced brackets
    if depth != 0 || bracket_depth != 0 {
        return None;
    }

    Some((content, i - 1))
}

/// Parse a single @name(params) annotation
fn parse_annotation(
    name: &str,
    params_str: &str,
    doc_span: &Span,
    id_gen: &mut IDGenerator,
) -> Option<Annotation> {
    let params = parse_params(params_str);

    Some(Annotation {
        name: name.to_string(),
        params,
        span: Span::new(doc_span.start, doc_span.end, id_gen.new_id()),
    })
}

/// Parse key: value pairs from params string
/// Handles nested arrays and properly splits on commas only at depth 0
fn parse_params(params_str: &str) -> Vec<(String, AnnotationValue)> {
    let mut params = Vec::new();
    let trimmed = params_str.trim();

    if trimmed.is_empty() {
        return params;
    }

    // Split by commas at depth 0
    let parts = split_at_depth_zero(trimmed, ',');

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Split key: value
        if let Some(colon_pos) = part.find(':') {
            let key = part[..colon_pos].trim().to_string();
            let value_str = part[colon_pos + 1..].trim();
            let value = parse_value(value_str);
            params.push((key, value));
        }
    }

    params
}

/// Split a string by a delimiter, but only at bracket/paren depth 0
fn split_at_depth_zero(s: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = '"';

    for (i, c) in s.chars().enumerate() {
        // Handle strings
        if !in_string && (c == '"' || c == '\'') {
            in_string = true;
            string_char = c;
            current.push(c);
        } else if in_string && c == string_char && (i == 0 || s.chars().nth(i - 1) != Some('\\')) {
            in_string = false;
            current.push(c);
        } else if in_string {
            current.push(c);
        } else {
            match c {
                '(' | '[' | '{' => {
                    depth += 1;
                    current.push(c);
                }
                ')' | ']' | '}' => {
                    depth -= 1;
                    current.push(c);
                }
                c if c == delimiter && depth == 0 => {
                    parts.push(current.trim().to_string());
                    current = String::new();
                }
                _ => current.push(c),
            }
        }
    }

    if !current.is_empty() {
        parts.push(current.trim().to_string());
    }

    parts
}

/// Parse a value string into AnnotationValue
/// Tries in order: number, boolean, array, then falls back to string
fn parse_value(value_str: &str) -> AnnotationValue {
    let trimmed = value_str.trim();

    // Try boolean
    if trimmed == "true" {
        return AnnotationValue::Boolean(true);
    }
    if trimmed == "false" {
        return AnnotationValue::Boolean(false);
    }

    // Try number (including negative and decimal)
    if let Ok(n) = trimmed.parse::<f64>() {
        return AnnotationValue::Number(n);
    }

    // Try array
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        let items = split_at_depth_zero(inner, ',');
        let array_values: Vec<AnnotationValue> = items
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| parse_value(s))
            .collect();
        return AnnotationValue::Array(array_values);
    }

    // String - remove quotes if present, otherwise treat as bare identifier
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        let unquoted = &trimmed[1..trimmed.len() - 1];
        return AnnotationValue::String(unquoted.to_string());
    }

    // Bare identifier or unquoted string
    AnnotationValue::String(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_span() -> Span {
        Span::new(0, 100, "test".to_string())
    }

    fn make_id_gen() -> IDGenerator {
        IDGenerator::new("test.pc")
    }

    #[test]
    fn test_parse_simple_frame() {
        let content = "/** @frame(x: 100, y: 200) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].name, "frame");
        assert_eq!(result.annotations[0].params.len(), 2);

        // Check x param
        let x_param = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "x")
            .unwrap();
        assert_eq!(x_param.1, AnnotationValue::Number(100.0));

        // Check y param
        let y_param = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "y")
            .unwrap();
        assert_eq!(y_param.1, AnnotationValue::Number(200.0));
    }

    #[test]
    fn test_parse_frame_with_all_params() {
        let content = "/** @frame(x: 100, y: 200, width: 300, height: 400) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].params.len(), 4);
    }

    #[test]
    fn test_parse_mixed_content() {
        let content = "/** This is a description @frame(x: 100, y: 200) more text */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert!(result.description.contains("This is a description"));
        assert!(result.description.contains("more text"));
        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].name, "frame");
    }

    #[test]
    fn test_parse_multiple_annotations() {
        let content = "/** @frame(x: 100, y: 200) @prop(variant: primary) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 2);
        assert_eq!(result.annotations[0].name, "frame");
        assert_eq!(result.annotations[1].name, "prop");
    }

    #[test]
    fn test_parse_annotation_without_params() {
        let content = "/** @deprecated */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].name, "deprecated");
        assert!(result.annotations[0].params.is_empty());
    }

    #[test]
    fn test_parse_string_values() {
        let content = r#"/** @prop(name: "Card", variant: primary) */"#;
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);

        let name_param = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "name")
            .unwrap();
        assert_eq!(name_param.1, AnnotationValue::String("Card".to_string()));

        let variant_param = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "variant")
            .unwrap();
        assert_eq!(
            variant_param.1,
            AnnotationValue::String("primary".to_string())
        );
    }

    #[test]
    fn test_parse_boolean_values() {
        let content = "/** @config(locked: true, visible: false) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);

        let locked = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "locked")
            .unwrap();
        assert_eq!(locked.1, AnnotationValue::Boolean(true));

        let visible = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "visible")
            .unwrap();
        assert_eq!(visible.1, AnnotationValue::Boolean(false));
    }

    #[test]
    fn test_parse_array_values() {
        let content = "/** @data(items: [1, 2, 3]) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);

        let items = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "items")
            .unwrap();

        if let AnnotationValue::Array(arr) = &items.1 {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], AnnotationValue::Number(1.0));
            assert_eq!(arr[1], AnnotationValue::Number(2.0));
            assert_eq!(arr[2], AnnotationValue::Number(3.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_negative_numbers() {
        let content = "/** @frame(x: -50, y: -100.5) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        let x = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "x")
            .unwrap();
        assert_eq!(x.1, AnnotationValue::Number(-50.0));

        let y = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "y")
            .unwrap();
        assert_eq!(y.1, AnnotationValue::Number(-100.5));
    }

    #[test]
    fn test_doc_comment_only_description() {
        let content = "/** Just a description without annotations */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert!(result.annotations.is_empty());
        assert_eq!(
            result.description,
            "Just a description without annotations"
        );
    }

    #[test]
    fn test_multiline_doc_comment() {
        let content = r#"/**
         * This is a card component.
         * @frame(x: 100, y: 200)
         * More description here.
         */"#;
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        assert!(result.description.contains("This is a card component"));
        assert!(result.description.contains("More description here"));
    }

    #[test]
    fn test_at_in_string_not_parsed_as_annotation() {
        let content = r#"/** @prop(email: "foo@bar.com") */"#;
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].name, "prop");

        let email = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "email")
            .unwrap();
        assert_eq!(
            email.1,
            AnnotationValue::String("foo@bar.com".to_string())
        );
    }

    // ==================== Additional Exhaustive Tests ====================

    #[test]
    fn test_empty_doc_comment() {
        let content = "/** */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert!(result.annotations.is_empty());
        assert!(result.description.is_empty());
    }

    #[test]
    fn test_nested_arrays() {
        let content = "/** @data(matrix: [[1, 2], [3, 4]]) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        let matrix = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "matrix")
            .unwrap();

        if let AnnotationValue::Array(outer) = &matrix.1 {
            assert_eq!(outer.len(), 2);
            if let AnnotationValue::Array(inner) = &outer[0] {
                assert_eq!(inner.len(), 2);
                assert_eq!(inner[0], AnnotationValue::Number(1.0));
            } else {
                panic!("Expected nested array");
            }
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_mixed_array_types() {
        let content = r#"/** @config(items: [1, "two", true, 4.5]) */"#;
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        let items = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "items")
            .unwrap();

        if let AnnotationValue::Array(arr) = &items.1 {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], AnnotationValue::Number(1.0));
            assert_eq!(arr[1], AnnotationValue::String("two".to_string()));
            assert_eq!(arr[2], AnnotationValue::Boolean(true));
            assert_eq!(arr[3], AnnotationValue::Number(4.5));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_empty_array() {
        let content = "/** @config(items: []) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        let items = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "items")
            .unwrap();

        if let AnnotationValue::Array(arr) = &items.1 {
            assert!(arr.is_empty());
        } else {
            panic!("Expected empty array");
        }
    }

    #[test]
    fn test_single_quoted_strings() {
        let content = r#"/** @meta(name: 'Card', variant: 'primary') */"#;
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        let name = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "name")
            .unwrap();
        assert_eq!(name.1, AnnotationValue::String("Card".to_string()));
    }

    #[test]
    fn test_decimal_numbers() {
        let content = "/** @frame(x: 10.5, y: -20.75, scale: 0.5) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].params.len(), 3);

        let scale = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "scale")
            .unwrap();
        assert_eq!(scale.1, AnnotationValue::Number(0.5));
    }

    #[test]
    fn test_annotation_at_very_start() {
        let content = "/**@frame(x: 0, y: 0)*/";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].name, "frame");
    }

    #[test]
    fn test_multiple_annotations_same_line() {
        let content = "/** @a(x: 1) @b(y: 2) @c(z: 3) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 3);
        assert_eq!(result.annotations[0].name, "a");
        assert_eq!(result.annotations[1].name, "b");
        assert_eq!(result.annotations[2].name, "c");
    }

    #[test]
    fn test_annotation_with_underscore_name() {
        let content = "/** @my_custom_annotation(foo_bar: baz_qux) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].name, "my_custom_annotation");

        let foo = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "foo_bar")
            .unwrap();
        assert_eq!(foo.1, AnnotationValue::String("baz_qux".to_string()));
    }

    #[test]
    fn test_annotation_with_numeric_suffix() {
        let content = "/** @variant2(enabled: true) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].name, "variant2");
    }

    #[test]
    fn test_description_interspersed_with_annotations() {
        let content = r#"/**
         * Start of description
         * @frame(x: 100, y: 200)
         * Middle of description
         * @meta(category: ui)
         * End of description
         */"#;
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 2);
        assert!(result.description.contains("Start of description"));
        assert!(result.description.contains("Middle of description"));
        assert!(result.description.contains("End of description"));
    }

    #[test]
    fn test_special_characters_in_string_value() {
        let content = r#"/** @meta(regex: "^[a-z]+$", path: "/foo/bar/baz") */"#;
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);

        let regex = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "regex")
            .unwrap();
        assert_eq!(regex.1, AnnotationValue::String("^[a-z]+$".to_string()));

        let path = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "path")
            .unwrap();
        assert_eq!(path.1, AnnotationValue::String("/foo/bar/baz".to_string()));
    }

    #[test]
    fn test_whitespace_variations() {
        let content = "/** @frame(  x:100  ,  y:200  ) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        assert_eq!(result.annotations[0].params.len(), 2);

        let x = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "x")
            .unwrap();
        assert_eq!(x.1, AnnotationValue::Number(100.0));
    }

    #[test]
    fn test_colon_in_string_value() {
        let content = r#"/** @meta(time: "12:30:45", url: "https://example.com") */"#;
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);

        let time = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "time")
            .unwrap();
        assert_eq!(time.1, AnnotationValue::String("12:30:45".to_string()));

        let url = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "url")
            .unwrap();
        assert_eq!(
            url.1,
            AnnotationValue::String("https://example.com".to_string())
        );
    }

    #[test]
    fn test_large_numbers() {
        let content = "/** @frame(x: 999999999, y: -999999999) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);

        let x = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "x")
            .unwrap();
        assert_eq!(x.1, AnnotationValue::Number(999999999.0));
    }

    #[test]
    fn test_boolean_case_sensitivity() {
        let content = "/** @config(a: true, b: false, c: True, d: FALSE) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);

        let a = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "a")
            .unwrap();
        assert_eq!(a.1, AnnotationValue::Boolean(true));

        let b = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "b")
            .unwrap();
        assert_eq!(b.1, AnnotationValue::Boolean(false));

        // "True" and "FALSE" are not boolean literals - they're strings
        let c = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "c")
            .unwrap();
        assert_eq!(c.1, AnnotationValue::String("True".to_string()));

        let d = result.annotations[0]
            .params
            .iter()
            .find(|(k, _)| k == "d")
            .unwrap();
        assert_eq!(d.1, AnnotationValue::String("FALSE".to_string()));
    }

    #[test]
    fn test_only_annotations_no_description() {
        let content = "/** @a(x: 1) @b(y: 2) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 2);
        assert!(result.description.trim().is_empty());
    }

    #[test]
    fn test_trailing_comma_in_params() {
        let content = "/** @frame(x: 100, y: 200,) */";
        let mut id_gen = make_id_gen();
        let result = parse_doc_comment(content, make_test_span(), &mut id_gen);

        assert_eq!(result.annotations.len(), 1);
        // Trailing comma should be handled gracefully (empty part ignored)
        assert_eq!(result.annotations[0].params.len(), 2);
    }
}
