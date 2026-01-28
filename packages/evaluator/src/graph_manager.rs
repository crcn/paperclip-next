/// Dependency graph manager
///
/// Manages the dependency graph for a collection of documents,
/// tracking which files import which other files and detecting circular dependencies.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Circular dependency detected: {path}")]
    CircularDependency { path: String },
}

/// Manages dependency graph (file -> files it depends on)
#[derive(Clone, Debug, Default)]
pub struct GraphManager {
    /// Dependency graph: file -> files it imports
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,

    /// Reverse lookup: file -> files that import it
    dependents: HashMap<PathBuf, Vec<PathBuf>>,
}

impl GraphManager {
    /// Create a new empty graph manager
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Add a dependency relationship: source depends on target
    pub fn add_dependency(&mut self, source: PathBuf, target: PathBuf) {
        // Add to dependencies
        self.dependencies
            .entry(source.clone())
            .or_insert_with(Vec::new)
            .push(target.clone());

        // Add to dependents (reverse lookup)
        self.dependents
            .entry(target)
            .or_insert_with(Vec::new)
            .push(source);
    }

    /// Set all dependencies for a file at once
    pub fn set_dependencies(&mut self, source: PathBuf, targets: Vec<PathBuf>) {
        // Remove old dependents relationships
        if let Some(old_targets) = self.dependencies.get(&source) {
            for old_target in old_targets {
                if let Some(deps) = self.dependents.get_mut(old_target) {
                    deps.retain(|p| p != &source);
                }
            }
        }

        // Add new dependencies
        for target in &targets {
            self.dependents
                .entry(target.clone())
                .or_insert_with(Vec::new)
                .push(source.clone());
        }

        self.dependencies.insert(source, targets);
    }

    /// Get dependencies for a file
    pub fn get_dependencies(&self, path: &Path) -> Option<&[PathBuf]> {
        self.dependencies.get(path).map(|v| v.as_slice())
    }

    /// Get dependents for a file (files that import this file)
    pub fn get_dependents(&self, path: &Path) -> Option<&[PathBuf]> {
        self.dependents.get(path).map(|v| v.as_slice())
    }

    /// Get all files in the dependency graph
    pub fn all_files(&self) -> HashSet<PathBuf> {
        let mut files = HashSet::new();
        files.extend(self.dependencies.keys().cloned());
        files.extend(self.dependents.keys().cloned());
        files
    }

    /// Detect circular dependencies using DFS
    ///
    /// Returns an error if a cycle is detected.
    pub fn detect_circular_dependencies(&self) -> Result<(), GraphError> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for file in self.dependencies.keys() {
            if !visited.contains(file) {
                self.dfs_detect_cycle(file, &mut visited, &mut recursion_stack)?;
            }
        }

