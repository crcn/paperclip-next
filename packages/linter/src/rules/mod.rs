mod a11y;
mod no_important;
mod no_negative_spacing;
mod no_viewport_units;

pub use a11y::A11yRule;
pub use no_important::NoImportantRule;
pub use no_negative_spacing::NoNegativeSpacingRule;
pub use no_viewport_units::NoViewportUnitsRule;

use crate::diagnostic::Diagnostic;
use paperclip_parser::ast::{StyleBlock, StyleDecl};

/// Trait for implementing lint rules
pub trait LintRule {
    /// Unique identifier for this rule
    fn name(&self) -> &'static str;

    /// Human-readable description
    fn description(&self) -> &'static str;

    /// Check a style declaration (reusable style mixin)
    fn check_style_decl(&self, _style: &StyleDecl) -> Vec<Diagnostic> {
        Vec::new()
    }

    /// Check a style block (inline styles on elements)
    fn check_style_block(&self, _style: &StyleBlock) -> Vec<Diagnostic> {
        Vec::new()
    }
}

/// Registry of all available lint rules
pub struct RuleRegistry {
    rules: Vec<Box<dyn LintRule>>,
}

impl RuleRegistry {
    /// Create a new registry with all built-in rules
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(NoImportantRule),
                Box::new(NoViewportUnitsRule),
                Box::new(NoNegativeSpacingRule),
            ],
        }
    }

    /// Get all registered rules
    pub fn rules(&self) -> &[Box<dyn LintRule>] {
        &self.rules
    }

    /// Create an empty registry
    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a custom rule to the registry
    pub fn add_rule(&mut self, rule: Box<dyn LintRule>) {
        self.rules.push(rule);
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for RuleRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuleRegistry")
            .field("rules", &format!("{} rules", self.rules.len()))
            .finish()
    }
}
