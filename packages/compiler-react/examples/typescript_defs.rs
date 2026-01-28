use paperclip_compiler_react::{compile_definitions, compile_to_react, CompileOptions};
use paperclip_parser::parse;

fn main() {
    let source = r#"
public token primaryColor #3366FF
public token spacing 16px

public component Button {
    variant disabled
    variant loading

    render button(type="button") {
        style {
            padding: 8px 16px
            background: #3366FF
            color: white
        }
        text {label}
    }
}

public component Card {
    slot header
    slot footer {
        div {
            text "Default footer"
        }
    }

    render div {
        style {
            border: 1px solid #ddd
            border-radius: 8px
            padding: 16px
        }
        div {
            text {title}
        }
        div {
            text {description}
        }
        header
        footer
    }
}
"#;

    println!("Compiling Paperclip to React + TypeScript...\n");

    match parse(source) {
        Ok(document) => {
            let options = CompileOptions {
                use_typescript: true,
                include_css_imports: true,
            };

            // Generate React code
            match compile_to_react(&document, options.clone()) {
                Ok(react_code) => {
                    println!("✅ React Code Generated!");
                    println!("{}", "=".repeat(80));
                    println!("{}", react_code);
                    println!("{}", "=".repeat(80));
                    println!();
                }
                Err(e) => {
                    eprintln!("❌ React compilation error: {}", e);
                    std::process::exit(1);
                }
            }

            // Generate TypeScript definitions
            match compile_definitions(&document, options) {
                Ok(defs_code) => {
                    println!("✅ TypeScript Definitions Generated!");
                    println!("{}", "=".repeat(80));
                    println!("{}", defs_code);
                    println!("{}", "=".repeat(80));
                    println!();

                    println!("📝 Usage Example:");
                    println!("{}", "-".repeat(80));
                    println!(
                        r#"
import {{ Button, Card, primaryColor, spacing }} from "./components";

export function App() {{
  return (
    <div>
      <Button
        label="Click me"
        disabled={{false}}
        loading={{true}}
      />

      <Card
        title="Welcome"
        description="This is a card component"
        header={{<h1>Custom Header</h1>}}
      />
    </div>
  );
}}
"#
                    );
                    println!("{}", "-".repeat(80));
                }
                Err(e) => {
                    eprintln!("❌ Definition compilation error: {}", e);
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
