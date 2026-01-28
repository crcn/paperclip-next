use crate::types::Type;
use std::collections::HashMap;
use std::rc::Rc;

/// Lexical scope for tracking variable bindings during type inference
/// Uses Rc for cheap forking without deep copies
#[derive(Debug, Clone)]
pub struct Scope {
    parent: Option<Rc<Scope>>,
    bindings: HashMap<String, Type>,
    /// Track if this is the root scope (component props)
    is_root: bool,
}

impl Scope {
    /// Create a new root scope for component props
    pub fn new() -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
            is_root: true,
        }
    }

    /// Create a child scope with the given parent
    /// Used for control flow (conditionals, loops) where new variables are introduced
    pub fn with_parent(parent: Rc<Scope>) -> Self {
        Self {
            parent: Some(parent),
            bindings: HashMap::new(),
            is_root: false,
        }
    }

    /// Bind a variable to a type, unifying with existing binding if present
    pub fn bind(&mut self, name: String, type_: Type) {
        self.bindings
            .entry(name)
            .and_modify(|existing| {
                *existing = Type::unify(existing, &type_);
            })
            .or_insert(type_);
    }

    /// Look up a variable in this scope or parent scopes
    pub fn lookup(&self, name: &str) -> Option<Type> {
        self.bindings
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    /// Refine an existing binding with a more specific type
    /// This is used for member access refinement where we learn more about a variable
    pub fn refine(&mut self, name: &str, type_: Type) {
        if self.bindings.contains_key(name) {
            // Refine in this scope
            self.bindings
                .entry(name.to_string())
                .and_modify(|existing| {
                    *existing = Type::unify(existing, &type_);
                });
        } else if let Some(parent) = &self.parent {
            // Need to propagate refinement up to where the binding exists
            // For now, we'll add it to this scope as an override
            // This is a simplification - a more sophisticated approach would
            // track refinements separately or use a different scope model
            self.bindings.insert(name.to_string(), type_);
        } else {
            // Variable not found, bind it here
            self.bind(name.to_string(), type_);
        }
    }

    /// Collect all bindings from the root scope only (component props)
    /// This ensures we don't include loop variables, conditionals, etc.
    pub fn collect_root_props(&self) -> HashMap<String, Type> {
        if self.is_root {
            self.bindings.clone()
        } else if let Some(parent) = &self.parent {
            parent.collect_root_props()
        } else {
            HashMap::new()
        }
    }

    /// Get all bindings including parent scopes (for debugging/testing)
    pub fn collect_all(&self) -> HashMap<String, Type> {
        let mut all = HashMap::new();

        // Collect from parents first (so children override)
        if let Some(parent) = &self.parent {
            all.extend(parent.collect_all());
        }

        // Override with current scope
        all.extend(self.bindings.clone());

        all
    }

    /// Check if a variable exists in any scope
    pub fn contains(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
            || self
                .parent
                .as_ref()
                .map(|p| p.contains(name))
                .unwrap_or(false)
    }

    /// Get a mutable reference to a binding in this scope only
    /// Returns None if the binding is in a parent scope
    pub fn get_local_mut(&mut self, name: &str) -> Option<&mut Type> {
        self.bindings.get_mut(name)
    }

    /// Promote all bindings from child scope to root
    /// Used when we want to ensure variables discovered in nested scopes
    /// are available as props (e.g., variables in conditionals)
    pub fn promote_to_root(&self, target: &mut Scope) {
        if !target.is_root {
            panic!("Can only promote to root scope");
        }

        for (name, type_) in &self.bindings {
            target.bind(name.clone(), type_.clone());
        }

        // Recursively promote from parent (but stop at root)
        if let Some(parent) = &self.parent {
            if !parent.is_root {
                parent.promote_to_root(target);
            }
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_binding() {
        let mut scope = Scope::new();
        scope.bind("x".to_string(), Type::String);

        assert_eq!(scope.lookup("x"), Some(Type::String));
    }

    #[test]
    fn test_parent_lookup() {
        let mut root = Scope::new();
        root.bind("x".to_string(), Type::String);

        let root_rc = Rc::new(root);
        let child = Scope::with_parent(root_rc);

        assert_eq!(child.lookup("x"), Some(Type::String));
    }

    #[test]
    fn test_binding_unification() {
        let mut scope = Scope::new();
        scope.bind("x".to_string(), Type::Unknown);
        scope.bind("x".to_string(), Type::Number);

        // Should unify Unknown with Number -> Number
        assert_eq!(scope.lookup("x"), Some(Type::Number));
    }

    #[test]
    fn test_child_scope_isolation() {
        let mut root = Scope::new();
        root.bind("x".to_string(), Type::String);

        let root_rc = Rc::new(root.clone());
        let mut child = Scope::with_parent(root_rc);
        child.bind("y".to_string(), Type::Number);

        // Child can see parent's binding
        assert_eq!(child.lookup("x"), Some(Type::String));
        // Parent cannot see child's binding
        assert_eq!(root.lookup("y"), None);
    }

    #[test]
    fn test_collect_root_props_only() {
        let mut root = Scope::new();
        root.bind("prop1".to_string(), Type::String);

        let root_rc = Rc::new(root.clone());
        let mut child = Scope::with_parent(root_rc);
        child.bind("loop_var".to_string(), Type::Number);

        // Only root props should be collected
        let props = child.collect_root_props();
        assert_eq!(props.len(), 1);
        assert!(props.contains_key("prop1"));
        assert!(!props.contains_key("loop_var"));
    }

    #[test]
    fn test_collect_all() {
        let mut root = Scope::new();
        root.bind("x".to_string(), Type::String);

        let root_rc = Rc::new(root);
        let mut child = Scope::with_parent(root_rc);
        child.bind("y".to_string(), Type::Number);

        let all = child.collect_all();
        assert_eq!(all.len(), 2);
        assert_eq!(all.get("x"), Some(&Type::String));
        assert_eq!(all.get("y"), Some(&Type::Number));
    }
}
