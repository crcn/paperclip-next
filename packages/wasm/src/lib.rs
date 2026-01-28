use wasm_bindgen::prelude::*;
use paperclip_parser::{parse_with_path, get_document_id};
use paperclip_compiler_react::{compile_to_react, compile_definitions, CompileOptions};
use paperclip_compiler_css::compile_to_css;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct CompileResult {
    code: String,
    types: Option<String>,
}

#[wasm_bindgen]
impl CompileResult {
    #[wasm_bindgen(getter)]
    pub fn code(&self) -> String {
        self.code.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn types(&self) -> Option<String> {
        self.types.clone()
    }
}

/// Compile a .pc file to React/JSX
#[wasm_bindgen(js_name = compileToReact)]
pub fn compile_to_react_js(source: &str, file_path: &str, generate_types: bool) -> Result<CompileResult, JsValue> {
    // Parse
    let doc = parse_with_path(source, file_path)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {:?}", e)))?;

    // Compile to React
    let options = CompileOptions {
        use_typescript: generate_types,
        include_css_imports: true,
    };

    let code = compile_to_react(&doc, options.clone())
        .map_err(|e| JsValue::from_str(&format!("Compile error: {:?}", e)))?;

    // Generate TypeScript definitions if requested
    let types = if generate_types {
        match compile_definitions(&doc, options) {
            Ok(defs) => Some(defs),
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(CompileResult { code, types })
}

/// Compile a .pc file to CSS
#[wasm_bindgen(js_name = compileToCss)]
pub fn compile_to_css_js(source: &str, file_path: &str) -> Result<String, JsValue> {
    // Parse
    let doc = parse_with_path(source, file_path)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {:?}", e)))?;

    // Compile to CSS
    compile_to_css(&doc)
        .map_err(|e| JsValue::from_str(&format!("CSS compile error: {:?}", e)))
}

/// Parse a .pc file and return the AST as JSON
#[wasm_bindgen(js_name = parse)]
pub fn parse_js(source: &str, file_path: &str) -> Result<String, JsValue> {
    let doc = parse_with_path(source, file_path)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {:?}", e)))?;

    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get document ID for a file path
#[wasm_bindgen(js_name = getDocumentId)]
pub fn get_document_id_js(file_path: &str) -> String {
    get_document_id(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_component() {
        let source = r#"
            public component Button {
                render button {
                    text "Click me"
                }
            }
        "#;

        let result = compile_to_react_js(source, "/test.pc", false);
        assert!(result.is_ok());

        let compiled = result.unwrap();
        assert!(compiled.code().contains("Button"));
        assert!(compiled.code().contains("button"));
    }

    #[test]
    fn test_compile_with_types() {
        let source = r#"
            public component Card {
                render div {
                    text "Card"
                }
            }
        "#;

        let result = compile_to_react_js(source, "/test.pc", true);
        assert!(result.is_ok());

        let compiled = result.unwrap();
        assert!(compiled.code().contains("Card"));
        assert!(compiled.types().is_some());
    }
}
