//! Lexer for Paperclip .pc files using logos
//!
//! Logos provides extremely fast lexing via compile-time DFA generation.

use logos::Logos;

/// Token types for Paperclip syntax
#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\r\n]+")]  // Skip whitespace
pub enum Token<'src> {
    // Keywords
    #[token("import")]
    Import,
    #[token("as")]
    As,
    #[token("public")]
    Public,
    #[token("token")]
    TokenKw,
    #[token("style")]
    Style,
    #[token("component")]
    Component,
    #[token("variant")]
    Variant,
    #[token("trigger")]
    Trigger,
    #[token("render")]
    Render,
    #[token("slot")]
    Slot,
    #[token("insert")]
    InsertKw,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("repeat")]
    Repeat,
    #[token("text")]
    Text,
    #[token("extends")]
    Extends,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("empty")]
    Empty,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice())]
    Ident(&'src str),

    // Literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        &s[1..s.len()-1]  // Strip quotes
    })]
    String(&'src str),

    #[regex(r"'([^'\\]|\\.)*'", |lex| {
        let s = lex.slice();
        &s[1..s.len()-1]  // Strip quotes
    })]
    SingleQuoteString(&'src str),

    #[regex(r"-?[0-9]+(\.[0-9]+)?", |lex| lex.slice())]
    Number(&'src str),

    #[regex(r"#[0-9a-fA-F]{3,8}", |lex| lex.slice())]
    Color(&'src str),

    // Units (dimensions)
    #[regex(r"-?[0-9]+(\.[0-9]+)?(px|em|rem|%|vh|vw|vmin|vmax|ch|ex|cm|mm|in|pt|pc|deg|rad|turn|s|ms)", |lex| lex.slice())]
    Dimension(&'src str),

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("=")]
    Eq,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token("<=")]
    Lte,
    #[token(">")]
    Gt,
    #[token(">=")]
    Gte,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Bang,
    #[token("?")]
    Question,

    // Punctuation
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
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semi,
    #[token(".")]
    Dot,

    // Comments
    #[regex(r"//[^\n]*", |lex| lex.slice())]
    LineComment(&'src str),

    #[regex(r"/\*\*[^*]*\*+(?:[^/*][^*]*\*+)*/", |lex| lex.slice())]
    DocComment(&'src str),

    #[regex(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/", |lex| lex.slice())]
    BlockComment(&'src str),
}

/// Span information for a token
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenSpan {
    pub start: usize,
    pub end: usize,
}

/// A token with its span
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken<'src> {
    pub token: Token<'src>,
    pub span: TokenSpan,
}

/// Lex source code into tokens with spans
pub fn lex(source: &str) -> impl Iterator<Item = Result<SpannedToken<'_>, LexError>> + '_ {
    Token::lexer(source).spanned().map(|(result, span)| {
        match result {
            Ok(token) => Ok(SpannedToken {
                token,
                span: TokenSpan {
                    start: span.start,
                    end: span.end,
                },
            }),
            Err(_) => Err(LexError {
                span: TokenSpan {
                    start: span.start,
                    end: span.end,
                },
                message: "Unexpected character".to_string(),
            }),
        }
    })
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub span: TokenSpan,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_keywords() {
        let source = "import public component style token variant";
        let tokens: Vec<_> = lex(source).filter_map(|r| r.ok()).collect();
        
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].token, Token::Import);
        assert_eq!(tokens[1].token, Token::Public);
        assert_eq!(tokens[2].token, Token::Component);
        assert_eq!(tokens[3].token, Token::Style);
        assert_eq!(tokens[4].token, Token::TokenKw);
        assert_eq!(tokens[5].token, Token::Variant);
    }

    #[test]
    fn test_lex_string() {
        let source = r#""hello world""#;
        let tokens: Vec<_> = lex(source).filter_map(|r| r.ok()).collect();
        
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::String("hello world"));
    }

    #[test]
    fn test_lex_color() {
        let source = "#3366FF #fff";
        let tokens: Vec<_> = lex(source).filter_map(|r| r.ok()).collect();
        
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::Color("#3366FF"));
        assert_eq!(tokens[1].token, Token::Color("#fff"));
    }

    #[test]
    fn test_lex_dimension() {
        let source = "16px 1.5rem 100%";
        let tokens: Vec<_> = lex(source).filter_map(|r| r.ok()).collect();
        
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Dimension("16px"));
        assert_eq!(tokens[1].token, Token::Dimension("1.5rem"));
        assert_eq!(tokens[2].token, Token::Dimension("100%"));
    }

    #[test]
    fn test_lex_doc_comment() {
        let source = "/** @frame(x: 100) */";
        let tokens: Vec<_> = lex(source).filter_map(|r| r.ok()).collect();
        
        assert_eq!(tokens.len(), 1);
        matches!(tokens[0].token, Token::DocComment(_));
    }
}
