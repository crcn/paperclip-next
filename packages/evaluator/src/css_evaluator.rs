use crate::utils::get_style_namespace;
use paperclip_bundle::Bundle;
use paperclip_parser::ast::*;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info, instrument};

pub type CssResult<T> = Result<T, CssError>;

#[derive(Error, Debug)]
pub enum CssError {
    #[error("Style evaluation error: {message}")]
    EvaluationError { message: String },

    #[error("Token '{name}' not found")]
    TokenNotFound { name: String },
}

/// CSS rule with selector and properties
#[derive(Debug, Clone, PartialEq)]
pub struct CssRule {
    pub selector: String,
    pub properties: HashMap<String, String>,
    pub media_query: Option<String>,
}

/// CSS document - collection of CSS rules
#[derive(Debug, Clone)]
pub struct VirtualCssDocument {
    pub rules: Vec<CssRule>,
}

impl VirtualCssDocument {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: CssRule) {
        self.rules.push(rule);
    }

    /// Convert to CSS text
    pub fn to_css(&self) -> String {
        let mut css = String::new();

        for rule in &self.rules {
            css.push_str(&rule.selector);
            css.push_str(" {\n");

            for (key, value) in &rule.properties {
                css.push_str("  ");
                css.push_str(key);
                css.push_str(": ");
                css.push_str(value);
                css.push_str(";\n");
            }

            css.push_str("}\n\n");
        }

        css
    }
}

impl Default for VirtualCssDocument {
    fn default() -> Self {
        Self::new()
    }
}

/// CSS Evaluator - extracts styles from PC components
pub struct CssEvaluator {
    tokens: HashMap<String, String>,
    triggers: HashMap<String, Vec<String>>,  // trigger name -> selectors
    document_id: String,
}

impl CssEvaluator {
    pub fn new() -> Self {
        Self::with_document_id("<anonymous>")
    }

    pub fn with_document_id(path: &str) -> Self {
        let document_id = paperclip_parser::get_document_id(path);
        Self {
            tokens: HashMap::new(),
            triggers: HashMap::new(),
            document_id,
        }
    }

    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    /// Get the registered tokens (for testing/debugging)
    pub fn tokens(&self) -> &HashMap<String, String> {
        &self.tokens
    }

    /// Evaluate a document to CSS
    #[instrument(skip(self, doc), fields(components = doc.components.len(), tokens = doc.tokens.len()))]
    pub fn evaluate(&mut self, doc: &Document) -> CssResult<VirtualCssDocument> {
        info!("Starting CSS evaluation");

        // Register tokens
        for token in &doc.tokens {
            debug!(token_name = %token.name, token_value = %token.value, "Registering CSS token");
            self.tokens.insert(token.name.clone(), token.value.clone());
        }

        // Register triggers
        for trigger in &doc.triggers {
            debug!(trigger_name = %trigger.name, selectors = ?trigger.selectors, "Registering CSS trigger");
            self.triggers.insert(trigger.name.clone(), trigger.selectors.clone());
        }

        let mut css_doc = VirtualCssDocument::new();

        // Extract global styles with CSS variables
        for style_decl in &doc.styles {
            debug!(style_name = %style_decl.name, "Processing global style");
            let rules = self.evaluate_style_decl(style_decl, &doc.styles)?;
            for rule in rules {
                css_doc.add_rule(rule);
            }
        }

        // Extract component styles
        for component in &doc.components {
            if component.public {
                debug!(component_name = %component.name, "Processing component styles");
                let rules =
                    self.evaluate_component_styles(&component.name, component, &doc.styles)?;
                for rule in rules {
                    css_doc.add_rule(rule);
                }
            }
        }

        // Extract top-level render styles
        for render in &doc.renders {
            let mut rules = Vec::new();
            self.extract_element_styles(render, None, &mut rules, &doc.styles, &[])?;
            for rule in rules {
                css_doc.add_rule(rule);
            }
        }

        info!(rules = css_doc.rules.len(), "CSS evaluation complete");
        Ok(css_doc)
    }

