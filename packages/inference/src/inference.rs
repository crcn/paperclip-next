use crate::error::InferenceResult;
use crate::options::InferenceOptions;
use crate::scope::Scope;
use crate::types::{LiteralType, ObjectType, PropertyType, Type};
use paperclip_parser::ast::*;
use std::collections::BTreeMap;
use std::rc::Rc;

/// Multi-pass type inference engine for Paperclip components
pub struct InferenceEngine {
    options: InferenceOptions,
}

impl InferenceEngine {
    /// Create a new inference engine with the given options
    pub fn new(options: InferenceOptions) -> Self {
        Self { options }
    }

    /// Main entry point: infer component props from a component AST
    /// Returns a map of prop names to their inferred types
    pub fn infer_component_props(
        &self,
        component: &Component,
    ) -> InferenceResult<BTreeMap<String, PropertyType>> {
        let mut scope = Scope::new();

        // Pass 1: Collect component signature (variants, slots)
        self.collect_component_signature(component, &mut scope);

        // Pass 2: Infer from body expressions
        if let Some(body) = &component.body {
            self.infer_from_element(body, &mut scope)?;
        }

        // Pass 3: Convert scope to props and finalize types
        Ok(self.scope_to_props(&scope))
    }

    /// Pass 1: Collect explicit component signature (variants and slots)
    fn collect_component_signature(&self, component: &Component, scope: &mut Scope) {
        // Variants → Boolean props (always optional)
        for variant in &component.variants {
            scope.bind(variant.name.clone(), Type::Boolean);
        }

        // Slots → Slot props (wrapped in Optional if has default content)
        for slot in &component.slots {
            let slot_type = if !slot.default_content.is_empty() {
                // Has default content, make it optional
                Type::Optional(Box::new(Type::Slot))
            } else {
                // Required slot
                Type::Slot
            };
            scope.bind(slot.name.clone(), slot_type);
        }
    }

    /// Pass 2: Infer types from element tree
    fn infer_from_element(&self, element: &Element, scope: &mut Scope) -> InferenceResult<()> {
        match element {
            Element::Tag {
                attributes,
                children,
                ..
            } => {
                // Infer from attributes
                for (_, expr) in attributes {
                    self.infer_from_expression(expr, scope)?;
                }

                // Infer from children
                for child in children {
                    self.infer_from_element(child, scope)?;
                }
            }

            Element::Text { content, .. } => {
                self.infer_from_expression(content, scope)?;
            }

            Element::Instance {
                props, children, ..
            } => {
                // Infer from instance props
                for (_, expr) in props {
                    self.infer_from_expression(expr, scope)?;
                }

                // Infer from children
                for child in children {
                    self.infer_from_element(child, scope)?;
                }
            }

            Element::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Infer from condition
                self.infer_from_expression(condition, scope)?;

                // Create child scope for then branch
                let scope_rc = Rc::new(scope.clone());
                let mut then_scope = Scope::with_parent(scope_rc);

                for child in then_branch {
                    self.infer_from_element(child, &mut then_scope)?;
                }

                // Promote any new bindings to root
                then_scope.promote_to_root(scope);

                // Handle else branch if present
                if let Some(else_br) = else_branch {
                    let scope_rc = Rc::new(scope.clone());
                    let mut else_scope = Scope::with_parent(scope_rc);

                    for child in else_br {
                        self.infer_from_element(child, &mut else_scope)?;
                    }

                    // Promote any new bindings to root
                    else_scope.promote_to_root(scope);
                }
            }

            Element::Repeat {
                collection,
                item_name,
                body,
                ..
            } => {
                // Infer collection type
                let collection_type = self.infer_from_expression(collection, scope)?;

                // Create child scope with item binding
                let scope_rc = Rc::new(scope.clone());
                let mut child_scope = Scope::with_parent(scope_rc);

                // Infer item type from collection
                let item_type = match collection_type {
                    Type::Array(inner) => *inner,
                    _ => Type::Any, // Collection might not be typed yet
                };

                child_scope.bind(item_name.clone(), item_type);

                // Infer from body
                for child in body {
                    self.infer_from_element(child, &mut child_scope)?;
                }

                // Note: We don't promote loop variables to root scope
                // Only the collection variable is in the root scope
            }

            Element::SlotInsert { .. } => {
                // Slots are already handled in signature collection
            }

            Element::Insert { content, .. } => {
                // Process insert directive content
                for child in content {
                    self.infer_from_element(child, scope)?;
                }
            }
        }