        Ok(())
    }

    /// DFS helper for cycle detection
    fn dfs_detect_cycle(
        &self,
        node: &Path,
        visited: &mut HashSet<PathBuf>,
        recursion_stack: &mut HashSet<PathBuf>,
    ) -> Result<(), GraphError> {
        visited.insert(node.to_path_buf());
        recursion_stack.insert(node.to_path_buf());

        if let Some(deps) = self.dependencies.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.dfs_detect_cycle(dep, visited, recursion_stack)?;
                } else if recursion_stack.contains(dep) {
                    // Found a cycle
                    return Err(GraphError::CircularDependency {
                        path: dep.to_string_lossy().to_string(),
                    });
                }
            }
        }

        recursion_stack.remove(node);
        Ok(())
    }

    /// Get topologically sorted files (dependencies first)
    ///
    /// Returns files in an order where each file comes after all its dependencies.
    /// Useful for evaluation order.
    pub fn topological_sort(&self) -> Result<Vec<PathBuf>, GraphError> {
        // Verify no cycles first
        self.detect_circular_dependencies()?;

        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // Calculate in-degree for all nodes (number of dependencies each has)
        for file in self.all_files() {
            let degree = self.dependencies.get(&file).map(|v| v.len()).unwrap_or(0);
            in_degree.insert(file, degree);
        }

        // Add nodes with no dependencies to queue
        for (file, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(file.clone());
            }
        }

        // Process queue
        while let Some(file) = queue.pop_front() {
            result.push(file.clone());

            // Reduce in-degree for dependent nodes
            if let Some(dependents) = self.dependents.get(&file) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Get transitive dependencies (all dependencies, recursively)
    pub fn get_transitive_dependencies(&self, path: &Path) -> HashSet<PathBuf> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(path.to_path_buf());

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(deps) = self.dependencies.get(&current) {
                for dep in deps {
                    if !visited.contains(dep) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        // Remove the starting file from results
        visited.remove(path);
        visited
    }

    /// Remove a file from the graph
    pub fn remove_file(&mut self, path: &Path) {
        // Remove from dependencies
        if let Some(deps) = self.dependencies.remove(path) {
            // Remove from dependents and clean up empty entries
            for dep in deps {
                if let Some(dependents) = self.dependents.get_mut(&dep) {
                    dependents.retain(|p| p != path);
                    // Remove entry if empty
                    if dependents.is_empty() {
                        self.dependents.remove(&dep);
                    }
                }
            }
        }

        // Remove from dependents and update dependencies
        if let Some(dependents) = self.dependents.remove(path) {
            for dependent in dependents {
                if let Some(deps) = self.dependencies.get_mut(&dependent) {
                    deps.retain(|p| p != path);
                    // Remove entry if empty
                    if deps.is_empty() {
                        self.dependencies.remove(&dependent);
                    }
                }
            }
        }
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.dependencies.clear();
        self.dependents.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_dependency() {
        let mut graph = GraphManager::new();
        let a = PathBuf::from("/a.pc");
        let b = PathBuf::from("/b.pc");

        graph.add_dependency(a.clone(), b.clone());

        assert_eq!(graph.get_dependencies(&a), Some(&[b.clone()][..]));
        assert_eq!(graph.get_dependents(&b), Some(&[a.clone()][..]));
    }

    #[test]
    fn test_detect_circular_dependency() {
        let mut graph = GraphManager::new();
        let a = PathBuf::from("/a.pc");
        let b = PathBuf::from("/b.pc");
        let c = PathBuf::from("/c.pc");

        // Create cycle: a -> b -> c -> a
        graph.add_dependency(a.clone(), b.clone());
        graph.add_dependency(b.clone(), c.clone());
        graph.add_dependency(c.clone(), a.clone());

        let result = graph.detect_circular_dependencies();
        assert!(result.is_err());
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = GraphManager::new();
        let a = PathBuf::from("/a.pc");
        let b = PathBuf::from("/b.pc");
        let c = PathBuf::from("/c.pc");

        // a depends on b, b depends on c
        graph.add_dependency(a.clone(), b.clone());
        graph.add_dependency(b.clone(), c.clone());

        let sorted = graph.topological_sort().unwrap();

        // c should come before b, b should come before a
        let c_pos = sorted.iter().position(|p| p == &c).unwrap();
        let b_pos = sorted.iter().position(|p| p == &b).unwrap();
        let a_pos = sorted.iter().position(|p| p == &a).unwrap();

        assert!(c_pos < b_pos);
        assert!(b_pos < a_pos);
    }

    #[test]
    fn test_transitive_dependencies() {
        let mut graph = GraphManager::new();
        let a = PathBuf::from("/a.pc");
        let b = PathBuf::from("/b.pc");
        let c = PathBuf::from("/c.pc");
        let d = PathBuf::from("/d.pc");

        // a -> b -> c, a -> d
        graph.add_dependency(a.clone(), b.clone());
        graph.add_dependency(b.clone(), c.clone());
        graph.add_dependency(a.clone(), d.clone());

        let transitive = graph.get_transitive_dependencies(&a);

        assert_eq!(transitive.len(), 3);
        assert!(transitive.contains(&b));
        assert!(transitive.contains(&c));
        assert!(transitive.contains(&d));
    }

    #[test]
    fn test_remove_file() {
        let mut graph = GraphManager::new();
        let a = PathBuf::from("/a.pc");
        let b = PathBuf::from("/b.pc");
        let c = PathBuf::from("/c.pc");

        graph.add_dependency(a.clone(), b.clone());
        graph.add_dependency(b.clone(), c.clone());

        graph.remove_file(&b);

        assert_eq!(graph.get_dependencies(&a), None);
        assert_eq!(graph.get_dependents(&c), None);
    }
}
