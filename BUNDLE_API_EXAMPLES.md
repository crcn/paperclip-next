# Bundle API Examples

Comprehensive examples showing how to use the new Bundle APIs for file resolution, asset tracking, and import aliases.

## Table of Contents

1. [Basic Bundle Usage](#basic-bundle-usage)
2. [MockFileSystem for Testing](#mockfilesystem-for-testing)
3. [Asset Tracking](#asset-tracking)
4. [Import Alias Resolution](#import-alias-resolution)
5. [Custom File System Integration](#custom-file-system-integration)
6. [Real-World Scenarios](#real-world-scenarios)

---

## Basic Bundle Usage

### Creating and Populating a Bundle

```rust
use paperclip_evaluator::Bundle;
use paperclip_parser::parse_with_path;
use std::path::PathBuf;

// Create a new empty bundle
let mut bundle = Bundle::new();

// Parse and add documents
let theme_source = r#"
    public style fontBase {
        font-family: Inter, sans-serif
    }
"#;
let theme_doc = parse_with_path(theme_source, "/project/theme.pc")?;
bundle.add_document(PathBuf::from("/project/theme.pc"), theme_doc);

let main_source = r#"
    import "./theme.pc" as theme

    public component App {
        render div {
            style extends theme.fontBase {
                padding: 16px
            }
        }
    }
"#;
let main_doc = parse_with_path(main_source, "/project/main.pc")?;
bundle.add_document(PathBuf::from("/project/main.pc"), main_doc);

// Build dependency graph (uses real file system)
let project_root = PathBuf::from("/project");
bundle.build_dependencies(&project_root)?;

// Now you can query the bundle
let deps = bundle.get_dependencies(&PathBuf::from("/project/main.pc"));
println!("main.pc imports: {:?}", deps);
```

---

## MockFileSystem for Testing

### Basic Mock Setup

```rust
use paperclip_evaluator::{Bundle, bundle::MockFileSystem};
use std::path::PathBuf;

// Create a mock file system for testing
let mut mock_fs = MockFileSystem::new();

// Add files that "exist" in the mock
mock_fs.add_file(PathBuf::from("/test/colors.pc"));
mock_fs.add_file(PathBuf::from("/test/fonts.pc"));
mock_fs.add_file(PathBuf::from("/test/app.pc"));

// Use with bundle
let mut bundle = Bundle::new();
// ... add documents ...

// Build dependencies with mock FS
bundle.build_dependencies_with_fs(&PathBuf::from("/test"), &mock_fs)?;
```

### Complete Test Example

```rust
#[test]
fn test_theme_system_with_mock_fs() {
    let mut bundle = Bundle::new();
    let mut mock_fs = MockFileSystem::new();

    // Define our mock file system structure
    mock_fs.add_file(PathBuf::from("/app/theme/colors.pc"));
    mock_fs.add_file(PathBuf::from("/app/theme/fonts.pc"));
    mock_fs.add_file(PathBuf::from("/app/components/button.pc"));

    // Add colors file
    let colors_source = r#"
        public token primaryColor #FF6B35
        public token secondaryColor #004E89
    "#;
    let colors_doc = parse_with_path(colors_source, "/app/theme/colors.pc").unwrap();
    bundle.add_document(PathBuf::from("/app/theme/colors.pc"), colors_doc);

    // Add fonts file
    let fonts_source = r#"
        public style heading {
            font-family: "Roboto Slab", serif
            font-weight: 700
        }
    "#;
    let fonts_doc = parse_with_path(fonts_source, "/app/theme/fonts.pc").unwrap();
    bundle.add_document(PathBuf::from("/app/theme/fonts.pc"), fonts_doc);

    // Add button component that imports both theme files
    let button_source = r#"
        import "../theme/colors.pc" as colors
        import "../theme/fonts.pc" as fonts

        public component Button {
            render button {
                style extends fonts.heading {
                    color: colors.primaryColor
                    background: colors.secondaryColor
                    padding: 12px 24px
                }
                text "Click me"
            }
        }
    "#;
    let button_doc = parse_with_path(button_source, "/app/components/button.pc").unwrap();
    bundle.add_document(PathBuf::from("/app/components/button.pc"), button_doc);

    // Build dependencies with mock FS
    bundle.build_dependencies_with_fs(&PathBuf::from("/app"), &mock_fs).unwrap();

    // Test alias resolution
    let color_token = bundle.find_token(
        "colors.primaryColor",
        &PathBuf::from("/app/components/button.pc")
    );
    assert!(color_token.is_some());
    assert_eq!(color_token.unwrap().0.value, "#FF6B35");

    let font_style = bundle.find_style(
        "fonts.heading",
        &PathBuf::from("/app/components/button.pc")
    );
    assert!(font_style.is_some());
    assert_eq!(font_style.unwrap().0.name, "heading");
}
```

---

## Asset Tracking

### Adding Assets to Bundle

```rust
use paperclip_evaluator::{AssetReference, AssetType};
use std::path::PathBuf;

let mut bundle = Bundle::new();

// Add an image asset
bundle.add_asset(AssetReference {
    path: "/images/hero.jpg".to_string(),
    asset_type: AssetType::Image,
    resolved_path: PathBuf::from("/project/public/images/hero.jpg"),
    source_file: PathBuf::from("/project/pages/home.pc"),
});

// Add a font asset
bundle.add_asset(AssetReference {
    path: "/fonts/inter-var.woff2".to_string(),
    asset_type: AssetType::Font,
    resolved_path: PathBuf::from("/project/public/fonts/inter-var.woff2"),
    source_file: PathBuf::from("/project/styles/typography.pc"),
});

// Query all assets
let all_assets = bundle.assets();
println!("Total assets: {}", all_assets.len());

// Filter by type
let images: Vec<_> = all_assets.iter()
    .filter(|a| matches!(a.asset_type, AssetType::Image))
    .collect();
println!("Images: {}", images.len());

// Find assets from a specific file
let home_assets: Vec<_> = all_assets.iter()
    .filter(|a| a.source_file == PathBuf::from("/project/pages/home.pc"))
    .collect();
println!("Assets from home.pc: {}", home_assets.len());
```

### Workspace Integration (Automatic Asset Tracking)

```rust
use paperclip_workspace::WorkspaceState;
use std::path::PathBuf;

let mut workspace = WorkspaceState::new();
let project_root = PathBuf::from("/project");

// When you update a file with assets, they're automatically extracted and added to bundle
let page_source = r#"
    public component Hero {
        render div {
            img { src: "/images/hero.jpg" }
            text "Welcome"
        }
    }
"#;

workspace.update_file(
    PathBuf::from("/project/pages/home.pc"),
    page_source.to_string(),
    &project_root,
)?;

// Assets are now in the bundle
let file_state = workspace.get_file(&PathBuf::from("/project/pages/home.pc")).unwrap();
println!("Assets in file: {}", file_state.assets.len());

// You can also query bundle directly (if you have access to it)
// workspace.bundle.assets() would give you all assets across all files
```

### Asset Type Detection

```rust
use paperclip_evaluator::AssetType;

// Helper to categorize assets
fn categorize_asset(path: &str) -> AssetType {
    if path.ends_with(".jpg") || path.ends_with(".png") || path.ends_with(".svg") {
        AssetType::Image
    } else if path.ends_with(".woff") || path.ends_with(".woff2") || path.ends_with(".ttf") {
        AssetType::Font
    } else if path.ends_with(".mp4") || path.ends_with(".webm") {
        AssetType::Video
    } else if path.ends_with(".mp3") || path.ends_with(".ogg") {
        AssetType::Audio
    } else {
        AssetType::Other
    }
}

// Generate asset manifest for deployment
fn generate_asset_manifest(bundle: &Bundle) -> serde_json::Value {
    use serde_json::json;

    let assets_by_type: HashMap<&str, Vec<&AssetReference>> = bundle.assets()
        .iter()
        .fold(HashMap::new(), |mut acc, asset| {
            let type_key = match asset.asset_type {
                AssetType::Image => "images",
                AssetType::Font => "fonts",
                AssetType::Video => "videos",
                AssetType::Audio => "audio",
                AssetType::Other => "other",
            };
            acc.entry(type_key).or_insert_with(Vec::new).push(asset);
            acc
        });

    json!({
        "images": assets_by_type.get("images").map(|v| v.len()).unwrap_or(0),
        "fonts": assets_by_type.get("fonts").map(|v| v.len()).unwrap_or(0),
        "total": bundle.assets().len(),
    })
}
```

---

## Import Alias Resolution

### Simple Namespaced Lookup

```rust
// After building bundle with dependencies...
let bundle = /* ... */;

// Find a style from an imported file
// In button.pc: import "./theme.pc" as theme
let style = bundle.find_style(
    "theme.fontBase",                      // namespaced reference
    &PathBuf::from("/project/button.pc")   // current file
);

match style {
    Some((style_decl, source_file)) => {
        println!("Found style: {}", style_decl.name);
        println!("From file: {}", source_file.display());
        println!("Properties: {:?}", style_decl.properties);
    }
    None => println!("Style not found"),
}
```

### Token Lookup

```rust
// Find a token from an imported file
// In app.pc: import "./colors.pc" as colors
let token = bundle.find_token(
    "colors.primaryColor",
    &PathBuf::from("/project/app.pc")
);

match token {
    Some((token_decl, source_file)) => {
        println!("Token: {} = {}", token_decl.name, token_decl.value);
    }
    None => println!("Token not found"),
}
```

### Component Lookup

```rust
// Find a component from an imported file
// In page.pc: import "./components.pc" as comp
let component = bundle.find_component(
    "comp.Button",
    &PathBuf::from("/project/page.pc")
);

match component {
    Some((component_decl, source_file)) => {
        println!("Component: {}", component_decl.name);
        println!("Public: {}", component_decl.public);
    }
    None => println!("Component not found"),
}
```

### Multiple Imports Example

```rust
// Setup: app.pc imports multiple files with different aliases
let app_source = r#"
    import "./theme/colors.pc" as colors
    import "./theme/fonts.pc" as fonts
    import "./ui/button.pc" as ui

    public component App {
        render div {
            ui.Button()
            style {
                color: colors.primary
                font-family: fonts.base
            }
        }
    }
"#;

// After building bundle...

// Each alias resolves to its specific file
let color = bundle.find_token("colors.primary", &PathBuf::from("/app.pc"));
// Looks in /theme/colors.pc ONLY

let font = bundle.find_token("fonts.base", &PathBuf::from("/app.pc"));
// Looks in /theme/fonts.pc ONLY

let button = bundle.find_component("ui.Button", &PathBuf::from("/app.pc"));
// Looks in /ui/button.pc ONLY
```

### Checking What a File Imports

```rust
// Get all dependencies of a file
let deps = bundle.get_dependencies(&PathBuf::from("/project/app.pc"));

if let Some(imports) = deps {
    println!("app.pc imports {} files:", imports.len());
    for import_path in imports {
        println!("  - {}", import_path.display());
    }
}

// Get reverse dependencies (who imports this file?)
let dependents = bundle.get_dependents(&PathBuf::from("/project/theme.pc"));

if let Some(importers) = dependents {
    println!("theme.pc is imported by:");
    for importer_path in importers {
        println!("  - {}", importer_path.display());
    }
}
```

---

## Custom File System Integration

### Implementing Custom FileSystem

```rust
use paperclip_evaluator::bundle::FileSystem;
use std::path::{Path, PathBuf};

// Example: File system that works with a virtual file system or cloud storage
struct CloudFileSystem {
    bucket: String,
    cached_files: HashSet<PathBuf>,
}

impl CloudFileSystem {
    fn new(bucket: String) -> Self {
        Self {
            bucket,
            cached_files: HashSet::new(),
        }
    }

    fn cache_file(&mut self, path: PathBuf) {
        self.cached_files.insert(path);
    }
}

impl FileSystem for CloudFileSystem {
    fn exists(&self, path: &Path) -> bool {
        // Check if file exists in cloud storage
        self.cached_files.contains(path) || self.check_cloud_storage(path)
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
        // For cloud FS, we might not need to canonicalize
        // Just return normalized path
        Ok(path.to_path_buf())
    }
}

impl CloudFileSystem {
    fn check_cloud_storage(&self, path: &Path) -> bool {
        // Pseudo-code for cloud check
        // cloud_client.object_exists(&self.bucket, path)
        false
    }
}

// Usage:
let cloud_fs = CloudFileSystem::new("my-paperclip-bucket".to_string());
bundle.build_dependencies_with_fs(&project_root, &cloud_fs)?;
```

### Testing with In-Memory File System

```rust
struct MemoryFileSystem {
    files: HashMap<PathBuf, String>,
}

impl MemoryFileSystem {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    fn add(&mut self, path: PathBuf, content: String) {
        self.files.insert(path, content);
    }
}

impl FileSystem for MemoryFileSystem {
    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
        Ok(path.to_path_buf())
    }
}

// Use in tests:
#[test]
fn test_with_memory_fs() {
    let mut mem_fs = MemoryFileSystem::new();
    mem_fs.add(
        PathBuf::from("/virtual/theme.pc"),
        "public style base { color: red }".to_string()
    );

    let mut bundle = Bundle::new();
    // ... add parsed documents ...

    bundle.build_dependencies_with_fs(&PathBuf::from("/virtual"), &mem_fs).unwrap();
}
```

---

## Real-World Scenarios

### Scenario 1: Design System with Theme Variants

```rust
// Project structure:
// /design-system
//   /themes
//     base.pc
//     dark.pc
//     light.pc
//   /components
//     button.pc
//     card.pc

fn build_design_system_bundle() -> Result<Bundle, Box<dyn std::error::Error>> {
    let mut bundle = Bundle::new();
    let root = PathBuf::from("/design-system");

    // Add base theme
    let base_theme = r#"
        public token borderRadius 4px
        public token spacing 8px

        public style reset {
            margin: 0
            padding: 0
            box-sizing: border-box
        }
    "#;
    bundle.add_document(
        root.join("themes/base.pc"),
        parse_with_path(base_theme, "/design-system/themes/base.pc")?
    );

    // Add light theme
    let light_theme = r#"
        import "./base.pc" as base

        public token background #FFFFFF
        public token foreground #000000

        public style lightTheme extends base.reset {
            background: background
            color: foreground
        }
    "#;
    bundle.add_document(
        root.join("themes/light.pc"),
        parse_with_path(light_theme, "/design-system/themes/light.pc")?
    );

    // Add button component
    let button = r#"
        import "../themes/base.pc" as base
        import "../themes/light.pc" as light

        public component Button {
            render button {
                style extends light.lightTheme {
                    padding: base.spacing
                    border-radius: base.borderRadius
                }
                text "Button"
            }
        }
    "#;
    bundle.add_document(
        root.join("components/button.pc"),
        parse_with_path(button, "/design-system/components/button.pc")?
    );

    // Build dependencies
    bundle.build_dependencies(&root)?;

    // Verify cross-file references work
    let border_radius = bundle.find_token(
        "base.borderRadius",
        &root.join("components/button.pc")
    );
    assert!(border_radius.is_some());

    Ok(bundle)
}
```

### Scenario 2: Asset Manifest Generation for Build Tool

```rust
use std::fs::File;
use std::io::Write;

fn generate_asset_manifest_file(
    bundle: &Bundle,
    output_path: &Path
) -> std::io::Result<()> {
    let mut manifest = Vec::new();

    // Group assets by source file
    let mut assets_by_file: HashMap<&PathBuf, Vec<&AssetReference>> = HashMap::new();
    for asset in bundle.assets() {
        assets_by_file
            .entry(&asset.source_file)
            .or_insert_with(Vec::new)
            .push(asset);
    }

    // Generate manifest content
    manifest.push("# Asset Manifest\n".to_string());
    manifest.push(format!("Total assets: {}\n\n", bundle.assets().len()));

    for (source_file, assets) in assets_by_file {
        manifest.push(format!("## {}\n", source_file.display()));
        for asset in assets {
            manifest.push(format!(
                "- [{:?}] {} -> {}\n",
                asset.asset_type,
                asset.path,
                asset.resolved_path.display()
            ));
        }
        manifest.push("\n".to_string());
    }

    // Write to file
    let mut file = File::create(output_path)?;
    file.write_all(manifest.join("").as_bytes())?;

    Ok(())
}

// Usage:
let bundle = /* ... built bundle ... */;
generate_asset_manifest_file(&bundle, Path::new("./asset-manifest.md"))?;
```

### Scenario 3: Hot Module Replacement (HMR)

```rust
struct HMRHandler {
    bundle: Bundle,
    file_watchers: HashMap<PathBuf, SystemTime>,
}

impl HMRHandler {
    fn handle_file_change(&mut self, changed_file: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        // Find all files that depend on the changed file
        let mut affected_files = vec![changed_file.to_path_buf()];

        if let Some(dependents) = self.bundle.get_dependents(changed_file) {
            affected_files.extend(dependents.iter().cloned());
        }

        // Re-parse and update the changed file
        let new_source = std::fs::read_to_string(changed_file)?;
        let new_doc = parse_with_path(&new_source, &changed_file.to_string_lossy())?;
        self.bundle.add_document(changed_file.to_path_buf(), new_doc);

        // Rebuild dependencies to update import aliases
        let project_root = changed_file.parent().unwrap_or(Path::new("/"));
        self.bundle.build_dependencies(project_root)?;

        // Return list of files that need to be re-evaluated
        Ok(affected_files)
    }
}

// Usage in watch mode:
let mut hmr = HMRHandler {
    bundle: /* ... */,
    file_watchers: HashMap::new(),
};

// When theme.pc changes, find all components that import it
let affected = hmr.handle_file_change(&PathBuf::from("/project/theme.pc"))?;
println!("Files affected by change: {:?}", affected);

// Re-evaluate only the affected files
for file in affected {
    // re-evaluate_file(&file);
}
```

### Scenario 4: Dependency Graph Visualization

```rust
fn generate_dependency_dot_graph(bundle: &Bundle) -> String {
    let mut dot = String::from("digraph Dependencies {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box];\n\n");

    // Get all files
    let all_files: HashSet<_> = bundle.get_all_files().collect();

    for file in all_files {
        let file_name = file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Add node
        dot.push_str(&format!("  \"{}\";\n", file_name));

        // Add edges for dependencies
        if let Some(deps) = bundle.get_dependencies(file) {
            for dep in deps {
                let dep_name = dep.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                dot.push_str(&format!("  \"{}\" -> \"{}\";\n", file_name, dep_name));
            }
        }
    }

    dot.push_str("}\n");
    dot
}

// Generate GraphViz file
let dot_content = generate_dependency_dot_graph(&bundle);
std::fs::write("dependencies.dot", dot_content)?;

// Then run: `dot -Tpng dependencies.dot -o dependencies.png`
```

### Scenario 5: Circular Dependency Detection

```rust
fn check_for_circular_dependencies(bundle: &Bundle) -> Result<(), String> {
    match bundle.build_dependencies(&project_root) {
        Ok(_) => {
            println!("✓ No circular dependencies detected");
            Ok(())
        }
        Err(BundleError::CircularDependency { path }) => {
            eprintln!("✗ Circular dependency detected involving: {}", path);

            // Try to trace the cycle
            let cycle = trace_dependency_cycle(bundle, &PathBuf::from(&path));
            eprintln!("Cycle: {}", cycle.join(" → "));

            Err(format!("Circular dependency: {}", path))
        }
        Err(e) => Err(format!("Bundle error: {}", e)),
    }
}

fn trace_dependency_cycle(bundle: &Bundle, start: &Path) -> Vec<String> {
    let mut visited = HashSet::new();
    let mut path = Vec::new();

    fn dfs(
        bundle: &Bundle,
        current: &Path,
        visited: &mut HashSet<PathBuf>,
        path: &mut Vec<String>,
    ) -> bool {
        if visited.contains(current) {
            path.push(current.display().to_string());
            return true;
        }

        visited.insert(current.to_path_buf());
        path.push(current.display().to_string());

        if let Some(deps) = bundle.get_dependencies(current) {
            for dep in deps {
                if dfs(bundle, dep, visited, path) {
                    return true;
                }
            }
        }

        path.pop();
        visited.remove(current);
        false
    }

    dfs(bundle, start, &mut visited, &mut path);
    path
}
```

---

## Performance Tips

### 1. Batch Bundle Operations

```rust
// BAD: Rebuilding dependencies for each file
for file in files {
    bundle.add_document(file.path, file.doc);
    bundle.build_dependencies(&root)?;  // Expensive!
}

// GOOD: Add all documents first, then build once
for file in files {
    bundle.add_document(file.path, file.doc);
}
bundle.build_dependencies(&root)?;  // Once at the end
```

### 2. Cache Bundle Between Builds

```rust
// Keep bundle alive across file updates
struct BuildCache {
    bundle: Bundle,
    last_modified: HashMap<PathBuf, SystemTime>,
}

impl BuildCache {
    fn incremental_update(&mut self, changed_file: &Path) {
        // Only update the changed file, not the whole bundle
        let new_doc = /* ... parse changed file ... */;
        self.bundle.add_document(changed_file.to_path_buf(), new_doc);
        self.bundle.build_dependencies(&project_root).ok();
    }
}
```

### 3. Lazy Asset Loading

```rust
// Don't extract assets until they're needed
struct LazyBundle {
    bundle: Bundle,
    assets_loaded: bool,
}

impl LazyBundle {
    fn get_assets(&mut self) -> &[AssetReference] {
        if !self.assets_loaded {
            // Extract assets on first access
            self.load_all_assets();
            self.assets_loaded = true;
        }
        self.bundle.assets()
    }
}
```

---

## Error Handling Patterns

```rust
use paperclip_evaluator::BundleError;

fn handle_bundle_errors(result: Result<(), BundleError>) {
    match result {
        Ok(_) => println!("Success!"),

        Err(BundleError::ImportNotFound { import_path, source_path }) => {
            eprintln!("Error: Cannot find import '{}' in {}", import_path, source_path);
            eprintln!("Hint: Check that the file exists and the path is correct");
        }

        Err(BundleError::CircularDependency { path }) => {
            eprintln!("Error: Circular dependency detected at {}", path);
            eprintln!("Hint: Review your import statements to break the cycle");
        }

        Err(BundleError::StyleNotFound { name }) => {
            eprintln!("Error: Style '{}' not found", name);
            eprintln!("Hint: Make sure the style is public and the import alias is correct");
        }

        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

These examples cover the main use cases for the Bundle APIs. For more details, see the test files:
- `packages/evaluator/src/tests_bundle.rs`
- `packages/evaluator/src/tests_bundle_filesystem.rs`
- `packages/workspace/tests/bundle_integration_test.rs`
