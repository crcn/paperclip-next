use crate::{compile_to_react, CompileOptions};
use paperclip_parser::parse;

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn test_simple_component() {
    let source = r#"
public component Button {
    render button {
        text "Click me"
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let result = compile_to_react(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated code:\n{}", result);

    assert!(result.contains("import React from \"react\""));
    assert!(result.contains("const _Button = (props, ref) => {"));
    assert!(result.contains("const Button = React.memo(React.forwardRef(_Button))"));
    assert!(result.contains("export { Button }"));
    assert!(result.contains("<button"));
    assert!(result.contains("Click me"));
}

#[test]
fn test_component_with_props() {
    let source = r#"
public component Card {
    render div {
        text {title}
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let result = compile_to_react(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated code:\n{}", result);

    assert!(result.contains("const _Card = (props, ref) => {"));
    assert!(result.contains("props.title"));
}

#[test]
fn test_component_with_attributes() {
    let source = r#"
public component Button {
    render button(class="btn", type="button") {
        text "Submit"
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let result = compile_to_react(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated code:\n{}", result);

    // className should be used instead of class
    assert!(result.contains("className=\"btn\""));
    assert!(result.contains("type=\"button\""));
}

#[test]
fn test_nested_elements() {
    let source = r#"
public component Card {
    render div {
        div {
            text "Header"
        }
        div {
            text "Body"
        }
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let result = compile_to_react(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated code:\n{}", result);

    assert!(result.contains("<div"));
    assert!(result.contains("Header"));
    assert!(result.contains("Body"));
}

#[test]
fn test_component_instance() {
    let source = r#"
component Inner {
    render span {
        text "Inner"
    }
}

public component Outer {
    render div {
        Inner()
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let result = compile_to_react(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated code:\n{}", result);

    assert!(result.contains("const _Inner"));
    assert!(result.contains("const _Outer"));
    assert!(result.contains("<Inner"));
}

#[test]
fn test_public_token() {
    let source = r#"
public token primaryColor #3366FF

public component Button {
    render button {
        text "Click"
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let result = compile_to_react(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated code:\n{}", result);

    assert!(result.contains("export const primaryColor = \"#3366FF\""));
}

#[test]
fn test_conditional_rendering() {
    let source = r#"
public component Greeting {
    render div {
        if {isLoggedIn} {
            text "Welcome back!"
        }
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let result = compile_to_react(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated code:\n{}", result);

    assert!(result.contains("props.isLoggedIn"));
    assert!(result.contains("Welcome back!"));
}

#[test]
fn test_repeat_element() {
    let source = r#"
public component List {
    render div {
        repeat item in {items} {
            div {
                text {item}
            }
        }
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let result = compile_to_react(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated code:\n{}", result);

    assert!(result.contains("props.items"));
    assert!(result.contains(".map"));
    assert!(result.contains("item"));
}
