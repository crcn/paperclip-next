//! Parser for extracting @view annotations from Paperclip doc comments

use crate::types::{ViewSpec, Viewport};
use crate::{Result, VisionError};
use paperclip_parser::ast::{Component, Document};
use paperclip_parser::parse_with_path;

/// Extract view specifications from a Paperclip document
pub fn extract_views(doc: &Document) -> Vec<ViewSpec> {
    let mut views = Vec::new();

    for component in &doc.components {
        let component_views = parse_component_views(component);

        // If no @view annotations found, auto-generate default view
        if component_views.is_empty() {
            views.push(ViewSpec::default_for(component.name.clone()));
        } else {
            views.extend(component_views);
        }
    }

    views
}

/// Parse @view annotations from component doc comments
fn parse_component_views(component: &Component) -> Vec<ViewSpec> {
    let mut views = Vec::new();

    // Extract doc comments from component (would need to add this to AST)
    // For now, we'll parse from a hypothetical doc_comments field
    // TODO: Extend parser AST to include doc comments

    // Placeholder: Return empty vec
    // Will implement after adding doc comment support to parser

    views
}

/// Parse a single @view annotation line
///
/// Formats:
/// - `@view default`
/// - `@view hover - Hover state`
/// - `@viewport mobile`
fn parse_view_line(line: &str, component_name: &str, current_viewport: &mut Viewport) -> Option<ViewSpec> {
    let trimmed = line.trim();

    // Parse @viewport directive
    if let Some(viewport_name) = trimmed.strip_prefix("@viewport") {
        let viewport_name = viewport_name.trim();
        *current_viewport = match viewport_name {
            "mobile" => Viewport::Mobile,
            "tablet" => Viewport::Tablet,
            "desktop" => Viewport::Desktop,
            _ => Viewport::Desktop,
        };
        return None;
    }

    // Parse @view directive
    if let Some(rest) = trimmed.strip_prefix("@view") {
        let rest = rest.trim();

        // Split on " - " for description
        let (name, description) = if let Some(dash_pos) = rest.find(" - ") {
            let (n, d) = rest.split_at(dash_pos);
            (n.trim(), Some(d[3..].trim().to_string()))
        } else {
            (rest, None)
        };

        return Some(ViewSpec {
            name: name.to_string(),
            description,
            viewport: *current_viewport,
            component_name: component_name.to_string(),
        });
    }

    None
}

/// Load and parse a .pc file, extracting view specifications
pub fn load_views_from_file(path: &std::path::Path) -> Result<(Document, Vec<ViewSpec>)> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| VisionError::Io(e))?;

    let doc = parse_with_path(&source, &path.to_string_lossy())
        .map_err(|e| VisionError::Parse(format!("{:?}", e)))?;

    let views = extract_views(&doc);

    Ok((doc, views))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_view_line_simple() {
        let mut viewport = Viewport::Desktop;
        let view = parse_view_line("@view default", "Button", &mut viewport);

        assert!(view.is_some());
        let view = view.unwrap();
        assert_eq!(view.name, "default");
        assert_eq!(view.component_name, "Button");
        assert_eq!(view.viewport, Viewport::Desktop);
        assert_eq!(view.description, None);
    }

    #[test]
    fn test_parse_view_line_with_description() {
        let mut viewport = Viewport::Desktop;
        let view = parse_view_line("@view hover - Hover state", "Button", &mut viewport);

        assert!(view.is_some());
        let view = view.unwrap();
        assert_eq!(view.name, "hover");
        assert_eq!(view.description, Some("Hover state".to_string()));
    }

    #[test]
    fn test_parse_viewport_directive() {
        let mut viewport = Viewport::Desktop;
        let view = parse_view_line("@viewport mobile", "Button", &mut viewport);

        assert!(view.is_none());
        assert_eq!(viewport, Viewport::Mobile);
    }
}
