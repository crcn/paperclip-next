use crate::context::{CompileOptions, CompilerContext};
use paperclip_parser::ast::*;
use std::collections::HashMap;

/// Compile a Paperclip document to React code
pub fn compile_to_react(document: &Document, options: CompileOptions) -> Result<String, String> {
    let ctx = CompilerContext::new(options);

    // Generate imports
    compile_imports(&document, &ctx);

    // Generate utility functions
    compile_utilities(&ctx);

    // Compile components
    for component in &document.components {
        compile_component(component, &ctx)?;
    }

    // Export public tokens
    for token in &document.tokens {
        if token.public {
            compile_token(token, &ctx);
        }
    }

    // Export public styles
    for style in &document.styles {
        if style.public {
            compile_style_export(style, &ctx);
        }
    }

    Ok(ctx.get_output())
}

fn compile_imports(document: &Document, ctx: &CompilerContext) {
    // Import CSS file if enabled
    if ctx.options.include_css_imports {
        ctx.add_line("import \"./styles.css\";");
    }

    // Import React
    ctx.add_line("import React from \"react\";");

    // Import from other .pc files
    for import in &document.imports {
        if let Some(alias) = &import.alias {
            ctx.add_line(&format!(
                "import * as {} from \"{}\";",
                alias, import.path
            ));
        } else {
            ctx.add_line(&format!("import \"{}\";", import.path));
        }
    }

    ctx.add("\n");
}

fn compile_utilities(ctx: &CompilerContext) {
    // Add utility functions that React components may need
    ctx.add_line("// Utility function to merge class names");
    ctx.add_line("const cx = (...classes) => classes.filter(Boolean).join(\" \");");
    ctx.add("\n");
}

fn compile_token(token: &TokenDecl, ctx: &CompilerContext) {
    ctx.add_line(&format!(
        "export const {} = \"{}\";",
        token.name, token.value
    ));
}

fn compile_style_export(style: &StyleDecl, ctx: &CompilerContext) {
    // Generate a class name based on the style name
    let class_name = format!("pc-{}", style.name.to_lowercase());
    ctx.add_line(&format!(
        "export const {} = \"{}\";",
        style.name, class_name
    ));
}

fn compile_component(component: &Component, ctx: &CompilerContext) -> Result<(), String> {
    let component_name = &component.name;

    // Start component function
    ctx.add_line(&format!(
        "const _{} = (props, ref) => {{",
        component_name
    ));
    ctx.indent();

    // Extract variants from props if any
    if !component.variants.is_empty() {
        compile_variant_extraction(component, ctx);
    }

    // Extract slots from props if any
    if !component.slots.is_empty() {
        compile_slot_extraction(component, ctx);
    }

    // Render the component body
    if let Some(body) = &component.body {
        ctx.add_line("return (");
        ctx.indent();
        compile_element(body, ctx, true)?;
        ctx.dedent();
        ctx.add_line(");");
    } else {
        ctx.add_line("return null;");
    }

    ctx.dedent();
    ctx.add_line("};");

    // Set display name
    ctx.add_line(&format!("_{}.displayName = \"{}\";", component_name, component_name));

    // Wrap with React.memo and forwardRef
    ctx.add_line(&format!(
        "const {} = React.memo(React.forwardRef(_{}));",
        component_name, component_name
    ));

    // Export if public
    if component.public {
        ctx.add_line(&format!("export {{ {} }};", component_name));
    }

    ctx.add("\n");
    Ok(())
}

fn compile_variant_extraction(component: &Component, ctx: &CompilerContext) {
    ctx.add("  const { ");
    let variant_names: Vec<&str> = component.variants.iter().map(|v| v.name.as_str()).collect();
    ctx.add(&variant_names.join(", "));
    ctx.add(" } = props;\n");
}

fn compile_slot_extraction(component: &Component, ctx: &CompilerContext) {
    for slot in &component.slots {
        ctx.add_line(&format!("  const {} = props.{};", slot.name, slot.name));
    }
}

