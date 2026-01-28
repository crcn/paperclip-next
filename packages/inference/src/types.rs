use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents an inferred type for Paperclip components
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Unknown type - transient, engine-internal (should not escape inference)
    /// Used during inference passes, should be converted to Any or specific type by end
    Unknown,

    /// Any type - explicitly dynamic/user-controlled
    /// This is the fallback for truly dynamic values
    Any,

    /// String literal or string type
    String,

    /// Number type
    Number,

    /// Boolean type
    Boolean,

    /// Null type
    Null,

    /// Slot/children type (React.ReactNode)
    Slot,

    /// Union of multiple types
    Union(Vec<Type>),

    /// Exact literal type
    Literal(LiteralType),

    /// Array type
    Array(Box<Type>),

    /// Optional type (T | undefined)
    Optional(Box<Type>),

    /// Function/callback type
    Function(FunctionType),

    /// Element reference (for render props)
    Element(ElementType),

    /// Object/map type with known properties
    Object(ObjectType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LiteralType {
    String(String),
    Number(OrderedFloat<f64>),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropertyType {
    pub type_: Type,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementType {
    pub tag: String,
    pub is_component: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectType {
    pub properties: BTreeMap<String, PropertyType>,
    /// Index signature for dynamic properties
    pub index_signature: Option<Box<Type>>,
}

impl Type {
    /// Unify two types, returning the most specific type that accommodates both
    pub fn unify(t1: &Type, t2: &Type) -> Type {
        use Type::*;

        // If types are equal, return either one
        if t1 == t2 {
            return t1.clone();
        }

        // Unknown unifies with anything, taking the other type
        match (t1, t2) {
            (Unknown, other) | (other, Unknown) => return other.clone(),
            _ => {}
        }

        // Any absorbs everything (explicit dynamic)
        match (t1, t2) {
            (Any, _) | (_, Any) => return Any,
            _ => {}
        }

        // Literal to base type unification
        match (t1, t2) {
            (Literal(LiteralType::String(_)), String) | (String, Literal(LiteralType::String(_))) => {
                return String;
            }
            (Literal(LiteralType::Number(_)), Number) | (Number, Literal(LiteralType::Number(_))) => {
                return Number;
            }
            (Literal(LiteralType::Boolean(_)), Boolean) | (Boolean, Literal(LiteralType::Boolean(_))) => {
                return Boolean;
            }
            _ => {}
        }

        // Object unification - merge properties
        match (t1, t2) {
            (Object(obj1), Object(obj2)) => {
                return Object(ObjectType::merge(obj1, obj2));
            }
            _ => {}
        }

        // Array unification
        match (t1, t2) {
            (Array(inner1), Array(inner2)) => {
                return Array(Box::new(Type::unify(inner1, inner2)));
            }
            _ => {}
        }

        // Optional unification
        match (t1, t2) {
            (Optional(inner1), Optional(inner2)) => {
                return Optional(Box::new(Type::unify(inner1, inner2)));
            }
            (Optional(inner), other) | (other, Optional(inner)) => {
                return Optional(Box::new(Type::unify(inner, other)));
            }
            _ => {}
        }

        // Union flattening and merging
        match (t1, t2) {
            (Union(types1), Union(types2)) => {
                let mut merged = types1.clone();
                merged.extend(types2.clone());
                return Union(merged).simplify();
            }
            (Union(types), other) | (other, Union(types)) => {
                let mut merged = types.clone();
                merged.push(other.clone());
                return Union(merged).simplify();
            }
            _ => {}
        }

        // Incompatible types - create union
        Union(vec![t1.clone(), t2.clone()]).simplify()
    }

    /// Simplify a type by removing duplicates and flattening unions
    pub fn simplify(self) -> Type {
        match self {
            Type::Union(types) => {
                let mut simplified = Vec::new();
                let mut seen = std::collections::HashSet::new();

                for t in types {
                    let t = t.simplify();

                    // Flatten nested unions
                    if let Type::Union(inner_types) = t {
                        for inner in inner_types {
                            if seen.insert(inner.clone()) {
                                simplified.push(inner);
                            }
                        }
                    } else {
                        if seen.insert(t.clone()) {
                            simplified.push(t);
                        }
                    }
                }

                // If only one type remains, unwrap the union
                match simplified.len() {
                    0 => Type::Unknown,
                    1 => simplified.into_iter().next().unwrap(),
                    _ => Type::Union(simplified),
                }
            }
            other => other,
        }
    }

    /// Check if this type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::Number | Type::Literal(LiteralType::Number(_))
        )
    }

    /// Check if this type is string-like
    pub fn is_stringlike(&self) -> bool {
        matches!(
            self,
            Type::String | Type::Literal(LiteralType::String(_))
        )
    }

    /// Check if this type is boolean-like
    pub fn is_boolean(&self) -> bool {
        matches!(
            self,
            Type::Boolean | Type::Literal(LiteralType::Boolean(_))
        )
    }

    /// Convert Unknown types to Any (for final output)
    pub fn finalize(self) -> Type {
        match self {
            Type::Unknown => Type::Any,
            Type::Union(types) => {
                Type::Union(types.into_iter().map(|t| t.finalize()).collect()).simplify()
            }
            Type::Array(inner) => Type::Array(Box::new(inner.finalize())),
            Type::Optional(inner) => Type::Optional(Box::new(inner.finalize())),
            Type::Object(obj) => Type::Object(ObjectType {
                properties: obj
                    .properties
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k,
                            PropertyType {
                                type_: v.type_.finalize(),
                                optional: v.optional,
                            },
                        )
                    })
                    .collect(),
                index_signature: obj.index_signature.map(|t| Box::new(t.finalize())),
            }),
            other => other,
        }
    }
}

