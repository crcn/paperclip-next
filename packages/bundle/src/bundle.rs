//! # Bundle - Document Collection with Dependency Graph
//!
//! Manages parsed Paperclip documents with dependency tracking and name resolution.
//!
//! ## Purpose
//!
//! Bundle is the **single source of truth** for document lifetimes in the Paperclip system.
//! It maintains parsed documents, tracks dependencies between them, resolves imports, and
//! manages shared assets.
//!
//! ## Document Lifetime Ownership
//!
//! **CRITICAL RULE: Bundle owns all Document lifetimes.**
//!
//! Clients should **avoid holding long-lived `&Document` references**. Instead:
//!
//! ### ❌ Avoid - Ties client lifetime to Bundle
//! ```rust
//! let doc: &Document = bundle.get_document(path)?;
//! cache.store(doc);  // Dangerous if bundle rebuilds
//! ```
//!
//! ### ✅ Prefer - Client gets IDs or copies, not refs
//! ```rust
//! let doc_id: &str = bundle.get_document_id(path)?;
//! let component: Component = bundle.find_component("Button", path)?.clone();
//! ```
//!
//! **Why this matters:**
//! - Enables incremental rebuilds without invalidating client state
//! - Prevents clients from observing intermediate states during updates
//! - Allows Bundle to optimize internal representation
//! - Makes Bundle's invariants easier to maintain
//!
//! ## Architecture
//!
//! Bundle delegates to specialized modules:
//! - **GraphManager**: Dependency graph (cycles, topological sort)
//! - **Resolver**: Name resolution (components, styles, tokens)
//!
//! ## Encapsulation
//!
//! All fields are private. Access only through public methods:
//! - `get_document()` - Access parsed documents
//! - `get_dependencies()` - Query dependency graph
//! - `find_component()` - Resolve component names
//! - `find_style()` - Resolve style mixins
//! - `find_token()` - Resolve design tokens
//!
//! ## Usage
//!
//! ```rust
//! use paperclip_bundle::Bundle;
//! use paperclip_parser::parse;
//! use std::path::PathBuf;
//!
//! let mut bundle = Bundle::new();
//!
//! // Add documents
//! let doc = parse("component Button { ... }")?;
//! bundle.add_document(PathBuf::from("button.pc"), doc);
//!
//! // Build dependency graph
//! bundle.build_dependencies(&project_root)?;
//!
//! // Query
//! if let Some(doc) = bundle.get_document(&path) {
//!     // Use document (short-lived reference only!)
//! }
//! ```

use crate::graph::{GraphError, GraphManager};
use crate::resolver::{Resolver, ResolverError};
use paperclip_parser::ast::*;
use paperclip_parser::get_document_id;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use thiserror::Error;

// Re-export FileSystem traits from common-rs
pub use paperclip_common::{FileSystem, MockFileSystem, RealFileSystem};

#[derive(Error, Debug)]
pub enum BundleError {
    #[error("Circular dependency detected: {path}")]
    CircularDependency { path: String },

    #[error("Import not found: {import_path} imported by {source_path}")]
    ImportNotFound {
        import_path: String,
        source_path: String,
    },

    #[error("Style '{name}' not found in bundle")]
    StyleNotFound { name: String },

    #[error("Token '{name}' not found in bundle")]
    TokenNotFound { name: String },

    #[error("Component '{name}' not found in bundle")]
    ComponentNotFound { name: String },
}

// Convert GraphError to BundleError
impl From<GraphError> for BundleError {
    fn from(err: GraphError) -> Self {
        match err {
            GraphError::CircularDependency { path } => BundleError::CircularDependency { path },
        }
    }
}

// Convert ResolverError to BundleError
impl From<ResolverError> for BundleError {
    fn from(err: ResolverError) -> Self {
        match err {
            ResolverError::ImportNotFound {
                import_path,
                source_path,
            } => BundleError::ImportNotFound {
                import_path,
                source_path,
            },
            ResolverError::StyleNotFound { name } => BundleError::StyleNotFound { name },
            ResolverError::TokenNotFound { name } => BundleError::TokenNotFound { name },
            ResolverError::ComponentNotFound { name } => BundleError::ComponentNotFound { name },
        }
    }
}

/// Asset reference extracted from AST
#[derive(Clone, Debug)]
pub struct AssetReference {
    pub path: String,
    pub asset_type: AssetType,
    pub resolved_path: PathBuf,
    pub source_file: PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AssetType {
    Image,
    Font,
    Video,
    Audio,
    Other,
}

/// Bundle - collection of parsed documents with dependency graph
///
/// Refactored to delegate to GraphManager and Resolver for better separation of concerns
#[derive(Clone, Debug)]
pub struct Bundle {
    /// All parsed documents (path -> AST)
    documents: HashMap<PathBuf, Document>,

