//! Targeted DOM emitter for single-component rendering
//!
//! This is NOT a general HTML compiler. This emits minimal, deterministic HTML
//! for a single component view with inline styles.

use crate::{Result, VisionError};
use paperclip_evaluator::{Evaluator, VNode, VirtualDomDocument};
use paperclip_parser::ast::Document;

/// Render a single component to standalone HTML
pub fn render_component_html(
    doc: &Document,
    component_name: &str,
) -> Result<String> {
    // Find the component
    let _component = doc
        .components
        .iter()
        .find(|c| c.name == component_name)
        .ok_or_else(|| VisionError::ComponentNotFound(component_name.to_string()))?;

    // Evaluate to Virtual DOM
    let mut evaluator = Evaluator::new();
    let vdom = evaluator
        .evaluate(doc)
        .map_err(|e| VisionError::Render(format!("{:?}", e)))?;

    // Find component root in vdom
    let component_root = find_component_root(&vdom)
        .ok_or_else(|| VisionError::Render(format!("Component {} not found in vdom", component_name)))?;

    // Emit HTML
    let body_html = emit_vnode_html(component_root, true);

    // Build complete HTML document
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }}
    </style>
</head>
<body>
    {}
</body>
</html>"#,
        body_html
    );

    Ok(html)
}

/// Find the root VNode for a specific component
fn find_component_root(vdom: &VirtualDomDocument) -> Option<&VNode> {
    // For now, assume the first matching component
    // TODO: Handle multiple instances, component hierarchy
    vdom.nodes.first()
}

/// Emit HTML for a single VNode with inline styles
fn emit_vnode_html(node: &VNode, is_root: bool) -> String {
    match node {
        VNode::Element {
            tag,
            attributes,
            styles,
            children,
            ..
        } => {
            let mut html = String::new();
            html.push('<');
            html.push_str(tag);

            // Add data-pc-root for boundary detection
            if is_root {
                html.push_str(r#" data-pc-root="true""#);
            }

            // Add attributes
            for (key, value) in attributes {
                html.push(' ');
                html.push_str(key);
                html.push_str(r#"=""#);
                html.push_str(&escape_html(value));
                html.push('"');
            }

            // Add inline styles
            if !styles.is_empty() {
                html.push_str(r#" style=""#);
                for (key, value) in styles {
                    html.push_str(key);
                    html.push_str(": ");
                    html.push_str(value);
                    html.push_str("; ");
                }
                html.push('"');
            }

            html.push('>');

            // Add children
            for child in children {
                html.push_str(&emit_vnode_html(child, false));
            }

            html.push_str("</");
            html.push_str(tag);
            html.push('>');

            html
        }
        VNode::Text { content } => escape_html(content),
        VNode::Comment { .. } => {
            // Skip comments in HTML output
            String::new()
        }
        VNode::Error { message, .. } => {
            // Render errors as visible divs for debugging
            format!(r#"<div style="color: red; border: 1px solid red; padding: 4px;">Error: {}</div>"#, escape_html(message))
        }
    }
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse_with_path;

    #[test]
    fn test_render_simple_button() {
        let source = r#"
            public component Button {
                render button {
                    style {
                        padding: 8px 16px
                        background: blue
                    }
                    text "Click me"
                }
            }
        "#;

        let doc = parse_with_path(source, "/test.pc").unwrap();
        let html = render_component_html(&doc, "Button").unwrap();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("data-pc-root"));
        assert!(html.contains("Click me"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("'quotes'"), "&#39;quotes&#39;");
    }
}
