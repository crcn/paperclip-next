---
title: "feat: Universal Frontend Builder - Connect Paperclip to Any Backend"
type: feat
date: 2026-02-03
---

# Universal Frontend Builder

## Overview

Transform Paperclip into a **universal frontend builder** that compiles production-quality UIs connected to any backend—WordPress, GraphQL APIs, custom REST services, or any data source. The system works equally well for human designers using the visual canvas AND AI agents building programmatically.

This positions Paperclip as the "front-end builder for any web app"—a layer that sits between your data (wherever it lives) and your production frontend code.

## Problem Statement / Motivation

### The Current Landscape

1. **WordPress customization is painful**: Themes are tightly coupled to PHP, making visual customization require developer intervention
2. **Headless CMS loses preview**: Going headless (WordPress REST/GraphQL) means losing WYSIWYG editing
3. **AI can't build production frontends**: Tools like v0 generate one-off components, but can't maintain a design system or connect to live data
4. **Visual builders don't compile**: Webflow/Framer output is runtime-dependent, not production code you own

### The Opportunity

Paperclip already has:
- ✅ Visual component authoring → Production React code
- ✅ Deterministic evaluation (critical for AI reproducibility)
- ✅ Expression system with variables, member access, operators
- ✅ Semantic identity (survives refactoring)
- ✅ Live preview streaming architecture

**What's missing**: A way to bind components to external data sources and a pattern for AI agents to leverage this.

## Proposed Solution

Add **data bindings** to the `.pc` language that:
1. Declare data sources inline in components
2. Bind element properties to data paths
3. Compile to production code with proper data fetching (React Query, SWR, etc.)
4. Work in the designer with live preview using real or mocked data

### Design Principles

1. **Data bindings in .pc files** (not separate config)
2. **Compiles to production code** (not a runtime)
3. **Works for humans AND AI agents equally**
4. **Schema-aware** (leverage GraphQL SDL / OpenAPI for type safety)
5. **Backend-agnostic** (REST, GraphQL, any fetch-able data)

## Technical Approach

### New Language Constructs

#### 1. Data Source Declaration

```paperclip
/**
 * @datasource posts
 * @endpoint https://myblog.com/wp-json/wp/v2/posts
 * @type rest
 */

/**
 * @datasource products
 * @endpoint https://api.mystore.com/graphql
 * @type graphql
 * @query {
 *   products(first: 10) {
 *     id
 *     title
 *     price
 *     image { url }
 *   }
 * }
 */
```

#### 2. Data Binding Syntax

Extend the existing expression system to support data source references:

```paperclip
component PostCard {
  // Bind to data source
  render div {
    text { data.posts[0].title }
    img(src={data.posts[0].featured_image})

    // Repeat over data
    repeat post in data.posts {
      div {
        h2 { text { post.title } }
        p { text { post.excerpt } }
      }
    }
  }
}
```

#### 3. Loading/Error States

```paperclip
component PostList {
  render div {
    // Conditional rendering based on data state
    switch data.posts.state {
      loading => div { text "Loading..." }
      error => div { text { data.posts.error } }
      ready => repeat post in data.posts.data {
        PostCard(post={post})
      }
    }
  }
}
```

### Compilation Targets

#### React Output

```typescript
// Generated from PostCard.pc
import { useQuery } from '@tanstack/react-query';

interface Post {
  id: number;
  title: string;
  excerpt: string;
  featured_image: string;
}

export const PostCard = React.memo(React.forwardRef<HTMLDivElement, PostCardProps>(
  function PostCard(props, ref) {
    const { data: posts, isLoading, error } = useQuery<Post[]>({
      queryKey: ['posts'],
      queryFn: () => fetch('https://myblog.com/wp-json/wp/v2/posts').then(r => r.json())
    });

    if (isLoading) return <div>Loading...</div>;
    if (error) return <div>{error.message}</div>;

    return (
      <div ref={ref}>
        {posts?.map(post => (
          <div key={post.id}>
            <h2>{post.title}</h2>
            <p>{post.excerpt}</p>
          </div>
        ))}
      </div>
    );
  }
));
```

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     .pc Source File                          │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ @datasource posts                                        ││
│  │ @endpoint https://myblog.com/wp-json/wp/v2/posts        ││
│  │                                                          ││
│  │ component PostCard {                                     ││
│  │   render div { text { data.posts[0].title } }           ││
│  │ }                                                        ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        Parser                                │
│  - Parse @datasource annotations                            │
│  - Parse data.* expressions                                 │
│  - Extract endpoint, type, query info                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Evaluator                               │
│  - Resolve data bindings against mock/live data             │
│  - Track data dependencies per component                    │
│  - Generate VDOM with data placeholders                     │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│  Compiler: React │ │  Compiler: Vue   │ │  Designer Preview│
│                  │ │                  │ │                  │
│  - Generate      │ │  - Generate      │ │  - Fetch live    │
│    useQuery()    │ │    composables   │ │    data          │
│  - Type infer    │ │  - Type infer    │ │  - Show loading  │
│    from schema   │ │    from schema   │ │    states        │
└──────────────────┘ └──────────────────┘ └──────────────────┘
```

### Schema Inference

For type-safe code generation, Paperclip can introspect schemas:

```paperclip
/**
 * @datasource products
 * @endpoint https://api.example.com/graphql
 * @type graphql
 * @schema https://api.example.com/graphql  // Introspect schema
 */
