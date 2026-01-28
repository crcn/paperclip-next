use paperclip_evaluator::{CssEvaluator, CssResult};
use paperclip_parser::ast::Document;

/// Compile a Paperclip document to CSS
pub fn compile_to_css(document: &Document) -> CssResult<String> {
    let mut evaluator = CssEvaluator::new();
    let css_doc = evaluator.evaluate(document)?;
    Ok(css_doc.to_css())
}

/// Compile with a specific document path (for proper ID generation)
pub fn compile_to_css_with_path(document: &Document, path: &str) -> CssResult<String> {
    let mut evaluator = CssEvaluator::with_document_id(path);
    let css_doc = evaluator.evaluate(document)?;
    Ok(css_doc.to_css())
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse;

    #[test]
    fn test_compile_simple_styles() {
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
        let css = compile_to_css(&document).expect("Failed to compile CSS");

        println!("Generated CSS:\n{}", css);

        assert!(css.contains("padding: 8px 16px"));
        assert!(css.contains("background: #3366FF"));
        assert!(css.contains("color: white"));
    }

    #[test]
    fn test_compile_with_tokens() {
        let source = r#"
public token primaryColor #3366FF
public token spacing 16px

public component Button {
    render button {
        style {
            padding: 8px 16px
            background: #3366FF
        }
        text "Click"
    }
}
"#;

        let document = parse(source).expect("Failed to parse");
        let css = compile_to_css(&document).expect("Failed to compile CSS");

        println!("Generated CSS:\n{}", css);

        // Tokens should be expanded to their values
        assert!(css.contains("16px") || css.contains("var("));
        assert!(css.contains("#3366FF") || css.contains("var("));
    }

    #[test]
    fn test_compile_multiple_components() {
        let source = r#"
public component Button {
    render button {
        style {
            padding: 8px 16px
        }
        text "Button"
    }
}

public component Card {
    render div {
        style {
            border: 1px solid #ddd
            padding: 16px
        }
        text "Card"
    }
}
"#;

        let document = parse(source).expect("Failed to parse");
        let css = compile_to_css(&document).expect("Failed to compile CSS");

        println!("Generated CSS:\n{}", css);

        // Should contain styles from both components
        assert!(css.contains("padding: 8px 16px"));
        assert!(css.contains("border: 1px solid #ddd"));
        assert!(css.contains("padding: 16px"));
    }

    #[test]
    fn test_compile_nested_elements() {
        let source = r#"
public component Card {
    render div {
        style {
            padding: 16px
        }
        div {
            style {
                color: #333
                font-size: 18px
            }
            text "Title"
        }
        div {
            style {
                color: #666
            }
            text "Body"
        }
    }
}
"#;

        let document = parse(source).expect("Failed to parse");
        let css = compile_to_css(&document).expect("Failed to compile CSS");

        println!("Generated CSS:\n{}", css);

        assert!(css.contains("padding: 16px"));
        assert!(css.contains("color: #333"));
        assert!(css.contains("font-size: 18px"));
        assert!(css.contains("color: #666"));
    }
}
