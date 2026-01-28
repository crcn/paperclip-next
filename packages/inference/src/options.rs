/// Configuration options for type inference
#[derive(Debug, Clone)]
pub struct InferenceOptions {
    /// Use strict type checking (fail on Unknown types in final output)
    /// When true, any Unknown types remaining after inference will be treated as errors
    pub strict: bool,

    /// Infer object property types from member access
    /// When true, expressions like `user.name` will infer `user: { name: any }`
    pub infer_object_properties: bool,

    /// Infer function signatures from call expressions
    /// Not yet fully implemented
    pub infer_functions: bool,

    /// Support nested member access (e.g., user.address.city)
    /// When true, will build nested object structures
    pub nested_member_access: bool,
}

impl Default for InferenceOptions {
    fn default() -> Self {
        Self {
            strict: false,
            infer_object_properties: true,
            infer_functions: false,
            nested_member_access: true,
        }
    }
}

impl InferenceOptions {
    /// Create a new options instance with strict mode enabled
    pub fn strict() -> Self {
        Self {
            strict: true,
            ..Default::default()
        }
    }

    /// Create a new options instance with all features enabled
    pub fn full() -> Self {
        Self {
            strict: false,
            infer_object_properties: true,
            infer_functions: true,
            nested_member_access: true,
        }
    }

    /// Create a new options instance with minimal inference
    pub fn minimal() -> Self {
        Self {
            strict: false,
            infer_object_properties: false,
            infer_functions: false,
            nested_member_access: false,
        }
    }
}
