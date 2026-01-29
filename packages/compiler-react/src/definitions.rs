use crate::compiler::CompileError;
use crate::context::{CompileOptions, CompilerContext};
use paperclip_inference::codegen::typescript::TypeScriptGenerator;
use paperclip_inference::{CodeGenerator, InferenceEngine, InferenceOptions};
use paperclip_parser::ast::*;

/// Compile TypeScript definition file (.d.ts) for a Paperclip document
pub fn compile_definitions(
    document: &Document,
    _options: CompileOptions,
) -> Result<String, CompileError> {
    let ctx = CompilerContext::new(_options);

    // Add imports
    ctx.add_line("import React from \"react\";");
    ctx.add("\n");

    // Generate type definitions for public components
    for component in &document.components {
        if component.public {
            compile_component_definition(component, &ctx);
        }
    }

    // Export public tokens
    for token in &document.tokens {
        if token.public {
            ctx.add_line(&format!("export const {}: string;", token.name));
        }
    }

    // Export public styles
    for style in &document.styles {
        if style.public {
            ctx.add_line(&format!("export const {}: string;", style.name));
        }
    }

    Ok(ctx.get_output())
}

fn compile_component_definition(component: &Component, ctx: &CompilerContext) {
    let component_name = &component.name;

    // Create inference engine
    let engine = InferenceEngine::new(InferenceOptions::default());
    let inferred_props = match engine.infer_component_props(component) {
        Ok(props) => props,
        Err(e) => {
            eprintln!(
                "Warning: Failed to infer props for {}: {}",
                component_name, e
            );
            std::collections::BTreeMap::new()
        }
    };

    // Create TypeScript generator
    let ts_gen = TypeScriptGenerator::new();

    // Generate props interface
    ctx.add_line(&format!("export interface {}Props {{", component_name));
    ctx.indent();

    // Always include ref
    ctx.add_line("ref?: React.Ref<any>;");

    // Add inferred props
    for (prop_name, prop_type) in inferred_props {
        let line = ts_gen.generate_property(&prop_name, &prop_type);
        ctx.add_line(&format!("{};", line));
    }

    ctx.dedent();
    ctx.add_line("}");
    ctx.add("\n");

    // Export component type
    ctx.add_line(&format!(
        "export const {}: React.FC<{}Props>;",
        component_name, component_name
    ));
    ctx.add("\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse;

    #[test]
    fn test_compile_simple_definition() {
        let source = r#"
public component Button {
    render button {
        text {label}
    }
}
"#;

        let document = parse(source).expect("Failed to parse");
        let result = compile_definitions(&document, CompileOptions::default())
            .expect("Failed to compile definitions");

        println!("Generated definitions:\n{}", result);

        assert!(result.contains("import React from \"react\""));
        assert!(result.contains("export interface ButtonProps {"));
        assert!(result.contains("label: any;"));
        assert!(result.contains("export const Button: React.FC<ButtonProps>;"));
    }

    #[test]
    fn test_compile_with_variant() {
        let source = r#"
public component Card {
    variant active

    render div {
        text "Card"
    }
}
"#;

        let document = parse(source).expect("Failed to parse");
        let result = compile_definitions(&document, CompileOptions::default())
            .expect("Failed to compile definitions");

        println!("Generated definitions:\n{}", result);

        assert!(result.contains("export interface CardProps {"));
        assert!(result.contains("active?: boolean;"));
        assert!(result.contains("export const Card: React.FC<CardProps>;"));
    }

    #[test]
    fn test_compile_with_slot() {
        let source = r#"
public component Layout {
    slot header
    slot footer {
        div {
            text "Default footer"
        }
    }

    render div {
        header
        footer
    }
}
"#;

        let document = parse(source).expect("Failed to parse");
        let result = compile_definitions(&document, CompileOptions::default())
            .expect("Failed to compile definitions");

        println!("Generated definitions:\n{}", result);

        assert!(result.contains("export interface LayoutProps {"));
        assert!(result.contains("header: React.ReactNode;"));
        assert!(result.contains("footer?: React.ReactNode;")); // Optional because has default
    }

    #[test]
    fn test_compile_with_tokens() {
        let source = r#"
public token primaryColor #3366FF
token internalSpacing 16px
public token fontSize 14px

public component Button {
    render button {
        text "Click"
    }
}
"#;

        let document = parse(source).expect("Failed to parse");
        let result = compile_definitions(&document, CompileOptions::default())
            .expect("Failed to compile definitions");

        println!("Generated definitions:\n{}", result);

        assert!(result.contains("export const primaryColor: string;"));
        assert!(result.contains("export const fontSize: string;"));
        assert!(!result.contains("internalSpacing")); // Not public
    }

    #[test]
    fn test_compile_multiple_props() {
        let source = r#"
public component UserCard {
    render div {
        div {
            text {name}
        }
        div {
            text {email}
        }
        div {
            text {age}
        }
    }
}
"#;

        let document = parse(source).expect("Failed to parse");
        let result = compile_definitions(&document, CompileOptions::default())
            .expect("Failed to compile definitions");

        println!("Generated definitions:\n{}", result);

        assert!(result.contains("export interface UserCardProps {"));
        assert!(result.contains("name: any;"));
        assert!(result.contains("email: any;"));
        assert!(result.contains("age: any;"));
    }
}
