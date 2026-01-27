//! AST serializer for roundtrip source generation
//!
//! SPIKE 0.4: Roundtrip Serialization
//!
//! The goal is to serialize an AST back to source code while:
//! 1. Preserving original formatting where possible
//! 2. Only changing the parts that were modified
//!
//! Strategy:
//! - Use spans to identify unchanged regions
//! - Copy unchanged text from original source
//! - Generate new text for modified nodes

use paperclip_proto::ast::*;

/// Serialize a document to source code
/// 
/// Uses the original source to preserve formatting for unchanged nodes
pub fn serialize_document(doc: &Document, original: &str) -> String {
    let mut output = String::new();
    let mut last_end = 0;
    
    for item in &doc.items {
        // If this item has a valid span and wasn't modified, copy from original
        if item.span.start >= last_end && item.span.end <= original.len() {
            // Copy any whitespace/comments between items
            if item.span.start > last_end {
                output.push_str(&original[last_end..item.span.start]);
            }
            
            // For now, always serialize fresh (TODO: detect unchanged nodes)
            output.push_str(&serialize_item(&item.node));
            last_end = item.span.end;
        } else {
            // New item, generate fresh
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(&serialize_item(&item.node));
        }
    }
    
    // Copy any trailing content
    if last_end < original.len() {
        output.push_str(&original[last_end..]);
    }
    
    output
}

fn serialize_item(item: &Item) -> String {
    match item {
        Item::Import(import) => serialize_import(import),
        Item::Token(token) => serialize_token(token),
        Item::Style(style) => serialize_style_def(style),
        Item::Component(component) => serialize_component(component),
    }
}

fn serialize_import(import: &Import) -> String {
    let mut s = format!("import \"{}\"", import.path.node);
    if let Some(alias) = &import.alias {
        s.push_str(&format!(" as {}", alias.node));
    }
    s.push('\n');
    s
}

fn serialize_token(token: &Token) -> String {
    let mut s = String::new();
    if token.is_public {
        s.push_str("public ");
    }
    s.push_str("token ");
    s.push_str(&token.name.node);
    s.push(' ');
    s.push_str(&serialize_token_value(&token.value.node));
    s.push('\n');
    s
}

fn serialize_token_value(value: &TokenValue) -> String {
    match value {
        TokenValue::Color(c) => c.clone(),
        TokenValue::Dimension(n, u) => format!("{}{}", n, u),
        TokenValue::String(s) => format!("\"{}\"", s),
        TokenValue::Number(n) => n.to_string(),
        TokenValue::Reference(r) => format!("var({})", r),
    }
}

fn serialize_style_def(style: &StyleDefinition) -> String {
    let mut s = String::new();
    if style.is_public {
        s.push_str("public ");
    }
    s.push_str("style ");
    s.push_str(&style.name.node);
    s.push_str(" {\n");
    
    for decl in &style.declarations {
        s.push_str("    ");
        s.push_str(&decl.node.property.node);
        s.push_str(": ");
        s.push_str(&serialize_style_value(&decl.node.value.node));
        s.push('\n');
    }
    
    s.push_str("}\n");
    s
}

fn serialize_component(component: &Component) -> String {
    let mut s = String::new();
    
    // Doc comment with @frame
    if let Some(doc) = &component.doc_comment {
        if let Some(frame) = &doc.frame {
            s.push_str(&format!(
                "/** @frame(x: {}, y: {}, width: {}",
                frame.x, frame.y, frame.width
            ));
            if let Some(height) = frame.height {
                s.push_str(&format!(", height: {}", height));
            }
            s.push_str(") */\n");
        }
    }
    
    if component.is_public {
        s.push_str("public ");
    }
    s.push_str("component ");
    s.push_str(&component.name.node);
    s.push_str(" {\n");
    
    // Variants
    for variant in &component.variants {
        s.push_str("    variant ");
        s.push_str(&variant.node.name.node);
        if !variant.node.triggers.is_empty() {
            s.push_str(" trigger { ");
            let triggers: Vec<_> = variant.node.triggers.iter()
                .map(|t| format!("\"{}\"", t.node))
                .collect();
            s.push_str(&triggers.join(", "));
            s.push_str(" }");
        }
        s.push('\n');
    }
    
    // Render
    if let Some(render) = &component.render {
        s.push_str("    render ");
        s.push_str(&serialize_render_node(&render.node, 1));
    }
    
    s.push_str("}\n");
    s
}