impl ObjectType {
    /// Merge two object types, combining their properties
    pub fn merge(obj1: &ObjectType, obj2: &ObjectType) -> ObjectType {
        let mut properties = obj1.properties.clone();

        for (key, prop2) in &obj2.properties {
            properties
                .entry(key.clone())
                .and_modify(|prop1| {
                    // Unify property types
                    prop1.type_ = Type::unify(&prop1.type_, &prop2.type_);
                    // Property is optional if either is optional
                    prop1.optional = prop1.optional || prop2.optional;
                })
                .or_insert_with(|| prop2.clone());
        }

        let index_signature = match (&obj1.index_signature, &obj2.index_signature) {
            (Some(t1), Some(t2)) => Some(Box::new(Type::unify(t1, t2))),
            (Some(t), None) | (None, Some(t)) => Some(t.clone()),
            (None, None) => None,
        };

        ObjectType {
            properties,
            index_signature,
        }
    }

    /// Add a property to this object type
    pub fn add_property(&mut self, name: String, type_: Type, optional: bool) {
        self.properties
            .entry(name)
            .and_modify(|prop| {
                prop.type_ = Type::unify(&prop.type_, &type_);
                prop.optional = prop.optional || optional;
            })
            .or_insert_with(|| PropertyType { type_, optional });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_same_types() {
        let t1 = Type::String;
        let t2 = Type::String;
        assert_eq!(Type::unify(&t1, &t2), Type::String);
    }

    #[test]
    fn test_unify_unknown_with_concrete() {
        let t1 = Type::Unknown;
        let t2 = Type::Number;
        assert_eq!(Type::unify(&t1, &t2), Type::Number);
        assert_eq!(Type::unify(&t2, &t1), Type::Number);
    }

    #[test]
    fn test_unify_any_absorbs() {
        let t1 = Type::Any;
        let t2 = Type::String;
        assert_eq!(Type::unify(&t1, &t2), Type::Any);
    }

    #[test]
    fn test_unify_literal_to_base() {
        let t1 = Type::Literal(LiteralType::String("hello".to_string()));
        let t2 = Type::String;
        assert_eq!(Type::unify(&t1, &t2), Type::String);
    }

    #[test]
    fn test_unify_incompatible_creates_union() {
        let t1 = Type::String;
        let t2 = Type::Number;
        let result = Type::unify(&t1, &t2);
        assert!(matches!(result, Type::Union(_)));
    }

    #[test]
    fn test_simplify_removes_duplicates() {
        let union = Type::Union(vec![Type::String, Type::String, Type::Number]);
        let simplified = union.simplify();
        if let Type::Union(types) = simplified {
            assert_eq!(types.len(), 2);
        } else {
            panic!("Expected union");
        }
    }

    #[test]
    fn test_object_merge() {
        let mut obj1 = ObjectType {
            properties: BTreeMap::new(),
            index_signature: None,
        };
        obj1.add_property("name".to_string(), Type::String, false);

        let mut obj2 = ObjectType {
            properties: BTreeMap::new(),
            index_signature: None,
        };
        obj2.add_property("age".to_string(), Type::Number, false);

        let merged = ObjectType::merge(&obj1, &obj2);
        assert_eq!(merged.properties.len(), 2);
        assert!(merged.properties.contains_key("name"));
        assert!(merged.properties.contains_key("age"));
    }
}
