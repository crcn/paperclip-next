# Architecture Questions & Answers

## Q1: Asset Deduplication - What if hero.jpg is used 100 times?

**Current Behavior:**
```rust
// If hero.jpg appears in 100 PC files, we get 100 AssetReference entries
bundle.assets().len()  // Returns 100
```

**Problem:** Duplication wastes memory and makes queries inefficient.

**Proposed Solution:** Add asset deduplication with source tracking

### Option A: Deduplicate at Bundle Level (Recommended)

```rust
pub struct Bundle {
    // ...existing fields...

    /// Deduplicated assets with source file tracking
    /// Maps asset path -> (AssetReference, Set<source_files>)
    assets: HashMap<String, (AssetReference, HashSet<PathBuf>)>,
}

impl Bundle {
    pub fn add_asset(&mut self, asset: AssetReference) {
        self.assets
            .entry(asset.path.clone())
            .and_modify(|(_, sources)| {
                sources.insert(asset.source_file.clone());
            })
            .or_insert_with(|| (asset.clone(), {
                let mut set = HashSet::new();
                set.insert(asset.source_file.clone());
                set
            }));
    }

    /// Get unique assets
    pub fn unique_assets(&self) -> impl Iterator<Item = &AssetReference> {
        self.assets.values().map(|(asset, _)| asset)
    }

    /// Get all source files that use an asset
    pub fn asset_users(&self, asset_path: &str) -> Option<&HashSet<PathBuf>> {
        self.assets.get(asset_path).map(|(_, sources)| sources)
    }

    /// Get assets used by a specific file
    pub fn assets_for_file(&self, file: &Path) -> Vec<&AssetReference> {
        self.assets.values()
            .filter(|(_, sources)| sources.contains(file))
            .map(|(asset, _)| asset)
            .collect()
    }
}
```

**Usage:**
```rust
// Add assets (deduplicates automatically)
bundle.add_asset(hero_jpg_from_home);  // First occurrence
bundle.add_asset(hero_jpg_from_about); // Adds to sources
// ... 98 more times ...

// Query
println!("Unique assets: {}", bundle.unique_assets().count());  // 1
println!("hero.jpg used by: {:?}", bundle.asset_users("/images/hero.jpg"));
// Output: {/project/home.pc, /project/about.pc, ...}

// Get assets for specific file
let home_assets = bundle.assets_for_file(&PathBuf::from("/project/home.pc"));
```

### Option B: Keep All References (Current)

Pros:
- Simple implementation
- Preserves all context (can see every usage)
- Easy to iterate per-file

Cons:
- Wastes memory
- Harder to count unique assets
- Need to deduplicate in queries

**Recommendation:** Implement Option A for better performance and cleaner API.

---

## Q2: In WorkspaceState, why Bundle AND FileState?

**Current Structure:**
```rust
pub struct WorkspaceState {
    files: HashMap<PathBuf, FileState>,  // Per-file cache
    bundle: Bundle,                       // Cross-file data
}

pub struct FileState {
    pub source: String,              // Source code
    pub ast: Document,               // Parsed AST
    pub vdom: VirtualDomDocument,    // Evaluated DOM
    pub css: VirtualCssDocument,     // Evaluated CSS
    pub assets: Vec<AssetReference>, // Assets from this file
    pub version: u64,                // For change tracking
    pub document_id: String,         // CRC32 ID
}
```

### Why Both?

**FileState** stores **per-file output** from evaluation:
- `vdom` - The virtual DOM for **this file's** components
- `css` - The CSS rules for **this file's** styles
- `assets` - Assets referenced in **this file**
- `version` - Increments on each file update (for diffing)

**Bundle** stores **cross-file input** for evaluation:
- All parsed ASTs (needed to resolve imports)
- Dependency graph (who imports whom)
- Import alias mappings (namespace → file)
- Global asset registry (deduplicated)

### They Serve Different Purposes

```rust
// Scenario: Update theme.pc

// 1. Bundle tracks dependencies
let affected_files = bundle.get_dependents(&theme_path);
// Returns: [button.pc, card.pc, page.pc]

// 2. FileState tracks what changed
for file in affected_files {
    let old_state = workspace.files.get(&file);
    let old_version = old_state.version;  // e.g., 5

    // Re-evaluate with updated bundle
    workspace.update_file(file, source, root)?;

    let new_state = workspace.files.get(&file);
    let new_version = new_state.version;  // Now 6

    // Generate patches by diffing old vs new
    let patches = diff_vdocument(&old_state.vdom, &new_state.vdom);
}
```

### Could We Eliminate FileState?

**No**, because:

1. **Version Tracking** - Bundle doesn't track file versions for diffing
2. **Evaluation Output** - Bundle stores AST (input), not VDOM/CSS (output)
3. **HMR/Patching** - Need old VDOM to diff against new VDOM
4. **Performance** - Cache evaluated results instead of re-evaluating from scratch

### Could We Eliminate Bundle?

**No**, because:

