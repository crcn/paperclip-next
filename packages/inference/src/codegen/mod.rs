pub mod rust;
pub mod typescript;

use crate::types::{PropertyType, Type};

/// Plugin trait for generating code from inferred types
/// Implementations can target different languages (TypeScript, Rust, etc.)
pub trait CodeGenerator {
    /// Generate code for a single type
    fn generate_type(&self, type_: &Type) -> String;

    /// Generate code for a single property (name + type + optional marker)
    fn generate_property(&self, name: &str, prop: &PropertyType) -> String;

    /// Generate a complete interface/struct definition
    fn generate_interface(&self, name: &str, props: &[(String, PropertyType)]) -> String;
}
