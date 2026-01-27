//! Evaluation context for tracking tokens, imports, and styles

use paperclip_proto::ast::{TokenValue, StyleDeclaration, Spanned};
use std::collections::HashMap;

/// Context for evaluation, tracks resolved values
pub struct EvalContext<'a> {
    pub path: &'a str,
    
    /// Resolved token values
    tokens: HashMap<String, ResolvedToken>,
    
    /// Style mixins
    styles: HashMap<String, Vec<Spanned<StyleDeclaration>>>,
    
    /// Import aliases
    imports: HashMap<String, String>,
    
    /// Counter for generating unique class names
    class_counter: usize,
    
    /// Current component being evaluated
    pub current_component: Option<String>,
    
    /// Active variant names
    pub active_variants: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ResolvedToken {
    pub name: String,
    pub value: String, // CSS value string
}

impl<'a> EvalContext<'a> {
    pub fn new(path: &'a str) -> Self {
        Self {
            path,
            tokens: HashMap::new(),
            styles: HashMap::new(),
            imports: HashMap::new(),
            class_counter: 0,
            current_component: None,
            active_variants: Vec::new(),
        }
    }
    
    /// Add an import alias
    pub fn add_import(&mut self, path: &str, alias: Option<&str>) {
        if let Some(alias) = alias {
            self.imports.insert(alias.to_string(), path.to_string());
        }
    }
    
    /// Add a token definition
    pub fn add_token(&mut self, name: &str, value: &TokenValue) {
        let css_value = token_value_to_css(value);
        self.tokens.insert(name.to_string(), ResolvedToken {
            name: name.to_string(),
            value: css_value,
        });
    }
    
    /// Resolve a token reference
    pub fn resolve_token(&self, name: &str) -> Option<&ResolvedToken> {
        // Handle namespaced tokens (e.g., "tokens.primary")
        if let Some(dot_pos) = name.find('.') {
            let namespace = &name[..dot_pos];
            let token_name = &name[dot_pos + 1..];
            
            // For now, just look up the token name directly
            // TODO: properly resolve imports
            return self.tokens.get(token_name);
        }
        
        self.tokens.get(name)
    }
    
    /// Add a style mixin
    pub fn add_style(&mut self, name: &str, declarations: &[Spanned<StyleDeclaration>]) {
        self.styles.insert(name.to_string(), declarations.to_vec());
    }
    
    /// Get a style mixin
    pub fn get_style(&self, name: &str) -> Option<&[Spanned<StyleDeclaration>]> {
        self.styles.get(name).map(|v| v.as_slice())
    }
    
    /// Generate a unique class name
    pub fn generate_class_name(&mut self, base: &str) -> String {
        self.class_counter += 1;
        format!("pc-{}-{}", base, self.class_counter)
    }
    
    /// Generate a source ID for a node
    pub fn generate_source_id(&mut self, node_type: &str) -> String {
        self.class_counter += 1;
        let component = self.current_component.as_deref().unwrap_or("root");
        format!("{}/{}/{}", component, node_type, self.class_counter)
    }
}

/// Convert a token value to CSS string
fn token_value_to_css(value: &TokenValue) -> String {
    match value {
        TokenValue::Color(c) => c.clone(),
        TokenValue::Dimension(n, u) => format!("{}{}", n, u),
        TokenValue::String(s) => format!("\"{}\"", s),
        TokenValue::Number(n) => n.to_string(),
        TokenValue::Reference(r) => format!("var(--{})", r),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_resolution() {
        let mut ctx = EvalContext::new("test.pc");
        ctx.add_token("primary", &TokenValue::Color("#3366FF".to_string()));
        
        let resolved = ctx.resolve_token("primary").unwrap();
        assert_eq!(resolved.value, "#3366FF");
    }
    
    #[test]
    fn test_class_name_generation() {
        let mut ctx = EvalContext::new("test.pc");
        
        let class1 = ctx.generate_class_name("button");
        let class2 = ctx.generate_class_name("button");
        
        assert_ne!(class1, class2);
    }
}
