pub mod css_evaluator;
pub mod css_optimizer;
pub mod css_minifier;
pub mod css_splitter;
pub mod css_differ;
pub mod evaluator;
pub mod override_resolution;
pub mod utils;
pub mod validator;
pub mod vdom;
pub mod vdom_differ;

#[cfg(test)]
mod tests_comprehensive;

#[cfg(test)]
mod tests_edge_cases;

#[cfg(test)]
mod tests_cssom;

#[cfg(test)]
mod tests_bundle;

#[cfg(test)]
mod tests_bundle_filesystem;

#[cfg(test)]
mod tests_semantic_id;

#[cfg(test)]
mod tests_semantic_id_diffing;

#[cfg(test)]
mod tests_slots;

#[cfg(test)]
mod tests_error_recovery;

#[cfg(test)]
mod tests_expressions;

#[cfg(test)]
mod tests_integration;

pub use css_evaluator::CssRule;
pub use css_evaluator::{CssError, CssEvaluator, CssResult, VirtualCssDocument};
pub use evaluator::{EvalContext, EvalError, EvalResult, Evaluator, Value};
pub use override_resolution::{OverrideResolver, ResolvedOverride};
pub use validator::{ValidationLevel, ValidationWarning, Validator};
pub use vdom::CssRule as VDomCssRule;
pub use vdom::{VNode, VirtualDomDocument};
pub use vdom_differ::{diff_vdocument, VDocPatch};
