#[cfg(test)]
mod debug_tests {
    use crate::tokenizer::tokenize;

    #[test]
    fn debug_style_tokens() {
        let source = r#"
            component Card {
                render div {
                    style {
                        padding: 16px
                        background: #FF0000
                    }
                    text "Hello"
                }
            }
        "#;

        let tokens = tokenize(source);
        for (i, (token, span)) in tokens.iter().enumerate() {
            println!("{}: {:?} at {:?}", i, token, span);
        }
    }
}
