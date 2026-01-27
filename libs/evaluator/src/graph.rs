//! Dependency graph management for incremental evaluation

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use paperclip_proto::virt::EvaluatedModule;

/// Manages the dependency graph for incremental updates
pub struct GraphManager {
    /// File dependency graph: file -> files it imports
    file_deps: HashMap<PathBuf, HashSet<PathBuf>>,
    
    /// Reverse dependency graph: file -> files that import it
    reverse_deps: HashMap<PathBuf, HashSet<PathBuf>>,
    
    /// Cached evaluation results
    cache: HashMap<PathBuf, EvaluatedModule>,
}

impl GraphManager {
    pub fn new() -> Self {
        Self {
            file_deps: HashMap::new(),
            reverse_deps: HashMap::new(),
            cache: HashMap::new(),
        }
    }
    
    /// Register a file's dependencies
    pub fn register_deps(&mut self, path: PathBuf, deps: Vec<PathBuf>) {
        // Clear old reverse deps for this file
        if let Some(old_deps) = self.file_deps.get(&path) {
            for dep in old_deps {
                if let Some(rev) = self.reverse_deps.get_mut(dep) {
                    rev.remove(&path);
                }
            }
        }
        
        // Add new reverse deps
        for dep in &deps {
            self.reverse_deps
                .entry(dep.clone())
                .or_default()
                .insert(path.clone());
        }
        
        self.file_deps.insert(path, deps.into_iter().collect());
    }
    
    /// Get files that need re-evaluation when a file changes
    pub fn get_invalidated(&self, path: &PathBuf) -> Vec<PathBuf> {
        let mut invalidated = vec![path.clone()];
        let mut visited = HashSet::new();
        visited.insert(path.clone());
        
        // BFS to find all dependent files
        let mut queue = vec![path.clone()];
        while let Some(current) = queue.pop() {
            if let Some(dependents) = self.reverse_deps.get(&current) {
                for dep in dependents {
                    if !visited.contains(dep) {
                        visited.insert(dep.clone());
                        invalidated.push(dep.clone());
                        queue.push(dep.clone());
                    }
                }
            }
        }
        
        invalidated
    }
    
    /// Invalidate cached results for a file and its dependents
    pub fn invalidate(&mut self, path: &PathBuf) {
        let invalidated = self.get_invalidated(path);
        for p in invalidated {
            self.cache.remove(&p);
        }
    }
    
    /// Cache an evaluation result
    pub fn cache_result(&mut self, path: PathBuf, result: EvaluatedModule) {
        self.cache.insert(path, result);
    }
    
    /// Get a cached result
    pub fn get_cached(&self, path: &PathBuf) -> Option<&EvaluatedModule> {
        self.cache.get(path)
    }
    
    /// Check if a file needs re-evaluation
    pub fn needs_evaluation(&self, path: &PathBuf) -> bool {
        !self.cache.contains_key(path)
    }
}

impl Default for GraphManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_tracking() {
        let mut graph = GraphManager::new();
        
        let main = PathBuf::from("main.pc");
        let tokens = PathBuf::from("tokens.pc");
        let components = PathBuf::from("components.pc");
        
        // main imports tokens and components
        graph.register_deps(main.clone(), vec![tokens.clone(), components.clone()]);
        // components imports tokens
        graph.register_deps(components.clone(), vec![tokens.clone()]);
        
        // If tokens changes, both main and components should be invalidated
        let invalidated = graph.get_invalidated(&tokens);
        
        assert!(invalidated.contains(&tokens));
        assert!(invalidated.contains(&main));
        assert!(invalidated.contains(&components));
    }
}
