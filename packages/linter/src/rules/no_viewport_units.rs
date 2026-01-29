use crate::diagnostic::Diagnostic;
use crate::rules::LintRule;
use paperclip_parser::ast::{StyleBlock, StyleDecl};
use regex::Regex;

/// Lint rule that prevents use of vw and vh units
pub struct NoViewportUnitsRule;

impl LintRule for NoViewportUnitsRule {
    fn name(&self) -> &'static str {
        "no-viewport-units"
    }

    fn description(&self) -> &'static str {
        "Disallow vw and vh units in CSS"
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

    // Regex to match vw or vh units (with word boundaries to avoid false matches)
    let re = Regex::new(r"\b\d+(\.\d+)?(vw|vh)\b").unwrap();

    for (property, value) in properties {
        if let Some(matched) = re.find(value) {
            let unit = if matched.as_str().contains("vw") {
                "vw"
            } else {
                "vh"
            };

            diagnostics.push(
                Diagnostic::error(
                    "no-viewport-units",
                    format!(
                        "Avoid using {} units in '{}'. Viewport units can cause layout issues on mobile devices.",
                        unit, property
                    ),
                    span.clone(),
                )
                .with_suggestion(format!(
                    "Consider using percentages, rem, or em units instead of {} in '{}'",
                    unit, property
                )),
            );
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
    fn test_detects_vw_units() {
        let rule = NoViewportUnitsRule;
        let mut properties = HashMap::new();
        properties.insert("width".to_string(), "50vw".to_string());

        let style = StyleDecl {
            public: false,
            name: "test".to_string(),
            extends: vec![],
            properties,
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = rule.check_style_decl(&style);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "no-viewport-units");
        assert!(diagnostics[0].message.contains("vw"));
    }

    #[test]
    fn test_detects_vh_units() {
        let rule = NoViewportUnitsRule;
        let mut properties = HashMap::new();
        properties.insert("height".to_string(), "100vh".to_string());

        let style = StyleDecl {
            public: false,
            name: "test".to_string(),
            extends: vec![],
            properties,
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = rule.check_style_decl(&style);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "no-viewport-units");
        assert!(diagnostics[0].message.contains("vh"));
    }

    #[test]
    fn test_allows_other_units() {
        let rule = NoViewportUnitsRule;
        let mut properties = HashMap::new();
        properties.insert("width".to_string(), "50%".to_string());
        properties.insert("height".to_string(), "100px".to_string());
        properties.insert("font-size".to_string(), "1.5rem".to_string());

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