```

This enables:
1. **Auto-complete in editor**: Show available fields from schema
2. **Type-safe compiled output**: Generate TypeScript interfaces from schema
3. **Validation**: Warn if binding references non-existent fields

### AI Agent Integration

The key insight: Paperclip's deterministic evaluation and semantic IDs make it ideal for AI manipulation.

#### Agent Workflow

```
1. Agent receives task: "Create a product listing page for my Shopify store"

2. Agent introspects data source:
   - Fetch GraphQL schema from Shopify API
   - Understand available types: Product, Collection, Image, etc.

3. Agent generates .pc file:
   - Declare datasource with Shopify endpoint
   - Create components with data bindings
   - Use design tokens for styling

4. Agent compiles to preview:
   - Workspace server evaluates with live data
   - Agent "sees" the result via screenshot

5. Agent iterates:
   - Modify .pc file based on visual feedback
   - Semantic IDs ensure stable patches
   - Determinism ensures reproducible results
```

#### Why This Works for AI

1. **Declarative format**: .pc files are structured, predictable
2. **Deterministic**: Same input → same output, every time
3. **Semantic IDs**: Changes are stable, not position-based
4. **Live feedback loop**: Agent can "see" results via preview streaming
5. **Schema-aware**: Types provide guardrails for generation

## Implementation Phases

### Phase 1: Parser Extensions

**New AST nodes:**

```rust
// ast.rs additions

/// Data source declaration (from doc comment annotations)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataSourceDecl {
    pub name: String,
    pub endpoint: String,
    pub source_type: DataSourceType,
    pub query: Option<String>,        // For GraphQL
    pub schema_url: Option<String>,   // For introspection
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataSourceType {
    Rest,
    GraphQL,
}

// Extend Expression enum
pub enum Expression {
    // ... existing variants ...

    /// Data reference (data.sourceName.path)
    DataRef {
        source: String,
        path: Vec<String>,
        span: Span,
    },
}
```

**Files to modify:**
- `packages/parser/src/ast.rs` - Add DataSourceDecl, extend Expression
- `packages/parser/src/parser.rs` - Parse @datasource annotations
- `packages/parser/src/tokenizer.rs` - Tokenize data.* expressions

### Phase 2: Evaluator Support

**Handle data bindings during evaluation:**

```rust
// evaluator.rs additions

pub struct EvalContext {
    // ... existing fields ...

    /// Data sources and their current values (for preview)
    pub data_sources: HashMap<String, DataSourceValue>,
}

pub enum DataSourceValue {
    Loading,
    Ready(serde_json::Value),
    Error(String),
}
```

**For preview mode:**
- Fetch data from endpoints
- Cache responses
- Populate `data.*` expressions

**For compilation mode:**
- Track which components use which data sources
- Generate dependency metadata

**Files to modify:**
- `packages/evaluator/src/evaluator.rs` - Handle DataRef expressions
- `packages/evaluator/src/context.rs` - Add data source state

### Phase 3: Compiler Extensions

**React compiler generates data fetching code:**

```rust
// compiler-react additions

fn generate_data_hook(&self, source: &DataSourceDecl) -> String {
    match source.source_type {
        DataSourceType::Rest => {
            format!(
                r#"const {{ data: {name}, isLoading: {name}Loading, error: {name}Error }} = useQuery({{
                    queryKey: ['{name}'],
                    queryFn: () => fetch('{endpoint}').then(r => r.json())
                }});"#,
                name = source.name,
                endpoint = source.endpoint
            )
        }
        DataSourceType::GraphQL => {
            // Generate with graphql-request or urql
        }
    }
}
```

**Files to modify:**
- `packages/compiler-react/src/lib.rs` - Generate data fetching hooks
- New: `packages/compiler-react/src/data_hooks.rs` - Data hook generation

### Phase 4: Designer Integration

**Live data preview in the visual editor:**

1. Workspace server fetches data on file change
2. Evaluator populates data bindings
3. Preview shows actual data (or loading states)
4. Side panel shows data source configuration

**Files to modify:**
- `packages/workspace/src/lib.rs` - Data fetching service
- `packages/designer/src/components/` - Data source panel

### Phase 5: Schema Introspection

**Auto-complete and type generation:**

1. Fetch GraphQL schema via introspection
2. Parse OpenAPI specs
3. Generate TypeScript interfaces
4. Provide auto-complete in editor

**New package:**
- `packages/schema-introspection/` - Schema fetching and parsing

## Use Case Examples

### WordPress Blog

```paperclip
/**
 * @datasource posts
 * @endpoint https://myblog.com/wp-json/wp/v2/posts
 * @type rest
 */