    /// Evaluate a bundle to CSS (supports cross-file imports)
    #[instrument(skip(self, bundle), fields(entry = %entry_path.display()))]
    pub fn evaluate_bundle(
        &mut self,
        bundle: &Bundle,
        entry_path: &Path,
    ) -> CssResult<VirtualCssDocument> {
        info!("Starting bundle CSS evaluation");

        // Get entry document
        let entry_doc =
            bundle
                .get_document(entry_path)
                .ok_or_else(|| CssError::EvaluationError {
                    message: format!("Entry document not found: {}", entry_path.display()),
                })?;

        // Update document_id for the entry file
        if let Some(doc_id) = bundle.get_document_id(entry_path) {
            self.document_id = doc_id.to_string();
        }

        // Register tokens from entry file
        for token in &entry_doc.tokens {
            debug!(token_name = %token.name, token_value = %token.value, "Registering CSS token");
            self.tokens.insert(token.name.clone(), token.value.clone());
        }

        // Register tokens from imported files
        if let Some(deps) = bundle.get_dependencies(entry_path) {
            for dep_path in deps {
                if let Some(dep_doc) = bundle.get_document(dep_path) {
                    for token in &dep_doc.tokens {
                        if token.public {
                            debug!(token_name = %token.name, from_file = %dep_path.display(), "Registering imported token");
                            self.tokens.insert(token.name.clone(), token.value.clone());
                        }
                    }
                }
            }
        }

        let mut css_doc = VirtualCssDocument::new();

        // Collect all styles (entry + imported) for extends resolution
        let mut all_styles = entry_doc.styles.clone();
        if let Some(deps) = bundle.get_dependencies(entry_path) {
            for dep_path in deps {
                if let Some(dep_doc) = bundle.get_document(dep_path) {
                    for style in &dep_doc.styles {
                        if style.public {
                            all_styles.push(style.clone());
                        }
                    }
                }
            }
        }

        // Extract global styles from entry file
        for style_decl in &entry_doc.styles {
            debug!(style_name = %style_decl.name, "Processing global style");
            let rules = self.evaluate_style_decl(style_decl, &all_styles)?;
            for rule in rules {
                css_doc.add_rule(rule);
            }
        }

        // Extract global styles from imported files (public styles only)
        // This ensures CSS variables are available for instant theme updates
        if let Some(deps) = bundle.get_dependencies(entry_path) {
            for dep_path in deps {
                if let Some(dep_doc) = bundle.get_document(dep_path) {
                    for style_decl in &dep_doc.styles {
                        if style_decl.public {
                            debug!(style_name = %style_decl.name, from_file = %dep_path.display(), "Processing imported global style");
                            let rules = self.evaluate_style_decl(style_decl, &all_styles)?;
                            for rule in rules {
                                css_doc.add_rule(rule);
                            }
                        }
                    }
                }
            }
        }

        // Extract component styles from entry file
        for component in &entry_doc.components {
            if component.public {
                debug!(component_name = %component.name, "Processing component styles");
                let rules =
                    self.evaluate_component_styles(&component.name, component, &all_styles)?;
                for rule in rules {
                    css_doc.add_rule(rule);
                }
            }
        }

        info!(
            rules = css_doc.rules.len(),
            "Bundle CSS evaluation complete"
        );
        Ok(css_doc)
    }

    /// Evaluate a style declaration to CSS rules (with CSS variables)
    fn evaluate_style_decl(
        &mut self,
        style_decl: &StyleDecl,
        all_styles: &[StyleDecl],
    ) -> CssResult<Vec<CssRule>> {
        let mut rules = Vec::new();

        // Generate CSS custom properties (variables) for this style
        let mut variables = HashMap::new();
        for (property, value) in &style_decl.properties {
            let var_name = format!("--{}-{}-{}", style_decl.name, property, style_decl.span.id);
            let resolved_value = self.resolve_value(value)?;
            variables.insert(var_name, resolved_value);
        }

        // Create :root rule with CSS variables if we have properties
        if !variables.is_empty() {
            rules.push(CssRule {
                            media_query: None,
                selector: ":root".to_string(),
                properties: variables,
            });
        }

        // Create class rule that uses the variables
        let class_name = get_style_namespace(
            Some(&style_decl.name),
            &style_decl.span.id,
            None, // Not in component context
        );

        let mut class_properties = HashMap::new();

        // Handle extends - pull in CSS variables from extended styles
        for extend_ref in &style_decl.extends {
            if let Some(extended_style) = all_styles.iter().find(|s| &s.name == extend_ref) {
                // Pull in the CSS variables from the extended style
                for (property, value) in &extended_style.properties {
                    let var_name = format!(
                        "--{}-{}-{}",
                        extended_style.name, property, extended_style.span.id
                    );
                    let resolved_value = self.resolve_value(value)?;
                    // Reference the variable with fallback
                    let var_value = format!("var({}, {})", var_name, resolved_value);
                    class_properties.insert(property.clone(), var_value);
                }
            }
        }

        // Add local properties (can override extended properties)
        for (property, value) in &style_decl.properties {
            let var_name = format!("--{}-{}-{}", style_decl.name, property, style_decl.span.id);
            let resolved_value = self.resolve_value(value)?;
            // Use var() with fallback
            let var_value = format!("var({}, {})", var_name, resolved_value);
            class_properties.insert(property.clone(), var_value);
        }

        rules.push(CssRule {
                            media_query: None,
            selector: format!(".{}", class_name),
            properties: class_properties,
        });

        Ok(rules)
    }

