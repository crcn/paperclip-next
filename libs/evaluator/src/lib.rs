//! Paperclip Evaluator
//!
//! Transforms AST into Virtual DOM for preview rendering.
//!
//! Performance targets:
//! - Evaluate medium component in <20ms
//! - Support incremental evaluation (only re-evaluate changed nodes)
//! - Cache resolved token values

pub mod html;
pub mod css;
pub mod context;
pub mod graph;

use paperclip_parser::parse;
use paperclip_proto::ast::Document;
use paperclip_proto::virt::{EvaluatedModule, EvaluatedComponent, VirtualCSS};
use thiserror::Error;

pub use context::EvalContext;
pub use graph::GraphManager;

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Undefined reference: {0}")]
    UndefinedReference(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Evaluation error: {0}")]
    EvalError(String),
}

pub type EvalResult<T> = Result<T, EvalError>;

/// Evaluate a .pc file source to Virtual DOM
pub fn evaluate(source: &str, path: &str) -> EvalResult<EvaluatedModule> {
    // Parse source to AST
    let document = parse(source).map_err(|e| {
        EvalError::ParseError(format!("{} errors", e.len()))
    })?;
    
    // Create evaluation context
    let mut ctx = EvalContext::new(path);
    
    // Evaluate document
    evaluate_document(&document, &mut ctx)
}

/// Evaluate a parsed document
pub fn evaluate_document(doc: &Document, ctx: &mut EvalContext) -> EvalResult<EvaluatedModule> {
    let mut components = Vec::new();
    let mut css_rules = Vec::new();
    
    for item in &doc.items {
        match &item.node {
            paperclip_proto::ast::Item::Import(import) => {
                ctx.add_import(&import.path.node, import.alias.as_ref().map(|a| a.node.as_str()));
            }
            paperclip_proto::ast::Item::Token(token) => {
                ctx.add_token(&token.name.node, &token.value.node);
            }
            paperclip_proto::ast::Item::Style(style) => {
                ctx.add_style(&style.name.node, &style.declarations);
                // Generate CSS for style definition
                if let Some(rule) = css::evaluate_style_definition(style, ctx) {
                    css_rules.push(rule);
                }
            }
            paperclip_proto::ast::Item::Component(component) => {
                let evaluated = html::evaluate_component(component, ctx)?;
                
                // Generate CSS for component
                let component_css = css::evaluate_component_css(component, ctx);
                css_rules.extend(component_css);
                
                components.push(evaluated);
            }
        }
    }
    
    Ok(EvaluatedModule {
        path: ctx.path.to_string(),
        components,
        css: VirtualCSS { rules: css_rules },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_simple_component() {
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
        
        let result = evaluate(source, "button.pc").unwrap();
        
        assert_eq!(result.components.len(), 1);
        assert_eq!(result.components[0].name, "Button");
    }

    #[test]
    fn test_evaluate_with_tokens() {
        let source = r#"
public token primary #3366FF
public token spacing 16px

public component Card {
    render div {
        style {
            padding: var(spacing)
            background: var(primary)
        }
    }
}
"#;
        
        let result = evaluate(source, "card.pc").unwrap();
        
        assert_eq!(result.components.len(), 1);
        // CSS should have resolved token values
        assert!(!result.css.rules.is_empty());
    }
}
