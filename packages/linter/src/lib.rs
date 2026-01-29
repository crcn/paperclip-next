mod diagnostic;
mod linter;
mod rules;

pub use diagnostic::{Diagnostic, DiagnosticLevel};
pub use linter::{lint_document, LintOptions};
pub use rules::{LintRule, RuleRegistry};
