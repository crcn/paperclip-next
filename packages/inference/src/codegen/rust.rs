use crate::codegen::CodeGenerator;
use crate::types::{LiteralType, PropertyType, Type};

/// Rust code generator for inferred types (stub implementation)
/// This is a starting point for future Rust target support
pub struct RustGenerator {
    /// Configuration options (reserved for future use)
    _config: (),
}

impl RustGenerator {
    pub fn new() -> Self {
        Self { _config: () }
    }
}

impl Default for RustGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for RustGenerator {
    fn generate_type(&self, type_: &Type) -> String {
        match type_ {
            Type::Unknown | Type::Any => {
                // Fallback to serde_json::Value for dynamic types
                "serde_json::Value".to_string()
            }
            Type::String => "String".to_string(),
            Type::Number => "f64".to_string(),
            Type::Boolean => "bool".to_string(),
            Type::Null => "()".to_string(),

            Type::Array(inner) => {
                format!("Vec<{}>", self.generate_type(inner))
            }

            Type::Optional(inner) => {
                format!("Option<{}>", self.generate_type(inner))
            }

            Type::Literal(lit) => match lit {
                // For literal types in Rust, we just use the base type
                // Refinement types would require a different approach
                LiteralType::String(_) => "String".to_string(),
                LiteralType::Number(_) => "f64".to_string(),
                LiteralType::Boolean(_) => "bool".to_string(),
            },

            // TODO: Implement remaining types
            Type::Slot => {
                // Slots don't have a direct Rust equivalent yet
                // Could be Box<dyn Render> or similar in the future
                "serde_json::Value /* Slot */".to_string()
            }

            Type::Union(_) => {
                // Rust doesn't have union types
                // Could use an enum or serde_json::Value
                "serde_json::Value /* Union */".to_string()
            }

            Type::Function(_) => {
                // Function types would need trait bounds or Box<dyn Fn>
                "serde_json::Value /* Function */".to_string()
            }

            Type::Element(_) => {
                // Element types don't have a direct Rust equivalent
                "serde_json::Value /* Element */".to_string()
            }

            Type::Object(_) => {
                // Could generate a struct, but for now use serde_json::Value
                "serde_json::Value /* Object */".to_string()
            }
        }
    }

    fn generate_property(&self, name: &str, prop: &PropertyType) -> String {
        let rust_type = if prop.optional {
            format!("Option<{}>", self.generate_type(&prop.type_))
        } else {
            self.generate_type(&prop.type_)
        };

        // Add serde attribute for optional fields
        if prop.optional {
            format!(
                "#[serde(skip_serializing_if = \"Option::is_none\")]\n    pub {}: {}",
                name, rust_type
            )
        } else {
            format!("pub {}: {}", name, rust_type)
        }
    }

    fn generate_interface(&self, name: &str, props: &[(String, PropertyType)]) -> String {
        let mut lines = vec![
            "#[derive(Serialize, Deserialize, Debug, Clone)]".to_string(),
            format!("pub struct {} {{", name),
        ];

        for (prop_name, prop_type) in props {
            lines.push(format!("    {},", self.generate_property(prop_name, prop_type)));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_primitive_types() {
        let gen = RustGenerator::new();

        assert_eq!(gen.generate_type(&Type::String), "String");
        assert_eq!(gen.generate_type(&Type::Number), "f64");
        assert_eq!(gen.generate_type(&Type::Boolean), "bool");
        assert_eq!(gen.generate_type(&Type::Null), "()");
    }

    #[test]
    fn test_generate_array_type() {
        let gen = RustGenerator::new();

        let array_type = Type::Array(Box::new(Type::String));
        assert_eq!(gen.generate_type(&array_type), "Vec<String>");
    }

    #[test]
    fn test_generate_optional_type() {
        let gen = RustGenerator::new();

        let optional_type = Type::Optional(Box::new(Type::String));
        assert_eq!(gen.generate_type(&optional_type), "Option<String>");
    }

    #[test]
    fn test_generate_property() {
        let gen = RustGenerator::new();

        let prop = PropertyType {
            type_: Type::String,
            optional: false,
        };
        assert_eq!(gen.generate_property("name", &prop), "pub name: String");

        let optional_prop = PropertyType {
            type_: Type::Number,
            optional: true,
        };
        let result = gen.generate_property("age", &optional_prop);
        assert!(result.contains("pub age: Option<f64>"));
        assert!(result.contains("#[serde(skip_serializing_if"));
    }

    #[test]
    fn test_generate_interface() {
        let gen = RustGenerator::new();

        let props = vec![
            (
                "name".to_string(),
                PropertyType {
                    type_: Type::String,
                    optional: false,
                },
            ),
            (
                "age".to_string(),
                PropertyType {
                    type_: Type::Number,
                    optional: true,
                },
            ),
        ];

        let struct_def = gen.generate_interface("Person", &props);

        assert!(struct_def.contains("#[derive(Serialize, Deserialize, Debug, Clone)]"));
        assert!(struct_def.contains("pub struct Person {"));
        assert!(struct_def.contains("pub name: String"));
        assert!(struct_def.contains("pub age: Option<f64>"));
    }

    #[test]
    fn test_generate_any_fallback() {
        let gen = RustGenerator::new();

        assert_eq!(gen.generate_type(&Type::Any), "serde_json::Value");
        assert_eq!(gen.generate_type(&Type::Unknown), "serde_json::Value");
    }
}
