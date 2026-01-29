use crate::diagnostic::Diagnostic;
use crate::rules::{A11yRule, RuleRegistry};
use paperclip_parser::ast::{Component, Document, Element};

/// Options for configuring the linter
#[derive(Debug)]
pub struct LintOptions {
    /// Custom rule registry (uses default if None)
    pub registry: Option<RuleRegistry>,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self { registry: None }
    }
}

/// Lint a Paperclip document and return diagnostics
pub fn lint_document(document: &Document, options: LintOptions) -> Vec<Diagnostic> {
    let registry = options.registry.unwrap_or_default();
    let mut diagnostics = Vec::new();

    // Check all style declarations
    for style_decl in &document.styles {
        for rule in registry.rules() {
            diagnostics.extend(rule.check_style_decl(style_decl));
        }
    }

    // Check all components
    for component in &document.components {
        diagnostics.extend(lint_component(component, &registry));
    }

    diagnostics
}

/// Lint a component and its render tree
fn lint_component(component: &Component, registry: &RuleRegistry) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    if let Some(body) = &component.body {
        diagnostics.extend(lint_element(body, registry));
    }

    diagnostics
}

/// Recursively lint an element and its children
fn lint_element(element: &Element, registry: &RuleRegistry) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Run a11y checks on this element
    diagnostics.extend(A11yRule::check_element(element));

    match element {
        Element::Tag {
            styles, children, ..
        } => {
            // Check all style blocks on this element
            for style_block in styles {
                for rule in registry.rules() {
                    diagnostics.extend(rule.check_style_block(style_block));
                }
            }

            // Recursively check children
            for child in children {
                diagnostics.extend(lint_element(child, registry));
            }
        }
        Element::Instance { children, .. } => {
            // Recursively check children
            for child in children {
                diagnostics.extend(lint_element(child, registry));
            }
        }
        Element::Conditional {
            then_branch,
            else_branch,
            ..
        } => {
            // Check both branches
            for element in then_branch {
                diagnostics.extend(lint_element(element, registry));
            }
            if let Some(else_elements) = else_branch {
                for element in else_elements {
                    diagnostics.extend(lint_element(element, registry));
                }
            }
        }
        Element::Repeat { body, .. } => {
            // Check repeated elements
            for element in body {
                diagnostics.extend(lint_element(element, registry));
            }
        }
        Element::Insert { content, .. } => {
            // Check slot content
            for element in content {
                diagnostics.extend(lint_element(element, registry));
            }
        }
        Element::Text { .. } | Element::SlotInsert { .. } => {
            // No styles to check
        }
    }

    diagnostics
}
