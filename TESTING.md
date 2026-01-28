# Testing Spikes 0.4 & 0.5

This guide explains how to test the implemented spikes and verify they work correctly.

---

## Prerequisites

Make sure you have the following installed:
- **Rust** (1.70+): `rustc --version`
- **Node.js** (18+): `node --version`
- **Cargo**: Should come with Rust
- **Yarn** (1.x): `yarn --version` or install via `npm install -g yarn`

---

## Testing Spike 0.4: State Management + Patches (Rust)

### Run Unit Tests

```bash
# Test WorkspaceState (file caching, version tracking)
cargo test -p paperclip-workspace --lib state

# Expected output:
# running 3 tests
# test state::tests::test_workspace_state_creation ... ok
# test state::tests::test_file_caching ... ok
# test state::tests::test_version_increment ... ok

# Test VDocument Differ (patch generation)
cargo test -p paperclip-evaluator vdom_differ

# Expected output:
# running 4 tests
# test vdom_differ::tests::test_diff_create_node ... ok
# test vdom_differ::tests::test_diff_remove_node ... ok
# test vdom_differ::tests::test_diff_update_text ... ok
# test vdom_differ::tests::test_diff_update_attributes ... ok
```

### Verify Build

```bash
# Build entire workspace
cargo build

# Should complete without errors
# Look for: Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### What These Tests Validate

1. **State Caching**: Files are cached after first parse
2. **Version Tracking**: Version numbers increment on updates
3. **Patch Generation**: Minimal patches created by diffing
4. **Node Operations**: Create, Remove, Update patches work correctly

---

## Testing Spike 0.5: Live Component Preview (TypeScript)

### Install Dependencies

```bash
cd packages/client
yarn install

# Should install React 18 and dependencies
# Expected output:
# added 9 packages, and audited 57 packages
```

### Verify TypeScript Compilation

```bash
yarn build

# Should complete without errors
# Look for: No error output
```

### Run Development Server

```bash
yarn dev

# Expected output:
# VITE v5.x.x  ready in XXX ms
# ➜  Local:   http://localhost:5173/
```

### Test the Demo

1. **Open the hybrid demo**:
   - Navigate to http://localhost:5173/hybrid-demo.html
   - Or click the link in the terminal

2. **What you should see**:
   ```
   Hybrid Rendering Demo
   ┌─────────────────────────────────┐
   │ Choose a date                   │
   │ [2024-01-15 date input]         │
   │ Selected: 2024-01-15            │
   └─────────────────────────────────┘
   ```

3. **Test interactions**:
   - Click the date picker
   - Select a different date
   - Verify "Selected: [date]" updates

4. **Wait 2 seconds**:
   - Label should change to: "Updated: Pick a different date"
   - Date should reset to: 2024-12-25

5. **Open browser console**:
   ```
   🚀 Starting Hybrid Rendering Demo
   📦 Loading DatePicker component
   ✅ DatePicker registered
   🎨 Rendering initial state
   ✅ Initial render complete
   ✨ Demo running - watch for prop update in 2 seconds
   🔄 Updating component props
   ✅ Props updated
   ```

6. **Open React DevTools** (if installed):
   - Should see `DatePicker` component in the tree
   - Can inspect props and state

### What This Demo Validates

1. **Component Registration**: DatePicker successfully registered
2. **Dynamic Loading**: Component loaded via dynamic import
3. **React Integration**: Component renders using React 18 createRoot
4. **Hybrid Patching**: Component patches applied alongside DOM patches
5. **Prop Updates**: Component re-renders when props change
6. **Lifecycle**: React root properly mounted and updated

---

## Testing Both Spikes Together (Integration)

While the spikes are designed to work independently, here's how they'll integrate in production:

### Simulated End-to-End Flow

1. **Server Side** (Spike 0.4):
   ```bash
   # Start the workspace server (when available)
   cargo run --bin paperclip-server -- --root examples

   # Server will:
   # - Watch for file changes
   # - Parse and cache files
   # - Generate VDocument patches
   # - Stream patches via gRPC
   ```

2. **Client Side** (Spike 0.5):
   ```typescript
   // Client will:
   // - Receive VDocPatch[] from server
   // - Apply patches to VDocument state
   // - Generate DOM patches via diff()
   // - Apply via hybridPatchApplier
   // - Mount/update React components
   ```

### Full Integration (Future)

The complete system will be tested with:

```bash
# Terminal 1: Start server
cargo run --bin paperclip-server -- --root examples

