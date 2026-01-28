use paperclip_compiler_react::{compile_to_react, CompileOptions};
use paperclip_parser::parse;

fn main() {
    let source = r#"
public token primaryColor #3366FF
public token spacing 16px

public component Button {
    render button(type="button") {
        style {
            padding: 8px 16px
            background: #3366FF
            color: white
            border: none
            border-radius: 4px
        }
        text "Click me"
    }
}

public component Card {
    render div {
        style {
            border: 1px solid #ddd
            border-radius: 8px
            padding: 16px
        }
        div {
            text "Card Title"
        }
        div {
            text "Card content goes here"
        }
    }
}
"#;

    println!("Compiling Paperclip to React...\n");

    match parse(source) {
        Ok(document) => {
            let options = CompileOptions::default();
            match compile_to_react(&document, options) {
                Ok(react_code) => {
                    println!("✅ Successfully compiled!\n");
                    println!("Generated React code:");
                    println!("{}", "=".repeat(80));
                    println!("{}", react_code);
                    println!("{}", "=".repeat(80));
                }
                Err(e) => {
                    eprintln!("❌ Compilation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Parse error: {:?}", e);
            std::process::exit(1);
        }
    }
}