    /// Dependency graph manager
    graph: GraphManager,

    /// Import and name resolver
    resolver: Resolver,

    /// Deduplicated assets with source file tracking
    assets: HashMap<String, (AssetReference, HashSet<PathBuf>)>,

    /// Document IDs for each file (CRC32 of file path)
    document_ids: HashMap<PathBuf, String>,
}

impl Bundle {
    /// Create an empty bundle
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            graph: GraphManager::new(),
            resolver: Resolver::new(),
            assets: HashMap::new(),
            document_ids: HashMap::new(),
        }
    }

    /// Add a document to the bundle
    pub fn add_document(&mut self, path: PathBuf, document: Document) {
        // Canonicalize the path to ensure consistent lookups
        // (resolves symlinks like /var -> /private/var on macOS)
        let canonical_path = path.canonicalize().unwrap_or(path);
        let document_id = get_document_id(&canonical_path.to_string_lossy());
        self.document_ids.insert(canonical_path.clone(), document_id);
        self.documents.insert(canonical_path, document);
    }

    /// Build dependency graph from import statements
    pub fn build_dependencies(&mut self, project_root: &Path) -> Result<(), BundleError> {
        self.build_dependencies_with_fs(project_root, &RealFileSystem)
    }

    /// Build dependency graph asynchronously (requires 'async' feature)
    #[cfg(feature = "async")]
    pub async fn build_dependencies_async(
        &mut self,
        project_root: PathBuf,
    ) -> Result<(), BundleError> {
        self.build_dependencies_with_fs_async(project_root, RealFileSystem)
            .await
    }

    /// Build dependency graph asynchronously with custom file system (requires 'async' feature)
    #[cfg(feature = "async")]
    pub async fn build_dependencies_with_fs_async<F: FileSystem + Send + 'static>(
        &mut self,
        project_root: PathBuf,
        fs: F,
    ) -> Result<(), BundleError> {
        // Clone necessary data for the blocking task
        let documents = self.documents.clone();

        // Run the dependency building in a blocking task
        let result = tokio::task::spawn_blocking(move || {
            let mut temp_bundle = Bundle::new();
            temp_bundle.documents = documents;
            temp_bundle.build_dependencies_with_fs(&project_root, &fs)?;
            Ok::<_, BundleError>((temp_bundle.graph, temp_bundle.resolver))
        })
        .await
        .map_err(|e| BundleError::ImportNotFound {
            import_path: "spawn_blocking failed".to_string(),
            source_path: e.to_string(),
        })??;

        // Update self with the computed data
        self.graph = result.0;
        self.resolver = result.1;

        Ok(())
    }

    /// Build dependency graph with custom file system (for testing)
    pub fn build_dependencies_with_fs(
        &mut self,
        project_root: &Path,
        fs: &dyn FileSystem,
    ) -> Result<(), BundleError> {
        // Clear existing graph and resolver state
        self.graph.clear();
        self.resolver.clear();

        for (file_path, document) in &self.documents {
            let mut file_deps = Vec::new();

            for import in &document.imports {
                // Resolve import path using resolver
                let import_resolved =
                    self.resolver
                        .resolve_import_path(&import.path, file_path, project_root, fs)?;

                file_deps.push(import_resolved.clone());

                // Store alias mapping in resolver
                if let Some(ref alias) = import.alias {
                    self.resolver.add_alias(
                        file_path.clone(),
                        alias.clone(),
                        import_resolved.clone(),
                    );
                }
            }

            // Set dependencies in graph manager
            self.graph.set_dependencies(file_path.clone(), file_deps);
        }

        // Check for circular dependencies using graph manager
        self.graph.detect_circular_dependencies()?;

        Ok(())
    }

    /// Resolve import path relative to importing file

    /// Normalize a path by removing ./ and resolving ../

    /// Check if two paths refer to the same file

    /// Detect circular dependencies using DFS

    /// Get document by path
    pub fn get_document(&self, path: &Path) -> Option<&Document> {
        self.documents.get(path)
    }

    /// Get document ID for a file
    pub fn get_document_id(&self, path: &Path) -> Option<&str> {
        self.document_ids.get(path).map(|s| s.as_str())
    }

    /// Get all documents
    pub fn documents(&self) -> &HashMap<PathBuf, Document> {
        &self.documents
    }

    /// Get dependencies for a file
    pub fn get_dependencies(&self, path: &Path) -> Option<&[PathBuf]> {
        self.graph.get_dependencies(path)
    }

    /// Get dependents (files that import this file)
    pub fn get_dependents(&self, path: &Path) -> Option<&[PathBuf]> {
        self.graph.get_dependents(path)
    }

    /// Add asset reference
    /// Add an asset reference, deduplicating by path
    /// If the asset already exists, adds the source file to its users set
    pub fn add_asset(&mut self, asset: AssetReference) {
        self.assets
            .entry(asset.path.clone())
            .and_modify(|(_, sources)| {
                sources.insert(asset.source_file.clone());
            })
            .or_insert_with(|| {
                let mut sources = HashSet::new();
                sources.insert(asset.source_file.clone());
                (asset, sources)
            });
    }

    /// Get all unique assets (iterator)
    pub fn unique_assets(&self) -> impl Iterator<Item = &AssetReference> {
        self.assets.values().map(|(asset, _)| asset)
    }

    /// Get all source files that use a specific asset
    pub fn asset_users(&self, asset_path: &str) -> Option<&HashSet<PathBuf>> {
        self.assets.get(asset_path).map(|(_, sources)| sources)
    }

    /// Get all assets used by a specific source file
    pub fn assets_for_file(&self, file: &Path) -> Vec<&AssetReference> {
        self.assets
            .values()
            .filter(|(_, sources)| sources.contains(file))
            .map(|(asset, _)| asset)
            .collect()
    }

    /// Get total count of unique assets
    pub fn unique_asset_count(&self) -> usize {
        self.assets.len()
    }

    /// Deprecated: Use unique_assets() instead
    /// Returns a Vec for backwards compatibility
    #[deprecated(since = "0.2.0", note = "Use unique_assets() for better performance")]
    pub fn assets(&self) -> Vec<&AssetReference> {
        self.unique_assets().collect()
    }

    /// Look up a style declaration by name across the bundle
    /// Supports namespaced references like "theme.fontRegular"
    /// Searches in the given file and its imports
    pub fn find_style(
        &self,
        style_ref: &str,
        current_file: &Path,
    ) -> Option<(&StyleDecl, PathBuf)> {
        // Delegate to resolver
        self.resolver
            .find_style(style_ref, current_file, &self.documents)
            .ok()
    }

    /// Look up a token by name across the bundle
    /// Supports namespaced references like "theme.primaryColor"
    pub fn find_token(
        &self,
        token_ref: &str,
        current_file: &Path,
    ) -> Option<(&TokenDecl, PathBuf)> {
        // Delegate to resolver
        self.resolver
            .find_token(token_ref, current_file, &self.documents)
            .ok()
    }

    /// Look up a component by name across the bundle
    /// Supports namespaced references like "theme.Button"
    pub fn find_component(
        &self,
        component_ref: &str,
        current_file: &Path,
    ) -> Option<(&Component, PathBuf)> {
        // Delegate to resolver
        self.resolver
            .find_component(component_ref, current_file, &self.documents)
            .ok()
    }
}

