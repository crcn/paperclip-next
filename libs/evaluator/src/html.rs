//! HTML evaluation - AST to Virtual HTML

use paperclip_proto::ast::*;
use paperclip_proto::virt::*;
use crate::context::EvalContext;
use crate::EvalResult;

/// Evaluate a component to Virtual DOM
pub fn evaluate_component(component: &Component, ctx: &mut EvalContext) -> EvalResult<EvaluatedComponent> {
    ctx.current_component = Some(component.name.node.clone());
    
    // Extract frame from doc comment
    let frame = component.doc_comment.as_ref()
        .and_then(|dc| dc.frame.as_ref())
        .map(|f| Frame {
            x: f.x,
            y: f.y,
            width: f.width,
            height: f.height,
        });
    
    // Evaluate render tree
    let vdom = if let Some(render) = &component.render {
        evaluate_render_node(&render.node, ctx)?
    } else {
        VirtualNode::Fragment(vec![])
    };
    
    ctx.current_component = None;
    
    Ok(EvaluatedComponent {
        name: component.name.node.clone(),
        frame,
        vdom,
    })
}

/// Evaluate a render node
pub fn evaluate_render_node(node: &RenderNode, ctx: &mut EvalContext) -> EvalResult<VirtualNode> {
    match node {
        RenderNode::Element(el) => evaluate_element(el, ctx),
        RenderNode::Text(text) => evaluate_text(text, ctx),
        RenderNode::Slot(slot) => evaluate_slot(slot, ctx),
        RenderNode::Insert(insert) => evaluate_insert(insert, ctx),
        RenderNode::Condition(cond) => evaluate_condition(cond, ctx),
        RenderNode::Repeat(repeat) => evaluate_repeat(repeat, ctx),
        RenderNode::ComponentInstance(inst) => evaluate_instance(inst, ctx),
    }
}

fn evaluate_element(el: &Element, ctx: &mut EvalContext) -> EvalResult<VirtualNode> {
    let source_id = ctx.generate_source_id(&el.tag.node);
    
    // Evaluate attributes
    let mut attributes = Vec::new();
    for attr in &el.attributes {
        let value = evaluate_attribute_value(&attr.node.value.node, ctx);
        attributes.push((attr.node.name.node.clone(), value));
    }
    
    // Generate class names from styles
    let class_names: Vec<String> = el.styles.iter()
        .map(|_| ctx.generate_class_name(&el.tag.node))
        .collect();
    
    // Evaluate children
    let children: Vec<VirtualNode> = el.children.iter()
        .map(|child| evaluate_render_node(&child.node, ctx))
        .collect::<EvalResult<Vec<_>>>()?;
    
    Ok(VirtualNode::Element(VirtualElement {
        tag: el.tag.node.clone(),
        source_id,
        attributes,
        class_names,
        inline_styles: vec![], // TODO: handle dynamic styles
        children,
        live_component: None,
    }))
}

fn evaluate_text(text: &TextNode, ctx: &mut EvalContext) -> EvalResult<VirtualNode> {
    let content: String = text.parts.iter()
        .map(|part| match part {
            TextPart::Literal(s) => s.clone(),
            TextPart::Expression(expr) => evaluate_expression_to_string(expr, ctx),
        })
        .collect();
    
    Ok(VirtualNode::Text(content))
}

fn evaluate_slot(slot: &Slot, ctx: &mut EvalContext) -> EvalResult<VirtualNode> {
    // For now, render the default content
    // TODO: proper slot filling
    if slot.default.is_empty() {
        Ok(VirtualNode::Fragment(vec![]))
    } else {
        let children: Vec<VirtualNode> = slot.default.iter()
            .map(|child| evaluate_render_node(&child.node, ctx))
            .collect::<EvalResult<Vec<_>>>()?;
        
        Ok(VirtualNode::Fragment(children))
    }
}

fn evaluate_insert(_insert: &Insert, _ctx: &mut EvalContext) -> EvalResult<VirtualNode> {
    // Insert is only valid within component instances
    // During evaluation, we've already resolved slots
    Ok(VirtualNode::Fragment(vec![]))
}

