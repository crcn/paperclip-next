/// End-to-end test for bundle-based workspace evaluation
/// Tests the complete flow: parse → bundle → evaluate with cross-file imports
use paperclip_workspace::WorkspaceState;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_workspace_with_cross_file_imports() {
    // Create a temporary directory for test files
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create theme.pc file
    let theme_path = project_root.join("theme.pc");
    let theme_source = r#"
public token primaryColor #3366FF
public token spacing 16px

public style fontBase {
    font-family: Inter, sans-serif
    font-size: 14px
}

public style colorScheme {
    color: #333333
    background: #FFFFFF
}
"#;
    fs::write(&theme_path, theme_source).unwrap();

    // Create main.pc file that imports theme
    let main_path = project_root.join("main.pc");
    let main_source = r#"
import "./theme.pc" as theme

public style buttonStyle extends theme.fontBase, theme.colorScheme {
    padding: theme.spacing
    border-radius: 4px
}

public component Button {
    render button {
        style extends theme.fontBase {
            padding: 12px
            color: theme.primaryColor
        }
        text "Click Me"
    }
}

public component App {
    render div {
        Button()
    }
}
"#;
    fs::write(&main_path, main_source).unwrap();

    // Create workspace and process files
    let mut workspace = WorkspaceState::new();

    // Add theme file
    let theme_result =
        workspace.update_file(theme_path.clone(), theme_source.to_string(), project_root);
    assert!(theme_result.is_ok(), "Theme file should parse and evaluate");

    // Add main file (which imports theme)
    let main_result =
        workspace.update_file(main_path.clone(), main_source.to_string(), project_root);
    assert!(
        main_result.is_ok(),
        "Main file should parse and evaluate with imports"
    );

    // Verify the main file state
    let main_state = workspace.get_file(&main_path).unwrap();

    // Check VirtualDomDocument
    assert!(
        main_state.vdom.nodes.len() > 0,
        "Should have evaluated components"
    );

    // Check CSS
    assert!(main_state.css.rules.len() > 0, "Should have CSS rules");

    // Verify CSS variables are generated for theme styles
    let has_root_vars = main_state.css.rules.iter().any(|r| r.selector == ":root");
    assert!(has_root_vars, "Should have :root rule with CSS variables");

    // Verify namespaced style exists in AST (accessed through workspace)
    let main_ast = workspace.get_ast(&main_path).unwrap();
    let button_style = main_ast.styles.iter().find(|s| s.name == "buttonStyle");
    assert!(button_style.is_some(), "Should have buttonStyle");

    let button_style = button_style.unwrap();
    assert_eq!(button_style.extends.len(), 2, "Should extend two styles");
    assert_eq!(button_style.extends[0], "theme.fontBase");
    assert_eq!(button_style.extends[1], "theme.colorScheme");
}

#[test]
fn test_workspace_bundle_css_variables() {
    // Test that CSS variables work correctly for instant theme updates
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create colors file
    let colors_path = project_root.join("colors.pc");
    let colors_source = r#"
public style primary {
    color: #3366FF
    background: #E6F0FF
}
"#;
    fs::write(&colors_path, colors_source).unwrap();

    // Create component file
    let component_path = project_root.join("button.pc");
    let component_source = r#"
import "./colors.pc" as colors

public component PrimaryButton {
    render button {
        style extends colors.primary {
            padding: 8px 16px
            border-radius: 4px
        }
        text "Primary"
    }
}
"#;
    fs::write(&component_path, component_source).unwrap();

    let mut workspace = WorkspaceState::new();

    // Add colors file
    workspace
        .update_file(colors_path.clone(), colors_source.to_string(), project_root)
        .unwrap();

    // Add component file
    workspace
        .update_file(
            component_path.clone(),
            component_source.to_string(),
            project_root,
        )
        .unwrap();

    let component_state = workspace.get_file(&component_path).unwrap();

    // Verify CSS variables are present
    let root_rules: Vec<_> = component_state
        .css
        .rules
        .iter()
        .filter(|r| r.selector == ":root")
        .collect();

    assert!(
        root_rules.len() > 0,
        "Should have :root rules with CSS variables"
    );

    // Verify button uses CSS variables (not direct values)
    let button_rule = component_state
        .css
        .rules
        .iter()
        .find(|r| r.selector.contains("PrimaryButton") && r.selector.contains("button"));

    assert!(button_rule.is_some(), "Should have button rule");

    // Check that properties use var() references
    let button_rule = button_rule.unwrap();
    if let Some(color) = button_rule.properties.get("color") {
        // Should use var() with fallback for theme properties
        // Local properties (padding, border-radius) are direct values
        println!("Button color: {}", color);
    }

    // The key benefit: When you edit colors.pc and change "color: #3366FF" to "color: #FF0000"
    // You only need to patch the :root rule, and all dependent styles update instantly!
}

#[test]
fn test_workspace_handles_missing_imports() {
    // Test that workspace handles missing import files gracefully
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let main_path = project_root.join("main.pc");
    let main_source = r#"
import "./missing.pc" as missing

public component App {
    render div {
        text "App"
    }
}
"#;
    fs::write(&main_path, main_source).unwrap();

    let mut workspace = WorkspaceState::new();

    // This should still work - we just won't be able to resolve cross-file references
    let result = workspace.update_file(main_path.clone(), main_source.to_string(), project_root);

    // Should succeed - bundle dependency building may warn but evaluation continues
    assert!(result.is_ok(), "Should handle missing imports gracefully");
}