fn serialize_render_node(node: &RenderNode, indent: usize) -> String {
    let indent_str = "    ".repeat(indent);
    
    match node {
        RenderNode::Element(el) => serialize_element(el, indent),
        RenderNode::Text(text) => {
            let content: String = text.parts.iter()
                .map(|p| match p {
                    TextPart::Literal(s) => s.clone(),
                    TextPart::Expression(e) => format!("{{{}}}", serialize_expression(e)),
                })
                .collect();
            format!("{}text \"{}\"\n", indent_str, content)
        }
        RenderNode::Slot(slot) => {
            let mut s = format!("{}slot {}", indent_str, slot.name.node);
            if !slot.default.is_empty() {
                s.push_str(" {\n");
                for child in &slot.default {
                    s.push_str(&serialize_render_node(&child.node, indent + 1));
                }
                s.push_str(&format!("{}}}", indent_str));
            }
            s.push('\n');
            s
        }
        RenderNode::Insert(insert) => {
            let mut s = format!("{}insert {} {{\n", indent_str, insert.slot_name.node);
            for child in &insert.children {
                s.push_str(&serialize_render_node(&child.node, indent + 1));
            }
            s.push_str(&format!("{}}}\n", indent_str));
            s
        }
        RenderNode::Condition(cond) => {
            let mut s = format!("{}if {} {{\n", indent_str, serialize_expression(&cond.condition.node));
            for child in &cond.then_branch {
                s.push_str(&serialize_render_node(&child.node, indent + 1));
            }
            s.push_str(&format!("{}}}", indent_str));
            if let Some(else_branch) = &cond.else_branch {
                s.push_str(" else {\n");
                for child in else_branch {
                    s.push_str(&serialize_render_node(&child.node, indent + 1));
                }
                s.push_str(&format!("{}}}", indent_str));
            }
            s.push('\n');
            s
        }
        RenderNode::Repeat(repeat) => {
            let mut s = format!(
                "{}repeat {} as {}",
                indent_str,
                serialize_expression(&repeat.source.node),
                repeat.iterator.node
            );
            if let Some(index) = &repeat.index {
                s.push_str(&format!(", {}", index.node));
            }
            s.push_str(" {\n");
            for child in &repeat.body {
                s.push_str(&serialize_render_node(&child.node, indent + 1));
            }
            s.push_str(&format!("{}}}", indent_str));
            if let Some(empty) = &repeat.empty {
                s.push_str(" empty {\n");
                for child in empty {
                    s.push_str(&serialize_render_node(&child.node, indent + 1));
                }
                s.push_str(&format!("{}}}", indent_str));
            }
            s.push('\n');
            s
        }
        RenderNode::ComponentInstance(inst) => {
            let mut s = indent_str.clone();
            if let Some(ns) = &inst.namespace {
                s.push_str(&ns.node);
                s.push('.');
            }
            s.push_str(&inst.name.node);
            if !inst.props.is_empty() {
                s.push('(');
                let props: Vec<_> = inst.props.iter()
                    .map(|p| format!("{}={}", p.node.name.node, serialize_expression(&p.node.value.node)))
                    .collect();
                s.push_str(&props.join(", "));
                s.push(')');
            }
            if !inst.inserts.is_empty() {
                s.push_str(" {\n");
                for insert in &inst.inserts {
                    s.push_str(&format!("{}insert {} {{\n", "    ".repeat(indent + 1), insert.node.slot_name.node));
                    for child in &insert.node.children {
                        s.push_str(&serialize_render_node(&child.node, indent + 2));
                    }
                    s.push_str(&format!("{}}}\n", "    ".repeat(indent + 1)));
                }
                s.push_str(&format!("{}}}", indent_str));
            }
            s.push('\n');
            s
        }
    }
}

fn serialize_element(el: &Element, indent: usize) -> String {
    let indent_str = "    ".repeat(indent);
    let mut s = format!("{}{}", indent_str, el.tag.node);
    
    // Check if we have content
    let has_content = !el.attributes.is_empty() || !el.styles.is_empty() || !el.children.is_empty();
    
    if has_content {
        s.push_str(" {\n");
        
        // Attributes
        for attr in &el.attributes {
            s.push_str(&format!(
                "{}    {}={}\n",
                indent_str,
                attr.node.name.node,
                serialize_attribute_value(&attr.node.value.node)
            ));
        }
        
        // Styles
        for style_block in &el.styles {
            s.push_str(&serialize_style_block(&style_block.node, indent + 1));
        }
        
        // Children
        for child in &el.children {
            s.push_str(&serialize_render_node(&child.node, indent + 1));
        }
        
        s.push_str(&format!("{}}}\n", indent_str));
    } else {
        s.push('\n');
    }
    
    s
}

