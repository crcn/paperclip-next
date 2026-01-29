//! # Paperclip Inference Engine
//!
//! A standalone type inference engine for Paperclip components that enables
//! sophisticated type analysis for multiple compilation targets.
//!
//! ## Features
//!
//! - **Multi-pass inference**: Signature collection → body analysis → prop extraction
//! - **Binary operation constraints**: Infers `age: number` from `{age + 1}`
//! - **Member access tracking**: Infers object shapes from `{user.name}`
//! - **Nested member access**: Supports `{user.address.city}`
//! - **Plugin-based code generation**: TypeScript, Rust, and extensible to other targets
//! - **Lexical scoping**: Proper handling of control flow and nested scopes
//!
//! ## Example
//!
//! ```rust
//! use paperclip_inference::{InferenceEngine, InferenceOptions, CodeGenerator};
//! use paperclip_inference::codegen::typescript::TypeScriptGenerator;
//! use paperclip_parser::parse;
//!
//! let source = r#"
//! public component Counter {
//!     variant primary
//!     slot icon
//!     render div {
//!         text {count}
//!         text {user.name}
//!     }
//! }
//! "#;
//!
//! let doc = parse(source).unwrap();
//! let engine = InferenceEngine::new(InferenceOptions::default());
//! let props = engine.infer_component_props(&doc.components[0]).unwrap();
//!
//! // Generate TypeScript definitions
//! let ts_gen = TypeScriptGenerator::new();
//! for (name, prop) in &props {
//!     println!("{}: {}", name, ts_gen.generate_type(&prop.type_));
//! }
//! ```

pub mod codegen;
pub mod error;
pub mod inference;
pub mod options;
pub mod scope;
pub mod types;

// Re-export main types for convenience
pub use codegen::{rust::RustGenerator, typescript::TypeScriptGenerator, CodeGenerator};
pub use error::{InferenceError, InferenceResult};
pub use inference::InferenceEngine;
pub use options::InferenceOptions;
pub use scope::Scope;
pub use types::{ElementType, FunctionType, LiteralType, ObjectType, PropertyType, Type};
