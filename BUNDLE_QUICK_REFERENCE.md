# Bundle API Quick Reference

Quick lookup for common Bundle operations.

## Creating & Building

```rust
use paperclip_evaluator::Bundle;

// Create
let mut bundle = Bundle::new();

// Add documents
bundle.add_document(path, document);

// Build dependencies (production)
bundle.build_dependencies(&project_root)?;

// Build dependencies (testing)
bundle.build_dependencies_with_fs(&project_root, &mock_fs)?;
```

## Querying Dependencies

```rust
// What does this file import?
let deps = bundle.get_dependencies(&file_path);

// Who imports this file?
let importers = bundle.get_dependents(&file_path);

// Get document ID
let doc_id = bundle.get_document_id(&file_path);

// Get parsed document
let doc = bundle.get_document(&file_path);
```

## Resolving Imports

```rust
// Find style (supports "theme.fontBase")
let style = bundle.find_style("theme.fontBase", &current_file);

// Find token (supports "colors.primary")
let token = bundle.find_token("colors.primary", &current_file);

// Find component (supports "ui.Button")
let component = bundle.find_component("ui.Button", &current_file);
```

## Assets

```rust
// Add asset
bundle.add_asset(AssetReference {
    path: "/images/logo.png".to_string(),
    asset_type: AssetType::Image,
    resolved_path: PathBuf::from("/public/images/logo.png"),
    source_file: PathBuf::from("/src/home.pc"),
});

// Get all assets
let all = bundle.assets();

// Filter by type
let images = all.iter()
    .filter(|a| matches!(a.asset_type, AssetType::Image))
    .collect::<Vec<_>>();

// Filter by source file
let from_home = all.iter()
    .filter(|a| a.source_file == PathBuf::from("/src/home.pc"))
    .collect::<Vec<_>>();
```

## FileSystem Trait

```rust
use paperclip_evaluator::bundle::{FileSystem, MockFileSystem, RealFileSystem};

// Production (default)
let fs = RealFileSystem;

// Testing
let mut mock_fs = MockFileSystem::new();
mock_fs.add_file(PathBuf::from("/test/file.pc"));

// Use with bundle
bundle.build_dependencies_with_fs(&root, &mock_fs)?;

// Custom implementation
struct MyFS;
impl FileSystem for MyFS {
    fn exists(&self, path: &Path) -> bool { /* ... */ }
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error> { /* ... */ }
}
```

## Error Handling

```rust
use paperclip_evaluator::BundleError;

match bundle.build_dependencies(&root) {
    Ok(_) => { /* success */ }
    Err(BundleError::ImportNotFound { import_path, source_path }) => {
        eprintln!("Import {} not found in {}", import_path, source_path);
    }
    Err(BundleError::CircularDependency { path }) => {
        eprintln!("Circular dependency at {}", path);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Common Patterns

### Pattern: Build bundle from directory

```rust
fn load_bundle_from_dir(dir: &Path) -> Result<Bundle, Box<dyn Error>> {
    let mut bundle = Bundle::new();

    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension() == Some(OsStr::new("pc")) {
            let source = std::fs::read_to_string(&path)?;
            let doc = parse_with_path(&source, &path.to_string_lossy())?;
            bundle.add_document(path, doc);
        }
    }

    bundle.build_dependencies(dir)?;
    Ok(bundle)
}
```

### Pattern: Track changes

```rust
fn find_affected_files(bundle: &Bundle, changed: &Path) -> Vec<PathBuf> {
    let mut affected = vec![changed.to_path_buf()];

    if let Some(dependents) = bundle.get_dependents(changed) {
        affected.extend(dependents.iter().cloned());
    }

    affected
}
```

### Pattern: Validate all imports

```rust
fn validate_imports(bundle: &Bundle) -> Vec<String> {
    let mut errors = Vec::new();

    for (file, doc) in bundle.documents() {
        for import in &doc.imports {
            if !import_exists(bundle, file, &import.path) {
                errors.push(format!(
                    "{}: cannot find import '{}'",
                    file.display(),
                    import.path
                ));
            }
        }
    }

    errors
}
```

## Type Reference

### Bundle

```rust
pub struct Bundle {
    // Private fields
}
```

**Methods:**
- `new() -> Self`
- `add_document(path: PathBuf, doc: Document)`
- `build_dependencies(&mut self, root: &Path) -> Result<(), BundleError>`
- `build_dependencies_with_fs(&mut self, root: &Path, fs: &dyn FileSystem) -> Result<(), BundleError>`
- `get_dependencies(&self, path: &Path) -> Option<&[PathBuf]>`
- `get_dependents(&self, path: &Path) -> Option<&[PathBuf]>`
- `get_document(&self, path: &Path) -> Option<&Document>`
- `get_document_id(&self, path: &Path) -> Option<&str>`
- `find_style(&self, ref: &str, current: &Path) -> Option<(&StyleDecl, PathBuf)>`
- `find_token(&self, ref: &str, current: &Path) -> Option<(&TokenDecl, PathBuf)>`
- `find_component(&self, ref: &str, current: &Path) -> Option<(&Component, PathBuf)>`
- `add_asset(&mut self, asset: AssetReference)`
- `assets(&self) -> &[AssetReference]`

### AssetReference

```rust
pub struct AssetReference {
    pub path: String,              // Original path from source
    pub asset_type: AssetType,     // Type of asset
    pub resolved_path: PathBuf,    // Full filesystem path
    pub source_file: PathBuf,      // Which PC file contains this asset
}
```

### AssetType

```rust
pub enum AssetType {
    Image,
    Font,
    Video,
    Audio,
    Other,
}
```

### FileSystem

```rust
pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error>;
}

pub struct RealFileSystem;      // Uses actual filesystem
pub struct MockFileSystem;      // For testing
```

### BundleError

```rust
pub enum BundleError {
    CircularDependency { path: String },
    ImportNotFound { import_path: String, source_path: String },
    StyleNotFound { name: String },
    TokenNotFound { name: String },
    ComponentNotFound { name: String },
}
```

## Cheat Sheet

| Task | Code |
|------|------|
| Create bundle | `Bundle::new()` |
| Add file | `bundle.add_document(path, doc)` |
| Build deps | `bundle.build_dependencies(&root)?` |
| Mock FS | `MockFileSystem::new()` |
| Add mock file | `mock_fs.add_file(path)` |
| Get deps | `bundle.get_dependencies(&path)` |
| Get importers | `bundle.get_dependents(&path)` |
| Find style | `bundle.find_style("ns.name", &file)` |
| Find token | `bundle.find_token("ns.name", &file)` |
| Add asset | `bundle.add_asset(asset)` |
| Get assets | `bundle.assets()` |
| Get doc | `bundle.get_document(&path)` |

## Tips

✅ **DO:**
- Build dependencies once after adding all documents
- Use MockFileSystem for tests
- Track source_file in AssetReference for debugging
- Handle BundleError variants explicitly

❌ **DON'T:**
- Rebuild dependencies after every document add
- Assume imports will resolve without checking
- Ignore circular dependency errors
- Use filename-only matching (use Bundle's resolution)

## See Also

- [BUNDLE_API_EXAMPLES.md](./BUNDLE_API_EXAMPLES.md) - Detailed examples
- [BUNDLE_FIXES.md](./BUNDLE_FIXES.md) - Implementation details
- `packages/evaluator/src/bundle.rs` - Source code
- `packages/evaluator/src/tests_bundle_filesystem.rs` - Test examples