public component BlogIndex {
  render main {
    h1 { text "My Blog" }
    repeat post in data.posts {
      article {
        h2 { text { post.title.rendered } }
        div(dangerouslySetInnerHTML={post.content.rendered})
      }
    }
  }
}
```

### Shopify Storefront

```paperclip
/**
 * @datasource products
 * @endpoint https://mystore.myshopify.com/api/2024-01/graphql.json
 * @type graphql
 * @query {
 *   products(first: 12) {
 *     edges {
 *       node {
 *         id
 *         title
 *         handle
 *         priceRange { minVariantPrice { amount currencyCode } }
 *         images(first: 1) { edges { node { url } } }
 *       }
 *     }
 *   }
 * }
 */

public component ProductGrid {
  render div {
    style { display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px; }

    repeat product in data.products.edges {
      ProductCard(product={product.node})
    }
  }
}

component ProductCard {
  variant product

  render article {
    img(src={product.images.edges[0].node.url} alt={product.title})
    h3 { text { product.title } }
    span { text { "$" + product.priceRange.minVariantPrice.amount } }
  }
}
```

### Custom GraphQL API

```paperclip
/**
 * @datasource users
 * @endpoint https://api.myapp.com/graphql
 * @type graphql
 * @schema https://api.myapp.com/graphql
 * @query {
 *   users(where: { role: ADMIN }) {
 *     id
 *     name
 *     email
 *     avatar
 *   }
 * }
 */

public component AdminDashboard {
  render div {
    h1 { text "Admin Users" }

    switch data.users.state {
      loading => div { text "Loading users..." }
      error => div {
        style { color: red; }
        text { data.users.error }
      }
      ready => ul {
        repeat user in data.users.data {
          li {
            img(src={user.avatar} alt={user.name})
            span { text { user.name } }
            span { text { user.email } }
          }
        }
      }
    }
  }
}
```

## Alternative Approaches Considered

### 1. Separate Config Files

Instead of `@datasource` in .pc files, use separate `datasources.json`:

```json
{
  "posts": {
    "endpoint": "https://myblog.com/wp-json/wp/v2/posts",
    "type": "rest"
  }
}
```

**Rejected because:**
- Breaks co-location (data definition far from usage)
- Harder for AI to generate (must coordinate two files)
- Extra indirection for no benefit

### 2. Runtime Data Layer

Instead of compiling to `useQuery()`, include a Paperclip runtime that handles fetching.

**Rejected because:**
- Adds runtime dependency
- Defeats Paperclip's "compiles to production code" philosophy
- Less flexible (can't customize fetching strategy)

### 3. Props-Only Model

Only support data passed via props, no built-in fetching:

```paperclip
component PostList {
  variant posts: Post[]
  render div { repeat post in posts { ... } }
}
```

**Rejected because:**
- Requires external orchestration
- Can't preview with live data in designer
- Misses the opportunity for schema integration

## Success Metrics

1. **Adoption signal**: Users successfully connect to WordPress/Shopify/custom APIs
2. **AI effectiveness**: AI agents can build complete data-bound UIs without human intervention
3. **Preview quality**: Designer shows live data with <500ms refresh on edits
4. **Type safety**: 100% of data bindings validated against schema when available

## Dependencies & Prerequisites

- ✅ Parser expression system (exists)
- ✅ Evaluator context (exists)
- ✅ React compiler (exists)
- ⏳ Workspace streaming (exists, needs data fetching)
- ❌ Schema introspection (new)

## Risk Analysis & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Complex expression edge cases | Medium | Medium | Comprehensive test suite, error recovery |
| CORS issues in preview | High | Low | Proxy through workspace server |
| Schema drift | Medium | Medium | Cache schemas, version check |
| Performance with large datasets | Low | Medium | Pagination support, virtual scrolling |

## References & Research

### Internal References

- Expression system: `packages/parser/src/ast.rs:229-296`
- Evaluator context: `packages/evaluator/src/evaluator.rs:92-105`
- React compiler: `packages/compiler-react/src/lib.rs`

### External References

- [Builder.io Data Binding](https://www.builder.io/c/docs/data-binding) - State + interpolation pattern
- [Retool Queries](https://docs.retool.com/queries/quickstart) - Resource abstraction
- [AI SDK RSC](https://ai-sdk.dev/docs/ai-sdk-rsc/streaming-react-components) - Streaming components
- [GraphQL Codegen](https://the-guild.dev/graphql/codegen) - Schema → TypeScript
- [WordPress REST API](https://developer.wordpress.org/rest-api/) - Headless WordPress
- [Shopify Storefront API](https://shopify.dev/docs/api/storefront) - GraphQL commerce

### Similar Implementations

- [TeleportHQ Headless CMS](https://teleporthq.io/headless-cms-integration) - Visual builder + data
- [Storyblok Visual Editor](https://www.storyblok.com/docs/guide/essentials/visual-editor) - Live editing with data
- [v0 by Vercel](https://v0.dev) - AI component generation (no data binding)
