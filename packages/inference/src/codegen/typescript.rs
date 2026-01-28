use crate::codegen::CodeGenerator;
use crate::types::{ElementType, FunctionType, LiteralType, ObjectType, PropertyType, Type};

/// TypeScript code generator for inferred types
pub struct TypeScriptGenerator {
    /// Configuration options (reserved for future use)
    _config: (),
}

impl TypeScriptGenerator {
    pub fn new() -> Self {
        Self { _config: () }
    }

    /// Generate TypeScript type for an object
    fn generate_object_type(&self, obj: &ObjectType) -> String {
        let mut parts = vec!["{".to_string()];

        // Generate properties
        for (key, prop) in &obj.properties {
            let optional_marker = if prop.optional { "?" } else { "" };
            parts.push(format!(
                "  \"{}\"{}: {};",
                key,
                optional_marker,
                self.generate_type(&prop.type_)
            ));
        }

        // Generate index signature if present
        if let Some(index_type) = &obj.index_signature {
            parts.push(format!(
                "  [key: string]: {};",
                self.generate_type(index_type)
            ));
        }

        parts.push("}".to_string());
        parts.join("\n")
    }

    /// Generate TypeScript type for a function
    fn generate_function_type(&self, func: &FunctionType) -> String {
        let params: Vec<String> = func
            .params
            .iter()
            .enumerate()
            .map(|(i, t)| format!("arg{}: {}", i, self.generate_type(t)))
            .collect();

        format!(
            "({}) => {}",
            params.join(", "),
            self.generate_type(&func.return_type)
        )
    }

    /// Generate TypeScript type for an element
    fn generate_element_type(&self, elem: &ElementType) -> String {
        if elem.is_component {
            format!("React.ComponentProps<typeof {}>", elem.tag)
        } else {
            "React.HTMLAttributes<HTMLElement>".to_string()
        }
    }
}

impl Default for TypeScriptGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for TypeScriptGenerator {
    fn generate_type(&self, type_: &Type) -> String {
        match type_ {
            Type::Unknown => {
                // Unknown should not appear in final output
                // This is a safety fallback
                "any /* WARN: Unknown type */".to_string()
            }
            Type::Any => "any".to_string(),
            Type::String => "string".to_string(),
            Type::Number => "number".to_string(),
            Type::Boolean => "boolean".to_string(),
            Type::Null => "null".to_string(),
            Type::Slot => "React.ReactNode".to_string(),

            Type::Union(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.generate_type(t)).collect();
                type_strs.join(" | ")
            }

            Type::Literal(lit) => match lit {
                LiteralType::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
                LiteralType::Number(n) => n.to_string(),
                LiteralType::Boolean(b) => b.to_string(),
            },

            Type::Array(inner) => format!("Array<{}>", self.generate_type(inner)),

            Type::Optional(inner) => format!("{} | undefined", self.generate_type(inner)),

            Type::Function(func) => self.generate_function_type(func),

            Type::Element(elem) => self.generate_element_type(elem),

            Type::Object(obj) => self.generate_object_type(obj),
        }
    }

    fn generate_property(&self, name: &str, prop: &PropertyType) -> String {
        let optional_marker = if prop.optional { "?" } else { "" };
        format!("{}{}: {}", name, optional_marker, self.generate_type(&prop.type_))
    }

    fn generate_interface(&self, name: &str, props: &[(String, PropertyType)]) -> String {
        let mut lines = vec![format!("export interface {} {{", name)];

        for (prop_name, prop_type) in props {
            lines.push(format!("  {};", self.generate_property(prop_name, prop_type)));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_generate_primitive_types() {
        let gen = TypeScriptGenerator::new();

        assert_eq!(gen.generate_type(&Type::String), "string");
        assert_eq!(gen.generate_type(&Type::Number), "number");
        assert_eq!(gen.generate_type(&Type::Boolean), "boolean");
        assert_eq!(gen.generate_type(&Type::Any), "any");
        assert_eq!(gen.generate_type(&Type::Null), "null");
    }

    #[test]
    fn test_generate_literal_types() {
        let gen = TypeScriptGenerator::new();

        assert_eq!(
            gen.generate_type(&Type::Literal(LiteralType::String("hello".to_string()))),
            "\"hello\""
        );

        assert_eq!(
            gen.generate_type(&Type::Literal(LiteralType::Number(42.0.into()))),
            "42"
        );

        assert_eq!(
            gen.generate_type(&Type::Literal(LiteralType::Boolean(true))),
            "true"
        );
    }

    #[test]
    fn test_generate_array_type() {
        let gen = TypeScriptGenerator::new();

        let array_type = Type::Array(Box::new(Type::String));
        assert_eq!(gen.generate_type(&array_type), "Array<string>");
    }

    #[test]
    fn test_generate_union_type() {
        let gen = TypeScriptGenerator::new();

        let union_type = Type::Union(vec![Type::String, Type::Number]);
        assert_eq!(gen.generate_type(&union_type), "string | number");
    }

    #[test]
    fn test_generate_optional_type() {
        let gen = TypeScriptGenerator::new();

        let optional_type = Type::Optional(Box::new(Type::String));
        assert_eq!(gen.generate_type(&optional_type), "string | undefined");
    }

    #[test]
    fn test_generate_object_type() {
        let gen = TypeScriptGenerator::new();

        let mut properties = BTreeMap::new();
        properties.insert(
            "name".to_string(),
            PropertyType {
                type_: Type::String,
                optional: false,
            },
        );
        properties.insert(
            "age".to_string(),
            PropertyType {
                type_: Type::Number,
                optional: true,
            },
        );

        let obj_type = Type::Object(ObjectType {
            properties,
            index_signature: None,
        });

        let result = gen.generate_type(&obj_type);
        assert!(result.contains("\"name\": string"));
        assert!(result.contains("\"age\"?: number"));
    }

    #[test]
    fn test_generate_property() {
        let gen = TypeScriptGenerator::new();

        let prop = PropertyType {
            type_: Type::String,
            optional: false,
        };
        assert_eq!(gen.generate_property("name", &prop), "name: string");

        let optional_prop = PropertyType {
            type_: Type::Number,
            optional: true,
        };
        assert_eq!(
            gen.generate_property("age", &optional_prop),
            "age?: number"
        );
    }

    #[test]
    fn test_generate_interface() {
        let gen = TypeScriptGenerator::new();

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

        let interface = gen.generate_interface("Person", &props);

        assert!(interface.contains("export interface Person {"));
        assert!(interface.contains("name: string;"));
        assert!(interface.contains("age?: number;"));
        assert!(interface.ends_with("}"));
    }

    #[test]
    fn test_generate_slot_type() {
        let gen = TypeScriptGenerator::new();

        assert_eq!(gen.generate_type(&Type::Slot), "React.ReactNode");
    }
}