impl Default for Bundle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse_with_path;

    #[test]
    fn test_bundle_creation() {
        let mut bundle = Bundle::new();

        let source = r#"
            public style Button {
                padding: 8px
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").unwrap();
        bundle.add_document(PathBuf::from("/test.pc"), doc);

        assert_eq!(bundle.documents().len(), 1);
        assert!(bundle.get_document(Path::new("/test.pc")).is_some());
    }

    #[test]
    fn test_dependency_graph() {
        let mut bundle = Bundle::new();

        // Create base styles file
        let base_source = r#"
            public style BaseButton {
                padding: 8px
            }
        "#;
        let base_doc = parse_with_path(base_source, "/styles/base.pc").unwrap();
        bundle.add_document(PathBuf::from("/styles/base.pc"), base_doc);

        // Create main file that imports base
        let main_source = r#"
            import "./styles/base.pc"

            public component Button {
                render button {}
            }
        "#;
        let main_doc = parse_with_path(main_source, "/main.pc").unwrap();
        bundle.add_document(PathBuf::from("/main.pc"), main_doc);

        // Note: build_dependencies needs project_root, which we can't easily test here
        // This test verifies structure creation
        assert_eq!(bundle.documents().len(), 2);
    }

    #[test]
    fn test_find_style_in_bundle() {
        let mut bundle = Bundle::new();

        let source = r#"
            public style ButtonStyle {
                padding: 8px
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").unwrap();
        let path = PathBuf::from("/test.pc");
        bundle.add_document(path.clone(), doc);

        let result = bundle.find_style("ButtonStyle", &path);
        assert!(result.is_some());

        let (style, found_path) = result.unwrap();
        assert_eq!(style.name, "ButtonStyle");
        assert_eq!(found_path, path);
    }

    #[test]
    fn test_document_id_tracking() {
        let mut bundle = Bundle::new();

        let source = r#"
            public component Test {
                render div {}
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").unwrap();
        let path = PathBuf::from("/test.pc");
        bundle.add_document(path.clone(), doc);

        let doc_id = bundle.get_document_id(&path);
        assert!(doc_id.is_some());
        assert!(!doc_id.unwrap().is_empty());
    }
}