fn serialize_attribute_value(value: &AttributeValue) -> String {
    match value {
        AttributeValue::String(s) => format!("\"{}\"", s),
        AttributeValue::Boolean(b) => b.to_string(),
        AttributeValue::Expression(e) => format!("{{{}}}", serialize_expression(e)),
    }
}

fn serialize_style_block(block: &StyleBlock, indent: usize) -> String {
    let indent_str = "    ".repeat(indent);
    let mut s = format!("{}style", indent_str);
    
    // Extends
    if !block.extends.is_empty() {
        s.push_str(" extends ");
        let names: Vec<_> = block.extends.iter().map(|e| e.node.clone()).collect();
        s.push_str(&names.join(", "));
    }
    
    // Variant
    if let Some(variant) = &block.variant {
        s.push_str(" variant ");
        s.push_str(&variant.node);
    }
    
    s.push_str(" {\n");
    
    for decl in &block.declarations {
        s.push_str(&format!(
            "{}    {}: {}\n",
            indent_str,
            decl.node.property.node,
            serialize_style_value(&decl.node.value.node)
        ));
    }
    
    s.push_str(&format!("{}}}\n", indent_str));
    s
}

fn serialize_style_value(value: &StyleValue) -> String {
    match value {
        StyleValue::Keyword(k) => k.clone(),
        StyleValue::Color(c) => c.clone(),
        StyleValue::Dimension(n, u) => format!("{}{}", n, u),
        StyleValue::Number(n) => n.to_string(),
        StyleValue::String(s) => format!("\"{}\"", s),
        StyleValue::Reference(r) => format!("var({})", r),
        StyleValue::Function(name, args) => {
            let args_str: Vec<_> = args.iter().map(serialize_style_value).collect();
            format!("{}({})", name, args_str.join(", "))
        }
        StyleValue::List(values) => {
            let values_str: Vec<_> = values.iter().map(serialize_style_value).collect();
            values_str.join(" ")
        }
    }
}

fn serialize_expression(expr: &Expression) -> String {
    match expr {
        Expression::Identifier(name) => name.clone(),
        Expression::Number(n) => n.to_string(),
        Expression::String(s) => format!("\"{}\"", s),
        Expression::Boolean(b) => b.to_string(),
        Expression::MemberAccess(left, member) => {
            format!("{}.{}", serialize_expression(&left.node), member.node)
        }
        Expression::FunctionCall(name, args) => {
            let args_str: Vec<_> = args.iter()
                .map(|a| serialize_expression(&a.node))
                .collect();
            format!("{}({})", name.node, args_str.join(", "))
        }
        Expression::BinaryOp(left, op, right) => {
            let op_str = match op {
                BinaryOperator::Add => "+",
                BinaryOperator::Sub => "-",
                BinaryOperator::Mul => "*",
                BinaryOperator::Div => "/",
                BinaryOperator::Eq => "==",
                BinaryOperator::NotEq => "!=",
                BinaryOperator::Lt => "<",
                BinaryOperator::Lte => "<=",
                BinaryOperator::Gt => ">",
                BinaryOperator::Gte => ">=",
                BinaryOperator::And => "&&",
                BinaryOperator::Or => "||",
            };
            format!(
                "{} {} {}",
                serialize_expression(&left.node),
                op_str,
                serialize_expression(&right.node)
            )
        }
        Expression::UnaryOp(op, right) => {
            let op_str = match op {
                UnaryOperator::Not => "!",
                UnaryOperator::Neg => "-",
            };
            format!("{}{}", op_str, serialize_expression(&right.node))
        }
        Expression::Ternary(cond, then_expr, else_expr) => {
            format!(
                "{} ? {} : {}",
                serialize_expression(&cond.node),
                serialize_expression(&then_expr.node),
                serialize_expression(&else_expr.node)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse;

    #[test]
    fn test_roundtrip_simple() {
        let source = r#"public component Button {
    render button {
        text "Click me"
    }
}
"#;
        
        let doc = parse(source).unwrap();
        let serialized = serialize_document(&doc, source);
        
        // Parse again and verify it's valid
        let doc2 = parse(&serialized).unwrap();
        assert_eq!(doc.items.len(), doc2.items.len());
    }

    #[test]
    fn test_roundtrip_with_styles() {
        let source = r#"public component Card {
    render div {
        style {
            padding: 16px
            background: #ffffff
        }
    }
}
"#;
        
        let doc = parse(source).unwrap();
        let serialized = serialize_document(&doc, source);
        
        // Verify contains the style properties
        assert!(serialized.contains("padding:"));
        assert!(serialized.contains("background:"));
    }
}
