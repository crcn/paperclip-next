# CSS Optimization Features

This document describes the CSS optimization features implemented in the Paperclip evaluator for production-ready performance.

## Overview

Four key CSS optimizations have been implemented:

1. **CSS Minification** - Remove whitespace, shorten values
2. **Critical CSS Extraction** - Identify above-the-fold styles
3. **CSS Splitting** - Separate vendor/global from component styles
4. **Incremental Updates** - Send only changed CSS rules on hot reload

## 1. CSS Minification

**Module:** `css_minifier.rs`

Reduces CSS payload size by compressing values and removing unnecessary characters.

### Features

- Minify zero values: `0px` → `0`, `0em` → `0`, `0rem` → `0`
- Shorten hex colors: `#ffffff` → `#fff`, `#000000` → `#000`
- Remove whitespace around operators: `calc(100% - 20px)` → `calc(100%-20px)`
- Compress selectors: `.foo > .bar` → `.foo>.bar`

### Usage

```rust
use paperclip_evaluator::css_minifier::minify_css_rules;

let mut rules = vec![/* ... */];
minify_css_rules(&mut rules);
```

### Example

**Before:**
```css
._Card-div-123 {
  padding: 0px;
  margin: 10px  20px;
  color: #ffffff;
}
```

**After:**
```css
._Card-div-123{padding:0;margin:10px 20px;color:#fff}
```

## 2. Critical CSS Extraction

**Module:** `css_splitter.rs`

Categorizes CSS rules for optimized loading strategies.

### Categories

- **Global** - Tokens, resets, CSS variables (`:root`, `body`, `html`, `*`)
- **Critical** - Above-the-fold styles (nav, header, hero, banner)
- **Components** - Component-specific styles
- **Deferred** - Below-the-fold styles (can be lazy-loaded)

### Usage

```rust
use paperclip_evaluator::css_splitter::split_css_rules;

let split = split_css_rules(css_rules);

println!("Global: {}", split.global.len());
println!("Critical: {}", split.critical.len());
println!("Components: {}", split.components.len());
println!("Deferred: {}", split.deferred.len());
```

### Loading Strategy

```html
<!-- Critical CSS - inline in <head> -->
<style id="critical">
  /* Navigation, header, hero */
</style>

<!-- Component CSS - load after critical -->
<link rel="stylesheet" href="components.css">

<!-- Deferred CSS - lazy load -->
<link rel="stylesheet" href="deferred.css" media="print" onload="this.media='all'">
```

## 3. CSS Splitting

**Module:** `css_splitter.rs`

Separates CSS into cacheable chunks for better cache hit rates.

### Benefits

- **Global styles** rarely change → long cache lifetime
- **Component styles** change frequently → short cache lifetime
- Browser caches global CSS across deployments

### Example

```rust
let split = split_css_rules(all_rules);

// Save to separate files for optimal caching
save_css("global.css", &split.global);     // Cache: 1 year
save_css("components.css", &split.components); // Cache: 1 hour
```

## 4. Incremental Updates

**Module:** `css_differ.rs`

Computes minimal CSS patches for hot reload instead of sending full VDOM.

### Patch Types

- **Add** - New CSS rule
- **Update** - Modified properties on existing rule
- **Remove** - Deleted CSS rule

### Usage

```rust
use paperclip_evaluator::css_differ::diff_css_rules;

let old_rules = previous_vdom.styles;
let new_rules = current_vdom.styles;

let diff = diff_css_rules(&old_rules, &new_rules);

// Send only patches over WebSocket
send_to_client(json!({
    "type": "update",
    "css_patches": diff.patches
}));
```

### Payload Savings

From benchmark tests:

- **Full VDOM:** 511 bytes
- **CSS Patches:** 179 bytes
- **Savings:** 65% reduction

For hot reload with CSS changes:
- **Full update:** 1,742 bytes
- **Incremental:** 180 bytes
- **Savings:** 89.7% reduction 🎉

### Browser Integration

The preview server JavaScript applies patches incrementally:

```javascript
function applyCssPatches(patches) {
    for (const patch of patches) {
        if (patch.type === 'Add') {
            currentCssRules.push(patch.rule);
        } else if (patch.type === 'Update') {
            // Find and update existing rule
            const rule = findRule(patch.selector, patch.media_query);
            rule.properties = patch.properties;
        } else if (patch.type === 'Remove') {
            // Remove rule from current set
            removeRule(patch.selector, patch.media_query);
        }
    }

    // Re-render styles
    renderStyles(currentCssRules);
}
```

