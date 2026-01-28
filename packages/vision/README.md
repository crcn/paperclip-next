# Paperclip Vision

Screenshot capture and visual documentation for Paperclip components.

## Philosophy

This is **not** a visual testing framework or Storybook alternative.

This is **design truth extraction** - materializing canonical visual states directly from Paperclip components.

### Core Principles

- ✅ Views live with components (via `@view` doc comments)
- ✅ No runtime JavaScript required
- ✅ Deterministic rendering
- ✅ Component-scoped capture (not full viewport)
- ✅ Outputs are treated as build artifacts

## Installation

```bash
cargo install --path packages/vision
```

## Usage

### CLI

**Capture a single file:**
```bash
paperclip-vision capture button.pc
```

**Capture a directory:**
```bash
paperclip-vision capture src/components --output ./vision
```

**Specify viewport:**
```bash
paperclip-vision capture button.pc --viewport mobile
paperclip-vision capture button.pc --viewport tablet
paperclip-vision capture button.pc --viewport desktop
```

**High-DPI capture:**
```bash
paperclip-vision capture button.pc --scale 2.0
```

### Programmatic API

```rust
use paperclip_vision::{VisionCapture, CaptureOptions, Viewport};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let capture = VisionCapture::new(PathBuf::from("./vision"))?;

    let mut options = CaptureOptions::default();
    options.viewport = Viewport::Mobile;

    let screenshots = capture.capture_file(
        &PathBuf::from("button.pc"),
        options
    )?;

    for screenshot in screenshots {
        println!("Captured: {} -> {}",
            screenshot.view_name,
            screenshot.path.display()
        );
    }

    Ok(())
}
```

## @view Annotations

Define views using doc comments in your `.pc` files:

```javascript
/// @view default
/// @view hover - Hover state
/// @view disabled - Disabled button state
/// @viewport mobile
public component Button {
    render button {
        style {
            padding: 8px 16px
            background: blue
            color: white
        }
        text "Click me"
    }
}
```

### Annotation Syntax

**Basic view:**
```javascript
/// @view default
```

**View with description:**
```javascript
/// @view hover - Shows hover state styling
```

**Viewport directive:**
```javascript
/// @viewport mobile
/// @view default
```

Viewport applies to all subsequent `@view` annotations until changed.

### Auto-Generated Views

If no `@view` annotations are found, a `default` view is auto-generated for each public component.

## Output Structure

Vision generates a standardized output structure:

```
vision/
  Button/
    Button.default.png
    Button.hover.png
    Button.disabled.png
  Card/
    Card.default.png
  manifest.json
```

### Manifest JSON

Each capture generates a `manifest.json` with metadata:

```json
{
  "component_name": "Button",
  "source_path": "/path/to/button.pc",
  "generated_at": "2026-01-28T12:00:00Z",
  "views": [
    {
      "view_name": "default",
      "component_name": "Button",
      "path": "vision/Button.default.png",
      "width": 120,
      "height": 48,
      "viewport": "Desktop",
      "timestamp": "2026-01-28T12:00:00Z"
    }
  ]
}
```

## Capture Modes

### Component Bounds (Default)

Captures only the component's bounding box, eliminating whitespace:

```bash
paperclip-vision capture button.pc  # Default
```

Uses the `data-pc-root` attribute to identify component boundaries.

### Full Viewport

Captures the entire viewport:

```bash
# Not yet implemented
# paperclip-vision capture button.pc --capture-area viewport
```

## Viewport Presets

| Preset | Dimensions | Use Case |
|--------|-----------|----------|
| `mobile` | 375×667 | iPhone SE |
| `tablet` | 768×1024 | iPad |
| `desktop` | 1920×1080 | HD Display |

## How It Works

1. **Parse** - Extract `@view` annotations from doc comments
2. **Render** - Generate standalone HTML with inline styles
3. **Serve** - Start disposable HTTP server
4. **Capture** - Launch headless Chrome, screenshot component bounds
5. **Save** - Write PNG + metadata JSON

## Architecture

```
┌─────────────────────────────────────┐
│          .pc Source File            │
│  /// @view default                  │
│  /// @view hover                    │
│  component Button { ... }           │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         Parser (parser.rs)          │
│  Extract @view annotations          │
│  → ViewSpec[]                       │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│       Renderer (renderer.rs)        │
│  AST → Targeted DOM emitter         │
│  → Standalone HTML                  │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│        Server (server.rs)           │
│  Disposable HTTP server             │
│  → URL                              │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│       Capture (capture.rs)          │
│  headless_chrome                    │
│  Component bounds detection         │
│  → PNG + metadata                   │
└─────────────────────────────────────┘
```

## Limitations (v1)

- ✅ Static components only (no JS interactivity)
- ✅ Inline styles only (no external CSS)
- ✅ Single component per capture
- ✅ Basic viewport presets

These constraints are **features**, not bugs. They ensure deterministic, predictable output.

## Future Enhancements

- [ ] Visual diffing between captures
- [ ] Multiple components per screenshot
- [ ] Custom viewport dimensions
- [ ] JPEG output format
- [ ] Batch capture optimizations
- [ ] AI-based layout analysis

## Requirements

- Chrome or Chromium installed (for headless_chrome)
- Rust 1.70+

## License

MIT
