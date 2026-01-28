mod compiler;
mod context;
mod definitions;

pub use compiler::compile_to_react;
pub use context::{CompileOptions, CompilerContext};
pub use definitions::compile_definitions;

// Re-export from inference crate for convenience
pub use paperclip_inference::{
    CodeGenerator, InferenceEngine, InferenceOptions, PropertyType, RustGenerator, Type,
    TypeScriptGenerator,
};

#[cfg(test)]
mod tests;
