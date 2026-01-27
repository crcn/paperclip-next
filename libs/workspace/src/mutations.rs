//! Mutation application to AST

use paperclip_proto::ast::*;
use crate::proto::Mutation;

/// Apply a mutation to a document
pub fn apply_mutation(doc: &mut Document, mutation: &Mutation) -> Result<(), String> {
    match mutation {
        Mutation::InsertNode { parent_path, index, node_type, props } => {
            insert_node(doc, parent_path, *index, node_type, props)
        }
        Mutation::RemoveNode { path } => {
            remove_node(doc, path)
        }
        Mutation::UpdateNode { path, props } => {
            update_node(doc, path, props)
        }
        Mutation::MoveNode { from_path, to_parent, to_index } => {
            move_node(doc, from_path, to_parent, *to_index)
        }
        Mutation::UpdateStyle { element_path, property, value } => {
            update_style(doc, element_path, property, value)
        }
    }
}

fn insert_node(
    _doc: &mut Document,
    _parent_path: &[usize],
    _index: usize,
    _node_type: &str,
    _props: &serde_json::Value,
) -> Result<(), String> {
    // TODO: Implement node insertion
    // This requires navigating to the parent and inserting a new child
    Ok(())
}

fn remove_node(_doc: &mut Document, _path: &[usize]) -> Result<(), String> {
    // TODO: Implement node removal
    Ok(())
}

fn update_node(
    _doc: &mut Document,
    _path: &[usize],
    _props: &serde_json::Value,
) -> Result<(), String> {
    // TODO: Implement node update
    Ok(())
}

fn move_node(
    _doc: &mut Document,
    _from_path: &[usize],
    _to_parent: &[usize],
    _to_index: usize,
) -> Result<(), String> {
    // TODO: Implement node move
    Ok(())
}

fn update_style(
    doc: &mut Document,
    element_path: &[usize],
    property: &str,
    value: &str,
) -> Result<(), String> {
    // Navigate to the component and element
    if element_path.is_empty() {
        return Err("Empty element path".to_string());
    }
    
    let component_idx = element_path[0];
    if component_idx >= doc.items.len() {
        return Err("Component index out of bounds".to_string());
    }
    
    let item = &mut doc.items[component_idx];
    let component = match &mut item.node {
        Item::Component(c) => c,
        _ => return Err("Not a component".to_string()),
    };
    
    // Find the render node
    let render = component.render.as_mut()
        .ok_or_else(|| "No render block".to_string())?;
    
    // Navigate through the element path
    let mut current = &mut render.node;
    for &idx in &element_path[1..] {
        current = match current {
            RenderNode::Element(el) => {
                if idx >= el.children.len() {
                    return Err("Child index out of bounds".to_string());
                }
                &mut el.children[idx].node
            }
            _ => return Err("Not an element".to_string()),
        };
    }
    
    // Update the style
    if let RenderNode::Element(el) = current {
        // Find or create the style block
        if el.styles.is_empty() {
            // Create a new style block
            el.styles.push(Spanned::new(
                StyleBlock {
                    extends: vec![],
                    variant: None,
                    declarations: vec![],
                },
                Span::new(0, 0), // Will be updated during serialization
            ));
        }
        
        let style_block = &mut el.styles[0].node;
        
        // Find existing declaration or add new one
        let mut found = false;
        for decl in &mut style_block.declarations {
            if decl.node.property.node == property {
                decl.node.value = Spanned::new(
                    parse_style_value(value),
                    Span::new(0, 0),
                );
                found = true;
                break;
            }
        }
        
        if !found {
            style_block.declarations.push(Spanned::new(
                StyleDeclaration {
                    property: Spanned::new(property.to_string(), Span::new(0, 0)),
                    value: Spanned::new(parse_style_value(value), Span::new(0, 0)),
                },
                Span::new(0, 0),
            ));
        }
    }
    
    Ok(())
}

fn parse_style_value(value: &str) -> StyleValue {
    let trimmed = value.trim();
    
    // Check for color
    if trimmed.starts_with('#') {
        return StyleValue::Color(trimmed.to_string());
    }
    
    // Check for dimension
    let units = ["px", "em", "rem", "%", "vh", "vw", "vmin", "vmax"];
    for unit in units {
        if trimmed.ends_with(unit) {
            if let Ok(num) = trimmed[..trimmed.len() - unit.len()].parse::<f64>() {
                return StyleValue::Dimension(num, unit.to_string());
            }
        }
    }
    
    // Check for number
    if let Ok(num) = trimmed.parse::<f64>() {
        return StyleValue::Number(num);
    }
    
    // Default to keyword
    StyleValue::Keyword(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_style_value() {
        assert!(matches!(parse_style_value("#3366FF"), StyleValue::Color(_)));
        assert!(matches!(parse_style_value("16px"), StyleValue::Dimension(16.0, _)));
        assert!(matches!(parse_style_value("1.5rem"), StyleValue::Dimension(1.5, _)));
        assert!(matches!(parse_style_value("flex"), StyleValue::Keyword(_)));
    }
}