        Ok(())
    }

    /// Infer type from an expression and update scope
    fn infer_from_expression(&self, expr: &Expression, scope: &mut Scope) -> InferenceResult<Type> {
        match expr {
            Expression::Literal { value, .. } => {
                Ok(Type::Literal(LiteralType::String(value.clone())))
            }

            Expression::Number { value, .. } => {
                Ok(Type::Literal(LiteralType::Number((*value).into())))
            }

            Expression::Boolean { value, .. } => Ok(Type::Literal(LiteralType::Boolean(*value))),

            Expression::Variable { name, .. } => {
                // Look up in scope
                if let Some(existing_type) = scope.lookup(name) {
                    Ok(existing_type)
                } else {
                    // Bind as Unknown initially - will be refined or converted to Any
                    scope.bind(name.clone(), Type::Unknown);
                    Ok(Type::Unknown)
                }
            }

            Expression::Member {
                object, property, ..
            } => self.infer_member_access_simple(object, property, scope),

            Expression::Binary {
                left,
                operator,
                right,
                ..
            } => self.infer_binary_operation(left, operator, right, scope),

            Expression::Call {
                function: _,
                arguments,
                ..
            } => {
                // Infer argument types
                for arg in arguments {
                    self.infer_from_expression(arg, scope)?;
                }

                // For now, return Any (can be enhanced with function signatures)
                Ok(Type::Any)
            }

            Expression::Template { parts, .. } => {
                // Infer types of template expressions
                for part in parts {
                    if let TemplatePart::Expression(e) = part {
                        self.infer_from_expression(e, scope)?;
                    }
                }

                Ok(Type::String)
            }
        }
    }

    /// Infer type from member access where property is a String (e.g., user.name)
    fn infer_member_access_simple(
        &self,
        object: &Expression,
        property: &str,
        scope: &mut Scope,
    ) -> InferenceResult<Type> {
        if !self.options.infer_object_properties {
            // Just infer the object and return Any
            self.infer_from_expression(object, scope)?;
            return Ok(Type::Any);
        }

        // For nested access (e.g., user.address.city), we need to handle recursively
        if let Expression::Member {
            object: nested_object,
            property: nested_property,
            ..
        } = object
        {
            if self.options.nested_member_access {
                // Handle nested member access recursively
                return self.infer_nested_member_access(
                    nested_object,
                    nested_property,
                    property,
                    scope,
                );
            }
        }

        // Simple member access (e.g., user.name)
        if let Expression::Variable { name, .. } = object {
            let obj_type = scope.lookup(name).unwrap_or(Type::Unknown);

            match obj_type {
                Type::Object(mut obj) => {
                    // Object already exists, return property type or add it
                    if let Some(prop) = obj.properties.get(property) {
                        Ok(prop.type_.clone())
                    } else {
                        // Add new property
                        obj.add_property(property.to_string(), Type::Any, false);
                        scope.refine(name, Type::Object(obj));
                        Ok(Type::Any)
                    }
                }

                Type::Unknown | Type::Any => {
                    // Create object with this property
                    let mut obj = ObjectType {
                        properties: BTreeMap::new(),
                        index_signature: Some(Box::new(Type::Any)),
                    };
                    obj.add_property(property.to_string(), Type::Any, false);

                    scope.refine(name, Type::Object(obj));
                    Ok(Type::Any)
                }

                _ => {
                    // Object is already typed as something else
                    // This could be an error in strict mode, but for now return Any
                    Ok(Type::Any)
                }
            }
        } else {
            // Complex object expression
            let obj_type = self.infer_from_expression(object, scope)?;

            match obj_type {
                Type::Object(obj) => {
                    if let Some(prop) = obj.properties.get(property) {
                        Ok(prop.type_.clone())
                    } else if let Some(index_sig) = obj.index_signature {
                        Ok(*index_sig)
                    } else {
                        Ok(Type::Any)
                    }
                }
                _ => Ok(Type::Any),
            }
        }
    }

    /// Handle nested member access (e.g., user.address.city)
    fn infer_nested_member_access(
        &self,
        base_object: &Expression,
        base_property: &str,
        final_property: &str,
        scope: &mut Scope,
    ) -> InferenceResult<Type> {
        // Get the root variable name
        if let Expression::Variable { name, .. } = base_object {
            let obj_type = scope.lookup(name).unwrap_or(Type::Unknown);

            match obj_type {
                Type::Object(mut obj) => {
                    // Get or create the intermediate object
                    let intermediate_type = if let Some(prop) = obj.properties.get(base_property) {
                        prop.type_.clone()
                    } else {
                        // Create intermediate object
                        let mut intermediate = ObjectType {
                            properties: BTreeMap::new(),
                            index_signature: Some(Box::new(Type::Any)),
                        };
                        intermediate.add_property(final_property.to_string(), Type::Any, false);

                        let intermediate_type = Type::Object(intermediate);
                        obj.add_property(
                            base_property.to_string(),
                            intermediate_type.clone(),
                            false,
                        );
                        scope.refine(name, Type::Object(obj));

                        return Ok(Type::Any);
                    };

                    // Now handle the final property access
                    match intermediate_type {
                        Type::Object(mut intermediate) => {
                            if let Some(prop) = intermediate.properties.get(final_property) {
                                Ok(prop.type_.clone())
                            } else {
                                // Add final property
                                intermediate.add_property(
                                    final_property.to_string(),
                                    Type::Any,
                                    false,
                                );

                                // Update the entire chain
                                obj.add_property(
                                    base_property.to_string(),
                                    Type::Object(intermediate),
                                    false,
                                );
                                scope.refine(name, Type::Object(obj));

                                Ok(Type::Any)
                            }
                        }
                        _ => Ok(Type::Any),
                    }
                }

                Type::Unknown | Type::Any => {
                    // Create nested object structure
                    let mut intermediate = ObjectType {
                        properties: BTreeMap::new(),
                        index_signature: Some(Box::new(Type::Any)),
                    };
                    intermediate.add_property(final_property.to_string(), Type::Any, false);

                    let mut obj = ObjectType {
                        properties: BTreeMap::new(),
                        index_signature: Some(Box::new(Type::Any)),
                    };
                    obj.add_property(base_property.to_string(), Type::Object(intermediate), false);

                    scope.refine(name, Type::Object(obj));
                    Ok(Type::Any)
                }

                _ => Ok(Type::Any),
            }
        } else {
            // Complex nested expression - just return Any for now
            Ok(Type::Any)
        }
    }

    /// Infer type from binary operation and apply constraints
    fn infer_binary_operation(
        &self,
        left: &Expression,
        operator: &BinaryOp,
        right: &Expression,
        scope: &mut Scope,
    ) -> InferenceResult<Type> {
        use BinaryOp::*;

        let left_type = self.infer_from_expression(left, scope)?;
        let right_type = self.infer_from_expression(right, scope)?;

        match operator {
            // Arithmetic operators
            Add => {
                // Special case: string concatenation
                if left_type.is_stringlike() || right_type.is_stringlike() {
                    return Ok(Type::String);
                }

                // Otherwise, constrain both operands to numbers
                self.constrain_as_number(left, scope)?;
                self.constrain_as_number(right, scope)?;
                Ok(Type::Number)
            }

            Subtract | Multiply | Divide => {
                // These always require numbers
                self.constrain_as_number(left, scope)?;
                self.constrain_as_number(right, scope)?;
                Ok(Type::Number)
            }

            // Comparison operators
            LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual => {
                // Typically numeric comparison
                self.constrain_as_number(left, scope)?;
                self.constrain_as_number(right, scope)?;
                Ok(Type::Boolean)
            }

            // Equality operators
            Equals | NotEquals => {
                // Can compare any types
                Ok(Type::Boolean)
            }

            // Logical operators
            And | Or => {
                // Operands can be any type (truthy/falsy)
                // Result is boolean
                Ok(Type::Boolean)
            }
        }
    }

    /// Constrain an expression to be a number type
    fn constrain_as_number(&self, expr: &Expression, scope: &mut Scope) -> InferenceResult<()> {
        if let Expression::Variable { name, .. } = expr {
            let current_type = scope.lookup(name).unwrap_or(Type::Unknown);

            match current_type {
                Type::Unknown | Type::Any => {
                    // Refine to Number
                    scope.refine(name, Type::Number);
                }
                Type::Number | Type::Literal(LiteralType::Number(_)) => {
                    // Already a number, good
                }
                _ => {
                    // Type conflict - unify with Number
                    // This will create a Union if incompatible
                    scope.refine(name, Type::Number);
                }
            }
        }

        Ok(())
    }

    /// Convert scope bindings to component props
    fn scope_to_props(&self, scope: &Scope) -> BTreeMap<String, PropertyType> {
        let bindings = scope.collect_root_props();
        let mut props = BTreeMap::new();

        for (name, type_) in bindings {
            // Finalize type (convert Unknown to Any)
            let finalized_type = type_.finalize();

            // Determine if optional
            // - Boolean (variants) are always optional
            // - Slot wrapped in Optional is optional
            // - Optional types are optional
            // - Everything else is required
            let optional = matches!(finalized_type, Type::Boolean | Type::Optional(_));

            // Unwrap Optional for Slot types
            let unwrapped_type = if let Type::Optional(inner) = finalized_type {
                if matches!(*inner, Type::Slot) {
                    *inner
                } else {
                    Type::Optional(inner)
                }
            } else {
                finalized_type
            };

            props.insert(
                name,
                PropertyType {
                    type_: unwrapped_type,
                    optional,
                },
            );
        }

        props
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse;

    #[test]
    fn test_infer_basic_variable() {
        let source = r#"
public component Test {
    render div {
        text {message}
    }
}
"#;

        let doc = parse(source).unwrap();
        let engine = InferenceEngine::new(InferenceOptions::default());
        let props = engine.infer_component_props(&doc.components[0]).unwrap();

        assert!(props.contains_key("message"));
        // Should be Any (finalized from Unknown)
        assert_eq!(props["message"].type_, Type::Any);
    }

    #[test]
    #[ignore] // Parser doesn't support binary operations yet
    fn test_infer_binary_op() {
        let source = r#"
public component Counter {
    render div {
        text {count + 1}
    }
}
"#;

        let doc = parse(source).unwrap();
        let engine = InferenceEngine::new(InferenceOptions::default());
        let props = engine.infer_component_props(&doc.components[0]).unwrap();

        assert!(props.contains_key("count"));
        assert_eq!(props["count"].type_, Type::Number);
    }

    #[test]
    fn test_infer_member_access() {
        let source = r#"
public component UserCard {
    render div {
        text {user.name}
    }
}
"#;

        let doc = parse(source).unwrap();
        let engine = InferenceEngine::new(InferenceOptions::default());
        let props = engine.infer_component_props(&doc.components[0]).unwrap();

        assert!(props.contains_key("user"));
        if let Type::Object(obj) = &props["user"].type_ {
            assert!(obj.properties.contains_key("name"));
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_infer_variant() {
        let source = r#"
public component Button {
    variant primary
    render button {
        text {label}
    }
}
"#;

        let doc = parse(source).unwrap();
        let engine = InferenceEngine::new(InferenceOptions::default());
        let props = engine.infer_component_props(&doc.components[0]).unwrap();

        assert!(props.contains_key("primary"));
        assert_eq!(props["primary"].type_, Type::Boolean);
        assert!(props["primary"].optional);
    }

    #[test]
    fn test_infer_slot() {
        let source = r#"
public component Card {
    slot header
    render div {
        header
    }
}
"#;

        let doc = parse(source).unwrap();
        let engine = InferenceEngine::new(InferenceOptions::default());
        let props = engine.infer_component_props(&doc.components[0]).unwrap();

        assert!(props.contains_key("header"));
        assert_eq!(props["header"].type_, Type::Slot);
    }
}
