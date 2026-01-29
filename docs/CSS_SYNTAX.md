# Paperclip CSS Syntax Reference

Based on the original Paperclip language specification.

## Overview

Paperclip is a domain-specific language for building UI components that compile to HTML, CSS, and framework-specific code. It uses standard CSS syntax within `style {}` blocks and compiles to CSS with generated selectors.

**Current Implementation:** Standard CSS syntax (colons and semicolons required).

```paperclip
style {
    color: red;           // ✓ Current syntax
    padding: 10px 20px;   // ✓ Standard CSS
}
```

**Note:** Simplified syntax (no colons/semicolons) is planned for future versions.

## CSS Properties

Standard CSS properties work exactly as in regular CSS:

```paperclip
style {
    color: red;
    background: #333;
    padding: 10px 20px;
    width: calc(100% - 20px);
    background: url("./image.png");
    font-family: Inter, sans-serif;
}
```

## Tokens (CSS Variables)

Define reusable design values:

```paperclip
public token gray01 #333
public token fontFamily01 Inter, sans-serif
public token spacing04 16px
public token imageUrl url("./image.png")
```

Use in styles:

```paperclip
style {
    color: var(gray01);
    font-family: var(fontFamily01);
    padding: var(spacing04);
}
```

With imports:

```paperclip
import "./theme.pc" as theme

style {
    color: var(theme.primaryColor);
}
```

## Style Mixins

Reusable groups of CSS declarations that compile to CSS classes:

```paperclip
public style defaultFont {
    font-family: Inter, sans-serif;
    font-size: 11px;
    color: #333;
}

// Extend other styles
public style largeFont extends defaultFont {
    font-size: 16px;
}
```

Use in components:

```paperclip
div {
    style extends defaultFont {
        padding: 10px;
    }
}
```

## Triggers (Selectors & Media Queries)

Define CSS selectors or media queries for variants:

```paperclip
// Media query
public trigger mobile {
    "@media screen and (max-width: 400px)"
}

// Pseudo-class
public trigger hover {
    ":hover"
    ".hover"
}

// Prefers-color-scheme
public trigger darkMode {
    ".dark"
    "@media (prefers-color-scheme: dark)"
}
```

Multiple selectors/media queries can be included in one trigger.

## Variants

Define component states that activate based on triggers:

```paperclip
public component Button {
    variant hover trigger {
        ":hover"
        ".hover"
    }

    variant danger trigger {
        ".danger"
    }

    render button {
        style {
            background: blue;
            padding: 12px;
        }

        style variant hover {
            background: darkblue;
        }

        style variant danger {
            background: red;
        }

        slot children {
            text "Click me"
        }
    }
}
```

### Combination Variants (AND logic)

Multiple variants must all be active:

```paperclip
component Card {
    variant mobile trigger mobile
    variant dark trigger darkMode
    variant hover trigger { ":hover" }

    render div {
        style {
            background: white;
        }

        // Applies only when mobile AND dark AND hover
        style variant mobile + dark + hover {
            background: #000;
        }
    }
}
```

## Element Styles

Styles on elements compile to CSS with generated class names:

```paperclip
div myLabel {
    style {
        color: red;
        padding: 10px;
    }

    span nested {
        style {
            font-size: 14px;
        }
        text "Hello"
    }
}
```

This compiles to CSS like:

```css
.generated-class-123 {
    color: red;
    padding: 10px;
}

.generated-class-124 {
    font-size: 14px;
}
```

## Text Node Styles

Text can have styles (renders as `<span>` when styled):

```paperclip
text myLabel "Hello world" {
    style {
        color: red;
        font-weight: bold;
    }
}
```

## Complete Example

```paperclip
import "./theme.pc" as theme

public trigger mobile {
    "@media screen and (max-width: 768px)"
}

public style card {
    background: white;
    border: 1px solid var(theme.borderColor);
    border-radius: 8px;
    padding: 24px;
}

public component Card {
    variant mobile trigger mobile
    variant hover trigger { ":hover" }

    render div root {
        style extends card {
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        style variant hover {
            box-shadow: 0 4px 8px rgba(0,0,0,0.15);
        }

        style variant mobile {
            padding: 16px;
        }

        div header {
            style {
                margin-bottom: 16px;
                font-weight: 600;
            }
            slot headerContent {
                text "Default Header"
            }
        }

        div body {
            style {
                color: var(theme.textColor);
            }
            slot children
        }
    }
}
```

## Compilation

Paperclip compiles to:

1. **CSS** - All `style { }` blocks become CSS rules with generated class names
2. **HTML** - Element structure with generated class attributes
3. **Framework code** - React/Vue/etc. components that apply the classes

Example compilation:

**Input:**
```paperclip
div {
    style {
        color: red;
    }
    text "Hello"
}
```

**Output CSS:**
```css
.pc-123abc {
    color: red;
}
```

**Output HTML:**
```html
<div class="pc-123abc">Hello</div>
```

## Key Points

1. ✅ CSS properties work exactly the same as regular CSS
2. ✅ All standard CSS functions work (`calc()`, `var()`, `url()`, `rgba()`, etc.)
3. ✅ Compiles to real CSS with generated selectors
4. ✅ Supports all CSS pseudo-classes via triggers (`:hover`, `:focus`, `:active`, etc.)
5. ✅ Supports media queries via triggers
6. ✅ Style mixins become reusable CSS classes
7. ✅ Standard CSS syntax: colons and semicolons required (simplified syntax planned for future)

## See Also

- [Original Paperclip Repository](https://github.com/crcn/paperclip)
- Example files: `~/Developer/crcn/paperclip/libs/designer/src/ui/`
