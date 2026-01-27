//! CSS evaluation - AST styles to Virtual CSS

use paperclip_proto::ast::*;
use paperclip_proto::virt::CSSRule;
use crate::context::EvalContext;

/// Evaluate a style definition to CSS rule
pub fn evaluate_style_definition(style: &StyleDefinition, ctx: &mut EvalContext) -> Option<CSSRule> {
    if !style.is_public {
        return None; // Private styles are only used as mixins
    }
    
    let class_name = format!(".pc-{}", style.name.node);
    let declarations = evaluate_declarations(&style.declarations, ctx);
    
    Some(CSSRule {
        selector: class_name,
        declarations,
        media: None,
    })
}

/// Evaluate a component's styles to CSS rules
pub fn evaluate_component_css(component: &Component, ctx: &mut EvalContext) -> Vec<CSSRule> {
    let mut rules = Vec::new();
    
    if let Some(render) = &component.render {
        collect_element_css(&render.node, &component.name.node, &mut rules, ctx);
    }
    
    rules
}

fn collect_element_css(
    node: &RenderNode,
    component_name: &str,
    rules: &mut Vec<CSSRule>,
    ctx: &mut EvalContext,
) {
    match node {
        RenderNode::Element(el) => {
            // Process styles on this element
            for (i, style_block) in el.styles.iter().enumerate() {
                let class_name = format!(".pc-{}-{}", component_name.to_lowercase(), i + 1);
                
                let mut declarations = Vec::new();
                
                // First, include extended styles
                for extend_name in &style_block.node.extends {
                    if let Some(base_decls) = ctx.get_style(&extend_name.node) {
                        for decl in base_decls {
                            let (prop, value) = evaluate_declaration(&decl.node, ctx);
                            declarations.push((prop, value));
                        }
                    }
                }
                
                // Then add this block's declarations (may override)
                for decl in &style_block.node.declarations {
                    let (prop, value) = evaluate_declaration(&decl.node, ctx);
                    declarations.push((prop, value));
                }
                
                let media = style_block.node.variant.as_ref().and_then(|v| {
                    // Check if variant has a media query trigger
                    // TODO: look up variant definition
                    if v.node.starts_with('@') {
                        Some(v.node.clone())
                    } else {
                        None
                    }
                });
                
                // Handle variant selector (e.g., :hover)
                let selector = if let Some(variant) = &style_block.node.variant {
                    // Check if it's a pseudo-class
                    if variant.node.starts_with(':') || variant.node == "hover" || variant.node == "focus" {
                        format!("{}:{}", class_name, variant.node.trim_start_matches(':'))
                    } else {
                        class_name.clone()
                    }
                } else {
                    class_name
                };
                
                rules.push(CSSRule {
                    selector,
                    declarations,
                    media,
                });
            }
            
            // Recurse into children
            for child in &el.children {
                collect_element_css(&child.node, component_name, rules, ctx);
            }
        }
        RenderNode::Slot(slot) => {
            for child in &slot.default {
                collect_element_css(&child.node, component_name, rules, ctx);
            }
        }
        RenderNode::Condition(cond) => {
            for child in &cond.then_branch {
                collect_element_css(&child.node, component_name, rules, ctx);
            }
            if let Some(else_branch) = &cond.else_branch {
                for child in else_branch {
                    collect_element_css(&child.node, component_name, rules, ctx);
                }
            }
        }
        RenderNode::Repeat(repeat) => {
            for child in &repeat.body {
                collect_element_css(&child.node, component_name, rules, ctx);
            }
        }
        _ => {}
    }
}

fn evaluate_declarations(declarations: &[Spanned<StyleDeclaration>], ctx: &mut EvalContext) -> Vec<(String, String)> {
    declarations.iter()
        .map(|decl| evaluate_declaration(&decl.node, ctx))
        .collect()
}

fn evaluate_declaration(decl: &StyleDeclaration, ctx: &mut EvalContext) -> (String, String) {
    let property = decl.property.node.clone();
    let value = evaluate_style_value(&decl.value.node, ctx);
    (property, value)
}

fn evaluate_style_value(value: &StyleValue, ctx: &mut EvalContext) -> String {
    match value {
        StyleValue::Keyword(k) => k.clone(),
        StyleValue::Color(c) => c.clone(),
        StyleValue::Dimension(n, u) => format!("{}{}", n, u),
        StyleValue::Number(n) => n.to_string(),
        StyleValue::String(s) => format!("\"{}\"", s),
        StyleValue::Reference(name) => {
            // Resolve token reference
            if let Some(token) = ctx.resolve_token(name) {
                token.value.clone()
            } else {
                format!("var(--{})", name)
            }
        }
        StyleValue::Function(name, args) => {
            let args_str: Vec<String> = args.iter()
                .map(|a| evaluate_style_value(a, ctx))
                .collect();
            format!("{}({})", name, args_str.join(", "))
        }
        StyleValue::List(values) => {
            let values_str: Vec<String> = values.iter()
                .map(|v| evaluate_style_value(v, ctx))
                .collect();
            values_str.join(" ")
        }
    }
}

/// Serialize CSS rules to a string
pub fn serialize_css(css: &[CSSRule]) -> String {
    let mut output = String::new();
    
    for rule in css {
        if let Some(media) = &rule.media {
            output.push_str(&format!("@media {} {{\n", media));
        }
        
        output.push_str(&rule.selector);
        output.push_str(" {\n");
        
        for (prop, value) in &rule.declarations {
            output.push_str(&format!("  {}: {};\n", prop, value));
        }
        
        output.push_str("}\n");
        
        if rule.media.is_some() {
            output.push_str("}\n");
        }
        
        output.push('\n');
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_style_value_color() {
        let mut ctx = EvalContext::new("test.pc");
        let value = StyleValue::Color("#3366FF".to_string());
        
        assert_eq!(evaluate_style_value(&value, &mut ctx), "#3366FF");
    }

    #[test]
    fn test_evaluate_style_value_token_reference() {
        let mut ctx = EvalContext::new("test.pc");
        ctx.add_token("primary", &TokenValue::Color("#3366FF".to_string()));
        
        let value = StyleValue::Reference("primary".to_string());
        
        assert_eq!(evaluate_style_value(&value, &mut ctx), "#3366FF");
    }

    #[test]
    fn test_serialize_css() {
        let rules = vec![
            CSSRule {
                selector: ".button".to_string(),
                declarations: vec![
                    ("padding".to_string(), "8px 16px".to_string()),
                    ("background".to_string(), "#3366FF".to_string()),
                ],
                media: None,
            }
        ];
        
        let css = serialize_css(&rules);
        
        assert!(css.contains(".button"));
        assert!(css.contains("padding: 8px 16px"));
        assert!(css.contains("background: #3366FF"));
    }
}