    /// Evaluate component styles
    fn evaluate_component_styles(
        &mut self,
        component_name: &str,
        component: &Component,
        all_styles: &[StyleDecl],
    ) -> CssResult<Vec<CssRule>> {
        let mut rules = Vec::new();

        if let Some(body) = &component.body {
            // Extract styles from component body
            // Pass component name for scoping, all styles for extends, and variants for trigger resolution
            self.extract_element_styles(body, Some(component_name), &mut rules, all_styles, &component.variants)?;
        }

        Ok(rules)
    }

    /// Extract styles from element tree
    fn extract_element_styles(
        &mut self,
        element: &Element,
        component_name: Option<&str>,
        rules: &mut Vec<CssRule>,
        all_styles: &[StyleDecl],
        component_variants: &[Variant],
    ) -> CssResult<()> {
        match element {
            Element::Tag {
                tag_name,
                styles,
                children,
                span,
                ..
            } => {
                // Generate class name using AST ID
                let class_name =
                    get_style_namespace(Some(tag_name.as_str()), &span.id, component_name);

                // Collect styles from style blocks
                if !styles.is_empty() {
                    // Separate base styles from variant styles
                    let mut base_properties = HashMap::new();
                    let mut variant_styles: HashMap<Vec<String>, HashMap<String, String>> = HashMap::new();

                    for style_block in styles {
                        let mut properties = HashMap::new();

                        // Handle extends - pull in CSS variables from extended styles
                        for extend_ref in &style_block.extends {
                            if let Some(extended_style) =
                                all_styles.iter().find(|s| &s.name == extend_ref)
                            {
                                // Pull in the CSS variables from the extended style
                                for (property, value) in &extended_style.properties {
                                    let var_name = format!(
                                        "--{}-{}-{}",
                                        extended_style.name, property, extended_style.span.id
                                    );
                                    let resolved_value = self.resolve_value(value)?;
                                    // Reference the variable with fallback
                                    let var_value =
                                        format!("var({}, {})", var_name, resolved_value);
                                    properties.insert(property.clone(), var_value);
                                }
                            }
                        }

                        // Add local properties (can override extended properties)
                        for (key, value) in &style_block.properties {
                            let resolved_value = self.resolve_value(value)?;
                            properties.insert(key.clone(), resolved_value);
                        }

                        // Categorize as base or variant styles
                        if style_block.variants.is_empty() {
                            // Base styles
                            for (key, value) in properties {
                                base_properties.insert(key, value);
                            }
                        } else {
                            // Variant styles
                            variant_styles.entry(style_block.variants.clone())
                                .or_insert_with(HashMap::new)
                                .extend(properties);
                        }
                    }

                    // Generate base CSS rule
                    if !base_properties.is_empty() {
                        rules.push(CssRule {
                            media_query: None,
                            selector: format!(".{}", class_name),
                            properties: base_properties,
                        });
                    }

                    // Generate variant CSS rules
                    for (variant_names, properties) in variant_styles {
                        if properties.is_empty() {
                            continue;
                        }

                        // Resolve trigger selectors for these variants
                        let mut trigger_selectors = Vec::new();

                        for variant_name in &variant_names {
                            // Find the variant definition
                            if let Some(variant_def) = component_variants.iter().find(|v| &v.name == variant_name) {
                                // Resolve all trigger references in this variant
                                for trigger_ref in &variant_def.triggers {
                                    // Check if it's a trigger reference or inline selector
                                    if let Some(trigger_defs) = self.triggers.get(trigger_ref) {
                                        // It's a trigger reference - use its selectors
                                        trigger_selectors.extend(trigger_defs.clone());
                                    } else {
                                        // It's an inline selector
                                        trigger_selectors.push(trigger_ref.clone());
                                    }
                                }
                            } else {
                                // Variant not found, use name as class
                                trigger_selectors.push(format!(".{}", variant_name));
                            }
                        }

                        // Separate media queries from selectors
                        let mut media_queries = Vec::new();
                        let mut regular_selectors = Vec::new();

                        for trigger_selector in &trigger_selectors {
                            if trigger_selector.starts_with('@') {
                                media_queries.push(trigger_selector.clone());
                            } else {
                                regular_selectors.push(trigger_selector.clone());
                            }
                        }

                        // Generate rules for regular selectors
                        if !regular_selectors.is_empty() {
                            for selector in &regular_selectors {
                                let final_selector = if selector.starts_with('.') || selector.starts_with(':') {
                                    format!(".{}{}", class_name, selector)
                                } else {
                                    format!(".{} {}", class_name, selector)
                                };

                                rules.push(CssRule {
                                    selector: final_selector,
                                    properties: properties.clone(),
                                    media_query: None,
                                });
                            }
                        }

                        // Generate rules wrapped in media queries
                        for media_query in &media_queries {
                            rules.push(CssRule {
                                selector: format!(".{}", class_name),
                                properties: properties.clone(),
                                media_query: Some(media_query.clone()),
                            });
                        }

                        // If both exist, generate combined rule (media query + selector)
                        if !media_queries.is_empty() && !regular_selectors.is_empty() {
                            for media_query in &media_queries {
                                for selector in &regular_selectors {
                                    let final_selector = if selector.starts_with('.') || selector.starts_with(':') {
                                        format!(".{}{}", class_name, selector)
                                    } else {
                                        format!(".{} {}", class_name, selector)
                                    };

                                    rules.push(CssRule {
                                        selector: final_selector,
                                        properties: properties.clone(),
                                        media_query: Some(media_query.clone()),
                                    });
                                }
                            }
                        }

                        // If no trigger selectors resolved, fall back to variant name
                        if trigger_selectors.is_empty() {
                            let variant_classes = variant_names.join(".");
                            rules.push(CssRule {
                                selector: format!(".{}.{}", class_name, variant_classes),
                                properties,
                                media_query: None,
                            });
                        }
                    }
                }

                // Recurse into children
                for child in children {
                    self.extract_element_styles(child, component_name, rules, all_styles, component_variants)?;
                }
            }

            Element::Instance { children, .. } => {
                // Component instances - recurse into children
                for child in children {
                    self.extract_element_styles(child, component_name, rules, all_styles, component_variants)?;
                }
            }

            Element::Conditional {
                then_branch,
                else_branch,
                ..
            } => {
                // Extract from both branches
                for child in then_branch {
                    self.extract_element_styles(child, component_name, rules, all_styles, component_variants)?;
                }
                if let Some(else_br) = else_branch {
                    for child in else_br {
                        self.extract_element_styles(child, component_name, rules, all_styles, component_variants)?;
                    }
                }
            }

            Element::Repeat { body, .. } => {
                // Extract from repeat body
                for child in body {
                    self.extract_element_styles(child, component_name, rules, all_styles, component_variants)?;
                }
            }

            Element::Text { styles, span, .. } => {
                // Handle text with styles (wrapped in span at evaluation time)
                if !styles.is_empty() {
                    let class_name =
                        get_style_namespace(Some("span"), &span.id, component_name);

                    let mut base_properties = HashMap::new();
                    for style_block in styles {
                        for (key, value) in &style_block.properties {
                            let resolved_value = self.resolve_value(value)?;
                            base_properties.insert(key.clone(), resolved_value);
                        }
                    }

                    if !base_properties.is_empty() {
                        rules.push(CssRule {
                            media_query: None,
                            selector: format!(".{}", class_name),
                            properties: base_properties,
                        });
                    }
                }
            }

            Element::SlotInsert { .. } => {
                // No styles in slot inserts
            }

            Element::Insert { content, .. } => {
                // Extract styles from insert content
                for child in content {
                    self.extract_element_styles(child, component_name, rules, all_styles, component_variants)?;
                }
            }
        }

        Ok(())
    }

