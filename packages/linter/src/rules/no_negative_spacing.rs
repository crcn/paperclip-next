use crate::diagnostic::Diagnostic;
use crate::rules::LintRule;
use paperclip_parser::ast::{StyleBlock, StyleDecl};
use regex::Regex;

/// Lint rule that prevents negative margins and padding
pub struct NoNegativeSpacingRule;

impl LintRule for NoNegativeSpacingRule {
    fn name(&self) -> &'static str {
        "no-negative-spacing"
    }

    fn description(&self) -> &'static str {
        "Disallow negative values for margin and padding properties"
    }

    fn check_style_decl(&self, style: &StyleDecl) -> Vec<Diagnostic> {
        check_properties(&style.properties, &style.span)
    }

    fn check_style_block(&self, style: &StyleBlock) -> Vec<Diagnostic> {
        check_properties(&style.properties, &style.span)
    }
}

fn check_properties(
    properties: &std::collections::HashMap<String, String>,
    span: &paperclip_parser::ast::Span,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Regex to match negative values (e.g., -10px, -1rem, -5%)
    let re = Regex::new(r"-\d+(\.\d+)?(%|px|rem|em|vh|vw|pt|pc|in|cm|mm|ex|ch|vmin|vmax)\b")
        .unwrap();

    // Properties to check for negative values
    let spacing_properties = [
        "margin",
        "margin-top",
        "margin-right",
        "margin-bottom",
        "margin-left",
        "padding",
        "padding-top",
        "padding-right",
        "padding-bottom",
        "padding-left",
    ];

    for (property, value) in properties {
        let property_lower = property.to_lowercase();

        // Check if this is a spacing property
        if spacing_properties.contains(&property_lower.as_str()) {
            if re.is_match(value) {
                let is_margin = property_lower.starts_with("margin");
                let property_type = if is_margin { "margin" } else { "padding" };

                diagnostics.push(
                    Diagnostic::error(
                        "no-negative-spacing",
                        format!(
                            "Avoid using negative values for '{}'. Negative {} can cause unexpected layout behavior.",
                            property, property_type
                        ),
                        span.clone(),
                    )
                    .with_suggestion(format!(
                        "Consider restructuring your layout instead of using negative {} in '{}'",
                        property_type, property
                    )),
                );
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::ast::Span;
    use std::collections::HashMap;

    #[test]
    fn test_detects_negative_margin() {
        let rule = NoNegativeSpacingRule;
        let mut properties = HashMap::new();
        properties.insert("margin-top".to_string(), "-10px".to_string());

        let style = StyleDecl {
            public: false,
            name: "test".to_string(),
            extends: vec![],
            properties,
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = rule.check_style_decl(&style);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "no-negative-spacing");
        assert!(diagnostics[0].message.contains("margin"));
    }

    #[test]
    fn test_detects_negative_padding() {
        let rule = NoNegativeSpacingRule;
        let mut properties = HashMap::new();
        properties.insert("padding".to_string(), "-5px".to_string());

        let style = StyleDecl {
            public: false,
            name: "test".to_string(),
            extends: vec![],
            properties,
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = rule.check_style_decl(&style);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "no-negative-spacing");
        assert!(diagnostics[0].message.contains("padding"));
    }

    #[test]
    fn test_allows_positive_spacing() {
        let rule = NoNegativeSpacingRule;
        let mut properties = HashMap::new();
        properties.insert("margin".to_string(), "10px".to_string());
        properties.insert("padding".to_string(), "20px".to_string());

        let style = StyleDecl {
            public: false,
            name: "test".to_string(),
            extends: vec![],
            properties,
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = rule.check_style_decl(&style);
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_allows_negative_in_non_spacing_properties() {
        let rule = NoNegativeSpacingRule;
        let mut properties = HashMap::new();
        properties.insert("top".to_string(), "-10px".to_string());
        properties.insert("left".to_string(), "-5px".to_string());

        let style = StyleDecl {
            public: false,
            name: "test".to_string(),
            extends: vec![],
            properties,
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = rule.check_style_decl(&style);
        assert_eq!(diagnostics.len(), 0);
    }
}