fn compile_element(element: &Element, ctx: &CompilerContext, is_root: bool) -> Result<(), String> {
    match element {
        Element::Tag {
            tag_name,
            name: _element_name,
            attributes,
            styles,
            children,
            span: _,
        } => compile_tag(tag_name, attributes, styles, children, ctx, is_root),

        Element::Text { content, span: _ } => {
            compile_text_content(content, ctx);
            Ok(())
        }

        Element::Instance {
            name,
            props,
            children,
            span: _,
        } => compile_instance(name, props, children, ctx),

        Element::Conditional {
            condition,
            then_branch,
            else_branch,
            span: _,
        } => compile_conditional(condition, then_branch, else_branch, ctx),

        Element::Repeat {
            item_name,
            collection,
            body,
            span: _,
        } => compile_repeat(item_name, collection, body, ctx),

        Element::SlotInsert { name, span: _ } => {
            ctx.add(&format!("{{{}}}", name));
            Ok(())
        }

        Element::Insert { slot_name, content, span: _ } => {
            // Insert directive - compile content as fragment
            // This would typically be passed to the component instance
            ctx.add("<>");
            for child in content {
                compile_element(child, ctx, false)?;
            }
            ctx.add("</>");
            Ok(())
        }
    }
}

fn compile_tag(
    name: &str,
    attributes: &HashMap<String, Expression>,
    styles: &[StyleBlock],
    children: &[Element],
    ctx: &CompilerContext,
    is_root: bool,
) -> Result<(), String> {
    ctx.add(&format!("<{}", name));

    // Add ref for root element
    if is_root {
        ctx.add(" ref={ref}");
    }

    // Compile attributes
    for (attr_name, expr) in attributes {
        ctx.add(" ");
        compile_attribute(attr_name, expr, ctx)?;
    }

    // Add className if there are styles
    if !styles.is_empty() {
        ctx.add(" className={cx(");
        for (i, _style) in styles.iter().enumerate() {
            if i > 0 {
                ctx.add(", ");
            }
            ctx.add(&format!("\"pc-style-{}\"", i));
        }
        ctx.add(")}");
    }

    // Close opening tag or self-close
    if children.is_empty() {
        ctx.add(" />");
    } else {
        ctx.add(">");

        // Compile children
        for child in children {
            compile_element(child, ctx, false)?;
        }

        ctx.add(&format!("</{}>", name));
    }

    Ok(())
}

fn compile_attribute(name: &str, expr: &Expression, ctx: &CompilerContext) -> Result<(), String> {
    // Convert HTML attributes to React props
    let react_prop = match name {
        "class" => "className",
        _ => name,
    };

    ctx.add(react_prop);
    ctx.add("=");

    match expr {
        Expression::Literal { value, .. } => {
            ctx.add(&format!("\"{}\"", value));
        }
        _ => {
            ctx.add("{");
            compile_expression(expr, ctx)?;
            ctx.add("}");
        }
    }

    Ok(())
}

fn compile_text_content(expr: &Expression, ctx: &CompilerContext) {
    match expr {
        Expression::Literal { value, .. } => {
            ctx.add(value);
        }
        _ => {
            ctx.add("{");
            let _ = compile_expression(expr, ctx);
            ctx.add("}");
        }
    }
}

fn compile_instance(
    name: &str,
    props: &HashMap<String, Expression>,
    children: &[Element],
    ctx: &CompilerContext,
) -> Result<(), String> {
    ctx.add(&format!("<{}", name));

    // Compile props
    for (prop_name, expr) in props {
        ctx.add(" ");
        compile_attribute(prop_name, expr, ctx)?;
    }

    if children.is_empty() {
        ctx.add(" />");
    } else {
        ctx.add(">");
        for child in children {
            compile_element(child, ctx, false)?;
        }
        ctx.add(&format!("</{}>", name));
    }

    Ok(())
}

