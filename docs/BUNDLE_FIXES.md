# Bundle Architecture Fixes

## Summary

Fixed three critical issues with asset loading and file resolution in the bundle system:

1. **Assets now loaded into bundle** - Assets are properly tracked at bundle level
2. **Robust file resolution** - Implemented proper path normalization and matching
3. **Correct alias resolution** - Import aliases now map to specific imported files

## Changes Made

### 1. FileSystem Trait (packages/evaluator/src/bundle.rs)

Added abstraction for file system operations to enable mocking in tests:

```rust
pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error>;
}

pub struct RealFileSystem;  // Production implementation
pub struct MockFileSystem;  // Testing implementation
```

**Benefits:**
- Testable without real file I/O
- Clean separation of concerns
- Mockable for unit tests

### 2. Import Alias Mapping (packages/evaluator/src/bundle.rs)

Added `import_aliases` field to Bundle:

```rust
pub struct Bundle {
    ...
    /// Import alias mapping: (source_file, alias) -> resolved_path
    import_aliases: HashMap<(PathBuf, String), PathBuf>,
    ...
}
```

**How it works:**
- When `import "./theme.pc" as theme` is parsed
- Stores mapping: `(button.pc, "theme") -> theme.pc`
- When looking up `theme.fontRegular`:
  1. Split into namespace ("theme") and name ("fontRegular")
  2. Look up `(current_file, "theme")` in import_aliases
  3. Get resolved path (theme.pc)
  4. Look for "fontRegular" in that specific file

**Before (BROKEN):**
```rust
// Searched through ALL dependencies
for dep_path in deps {
    if let Some(dep_doc) = self.documents.get(dep_path) {
        // Check every dependency file!
    }
}
```

**After (FIXED):**
```rust
// Direct lookup using alias map
let key = (current_file.to_path_buf(), namespace.to_string());
if let Some(imported_file) = self.import_aliases.get(&key) {
    // Check only the specific imported file
    if let Some(dep_doc) = self.documents.get(imported_file) {
        // Find the style/token/component
    }
}
```

### 3. Improved Path Resolution (packages/evaluator/src/bundle.rs)

Implemented robust path matching with multiple strategies:

```rust
fn resolve_import_path(...) -> Result<PathBuf, BundleError> {
    // 1. Try resolved path as-is
    if self.documents.contains_key(&resolved) { ... }

    // 2. Try canonicalized (handles symlinks, makes absolute)
    if let Ok(canonicalized) = fs.canonicalize(&resolved) { ... }

    // 3. Try normalized path (remove ./ and ../)
    let normalized = self.normalize_path(&resolved);
    if self.documents.contains_key(&normalized) { ... }

    // 4. Fuzzy match (handles absolute vs relative)
    for existing_path in self.documents.keys() {
        if self.paths_match(&resolved, existing_path) { ... }
    }
}
```

**Benefits:**
- Handles temp directories (tests use `/var/folders/...`)
- Handles relative vs absolute paths
- Handles normalized vs non-normalized paths
- Much more robust than previous filename-only matching

### 4. Assets Loaded into Bundle (packages/workspace/src/state.rs)

Fixed asset tracking:

**Before:**
```rust
// Assets extracted but NOT added to bundle
let new_assets = extract_assets(&new_ast, project_root);

self.files.insert(path, FileState {
    assets: new_assets,  // Only in FileState!
    ...
});
```

**After:**
```rust
// Assets extracted AND added to bundle
let new_assets = extract_assets(&new_ast, project_root, &path);

// Add assets to bundle
for asset in &new_assets {
    self.bundle.add_asset(asset.clone());
}

self.files.insert(path, FileState {
    assets: new_assets,
    ...
});
```

**Benefits:**
- Assets tracked globally at bundle level
- Can query all assets across entire project
- Each asset knows its source file

### 5. Unified AssetReference Type

Removed duplicate AssetReference definition from workspace, now uses evaluator's:

```rust
// packages/workspace/src/state.rs
use paperclip_evaluator::{..., AssetReference, AssetType};

// packages/workspace/src/lib.rs
pub use paperclip_evaluator::{AssetReference, AssetType};
```

**Benefits:**
- Single source of truth
- Includes `source_file` field
- Consistent across codebase

## Test Coverage

### New Tests (packages/evaluator/src/tests_bundle_filesystem.rs)

Added 6 comprehensive tests:

1. **test_mock_filesystem_import_resolution** - Basic import resolution with mock FS
2. **test_alias_resolution_with_mock_fs** - Namespaced lookups via aliases
3. **test_multiple_imports_with_different_aliases** - Multiple imports in same file
4. **test_import_not_found_with_mock_fs** - Error handling for missing imports
5. **test_asset_tracking_in_bundle** - Single asset tracking
6. **test_assets_from_multiple_files** - Multi-file asset tracking

### Test Results

```
✅ 82 evaluator tests (was 76)
✅ 31 parser tests
✅ 19 workspace tests
✅ 3 bundle integration tests
✅ 9 general integration tests
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 144 total tests passing
```

## API Changes

### Public API Additions

**Bundle:**
```rust
// New methods
pub fn build_dependencies_with_fs(&mut self, root: &Path, fs: &dyn FileSystem) -> Result<(), BundleError>;

// New public types
pub trait FileSystem { ... }
pub struct RealFileSystem;
pub struct MockFileSystem;
```

**AssetReference now includes:**
```rust
pub struct AssetReference {
    pub path: String,
    pub asset_type: AssetType,
    pub resolved_path: PathBuf,
    pub source_file: PathBuf,  // NEW: track which file the asset came from
}
```

## Migration Notes

### For Users

No migration needed - all changes are internal improvements.

### For Testing

Can now use MockFileSystem for bundle tests:

```rust
let mut mock_fs = MockFileSystem::new();
mock_fs.add_file(PathBuf::from("/project/theme.pc"));
bundle.build_dependencies_with_fs(&root, &mock_fs)?;
```

## Performance Impact

**Improved:**
- Alias resolution is now O(1) hash lookup instead of O(n) linear search
- Path matching uses early returns for common cases

**No Change:**
- Asset extraction still happens per-file during parsing
- Bundle building still O(n) where n = number of imports

## Future Improvements

1. **Cache normalized paths** - Avoid repeated normalization
2. **Lazy asset extraction** - Only extract when assets are queried
3. **Asset deduplication** - Track unique assets across files
4. **Parallel dependency building** - Build import graph concurrently

## Related Files

- `packages/evaluator/src/bundle.rs` - Core bundle implementation
- `packages/evaluator/src/tests_bundle_filesystem.rs` - New test suite
- `packages/workspace/src/state.rs` - Workspace integration
- `packages/workspace/src/lib.rs` - Public API re-exports
