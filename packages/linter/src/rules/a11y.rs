use crate::diagnostic::{Diagnostic, DiagnosticLevel};
use paperclip_parser::ast::{Element, Expression};
use std::collections::HashMap;

/// Accessibility lint rules
pub struct A11yRule;

impl A11yRule {
    pub fn check_element(element: &Element) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        match element {
            Element::Tag {
                tag_name,
                attributes,
                children,
                span,
                ..
            } => {
                let tag_lower = tag_name.to_lowercase();

                // Check images for alt text
                if tag_lower == "img" {
                    if !has_attribute(attributes, "alt")
                        && !has_attribute(attributes, "aria-label")
                        && !has_attribute(attributes, "aria-labelledby")
                        && !has_attribute(attributes, "role")
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                "a11y-img-alt",
                                "Images must have alternative text for screen readers",
                                span.clone(),
                            )
                            .with_suggestion(
                                "Add an 'alt' attribute describing the image content, or use 'aria-label' if appropriate",
                            ),
                        );
                    }
                }

                // Check buttons for accessible text
                if tag_lower == "button" {
                    if !has_text_content(children)
                        && !has_attribute(attributes, "aria-label")
                        && !has_attribute(attributes, "aria-labelledby")
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                "a11y-button-text",
                                "Buttons must have accessible text content",
                                span.clone(),
                            )
                            .with_suggestion(
                                "Add text content inside the button, or use 'aria-label' to provide a label",
                            ),
                        );
                    }
                }

                // Check links for accessible text
                if tag_lower == "a" {
                    if !has_text_content(children)
                        && !has_attribute(attributes, "aria-label")
                        && !has_attribute(attributes, "aria-labelledby")
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                "a11y-link-text",
                                "Links must have accessible text content",
                                span.clone(),
                            )
                            .with_suggestion(
                                "Add text content inside the link, or use 'aria-label' to provide a label",
                            ),
                        );
                    }
                }

                // Check form inputs for labels
                if matches!(tag_lower.as_str(), "input" | "select" | "textarea") {
                    let input_type = get_attribute_value(attributes, "type")
                        .unwrap_or_else(|| "text".to_string());

                    // Skip hidden inputs
                    if input_type != "hidden" {
                        if !has_attribute(attributes, "aria-label")
                            && !has_attribute(attributes, "aria-labelledby")
                            && !has_attribute(attributes, "id")
                        {
                            diagnostics.push(
                                Diagnostic::warning(
                                    "a11y-input-label",
                                    format!(
                                        "Form {} elements should have associated labels",
                                        tag_lower
                                    ),
                                    span.clone(),
                                )
                                .with_suggestion(
                                    "Add an 'id' attribute to associate with a <label>, or use 'aria-label' for the input",
                                ),
                            );
                        }
                    }
                }

                // Check for invalid ARIA attributes
                if let Some(role) = get_attribute_value(attributes, "role") {
                    if !is_valid_aria_role(&role) {
                        diagnostics.push(
                            Diagnostic::error(
                                "a11y-invalid-aria-role",
                                format!("Invalid ARIA role: '{}'", role),
                                span.clone(),
                            )
                            .with_suggestion(
                                "Use a valid ARIA role such as: button, link, navigation, main, complementary, banner, contentinfo, etc.",
                            ),
                        );
                    }
                }

                // Check for semantic HTML - warn about using div/span for interactive elements
                if (tag_lower == "div" || tag_lower == "span")
                    && (has_attribute(attributes, "onclick")
                        || has_attribute(attributes, "onkeydown")
                        || has_attribute(attributes, "onkeyup"))
                {
                    diagnostics.push(
                        Diagnostic::warning(
                            "a11y-semantic-html",
                            format!(
                                "Use semantic HTML elements instead of <{}> for interactive content",
                                tag_lower
                            ),
                            span.clone(),
                        )
                        .with_suggestion(
                            "Consider using <button>, <a>, or add proper ARIA role and keyboard support",
                        ),
                    );
                }

                // Check heading hierarchy (basic check)
                if let Some(level) = get_heading_level(&tag_lower) {
                    // This is a simplified check - a full implementation would track heading hierarchy across the document
                    if level > 1 {
                        diagnostics.push(
                            Diagnostic {
                                level: DiagnosticLevel::Info,
                                rule: "a11y-heading-hierarchy".to_string(),
                                message: format!(
                                    "Ensure heading hierarchy is correct (h{} should follow appropriate parent heading)",
                                    level
                                ),
                                span: span.clone(),
                                suggestion: Some("Headings should follow sequential order (h1 -> h2 -> h3) without skipping levels".to_string()),
                            },
                        );
                    }
                }

                // Note: Children are checked by the linter engine's recursion
            }
            Element::Instance { .. }
            | Element::Conditional { .. }
            | Element::Repeat { .. }
            | Element::Insert { .. }
            | Element::Text { .. }
            | Element::SlotInsert { .. } => {
                // No element-specific checks needed
                // Children are checked by the linter engine's recursion
            }
        }

        diagnostics
    }
}