# Terminal 2: Start client
cd packages/client && yarn dev

# Terminal 3: Edit a file
echo 'component Button { <button>Updated</button> }' > examples/button.pc

# Result:
# - Server detects change
# - Generates patches (not full VDocument)
# - Client receives patches
# - Updates preview without full re-render
```

---

## Troubleshooting

### Rust Tests Fail

**Issue**: `cargo test` fails with compilation errors

**Solution**:
```bash
# Update dependencies
cargo update

# Clean build
cargo clean && cargo build

# Try again
cargo test
```

### TypeScript Compilation Fails

**Issue**: `yarn build` shows type errors

**Solution**:
```bash
# Reinstall dependencies
rm -rf node_modules yarn.lock
yarn install

# Check tsconfig.json has jsx: "react-jsx"
cat tsconfig.json | grep jsx

# Try again
yarn build
```

### Demo Page Blank

**Issue**: http://localhost:5173/hybrid-demo.html shows blank page

**Solution**:
1. Open browser console (F12)
2. Check for errors
3. Common issues:
   - Module not found: Check file paths
   - React not loaded: Run `yarn install`
   - Port conflict: Use different port with `yarn dev -- --port 3000`

### Component Not Rendering

**Issue**: Component placeholder shows but React component doesn't render

**Solution**:
1. Check console for errors
2. Verify React DevTools shows component
3. Check component is exported correctly:
   ```typescript
   // DatePicker.tsx must have:
   export function DatePicker(...) { ... }
   ```
4. Check registration uses correct export name:
   ```typescript
   registry.register({
     exportName: "DatePicker"  // Must match export
   });
   ```

---

## Performance Testing (Optional)

### Measure Patch Generation Time

```bash
# Add timing to state tests
cargo test -p paperclip-workspace state::tests -- --nocapture

# Look for timing logs (if added):
# Cached parse: <1ms
# Fresh parse: ~10ms
# Patch generation: <1ms
```

### Measure Client Rendering

```javascript
// In browser console:
console.time('patch-apply');
applier.apply(patches, rootElement);
console.timeEnd('patch-apply');

// Expected: <10ms for typical patch sets
```

---

## Validation Checklist

Use this checklist to verify both spikes work correctly:

### Spike 0.4 (Rust)
- [ ] All state tests pass
- [ ] All differ tests pass
- [ ] Cargo build succeeds
- [ ] No compilation warnings (except deprecation warnings)
- [ ] Protobuf types compile correctly

### Spike 0.5 (TypeScript)
- [ ] yarn install succeeds
- [ ] yarn build succeeds (no TypeScript errors)
- [ ] Dev server starts
- [ ] Demo page loads at /hybrid-demo.html
- [ ] DatePicker component renders
- [ ] Can interact with date picker
- [ ] Props update after 2 seconds
- [ ] Console shows success messages
- [ ] No errors in browser console
- [ ] React DevTools shows component tree

### Both Spikes
- [ ] Documentation is clear and complete
- [ ] Code is well-commented
- [ ] Architecture decisions are validated
- [ ] Ready to proceed to production implementation

---

## Next Steps After Testing

Once all tests pass:

1. ✅ Review test results with team
2. ✅ Validate architecture decisions
3. ⏭️ Plan production implementation (Phase 1: Complete server-side)
4. ⏭️ Set up CI/CD for continuous testing
5. ⏭️ Begin implementing production features

---

## Questions?

If you encounter issues not covered here:

1. Check `SPIKE_0.4_COMPLETE.md` for Rust implementation details
2. Check `SPIKE_0.5_COMPLETE.md` for TypeScript implementation details
3. Check `SPIKES_IMPLEMENTATION_COMPLETE.md` for architecture overview
4. Review code comments for specific behavior
5. Open an issue describing the problem with:
   - What you tried
   - Expected result
   - Actual result
   - Console output / error messages
