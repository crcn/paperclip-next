use paperclip_common::FileSystem;
/// Import and name resolution
///
/// Handles resolving import paths, alias mappings, and finding
/// components, tokens, and styles across the bundle.
use paperclip_parser::ast::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResolverError {
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

/// Handles import resolution and name lookups
#[derive(Clone, Debug, Default)]
pub struct Resolver {
    /// Import alias mapping: (source_file, alias) -> resolved_path
    /// Enables efficient lookup of "theme.fontRegular" style references
    import_aliases: HashMap<(PathBuf, String), PathBuf>,
}

impl Resolver {
    /// Create a new empty resolver
    pub fn new() -> Self {
        Self {
            import_aliases: HashMap::new(),
        }
    }

    /// Add an import alias mapping
    pub fn add_alias(&mut self, source_file: PathBuf, alias: String, target_path: PathBuf) {
        self.import_aliases
            .insert((source_file, alias), target_path);
    }

    /// Resolve an aliased import (e.g., "theme" -> "/path/to/theme.pc")
    pub fn resolve_alias(&self, source_file: &Path, alias: &str) -> Option<&PathBuf> {
        self.import_aliases
            .get(&(source_file.to_path_buf(), alias.to_string()))
    }

    /// Resolve import path relative to importing file
    pub fn resolve_import_path(
        &self,
        import_path: &str,
        importing_file: &Path,
        project_root: &Path,
        fs: &dyn FileSystem,
    ) -> Result<PathBuf, ResolverError> {
        // Resolve the import path
        let resolved = if import_path.starts_with("./") || import_path.starts_with("../") {
            // Relative import - resolve from importing file's directory
            importing_file
                .parent()
                .unwrap_or(project_root)
                .join(import_path)
        } else {
            // Absolute import - resolve from project root
            project_root.join(import_path)
        };

        // Check if file exists
        if !fs.exists(&resolved) {
            return Err(ResolverError::ImportNotFound {
                import_path: import_path.to_string(),
                source_path: importing_file.to_string_lossy().to_string(),
            });
        }

        // Canonicalize path (resolve symlinks, make absolute)
        fs.canonicalize(&resolved)
            .map_err(|_| ResolverError::ImportNotFound {
                import_path: import_path.to_string(),
                source_path: importing_file.to_string_lossy().to_string(),
            })
    }

    /// Find a style by name, checking aliases
    ///
    /// Handles both simple names ("myStyle") and aliased names ("theme.fontBold")
    pub fn find_style<'a>(
        &self,
        name: &str,
        requesting_file: &Path,
        documents: &'a HashMap<PathBuf, Document>,
    ) -> Result<(&'a StyleDecl, PathBuf), ResolverError> {
        // Check if name contains alias (e.g., "theme.fontBold")
        if let Some((alias, style_name)) = name.split_once('.') {
            // Aliased import
            if let Some(imported_file) = self.resolve_alias(requesting_file, alias) {
                if let Some(doc) = documents.get(imported_file) {
                    for style in &doc.styles {
                        if style.name == style_name && style.public {
                            return Ok((style, imported_file.clone()));
                        }
                    }
                }
            }
        } else {
            // Direct reference - search in requesting file first
            if let Some(doc) = documents.get(requesting_file) {
                for style in &doc.styles {
                    if style.name == name {
                        return Ok((style, requesting_file.to_path_buf()));
                    }
                }
            }
        }

        Err(ResolverError::StyleNotFound {
            name: name.to_string(),
        })
    }

    /// Find a token by name, checking aliases
    ///
    /// Handles both simple names ("myToken") and aliased names ("theme.primaryColor")
    pub fn find_token<'a>(
        &self,
        name: &str,
        requesting_file: &Path,
        documents: &'a HashMap<PathBuf, Document>,
    ) -> Result<(&'a TokenDecl, PathBuf), ResolverError> {
        // Check if name contains alias (e.g., "theme.primaryColor")
        if let Some((alias, token_name)) = name.split_once('.') {
            // Aliased import
            if let Some(imported_file) = self.resolve_alias(requesting_file, alias) {
                if let Some(doc) = documents.get(imported_file) {
                    for token in &doc.tokens {
                        if token.name == token_name && token.public {
                            return Ok((token, imported_file.clone()));
                        }
                    }
                }
            }
        } else {
            // Direct reference - search in requesting file first
            if let Some(doc) = documents.get(requesting_file) {
                for token in &doc.tokens {
                    if token.name == name {
                        return Ok((token, requesting_file.to_path_buf()));
                    }
                }
            }
        }

        Err(ResolverError::TokenNotFound {
            name: name.to_string(),
        })
    }

    /// Find a component by name, checking aliases
    ///
    /// Handles both simple names ("Button") and aliased names ("ui.Button")
    pub fn find_component<'a>(
        &self,
        name: &str,
        requesting_file: &Path,
        documents: &'a HashMap<PathBuf, Document>,
    ) -> Result<(&'a Component, PathBuf), ResolverError> {
        // Check if name contains alias (e.g., "ui.Button")
        if let Some((alias, component_name)) = name.split_once('.') {
            // Aliased import
            if let Some(imported_file) = self.resolve_alias(requesting_file, alias) {
                if let Some(doc) = documents.get(imported_file) {
                    for component in &doc.components {
                        if component.name == component_name && component.public {
                            return Ok((component, imported_file.clone()));
                        }
                    }
                }
            }
        } else {
            // Direct reference - search in requesting file first
            if let Some(doc) = documents.get(requesting_file) {
                for component in &doc.components {
                    if component.name == name {
                        return Ok((component, requesting_file.to_path_buf()));
                    }
                }
            }
        }

        Err(ResolverError::ComponentNotFound {
            name: name.to_string(),
        })
    }

    /// Clear all alias mappings
    pub fn clear(&mut self) {
        self.import_aliases.clear();
    }

    /// Get all aliases for debugging
    pub fn aliases(&self) -> &HashMap<(PathBuf, String), PathBuf> {
        &self.import_aliases
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_resolve_alias() {
        let mut resolver = Resolver::new();
        let source = PathBuf::from("/main.pc");
        let target = PathBuf::from("/theme.pc");

        resolver.add_alias(source.clone(), "theme".to_string(), target.clone());

        assert_eq!(resolver.resolve_alias(&source, "theme"), Some(&target));
        assert_eq!(resolver.resolve_alias(&source, "other"), None);
    }

    #[test]
    fn test_find_style_direct() {
        let resolver = Resolver::new();
        let file = PathBuf::from("/main.pc");

        let mut documents = HashMap::new();
        let mut doc = Document::new();
        doc.styles.push(StyleDecl {
            name: "myStyle".to_string(),
            public: true,
            properties: HashMap::new(),
            extends: Vec::new(),
            span: Span::new(0, 0, "test".to_string()),
        });
        documents.insert(file.clone(), doc);

        let result = resolver.find_style("myStyle", &file, &documents);
        assert!(result.is_ok());
        let (style, _) = result.unwrap();
        assert_eq!(style.name, "myStyle");
    }

    #[test]
    fn test_find_style_aliased() {
        let mut resolver = Resolver::new();
        let main_file = PathBuf::from("/main.pc");
        let theme_file = PathBuf::from("/theme.pc");

        // Set up alias
        resolver.add_alias(main_file.clone(), "theme".to_string(), theme_file.clone());

        // Set up documents
        let mut documents = HashMap::new();
        let mut theme_doc = Document::new();
        theme_doc.styles.push(StyleDecl {
            name: "fontBold".to_string(),
            public: true,
            properties: HashMap::new(),
            extends: Vec::new(),
            span: Span::new(0, 0, "test".to_string()),
        });
        documents.insert(theme_file.clone(), theme_doc);
        documents.insert(main_file.clone(), Document::new());

        let result = resolver.find_style("theme.fontBold", &main_file, &documents);
        assert!(result.is_ok());
        let (style, source_file) = result.unwrap();
        assert_eq!(style.name, "fontBold");
        assert_eq!(source_file, theme_file);
    }

    #[test]
    fn test_find_token_direct() {
        let resolver = Resolver::new();
        let file = PathBuf::from("/main.pc");

        let mut documents = HashMap::new();
        let mut doc = Document::new();
        doc.tokens.push(TokenDecl {
            name: "primaryColor".to_string(),
            value: "#blue".to_string(),
            public: true,
            span: Span::new(0, 0, "test".to_string()),
        });
        documents.insert(file.clone(), doc);

        let result = resolver.find_token("primaryColor", &file, &documents);
        assert!(result.is_ok());
        let (token, _) = result.unwrap();
        assert_eq!(token.name, "primaryColor");
    }

    #[test]
    fn test_find_component_direct() {
        let resolver = Resolver::new();
        let file = PathBuf::from("/main.pc");

        let mut documents = HashMap::new();
        let mut doc = Document::new();
        doc.components.push(Component {
            name: "Button".to_string(),
            public: true,
            doc_comment: None,
            script: None,
            frame: None,
            variants: Vec::new(),
            body: None,
            slots: Vec::new(),
            overrides: Vec::new(),
            span: Span::new(0, 0, "test".to_string()),
        });
        documents.insert(file.clone(), doc);

        let result = resolver.find_component("Button", &file, &documents);
        assert!(result.is_ok());
        let (component, _) = result.unwrap();
        assert_eq!(component.name, "Button");
    }
}
