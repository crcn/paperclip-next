/// Shared utilities for CSS and DOM evaluation
///
/// This module provides functions for generating synchronized class names
/// between CSS rules and DOM elements, ensuring styles apply correctly.

/// Generate a scoped class name for an element
///
/// This function creates deterministic, collision-free class names by combining:
/// - Element name (if present)
/// - Component name (if inside a component)
/// - AST node ID (always unique)
///
/// Examples:
/// - `get_style_namespace(Some("button"), "abc123", Some("Button"))` → `"_Button-button-abc123"`
/// - `get_style_namespace(Some("div"), "def456", None)` → `"_div-def456"`
/// - `get_style_namespace(None, "ghi789", Some("Card"))` → `"_Card-ghi789"`
pub fn get_style_namespace(
    element_name: Option<&str>,
    id: &str,
    component_name: Option<&str>,
) -> String {
    if let Some(name) = element_name {
        // Element has a name (e.g., "button", "div")
        let ns = if let Some(comp) = component_name {
            // Inside a component: prefix with component name
            format!("{}-{}", comp, name)
        } else {
            // Top-level element
            name.to_string()
        };

        // Always include ID for uniqueness
        format!("_{}-{}", ns, id)
    } else {
        // Element has no name (anonymous)
        if let Some(comp) = component_name {
            format!("_{}-{}", comp, id)
        } else {
            format!("_{}", id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_with_element_and_component() {
        let result = get_style_namespace(Some("button"), "abc123", Some("Button"));
        assert_eq!(result, "_Button-button-abc123");
    }

    #[test]
    fn test_namespace_with_element_no_component() {
        let result = get_style_namespace(Some("div"), "def456", None);
        assert_eq!(result, "_div-def456");
    }

    #[test]
    fn test_namespace_no_element_with_component() {
        let result = get_style_namespace(None, "ghi789", Some("Card"));
        assert_eq!(result, "_Card-ghi789");
    }

    #[test]
    fn test_namespace_no_element_no_component() {
        let result = get_style_namespace(None, "jkl012", None);
        assert_eq!(result, "_jkl012");
    }

    #[test]
    fn test_namespace_deterministic() {
        // Same inputs should always produce same output
        let result1 = get_style_namespace(Some("button"), "abc", Some("Btn"));
        let result2 = get_style_namespace(Some("button"), "abc", Some("Btn"));
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_namespace_unique_ids() {
        // Different IDs should produce different results
        let result1 = get_style_namespace(Some("button"), "abc", Some("Btn"));
        let result2 = get_style_namespace(Some("button"), "xyz", Some("Btn"));
        assert_ne!(result1, result2);
    }
}