    /// Resolve value (handle token references)
    fn resolve_value(&self, value: &str) -> CssResult<String> {
        // Check if value references a token
        if value.starts_with('{') && value.ends_with('}') {
            let token_name = &value[1..value.len() - 1];
            if let Some(token_value) = self.tokens.get(token_name) {
                Ok(token_value.clone())
            } else {
                Err(CssError::TokenNotFound {
                    name: token_name.to_string(),
                })
            }
        } else {
            Ok(value.to_string())
        }
    }
}

impl Default for CssEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse_with_path;

    #[test]
    fn test_evaluate_simple_component_styles() {
        let source = r#"
            public component Button {
                render button {
                    style {
                        padding: 8px 16px
                        background: #3366FF
                    }
                    text "Click me"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should have one rule for the button
        assert!(css_doc.rules.len() > 0);

        // Find the button rule
        let button_rule = css_doc
            .rules
            .iter()
            .find(|r| r.selector.contains("button"))
            .expect("Should have button rule");

        assert_eq!(
            button_rule.properties.get("padding"),
            Some(&"8px 16px".to_string())
        );
        assert_eq!(
            button_rule.properties.get("background"),
            Some(&"#3366FF".to_string())
        );
    }

    #[test]
    fn test_evaluate_with_tokens() {
        let source = r#"
            token primaryColor #FF0000

            public component Button {
                render div {
                    style {
                        background: #FF0000
                    }
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should have registered the token
        assert_eq!(
            evaluator.tokens().get("primaryColor"),
            Some(&"#FF0000".to_string())
        );

        // Find the div rule
        let div_rule = css_doc
            .rules
            .iter()
            .find(|r| r.selector.contains("div"))
            .expect("Should have div rule");

        // Background should be present
        assert_eq!(
            div_rule.properties.get("background"),
            Some(&"#FF0000".to_string())
        );
    }

    #[test]
    fn test_css_document_to_css() {
        let mut css_doc = VirtualCssDocument::new();

        let mut properties = HashMap::new();
        properties.insert("color".to_string(), "red".to_string());
        properties.insert("font-size".to_string(), "16px".to_string());

        css_doc.add_rule(CssRule {
                media_query: None,
            selector: ".button".to_string(),
            properties,
        });

        let css_text = css_doc.to_css();

        assert!(css_text.contains(".button"));
        assert!(css_text.contains("color: red"));
        assert!(css_text.contains("font-size: 16px"));
    }

    #[test]
    fn test_evaluate_global_styles() {
        let source = r#"
            public style ButtonStyle {
                padding: 8px
                background: blue
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should have 2 rules: :root with variables + class rule
        assert_eq!(css_doc.rules.len(), 2);

        // First rule should be :root with CSS variables
        assert_eq!(css_doc.rules[0].selector, ":root");
        assert!(css_doc.rules[0].properties.len() >= 2); // padding and background variables

        // Second rule should be the namespaced class
        assert!(css_doc.rules[1].selector.starts_with("._ButtonStyle-"));
        assert!(css_doc.rules[1].selector.contains(evaluator.document_id()));
        // Properties should use var() with fallbacks
        let padding = css_doc.rules[1].properties.get("padding").unwrap();
        assert!(padding.starts_with("var(--ButtonStyle-padding-"));
    }

    #[test]
    fn test_css_variable_extends() {
        // Test that style extends generates CSS variables for instant theme updates
        let source = r#"
            public style fontRegular {
                font-family: Helvetica
                font-weight: 600
            }

            public component Button {
                render button {
                    style extends fontRegular {
                        padding: 8px
                    }
                    text "Click"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should have:
        // 1. :root rule with fontRegular variables
        // 2. .fontRegular class rule
        // 3. .Button-button class rule (that uses fontRegular variables)
        assert!(css_doc.rules.len() >= 3);

        // Find the :root rule with fontRegular variables
        let root_rule = css_doc
            .rules
            .iter()
            .find(|r| {
                r.selector == ":root" && r.properties.keys().any(|k| k.contains("fontRegular"))
            })
            .expect("Should have :root rule with fontRegular variables");

        // Check that CSS variables exist
        assert!(root_rule
            .properties
            .keys()
            .any(|k| k.contains("--fontRegular-font-family-")));
        assert!(root_rule
            .properties
            .keys()
            .any(|k| k.contains("--fontRegular-font-weight-")));

        // Find the button rule
        let button_rule = css_doc
            .rules
            .iter()
            .find(|r| r.selector.contains("Button") && r.selector.contains("button"))
            .expect("Should have button rule");

        // Button should use var() references to fontRegular variables
        let font_family = button_rule
            .properties
            .get("font-family")
            .expect("Button should have font-family");
        assert!(font_family.starts_with("var(--fontRegular-font-family-"));
        assert!(font_family.contains("Helvetica")); // Fallback value

        let font_weight = button_rule
            .properties
            .get("font-weight")
            .expect("Button should have font-weight");
        assert!(font_weight.starts_with("var(--fontRegular-font-weight-"));
        assert!(font_weight.contains("600")); // Fallback value

        // Button should also have its own padding
        assert_eq!(
            button_rule.properties.get("padding"),
            Some(&"8px".to_string())
        );
    }

    #[test]
    fn test_nested_element_styles() {
        let source = r#"
            public component Card {
                render div {
                    style {
                        padding: 16px
                    }
                    div {
                        style {
                            margin: 8px
                        }
                        text "Content"
                    }
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id("/test.pc");
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Should have rules for both divs
        assert!(css_doc.rules.len() >= 2);

        // Verify nested structure in selectors
        let has_nested_selectors = css_doc
            .rules
            .iter()
            .any(|r| r.selector.contains("Card") && r.selector.contains("div"));

        assert!(has_nested_selectors, "Should have nested selectors");
    }

    #[test]
    fn test_document_id_in_class_names() {
        let source = r#"
            public style ButtonStyle {
                padding: 8px
                background: blue
            }

            public component Button {
                render button {
                    style {
                        color: red
                    }
                    text "Click"
                }
            }
        "#;

        let path = "/entry.pc";
        let doc = parse_with_path(source, path).expect("Failed to parse");
        let mut evaluator = CssEvaluator::with_document_id(path);
        let css_doc = evaluator.evaluate(&doc).expect("Failed to evaluate");

        // Get the document ID that should be in all class names
        let doc_id = paperclip_parser::get_document_id(path);
        println!("Document ID: {}", doc_id);

        // Print all selectors for debugging
        for rule in &css_doc.rules {
            println!("Selector: {}", rule.selector);
        }

        // Find the ButtonStyle class rule (not :root)
        let button_style_rule = css_doc
            .rules
            .iter()
            .find(|r| r.selector.contains("ButtonStyle") && r.selector != ":root")
            .expect("Should have ButtonStyle class rule");

        // Verify it contains the document ID
        assert!(
            button_style_rule.selector.contains(&doc_id),
            "ButtonStyle selector '{}' should contain document ID '{}'",
            button_style_rule.selector,
            doc_id
        );

        // Verify format: _ButtonStyle-{docId}-{seqNum}
        assert!(
            button_style_rule.selector.starts_with("._ButtonStyle-"),
            "ButtonStyle should start with '._ButtonStyle-'"
        );

        // Find the button element rule
        let button_element_rule = css_doc
            .rules
            .iter()
            .find(|r| r.selector.contains("Button") && r.selector.contains("button"))
            .expect("Should have button element rule");

        // Verify it contains the document ID
        assert!(
            button_element_rule.selector.contains(&doc_id),
            "Button element selector '{}' should contain document ID '{}'",
            button_element_rule.selector,
            doc_id
        );

        // Verify format: _Button-button-{docId}-{seqNum}
        assert!(
            button_element_rule.selector.starts_with("._Button-button-"),
            "Button element should start with '._Button-button-'"
        );

        println!("\n✓ All class names properly include document ID");
        println!("✓ Public styles are namespaced (not global)");
        println!("✓ Component elements are namespaced");
    }
}