fn evaluate_condition(cond: &Condition, ctx: &mut EvalContext) -> EvalResult<VirtualNode> {
    // Evaluate the condition
    let result = evaluate_expression_to_bool(&cond.condition.node, ctx);
    
    if result {
        let children: Vec<VirtualNode> = cond.then_branch.iter()
            .map(|child| evaluate_render_node(&child.node, ctx))
            .collect::<EvalResult<Vec<_>>>()?;
        Ok(VirtualNode::Fragment(children))
    } else if let Some(else_branch) = &cond.else_branch {
        let children: Vec<VirtualNode> = else_branch.iter()
            .map(|child| evaluate_render_node(&child.node, ctx))
            .collect::<EvalResult<Vec<_>>>()?;
        Ok(VirtualNode::Fragment(children))
    } else {
        Ok(VirtualNode::Fragment(vec![]))
    }
}

fn evaluate_repeat(repeat: &Repeat, ctx: &mut EvalContext) -> EvalResult<VirtualNode> {
    // For preview, we use sample data
    // TODO: get sample data from context
    
    // For now, just render the body once as a template indicator
    let children: Vec<VirtualNode> = repeat.body.iter()
        .map(|child| evaluate_render_node(&child.node, ctx))
        .collect::<EvalResult<Vec<_>>>()?;
    
    Ok(VirtualNode::Fragment(children))
}

fn evaluate_instance(inst: &ComponentInstance, ctx: &mut EvalContext) -> EvalResult<VirtualNode> {
    let source_id = ctx.generate_source_id(&inst.name.node);
    
    // Check if this is a live component (external React component)
    // For now, render a placeholder
    // TODO: look up component in registry
    
    // Evaluate props
    let props: Vec<(String, String)> = inst.props.iter()
        .map(|prop| {
            let value = evaluate_expression_to_string(&prop.node.value.node, ctx);
            (prop.node.name.node.clone(), value)
        })
        .collect();
    
    let props_json = serde_json::to_string(&props.into_iter().collect::<std::collections::HashMap<_, _>>())
        .unwrap_or_else(|_| "{}".to_string());
    
    // Create placeholder for component
    Ok(VirtualNode::Element(VirtualElement {
        tag: "div".to_string(),
        source_id,
        attributes: vec![
            ("data-component".to_string(), inst.name.node.clone()),
        ],
        class_names: vec![],
        inline_styles: vec![],
        children: vec![VirtualNode::Text(format!("<{} />", inst.name.node))],
        live_component: Some(LiveComponentRef {
            component_id: format!("@local/{}", inst.name.node),
            props: props_json,
        }),
    }))
}

fn evaluate_attribute_value(value: &AttributeValue, ctx: &mut EvalContext) -> String {
    match value {
        AttributeValue::String(s) => s.clone(),
        AttributeValue::Boolean(b) => b.to_string(),
        AttributeValue::Expression(expr) => evaluate_expression_to_string(expr, ctx),
    }
}

fn evaluate_expression_to_string(expr: &Expression, ctx: &mut EvalContext) -> String {
    match expr {
        Expression::String(s) => s.clone(),
        Expression::Number(n) => n.to_string(),
        Expression::Boolean(b) => b.to_string(),
        Expression::Identifier(name) => {
            // Try to resolve as token
            if let Some(token) = ctx.resolve_token(name) {
                token.value.clone()
            } else {
                format!("{{{}}}", name) // Keep as placeholder
            }
        }
        Expression::MemberAccess(left, member) => {
            let left_str = evaluate_expression_to_string(&left.node, ctx);
            format!("{}.{}", left_str, member.node)
        }
        Expression::BinaryOp(left, op, right) => {
            let l = evaluate_expression_to_string(&left.node, ctx);
            let r = evaluate_expression_to_string(&right.node, ctx);
            format!("{} {:?} {}", l, op, r)
        }
        _ => "{expr}".to_string(),
    }
}

fn evaluate_expression_to_bool(expr: &Expression, _ctx: &mut EvalContext) -> bool {
    match expr {
        Expression::Boolean(b) => *b,
        Expression::Number(n) => *n != 0.0,
        Expression::String(s) => !s.is_empty(),
        _ => true, // Default to true for preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_simple_element() {
        let mut ctx = EvalContext::new("test.pc");
        
        let element = Element {
            tag: Spanned::new("div".to_string(), Span::new(0, 3)),
            attributes: vec![],
            styles: vec![],
            children: vec![],
        };
        
        let result = evaluate_element(&element, &mut ctx).unwrap();
        
        match result {
            VirtualNode::Element(el) => {
                assert_eq!(el.tag, "div");
            }
            _ => panic!("Expected element"),
        }
    }
}