fn compile_conditional(
    condition: &Expression,
    then_branch: &[Element],
    else_branch: &Option<Vec<Element>>,
    ctx: &CompilerContext,
) -> Result<(), String> {
    ctx.add("{");
    compile_expression(condition, ctx)?;
    ctx.add(" ? (");

    // Compile then branch
    if then_branch.len() == 1 {
        compile_element(&then_branch[0], ctx, false)?;
    } else {
        ctx.add("<>");
        for element in then_branch {
            compile_element(element, ctx, false)?;
        }
        ctx.add("</>");
    }

    ctx.add(") : ");

    // Compile else branch
    if let Some(else_elements) = else_branch {
        ctx.add("(");
        if else_elements.len() == 1 {
            compile_element(&else_elements[0], ctx, false)?;
        } else {
            ctx.add("<>");
            for element in else_elements {
                compile_element(element, ctx, false)?;
            }
            ctx.add("</>");
        }
        ctx.add(")");
    } else {
        ctx.add("null");
    }

    ctx.add("}");
    Ok(())
}

fn compile_repeat(
    item_name: &str,
    collection: &Expression,
    body: &[Element],
    ctx: &CompilerContext,
) -> Result<(), String> {
    ctx.add("{");
    compile_expression(collection, ctx)?;
    ctx.add(&format!("?.map(({}, index) => (", item_name));

    // Wrap in fragment if multiple elements
    if body.len() > 1 {
        ctx.add("<React.Fragment key={index}>");
    }

    for element in body.iter() {
        if body.len() == 1 {
            // Add key to single element
            ctx.add(&format!("<span key={{index}}>"));
        }
        compile_element(element, ctx, false)?;
        if body.len() == 1 {
            ctx.add("</span>");
        }
    }

    if body.len() > 1 {
        ctx.add("</React.Fragment>");
    }

    ctx.add("))}");
    Ok(())
}

fn compile_expression(expr: &Expression, ctx: &CompilerContext) -> Result<(), String> {
    match expr {
        Expression::Literal { value, .. } => {
            ctx.add(&format!("\"{}\"", value));
        }
        Expression::Number { value, .. } => {
            ctx.add(&value.to_string());
        }
        Expression::Boolean { value, .. } => {
            ctx.add(&value.to_string());
        }
        Expression::Variable { name, .. } => {
            ctx.add(&format!("props.{}", name));
        }
        Expression::Member { object, property, .. } => {
            compile_expression(object, ctx)?;
            ctx.add(&format!(".{}", property));
        }
        Expression::Binary {
            left,
            operator,
            right,
            ..
        } => {
            ctx.add("(");
            compile_expression(left, ctx)?;
            ctx.add(" ");
            compile_operator(operator, ctx);
            ctx.add(" ");
            compile_expression(right, ctx)?;
            ctx.add(")");
        }
        Expression::Call {
            function,
            arguments,
            ..
        } => {
            ctx.add(&format!("{}(", function));
            for (i, arg) in arguments.iter().enumerate() {
                if i > 0 {
                    ctx.add(", ");
                }
                compile_expression(arg, ctx)?;
            }
            ctx.add(")");
        }
        Expression::Template { parts, .. } => {
            ctx.add("`");
            for part in parts {
                match part {
                    TemplatePart::Literal(s) => ctx.add(s),
                    TemplatePart::Expression(e) => {
                        ctx.add("${");
                        compile_expression(e, ctx)?;
                        ctx.add("}");
                    }
                }
            }
            ctx.add("`");
        }
    }
    Ok(())
}

fn compile_operator(op: &BinaryOp, ctx: &CompilerContext) {
    let op_str = match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Equals => "===",
        BinaryOp::NotEquals => "!==",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanOrEqual => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanOrEqual => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
    };
    ctx.add(op_str);
}
