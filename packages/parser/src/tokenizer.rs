use logos::Logos;
use std::fmt;

/// Token types for the Paperclip language
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token<'src> {
    // Keywords
    #[token("component")]
    Component,

    #[token("public")]
    Public,

    #[token("render")]
    Render,

    #[token("style")]
    Style,

    #[token("token")]
    TokenKeyword,

    #[token("variant")]
    Variant,

    #[token("slot")]
    Slot,

    #[token("import")]
    Import,

    #[token("as")]
    As,

    #[token("from")]
    From,

    #[token("extends")]
    Extends,

    #[token("trigger")]
    Trigger,

    #[token("if")]
    If,

    #[token("repeat")]
    Repeat,

    #[token("text")]
    Text,

    #[token("div")]
    Div,

    #[token("span")]
    Span,

    #[token("button")]
    Button,

    #[token("img")]
    Img,

    #[token("input")]
    Input,

    // Identifiers (including CSS properties with dashes like margin-bottom)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice())]
    Ident(&'src str),

    // String literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice())]
    String(&'src str),

    // Numbers
    #[regex(r"-?[0-9]+(\.[0-9]+)?", |lex| lex.slice())]
    Number(&'src str),

    // Color values
    #[regex(r"#[0-9a-fA-F]{3,8}", |lex| lex.slice())]
    Color(&'src str),

    // CSS units
    #[regex(r"-?[0-9]+(\.[0-9]+)?(px|em|rem|%|vh|vw)", |lex| lex.slice())]
    CssUnit(&'src str),

    // Symbols
    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("<")]
    LAngle,

    #[token(">")]
    RAngle,

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token("=")]
    Equals,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("!")]
    Bang,

    #[token("@")]
    At,

    #[token("$")]
    Dollar,

    #[token("&")]
    Ampersand,

    #[token("|")]
    Pipe,
}

impl<'src> fmt::Display for Token<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Component => write!(f, "component"),
            Token::Public => write!(f, "public"),
            Token::Render => write!(f, "render"),
            Token::Style => write!(f, "style"),
            Token::TokenKeyword => write!(f, "token"),
            Token::Variant => write!(f, "variant"),
            Token::Slot => write!(f, "slot"),
            Token::Import => write!(f, "import"),
            Token::As => write!(f, "as"),
            Token::From => write!(f, "from"),
            Token::Extends => write!(f, "extends"),
            Token::Trigger => write!(f, "trigger"),
            Token::If => write!(f, "if"),
            Token::Repeat => write!(f, "repeat"),
            Token::Text => write!(f, "text"),
            Token::Div => write!(f, "div"),
            Token::Span => write!(f, "span"),
            Token::Button => write!(f, "button"),
            Token::Img => write!(f, "img"),
            Token::Input => write!(f, "input"),
            Token::Ident(s) => write!(f, "identifier '{}'", s),
            Token::String(s) => write!(f, "string {}", s),
            Token::Number(n) => write!(f, "number {}", n),
            Token::Color(c) => write!(f, "color {}", c),
            Token::CssUnit(u) => write!(f, "css unit {}", u),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LAngle => write!(f, "<"),
            Token::RAngle => write!(f, ">"),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Equals => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Bang => write!(f, "!"),
            Token::At => write!(f, "@"),
            Token::Dollar => write!(f, "$"),
            Token::Ampersand => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
        }
    }
}

/// Tokenize a source string
pub fn tokenize(source: &str) -> Vec<(Token, std::ops::Range<usize>)> {
    let lexer = Token::lexer(source);
    lexer
        .spanned()
        .filter_map(|(result, span)| result.ok().map(|token| (token, span)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let source = "component public render style token";
        let tokens = tokenize(source);

        assert_eq!(tokens[0].0, Token::Component);
        assert_eq!(tokens[1].0, Token::Public);
        assert_eq!(tokens[2].0, Token::Render);
        assert_eq!(tokens[3].0, Token::Style);
        assert_eq!(tokens[4].0, Token::TokenKeyword);
    }

    #[test]
    fn test_identifiers() {
        let source = "Button myComponent _private";
        let tokens = tokenize(source);

        assert_eq!(tokens[0].0, Token::Ident("Button"));
        assert_eq!(tokens[1].0, Token::Ident("myComponent"));
        assert_eq!(tokens[2].0, Token::Ident("_private"));
    }

    #[test]
    fn test_strings() {
        let source = r#""hello world" "escaped \"quote\"" "#;
        let tokens = tokenize(source);

        assert!(matches!(tokens[0].0, Token::String(_)));
        assert!(matches!(tokens[1].0, Token::String(_)));
    }

    #[test]
    fn test_numbers_and_colors() {
        let source = "42 3.14 -10 #FF0000 #333";
        let tokens = tokenize(source);

        assert_eq!(tokens[0].0, Token::Number("42"));
        assert_eq!(tokens[1].0, Token::Number("3.14"));
        assert_eq!(tokens[2].0, Token::Number("-10"));
        assert_eq!(tokens[3].0, Token::Color("#FF0000"));
        assert_eq!(tokens[4].0, Token::Color("#333"));
    }

    #[test]
    fn test_css_units() {
        let source = "16px 1.5em 100% 50vh";
        let tokens = tokenize(source);

        assert_eq!(tokens[0].0, Token::CssUnit("16px"));
        assert_eq!(tokens[1].0, Token::CssUnit("1.5em"));
        assert_eq!(tokens[2].0, Token::CssUnit("100%"));
        assert_eq!(tokens[3].0, Token::CssUnit("50vh"));
    }

    #[test]
    fn test_component_structure() {
        let source = r#"
            public component Button {
                render button {
                    style {
                        background: #FF0000
                    }
                }
            }
        "#;

        let tokens = tokenize(source);

        assert!(tokens.iter().any(|(t, _)| *t == Token::Public));
        assert!(tokens.iter().any(|(t, _)| *t == Token::Component));
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Ident("Button"))));
        assert!(tokens.iter().any(|(t, _)| *t == Token::LBrace));
        assert!(tokens.iter().any(|(t, _)| *t == Token::RBrace));
    }

    #[test]
    fn test_comments_ignored() {
        let source = r#"
            // Single line comment
            component Button /* block comment */ {
                /* Multi-line
                   comment */
                render button
            }
        "#;

        let tokens = tokenize(source);

        // Comments should be skipped
        assert!(!tokens.iter().any(|(t, _)| matches!(t, Token::Slash) && matches!(tokens.get(1), Some((Token::Slash, _)))));
    }
}
