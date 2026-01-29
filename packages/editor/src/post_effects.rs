//! # Post-Effect System
//!
//! Mutations trigger cascading effects to maintain document integrity.
//!
//! ## Design
//!
//! When a mutation is applied, it may require additional changes to keep the
//! document consistent. For example:
//! - Deleting a node → cleanup overrides pointing to it
//! - Moving a node → update override paths
//! - Renaming a component → update all instance references
//! - Deleting a component → remove all instances of it
//!
//! Post-effects are:
//! - **Deterministic**: Same mutation always produces same effects
//! - **Order-independent**: Effects don't depend on application order
//! - **Minimal**: Only generate necessary secondary mutations
//! - **Composable**: Multiple effects can be triggered by one mutation

use crate::mutations::Mutation;
use paperclip_parser::ast::Document;

/// Post-effect that can be triggered by a mutation
pub trait PostEffect: std::fmt::Debug {
    /// Analyze the mutation and generate secondary mutations if needed
    fn analyze(&self, mutation: &Mutation, doc: &Document) -> Vec<Mutation>;
}

/// Cleanup overrides that point to deleted nodes
#[derive(Debug)]
pub struct CleanupOrphanedOverrides;

impl PostEffect for CleanupOrphanedOverrides {
    fn analyze(&self, mutation: &Mutation, doc: &Document) -> Vec<Mutation> {
        match mutation {
            Mutation::RemoveNode { node_id } => {
                // TODO: Find all overrides targeting this node and its descendants
                // For now, return empty vec - proper implementation needs override system
                vec![]
            }
            _ => vec![],
        }
    }
}

/// Update instance references when a component is renamed
#[derive(Debug)]
pub struct UpdateInstanceReferences;

impl PostEffect for UpdateInstanceReferences {
    fn analyze(&self, mutation: &Mutation, _doc: &Document) -> Vec<Mutation> {
        // TODO: Implement when we have component rename mutation
        // Would scan all Instance elements and update their component_name
        vec![]
    }
}

/// Remove all instances when a component is deleted
#[derive(Debug)]
pub struct CleanupDeletedComponentInstances;

impl PostEffect for CleanupDeletedComponentInstances {
    fn analyze(&self, mutation: &Mutation, doc: &Document) -> Vec<Mutation> {
        // TODO: Implement when we have DeleteComponent mutation
        // Would find all Instance elements referencing the deleted component
        // and generate RemoveNode mutations for each
        vec![]
    }
}

/// Update override paths when nodes are moved
#[derive(Debug)]
pub struct ReparentOverrides;

impl PostEffect for ReparentOverrides {
    fn analyze(&self, mutation: &Mutation, _doc: &Document) -> Vec<Mutation> {
        match mutation {
            Mutation::MoveElement { .. } => {
                // TODO: Find overrides with paths containing the moved node
                // Update their paths to reflect new parent
                vec![]
            }
            _ => vec![],
        }
    }
}

/// Post-effect engine that applies all registered effects
#[derive(Debug)]
pub struct PostEffectEngine {
    effects: Vec<Box<dyn PostEffect>>,
}

impl PostEffectEngine {
    /// Create engine with default effects
    pub fn new() -> Self {
        Self {
            effects: vec![
                Box::new(CleanupOrphanedOverrides),
                Box::new(UpdateInstanceReferences),
                Box::new(CleanupDeletedComponentInstances),
                Box::new(ReparentOverrides),
            ],
        }
    }

    /// Analyze a mutation and generate all secondary mutations
    pub fn analyze(&self, mutation: &Mutation, doc: &Document) -> Vec<Mutation> {
        let mut secondary_mutations = Vec::new();

        for effect in &self.effects {
            let mut effect_mutations = effect.analyze(mutation, doc);
            secondary_mutations.append(&mut effect_mutations);
        }

        secondary_mutations
    }

    /// Apply a mutation with all its post-effects
    pub fn apply_with_effects(
        &self,
        mutation: Mutation,
        doc: &mut Document,
    ) -> Result<Vec<Mutation>, crate::MutationError> {
        // Track all mutations applied (for undo)
        let mut applied_mutations = vec![mutation.clone()];

        // Apply primary mutation
        mutation.apply(doc)?;

        // Generate and apply secondary mutations
        let secondary = self.analyze(&mutation, doc);
        for secondary_mutation in secondary {
            secondary_mutation.apply(doc)?;
            applied_mutations.push(secondary_mutation);
        }

        Ok(applied_mutations)
    }
}

impl Default for PostEffectEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse;

    #[test]
    fn test_post_effect_engine_creation() {
        let engine = PostEffectEngine::new();
        assert_eq!(engine.effects.len(), 4);
    }

    #[test]
    fn test_analyze_returns_empty_for_simple_mutations() {
        let source = r#"
            component Test {
                render div {
                    text "Hello"
                }
            }
        "#;
        let doc = parse(source).unwrap();
        let engine = PostEffectEngine::new();

        let mutation = Mutation::UpdateText {
            node_id: "test-123".to_string(),
            content: "World".to_string(),
        };

        let secondary = engine.analyze(&mutation, &doc);

        // For now, should return empty since we haven't implemented override system
        assert_eq!(secondary.len(), 0);
    }

    #[test]
    fn test_apply_with_effects_applies_primary_mutation() {
        let source = r#"
            component Test {
                render div {
                    text "Hello"
                }
            }
        "#;
        let mut doc = parse(source).unwrap();
        let engine = PostEffectEngine::new();

        // Get the text node ID (it's inside the div)
        let div = doc.components[0].body.as_ref().unwrap();
        let text_id = div.children().unwrap()[0].span().id.clone();

        let mutation = Mutation::UpdateText {
            node_id: text_id,
            content: "World".to_string(),
        };

        let result = engine.apply_with_effects(mutation, &mut doc);
        assert!(result.is_ok());

        let applied = result.unwrap();
        assert_eq!(applied.len(), 1); // Only primary mutation (no secondary effects yet)
    }
}
