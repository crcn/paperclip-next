use paperclip_parser::ast::*;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during HTML compilation
#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Compilation error: {0}")]
    Generic(String),
}

impl From<String> for CompileError {
    fn from(s: String) -> Self {
        CompileError::Generic(s)
    }
}

impl From<&str> for CompileError {
    fn from(s: &str) -> Self {
        CompileError::Generic(s.to_string())
    }
}

/// Options for HTML compilation
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Include inline styles
    pub inline_styles: bool,
    /// Use class names (requires separate CSS compilation)
    pub use_classes: bool,
    /// Pretty print HTML
    pub pretty: bool,
    /// Indentation string
    pub indent: String,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            inline_styles: true,
            use_classes: false,
            pretty: true,
            indent: "  ".to_string(),
        }
    }
}

struct Context {
    options: CompileOptions,
    depth: usize,
    buffer: String,
}

impl Context {
    fn new(options: CompileOptions) -> Self {
        Self {
            options,
            depth: 0,
            buffer: String::new(),
        }
    }

    fn add(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    fn add_line(&mut self, text: &str) {
        if self.options.pretty {
            self.add_indent();
        }
        self.add(text);
        if self.options.pretty {
            self.add("\n");
        }
    }

    fn add_indent(&mut self) {
        let indent = self.options.indent.clone();
        for _ in 0..self.depth {
            self.add(&indent);
        }
    }

    fn indent(&mut self) {
        self.depth += 1;
    }

    fn dedent(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    fn get_output(self) -> String {
        self.buffer
    }
}

/// Compile a Paperclip document to HTML
pub fn compile_to_html(
    document: &Document,
    options: CompileOptions,
) -> Result<String, CompileError> {
    let mut ctx = Context::new(options);

    // Add DOCTYPE
    ctx.add_line("<!DOCTYPE html>");
    ctx.add_line("<html>");
    ctx.indent();

    // Add head
    compile_head(&document, &mut ctx);

    // Add body with components
    ctx.add_line("<body>");
    ctx.indent();

    for component in &document.components {
        if component.public {
            compile_component_as_html(component, &mut ctx)?;
        }
    }

    ctx.dedent();
    ctx.add_line("</body>");

    ctx.dedent();
    ctx.add_line("</html>");

    Ok(ctx.get_output())
}

fn compile_head(_document: &Document, ctx: &mut Context) {
    ctx.add_line("<head>");
    ctx.indent();

    ctx.add_line("<meta charset=\"UTF-8\">");
    ctx.add_line("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">");
    ctx.add_line("<title>Paperclip Components</title>");

    // Add inline styles if using class names
    if ctx.options.use_classes {
        ctx.add_line("<style>");
        ctx.indent();
        ctx.add_line("/* Generated styles - compile CSS separately for production */");
        ctx.add_line("/* paperclip compile --target css */");
        ctx.dedent();
        ctx.add_line("</style>");
    }

    ctx.dedent();
    ctx.add_line("</head>");
}

fn compile_component_as_html(component: &Component, ctx: &mut Context) -> Result<(), CompileError> {
    // Add component as a section with ID
    ctx.add_line(&format!(
        "<section id=\"{}\" class=\"paperclip-component\">",
        component.name.to_lowercase()
    ));
    ctx.indent();

    if let Some(body) = &component.body {
        compile_element(body, ctx)?;
    }

    ctx.dedent();
    ctx.add_line("</section>");

    Ok(())
}

fn compile_element(element: &Element, ctx: &mut Context) -> Result<(), CompileError> {
    match element {
        Element::Tag {
            tag_name,
            name: _element_name,
            attributes,
            styles,
            children,
            span: _,
        } => compile_tag(tag_name, attributes, styles, children, ctx),

        Element::Text { content, span: _ } => {
            compile_text_content(content, ctx);
            Ok(())
        }

        Element::Instance {
            name,
            props: _,
            children,
            span: _,
        } => {
            // Render instance as a div with component name as class
            ctx.add_line(&format!(
                "<div class=\"component-{}\">",
                name.to_lowercase()
            ));
            ctx.indent();
            for child in children {
                compile_element(child, ctx)?;
            }
            ctx.dedent();
            ctx.add_line("</div>");
            Ok(())
        }

        Element::Conditional {
            condition: _,
            then_branch,
            else_branch: _,
            span: _,
        } => {
            // For static HTML, just render the then branch
            for element in then_branch {
                compile_element(element, ctx)?;
            }
            Ok(())
        }

        Element::Repeat {
            item_name: _,
            collection: _,
            body,
            span: _,
        } => {
            // For static HTML, render once as sample
            ctx.add_line("<!-- Repeat: rendered once as sample -->");
            for element in body {
                compile_element(element, ctx)?;
            }
            Ok(())
        }

        Element::SlotInsert { name, span: _ } => {
            ctx.add_line(&format!("<!-- Slot: {} -->", name));
            Ok(())
        }

        Element::Insert {
            slot_name: _,
            content,
            span: _,
        } => {
            // Insert directive - render content
            ctx.add_line("<!-- Insert directive -->");
            for element in content {
                compile_element(element, ctx)?;
            }
            Ok(())
        }
    }
}

fn compile_tag(
    name: &str,
    attributes: &HashMap<String, Expression>,
    styles: &[StyleBlock],
    children: &[Element],
    ctx: &mut Context,
) -> Result<(), CompileError> {
    // Opening tag
    if ctx.options.pretty {
        ctx.add_indent();
    }
    ctx.add(&format!("<{}", name));

    // Add attributes
    for (attr_name, expr) in attributes {
        ctx.add(" ");
        compile_attribute(attr_name, expr, ctx)?;
    }

    // Add inline styles
    if ctx.options.inline_styles && !styles.is_empty() {
        ctx.add(" style=\"");
        for (i, style_block) in styles.iter().enumerate() {
            if i > 0 {
                ctx.add(" ");
            }
            for (key, value) in &style_block.properties {
                ctx.add(&format!("{}: {}; ", key, value));
            }
        }
        ctx.add("\"");
    }

    // Self-closing tags
    if children.is_empty() && is_self_closing(name) {
        ctx.add(" />");
        if ctx.options.pretty {
            ctx.add("\n");
        }
        return Ok(());
    }

    ctx.add(">");

    // Children
    if !children.is_empty() {
        if ctx.options.pretty && has_element_children(children) {
            ctx.add("\n");
        }
        ctx.indent();

        for child in children {
            compile_element(child, ctx)?;
        }

        ctx.dedent();
        if ctx.options.pretty && has_element_children(children) {
            ctx.add_indent();
        }
    }

    // Closing tag
    ctx.add(&format!("</{}>", name));
    if ctx.options.pretty {
        ctx.add("\n");
    }

    Ok(())
}

fn compile_attribute(name: &str, expr: &Expression, ctx: &mut Context) -> Result<(), CompileError> {
    ctx.add(name);
    ctx.add("=\"");

    match expr {
        Expression::Literal { value, .. } => {
            ctx.add(&escape_html(value));
        }
        Expression::Number { value, .. } => {
            ctx.add(&value.to_string());
        }
        Expression::Boolean { value, .. } => {
            ctx.add(&value.to_string());
        }
        _ => {
            // For dynamic expressions, use placeholder
            ctx.add("[dynamic]");
        }
    }

    ctx.add("\"");
    Ok(())
}

fn compile_text_content(expr: &Expression, ctx: &mut Context) {
    match expr {
        Expression::Literal { value, .. } => {
            ctx.add(&escape_html(value));
        }
        Expression::Number { value, .. } => {
            ctx.add(&value.to_string());
        }
        Expression::Boolean { value, .. } => {
            ctx.add(&value.to_string());
        }
        _ => {
            // For dynamic expressions, use placeholder
            ctx.add("[dynamic]");
        }
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn is_self_closing(tag: &str) -> bool {
    matches!(
        tag,
        "img"
            | "input"
            | "br"
            | "hr"
            | "meta"
            | "link"
            | "area"
            | "base"
            | "col"
            | "embed"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn has_element_children(children: &[Element]) -> bool {
    children
        .iter()
        .any(|child| !matches!(child, Element::Text { .. }))
}