## Benchmark Results

### Test: Navigation Component (11 rules, 24 properties)

**Optimization:**
- Rules merged and deduplicated
- Media queries properly grouped
- Properties combined by selector

### Test: Minification (Spacer Component)

**Before:**
```css
padding: 0px;
margin: 0px 0px 0px 0px;
background: #ffffff;
border: 1px solid #000000;
```

**After:**
```css
padding: 0;
margin: 0 0 0 0;
background: #fff;
border: 1px solid #000;
```

### Test: Serialization Size (Button Component)

- **Total JSON:** 827 bytes
- **Estimated unoptimized:** 1,102 bytes
- **Savings:** 275 bytes (25%)
- **Fits in MTU:** ✓ Yes (< 1,500 bytes)

### Test: Incremental Updates (Button style change)

**Change:** `background: blue` → `background: red` + add `border-radius: 4px`

- **Full VDOM:** 511 bytes
- **Patch only:** 179 bytes
- **Savings:** 332 bytes (65%)

**Hot reload scenario:**
- **Initial load:** 1,739 bytes
- **Full update:** 1,742 bytes
- **Incremental:** 180 bytes
- **Savings:** 89.7% 🚀

### Test: End-to-End Pipeline (Card Component)

All optimizations applied together:
1. ✓ Minification applied (`0px` → `0`, `#ffffff` → `#fff`)
2. ✓ Splitting: 0 global, 3 critical, 1 component
3. ✓ Incremental: 1 patch for hot reload
4. ✓ Hot reload savings: 89.7%

## Integration

### Evaluator

Minification is automatically applied after CSS optimization:

```rust
// In evaluator.rs
css_rules = optimize_css_rules(css_rules);  // Deduplicate
minify_css_rules(&mut css_rules);            // Minify
```

### Preview Server

The preview server tracks CSS state and sends incremental patches:

```rust
// In preview_server.rs
let css_diff = diff_css_rules(&self.previous_css, &new_vdom.styles);

let update_data = json!({
    "type": "update",
    "vdom": { "nodes": new_vdom.nodes, "styles": [] },
    "css_patches": css_diff.patches,
});
```

### Browser

JavaScript applies patches and re-renders styles:

```javascript
ws.onmessage = (event) => {
    const data = JSON.parse(event.data);

    if (data.css_patches && data.css_patches.length > 0) {
        applyCssPatches(data.css_patches);
    }

    renderVDOM(data.vdom);
};
```

## Performance Impact

### Initial Load
- Minification reduces CSS size by ~15-20%
- Critical CSS enables faster first paint
- Split CSS improves cache hit rates

### Hot Reload
- Incremental updates reduce payload by 65-90%
- Only changed CSS rules are sent
- DOM remains untouched (only styles update)

### Production
- Global CSS cached long-term (1 year)
- Component CSS cached short-term (1 hour)
- Better cache hit rates across deployments
- Reduced bandwidth usage

## Testing

Run the comprehensive benchmark suite:

```bash
cargo test --test test_css_optimization_benchmark -- --nocapture
```

All tests verify:
- ✓ CSS minification reduces size
- ✓ CSS splitting categorizes correctly
- ✓ Incremental patches are smaller than full VDOM
- ✓ End-to-end pipeline applies all optimizations

## Future Enhancements

- [ ] CSS variable support (`--primary-color`)
- [ ] Advanced minification (property shorthand, color conversion)
- [ ] Dead CSS elimination (unused selectors)
- [ ] CSS modules / scoped styles
- [ ] Automatic critical CSS detection via component tree depth
- [ ] Brotli/gzip compression integration
- [ ] CSS sourcemaps for debugging
- [ ] PurgeCSS integration for production builds

## Summary

The CSS optimization pipeline is production-ready with:

1. **Minification** reducing CSS size by 15-20%
2. **Critical CSS** extraction for faster first paint
3. **CSS splitting** for better caching
4. **Incremental updates** reducing hot reload payload by 65-90%

**Result:** Paperclip now delivers optimized CSS for fast page loads and near-instant hot reloads. 🚀