/// Check if an element has a specific attribute
fn has_attribute(attributes: &HashMap<String, Expression>, name: &str) -> bool {
    attributes.contains_key(name)
}

/// Get the string value of an attribute if it's a literal
fn get_attribute_value(attributes: &HashMap<String, Expression>, name: &str) -> Option<String> {
    attributes.get(name).and_then(|expr| match expr {
        Expression::Literal { value, .. } => Some(value.clone()),
        _ => None,
    })
}

/// Check if children contain text content
fn has_text_content(children: &[Element]) -> bool {
    children
        .iter()
        .any(|child| matches!(child, Element::Text { .. }))
}

/// Check if a role is a valid ARIA role
fn is_valid_aria_role(role: &str) -> bool {
    matches!(
        role,
        // Document structure roles
        "article"
            | "complementary"
            | "contentinfo"
            | "definition"
            | "directory"
            | "document"
            | "feed"
            | "figure"
            | "group"
            | "heading"
            | "img"
            | "list"
            | "listitem"
            | "main"
            | "math"
            | "navigation"
            | "none"
            | "note"
            | "presentation"
            | "region"
            | "separator"
            | "table"
            | "toolbar"
            // Widget roles
            | "alert"
            | "alertdialog"
            | "button"
            | "checkbox"
            | "dialog"
            | "gridcell"
            | "link"
            | "log"
            | "marquee"
            | "menuitem"
            | "menuitemcheckbox"
            | "menuitemradio"
            | "option"
            | "progressbar"
            | "radio"
            | "scrollbar"
            | "searchbox"
            | "slider"
            | "spinbutton"
            | "status"
            | "switch"
            | "tab"
            | "tabpanel"
            | "textbox"
            | "timer"
            | "tooltip"
            | "treeitem"
            // Landmark roles
            | "banner"
            | "form"
            | "search"
            // Live region roles
            | "application"
            | "atomic"
            | "busy"
            | "live"
            | "relevant"
            // Composite roles
            | "combobox"
            | "grid"
            | "listbox"
            | "menu"
            | "menubar"
            | "radiogroup"
            | "tablist"
            | "tree"
            | "treegrid"
    )
}

/// Get heading level from tag name (h1-h6)
fn get_heading_level(tag: &str) -> Option<u8> {
    match tag {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::ast::Span;

    #[test]
    fn test_img_without_alt() {
        let element = Element::Tag {
            tag_name: "img".to_string(),
            name: None,
            attributes: HashMap::new(),
            styles: vec![],
            children: vec![],
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = A11yRule::check_element(&element);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "a11y-img-alt");
    }

    #[test]
    fn test_img_with_alt() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "alt".to_string(),
            Expression::Literal {
                value: "Description".to_string(),
                span: Span::new(0, 10, "test".to_string()),
            },
        );

        let element = Element::Tag {
            tag_name: "img".to_string(),
            name: None,
            attributes,
            styles: vec![],
            children: vec![],
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = A11yRule::check_element(&element);
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_button_without_text() {
        let element = Element::Tag {
            tag_name: "button".to_string(),
            name: None,
            attributes: HashMap::new(),
            styles: vec![],
            children: vec![],
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = A11yRule::check_element(&element);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "a11y-button-text");
    }

    #[test]
    fn test_button_with_text() {
        let element = Element::Tag {
            tag_name: "button".to_string(),
            name: None,
            attributes: HashMap::new(),
            styles: vec![],
            children: vec![Element::Text {
                content: Expression::Literal {
                    value: "Click me".to_string(),
                    span: Span::new(0, 10, "test".to_string()),
                },
                styles: vec![],
                span: Span::new(0, 10, "test".to_string()),
            }],
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = A11yRule::check_element(&element);
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_invalid_aria_role() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "role".to_string(),
            Expression::Literal {
                value: "invalid-role".to_string(),
                span: Span::new(0, 10, "test".to_string()),
            },
        );

        let element = Element::Tag {
            tag_name: "div".to_string(),
            name: None,
            attributes,
            styles: vec![],
            children: vec![],
            span: Span::new(0, 10, "test".to_string()),
        };

        let diagnostics = A11yRule::check_element(&element);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "a11y-invalid-aria-role");
    }
}
