use crate::diagnostic::Diagnostic;
use crate::rules::LintRule;
use paperclip_parser::ast::{StyleBlock, StyleDecl};

/// Lint rule that prevents use of !important in CSS
pub struct NoImportantRule;

impl LintRule for NoImportantRule {
    fn name(&self) -> &'static str {
        "no-important"
    }

    fn description(&self) -> &'static str {
        "Disallow !important in CSS declarations"
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

    for (property, value) in properties {
        if value.contains("!important") {
            diagnostics.push(
                Diagnostic::error(
                    "no-important",
                    format!(
                        "Avoid using !important in '{}'. It makes styles harder to override and maintain.",
                        property
                    ),
                    span.clone(),
                )
                .with_suggestion(format!(
                    "Remove !important from '{}' and use more specific selectors or adjust specificity instead",
                    property
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
    fn test_detects_important_in_style_decl() {
        let rule = NoImportantRule;
        let mut properties = HashMap::new();
        properties.insert("color".to_string(), "red !important".to_string());

        let style = StyleDecl {
            public: false,
            name: "test".to_string(),
            extends: vec![],
            properties,
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = rule.check_style_decl(&style);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "no-important");
    }

    #[test]
    fn test_allows_normal_styles() {
        let rule = NoImportantRule;
        let mut properties = HashMap::new();
        properties.insert("color".to_string(), "red".to_string());

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