1. **Import Resolution** - Need dependency graph to resolve `theme.fontRegular`
2. **Circular Detection** - Need full graph to detect cycles
3. **HMR Propagation** - Need dependents list to know what to re-evaluate
4. **Cross-File Queries** - Need to find styles/tokens in imported files

### Ideal Architecture

```
Bundle (Cross-File Data)           FileState (Per-File Cache)
├─ documents: AST storage          ├─ source: string
├─ dependencies: import graph      ├─ ast: Document (redundant with Bundle?)
├─ import_aliases: namespace map   ├─ vdom: VirtualDomDocument (output)
└─ assets: deduplicated registry   ├─ css: VirtualCssDocument (output)
                                   ├─ assets: redundant with Bundle
                                   └─ version: for diffing
```

### Potential Optimization

Remove redundant data from FileState:

```rust
pub struct FileState {
    pub source: String,                  // Keep - useful for debugging
    // pub ast: Document,                // REMOVE - duplicate of bundle.documents
    pub vdom: VirtualDomDocument,        // Keep - evaluation output
    pub css: VirtualCssDocument,         // Keep - evaluation output
    // pub assets: Vec<AssetReference>,  // REMOVE - use bundle.assets_for_file()
    pub version: u64,                    // Keep - for version tracking
    pub document_id: String,             // Keep - convenient cache
}
```

**Benefits:**
- Less memory duplication
- Single source of truth for AST and assets
- Simpler to keep in sync

**Trade-offs:**
- Slightly more indirection (need bundle reference to get AST)
- Less self-contained FileState

---

## Q3: Making build_dependencies_with_fs async

**Current Signature:**
```rust
pub fn build_dependencies_with_fs(
    &mut self,
    project_root: &Path,
    fs: &dyn FileSystem,
) -> Result<(), BundleError>
```

**Why make it async?**
- File I/O operations (`exists()`, `canonicalize()`) can block
- Better for servers handling many concurrent requests
- Fits async Rust ecosystem (tokio, async-std)

### Approach 1: Async Trait (Most Correct)

```rust
#[async_trait]
pub trait FileSystem {
    async fn exists(&self, path: &Path) -> bool;
    async fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error>;
}

pub struct RealFileSystem;

#[async_trait]
impl FileSystem for RealFileSystem {
    async fn exists(&self, path: &Path) -> bool {
        tokio::fs::metadata(path).await.is_ok()
    }

    async fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
        tokio::fs::canonicalize(path).await
    }
}

impl Bundle {
    pub async fn build_dependencies_with_fs(
        &mut self,
        project_root: &Path,
        fs: &dyn FileSystem,
    ) -> Result<(), BundleError> {
        // ...
        let resolved = /* ... */;
        if fs.exists(&resolved).await {  // .await here
            return Ok(resolved);
        }
        // ...
    }
}
```

**Pros:**
- Truly async I/O
- Non-blocking
- Scales well

**Cons:**
- Breaking change (all callers must use .await)
- Requires async-trait crate for trait objects
- More complex

### Approach 2: Async Wrapper (Easier Migration)

Keep FileSystem sync, but make the public API async:

```rust
// Keep FileSystem sync
pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error>;
}

impl Bundle {
    // Public async API
    pub async fn build_dependencies_with_fs_async(
        &mut self,
        project_root: &Path,
        fs: &dyn FileSystem,
    ) -> Result<(), BundleError> {
        // Spawn blocking task for sync work
        let bundle_clone = self.clone();
        let root = project_root.to_path_buf();

        tokio::task::spawn_blocking(move || {
            bundle_clone.build_dependencies_with_fs_sync(&root, fs)
        }).await?
    }

    // Private sync implementation
    fn build_dependencies_with_fs_sync(
        &mut self,
        project_root: &Path,
        fs: &dyn FileSystem,
    ) -> Result<(), BundleError> {
        // Current implementation
    }
}
```

**Pros:**
- No breaking changes to FileSystem trait
- Easier migration
- Async where it matters (public API)

**Cons:**
- Still blocks thread pool
- Not truly async

### Recommendation

**For now:** Approach 2 (async wrapper)
- Easy to implement
- Backwards compatible
- Can migrate to Approach 1 later

**Long term:** Approach 1 (fully async)
- Better performance
- More idiomatic async Rust

Would you like me to implement one of these approaches?

---

## Summary of Recommended Changes

1. **Asset Deduplication**
   - Change `Bundle.assets` from `Vec` to `HashMap<String, (AssetReference, HashSet<PathBuf>)>`
   - Add `unique_assets()`, `asset_users()`, `assets_for_file()` methods
   - Update `add_asset()` to deduplicate

2. **FileState Optimization**
   - Remove `ast` field (use `bundle.get_document()` instead)
   - Remove `assets` field (use `bundle.assets_for_file()` instead)
   - Keep `vdom`, `css`, `version`, `document_id`, `source`

3. **Async Build Dependencies**
   - Add `build_dependencies_with_fs_async()` with spawn_blocking
   - Keep existing sync method for compatibility
   - Consider full async trait migration later

Would you like me to implement these changes?
