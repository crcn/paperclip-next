use crate::{compile_to_html, CompileOptions};
use paperclip_parser::parse;

#[test]
fn test_compile_simple_component() {
    let source = r#"
public component Button {
    render button {
        text "Click me"
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let html = compile_to_html(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated HTML:\n{}", html);

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<button>"));
    assert!(html.contains("Click me"));
    assert!(html.contains("</button>"));
}

#[test]
fn test_compile_with_attributes() {
    let source = r#"
public component Button {
    render button(type="button", class="btn") {
        text "Submit"
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let html = compile_to_html(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated HTML:\n{}", html);

    assert!(html.contains("type=\"button\""));
    assert!(html.contains("class=\"btn\""));
    assert!(html.contains("Submit"));
}

#[test]
fn test_compile_with_inline_styles() {
    let source = r#"
public component Button {
    render button {
        style {
            padding: 8px 16px
            background: #3366FF
            color: white
        }
        text "Click"
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let options = CompileOptions {
        inline_styles: true,
        ..Default::default()
    };
    let html = compile_to_html(&document, options).expect("Failed to compile");

    println!("Generated HTML:\n{}", html);

    assert!(html.contains("style=\""));
    assert!(html.contains("padding: 8px 16px"));
    assert!(html.contains("background: #3366FF"));
    assert!(html.contains("color: white"));
}

#[test]
fn test_compile_nested_elements() {
    let source = r#"
public component Card {
    render div {
        div {
            text "Title"
        }
        div {
            text "Body"
        }
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let html = compile_to_html(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated HTML:\n{}", html);

    assert!(html.contains("<div>"));
    assert!(html.contains("Title"));
    assert!(html.contains("Body"));
    assert!(html.contains("</div>"));
}

#[test]
fn test_compile_self_closing_tags() {
    let source = r#"
public component Image {
    render img(src="photo.jpg", alt="Photo")
}
"#;

    let document = parse(source).expect("Failed to parse");
    let html = compile_to_html(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated HTML:\n{}", html);

    assert!(html.contains("<img"));
    assert!(html.contains("src=\"photo.jpg\""));
    assert!(html.contains("alt=\"Photo\""));
    assert!(html.contains("/>"));
}

#[test]
fn test_compile_multiple_components() {
    let source = r#"
public component Header {
    render div {
        text "Header"
    }
}

public component Footer {
    render div {
        text "Footer"
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let html = compile_to_html(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated HTML:\n{}", html);

    assert!(html.contains("Header"));
    assert!(html.contains("Footer"));
    assert!(html.contains("id=\"header\""));
    assert!(html.contains("id=\"footer\""));
}

#[test]
fn test_compile_without_pretty_print() {
    let source = r#"
public component Button {
    render button {
        text "Click"
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let options = CompileOptions {
        pretty: false,
        ..Default::default()
    };
    let html = compile_to_html(&document, options).expect("Failed to compile");

    println!("Generated HTML:\n{}", html);

    // Should be compact, no extra newlines
    assert!(!html.contains("\n  "));
}

#[test]
fn test_escape_html_entities() {
    let source = r#"
public component Message {
    render div {
        text "Hello <world> & friends"
    }
}
"#;

    let document = parse(source).expect("Failed to parse");
    let html = compile_to_html(&document, CompileOptions::default()).expect("Failed to compile");

    println!("Generated HTML:\n{}", html);

    assert!(html.contains("&lt;world&gt;"));
    assert!(html.contains("&amp;"));
    assert!(html.contains("friends"));
}
