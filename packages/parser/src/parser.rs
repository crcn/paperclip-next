use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::tokenizer::{Token, tokenize};
use std::collections::HashMap;

/// Parser for Paperclip language
pub struct Parser<'src> {
    tokens: Vec<(Token<'src>, std::ops::Range<usize>)>,
    pos: usize,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        let tokens = tokenize(source);
        Self { tokens, pos: 0 }
    }

    /// Parse a complete document
    pub fn parse_document(&mut self) -> ParseResult<Document> {
        let mut doc = Document::new();

        while !self.is_at_end() {
            match self.peek() {
                Some((Token::Import, _)) => {
                    doc.imports.push(self.parse_import()?);
                }
                Some((Token::Public, _)) => {
                    self.advance(); // consume 'public'
                    match self.peek() {
                        Some((Token::TokenKeyword, _)) => {
                            doc.tokens.push(self.parse_token_decl(true)?);
                        }
                        Some((Token::Style, _)) => {
                            doc.styles.push(self.parse_style_decl(true)?);
                        }
                        Some((Token::Component, _)) => {
                            doc.components.push(self.parse_component(true)?);
                        }
                        _ => {
                            return Err(ParseError::invalid_syntax(
                                self.current_pos(),
                                "Expected 'token', 'style', or 'component' after 'public'",
                            ));
                        }
                    }
                }
                Some((Token::TokenKeyword, _)) => {
                    doc.tokens.push(self.parse_token_decl(false)?);
                }
                Some((Token::Style, _)) => {
                    doc.styles.push(self.parse_style_decl(false)?);
                }
                Some((Token::Component, _)) => {
                    doc.components.push(self.parse_component(false)?);
                }
                _ => {
                    return Err(ParseError::invalid_syntax(
                        self.current_pos(),
                        format!("Unexpected token: {:?}", self.peek()),
                    ));
                }
            }
        }

        Ok(doc)
    }

    /// Parse an import statement
    fn parse_import(&mut self) -> ParseResult<Import> {
        let start = self.current_pos();
        self.expect(Token::Import)?;

        let path = self.expect_string()?;

        let alias = if self.match_token(Token::As) {
            Some(self.expect_ident()?)
        } else {
            None
        };

        let end = self.current_pos();

        Ok(Import {
            path,
            alias,
            span: Span::new(start, end),
        })
    }

    /// Parse a token declaration
    fn parse_token_decl(&mut self, public: bool) -> ParseResult<TokenDecl> {
        let start = self.current_pos();
        self.expect(Token::TokenKeyword)?;

        let name = self.expect_ident()?;
        let value = self.parse_token_value()?;

        let end = self.current_pos();

        Ok(TokenDecl {
            public,
            name,
            value,
            span: Span::new(start, end),
        })
    }

    /// Parse a token value (color, number, string, etc.)
    fn parse_token_value(&mut self) -> ParseResult<String> {
        match self.peek() {
            Some((Token::Color(c), _)) => {
                let val = c.to_string();
                self.advance();
                Ok(val)
            }
            Some((Token::Number(n), _)) => {
                let val = n.to_string();
                self.advance();
                Ok(val)
            }
            Some((Token::String(s), _)) => {
                let val = s.to_string();
                self.advance();
                Ok(val)
            }
            Some((Token::CssUnit(u), _)) => {
                let val = u.to_string();
                self.advance();
                Ok(val)
            }
            Some((Token::Ident(i), _)) => {
                let val = i.to_string();
                self.advance();
                // Support multi-word values like "Inter, sans-serif"
                while self.match_token(Token::Comma) {
                    let next = self.expect_ident()?;
                    return Ok(format!("{}, {}", val, next));
                }
                Ok(val)
            }
            _ => Err(ParseError::invalid_syntax(
                self.current_pos(),
                "Expected token value",
            )),
        }
    }

    /// Parse a style declaration
    fn parse_style_decl(&mut self, public: bool) -> ParseResult<StyleDecl> {
        let start = self.current_pos();
        self.expect(Token::Style)?;

        let name = self.expect_ident()?;

        let mut extends = Vec::new();
        if self.match_token(Token::Extends) {
            extends.push(self.expect_ident()?);
            while self.match_token(Token::Comma) {
                extends.push(self.expect_ident()?);
            }
        }

        self.expect(Token::LBrace)?;
        let properties = self.parse_style_properties()?;
        self.expect(Token::RBrace)?;

        let end = self.current_pos();

        Ok(StyleDecl {
            public,
            name,
            extends,
            properties,
            span: Span::new(start, end),
        })
    }

    /// Parse style properties
    fn parse_style_properties(&mut self) -> ParseResult<HashMap<String, String>> {
        let mut properties = HashMap::new();

        while !self.check(Token::RBrace) && !self.is_at_end() {
            let prop_name = self.expect_ident()?;
            self.expect(Token::Colon)?;

            let value = self.parse_style_value()?;
            properties.insert(prop_name, value);

            // Optional semicolon
            self.match_token(Token::Semicolon);
        }

        Ok(properties)
    }

    /// Parse a style property value
    fn parse_style_value(&mut self) -> ParseResult<String> {
        let mut value = String::new();
        let mut token_count = 0;

        // Collect tokens until semicolon or closing brace
        // Stop early if we see an identifier that could start a new property (identifier followed by colon)
        while !self.check(Token::Semicolon) && !self.check(Token::RBrace) && !self.is_at_end() {
            match self.peek() {
                Some((Token::Ident(s), _)) => {
                    // Check if next token is colon (would indicate start of new property)
                    if token_count > 0 && self.peek_ahead(1).map(|(t, _)| matches!(t, Token::Colon)).unwrap_or(false) {
                        break;
                    }
                    if !value.is_empty() {
                        value.push(' ');
                    }
                    value.push_str(s);
                    self.advance();
                    token_count += 1;
                }
                Some((Token::Number(n), _)) => {
                    if !value.is_empty() {
                        value.push(' ');
                    }
                    value.push_str(n);
                    self.advance();
                    token_count += 1;
                }
                Some((Token::Color(c), _)) => {
                    if !value.is_empty() {
                        value.push(' ');
                    }
                    value.push_str(c);
                    self.advance();
                    token_count += 1;
                }
                Some((Token::CssUnit(u), _)) => {
                    if !value.is_empty() {
                        value.push(' ');
                    }
                    value.push_str(u);
                    self.advance();
                    token_count += 1;
                }
                Some((Token::String(s), _)) => {
                    if !value.is_empty() {
                        value.push(' ');
                    }
                    value.push_str(s);
                    self.advance();
                    token_count += 1;
                }
                Some((Token::LParen, _)) => {
                    value.push('(');
                    self.advance();
                }
                Some((Token::RParen, _)) => {
                    value.push(')');
                    self.advance();
                }
                Some((Token::Comma, _)) => {
                    value.push(',');
                    self.advance();
                }
                _ => break,
            }
        }

        Ok(value.trim().to_string())
    }

    /// Parse a component
    fn parse_component(&mut self, public: bool) -> ParseResult<Component> {
        let start = self.current_pos();
        self.expect(Token::Component)?;

        let name = self.expect_ident()?;

        self.expect(Token::LBrace)?;

        let mut variants = Vec::new();
        let mut slots = Vec::new();
        let mut body = None;

        while !self.check(Token::RBrace) && !self.is_at_end() {
            match self.peek() {
                Some((Token::Variant, _)) => {
                    variants.push(self.parse_variant()?);
                }
                Some((Token::Slot, _)) => {
                    slots.push(self.parse_slot()?);
                }
                Some((Token::Render, _)) => {
                    self.advance();
                    body = Some(self.parse_element()?);
                }
                _ => {
                    return Err(ParseError::invalid_syntax(
                        self.current_pos(),
                        "Expected 'variant', 'slot', or 'render'",
                    ));
                }
            }
        }

        self.expect(Token::RBrace)?;

        let end = self.current_pos();

        Ok(Component {
            public,
            name,
            variants,
            slots,
            body,
            span: Span::new(start, end),
        })
    }

    /// Parse a variant
    fn parse_variant(&mut self) -> ParseResult<Variant> {
        let start = self.current_pos();
        self.expect(Token::Variant)?;

        let name = self.expect_ident()?;

        let mut triggers = Vec::new();
        if self.match_token(Token::Trigger) {
            self.expect(Token::LBrace)?;
            while !self.check(Token::RBrace) && !self.is_at_end() {
                triggers.push(self.expect_string()?);
                if !self.match_token(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::RBrace)?;
        }

        let end = self.current_pos();

        Ok(Variant {
            name,
            triggers,
            span: Span::new(start, end),
        })
    }

    /// Parse a slot
    fn parse_slot(&mut self) -> ParseResult<Slot> {
        let start = self.current_pos();
        self.expect(Token::Slot)?;

        let name = self.expect_ident()?;

        let mut default_content = Vec::new();
        if self.match_token(Token::LBrace) {
            while !self.check(Token::RBrace) && !self.is_at_end() {
                default_content.push(self.parse_element()?);
            }
            self.expect(Token::RBrace)?;
        }

        let end = self.current_pos();

        Ok(Slot {
            name,
            default_content,
            span: Span::new(start, end),
        })
    }

    /// Parse an element
    fn parse_element(&mut self) -> ParseResult<Element> {
        let start = self.current_pos();

        match self.peek() {
            Some((Token::Text, _)) => {
                self.advance();
                let content = self.parse_expression()?;
                let end = self.current_pos();
                Ok(Element::Text {
                    content,
                    span: Span::new(start, end),
                })
            }
            Some((Token::Div, _)) => {
                self.advance();
                self.parse_tag_element("div".to_string(), start)
            }
            Some((Token::Span, _)) => {
                self.advance();
                self.parse_tag_element("span".to_string(), start)
            }
            Some((Token::Button, _)) => {
                self.advance();
                self.parse_tag_element("button".to_string(), start)
            }
            Some((Token::Img, _)) => {
                self.advance();
                self.parse_tag_element("img".to_string(), start)
            }
            Some((Token::Input, _)) => {
                self.advance();
                self.parse_tag_element("input".to_string(), start)
            }
            Some((Token::If, _)) => self.parse_conditional(start),
            Some((Token::Repeat, _)) => self.parse_repeat(start),
            Some((Token::Ident(_), _)) => {
                let name = self.expect_ident()?;
                // Could be component instance or slot insert
                if self.check(Token::LParen) {
                    self.parse_instance(name, start)
                } else {
                    // Treat as slot insert
                    let end = self.current_pos();
                    Ok(Element::SlotInsert {
                        name,
                        span: Span::new(start, end),
                    })
                }
            }
            _ => Err(ParseError::invalid_syntax(
                self.current_pos(),
                "Expected element",
            )),
        }
    }

    /// Parse a tag element (div, span, etc.)
    fn parse_tag_element(&mut self, name: String, start: usize) -> ParseResult<Element> {
        let mut attributes = HashMap::new();
        let mut styles = Vec::new();
        let mut children = Vec::new();

        if self.match_token(Token::LParen) {
            // Parse attributes
            while !self.check(Token::RParen) && !self.is_at_end() {
                let attr_name = self.expect_ident()?;
                self.expect(Token::Equals)?;
                let attr_value = self.parse_expression()?;
                attributes.insert(attr_name, attr_value);

                if !self.match_token(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::RParen)?;
        }

        if self.match_token(Token::LBrace) {
            while !self.check(Token::RBrace) && !self.is_at_end() {
                if self.check(Token::Style) {
                    styles.push(self.parse_style_block()?);
                } else {
                    children.push(self.parse_element()?);
                }
            }
            self.expect(Token::RBrace)?;
        }

        let end = self.current_pos();

        Ok(Element::Tag {
            name,
            attributes,
            styles,
            children,
            span: Span::new(start, end),
        })
    }

    /// Parse a style block
    fn parse_style_block(&mut self) -> ParseResult<StyleBlock> {
        let start = self.current_pos();
        self.expect(Token::Style)?;

        let variant = None; // Simplified for vertical slice

        let mut extends = Vec::new();
        if self.match_token(Token::Extends) {
            extends.push(self.expect_ident()?);
        }

        self.expect(Token::LBrace)?;
        let properties = self.parse_style_properties()?;
        self.expect(Token::RBrace)?;

        let end = self.current_pos();

        Ok(StyleBlock {
            variant,
            extends,
            properties,
            span: Span::new(start, end),
        })
    }

    /// Parse a conditional
    fn parse_conditional(&mut self, start: usize) -> ParseResult<Element> {
        self.expect(Token::If)?;
        let condition = self.parse_expression()?;

        self.expect(Token::LBrace)?;
        let mut then_branch = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            then_branch.push(self.parse_element()?);
        }
        self.expect(Token::RBrace)?;

        let else_branch = None; // Simplified for vertical slice

        let end = self.current_pos();

        Ok(Element::Conditional {
            condition,
            then_branch,
            else_branch,
            span: Span::new(start, end),
        })
    }

    /// Parse a repeat
    fn parse_repeat(&mut self, start: usize) -> ParseResult<Element> {
        self.expect(Token::Repeat)?;
        let item_name = self.expect_ident()?;
        // Expect 'in'
        if let Some((Token::Ident("in"), _)) = self.peek() {
            self.advance();
        }
        let collection = self.parse_expression()?;

        self.expect(Token::LBrace)?;
        let mut body = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            body.push(self.parse_element()?);
        }
        self.expect(Token::RBrace)?;

        let end = self.current_pos();

        Ok(Element::Repeat {
            item_name,
            collection,
            body,
            span: Span::new(start, end),
        })
    }

    /// Parse a component instance
    fn parse_instance(&mut self, name: String, start: usize) -> ParseResult<Element> {
        let mut props = HashMap::new();
        let mut children = Vec::new();

        if self.match_token(Token::LParen) {
            while !self.check(Token::RParen) && !self.is_at_end() {
                let prop_name = self.expect_ident()?;
                self.expect(Token::Equals)?;
                let prop_value = self.parse_expression()?;
                props.insert(prop_name, prop_value);

                if !self.match_token(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::RParen)?;
        }

        if self.match_token(Token::LBrace) {
            while !self.check(Token::RBrace) && !self.is_at_end() {
                children.push(self.parse_element()?);
            }
            self.expect(Token::RBrace)?;
        }

        let end = self.current_pos();

        Ok(Element::Instance {
            name,
            props,
            children,
            span: Span::new(start, end),
        })
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_pos();

        match self.peek() {
            Some((Token::String(s), _)) => {
                let val = s.trim_matches('"').to_string();
                self.advance();
                let end = self.current_pos();
                Ok(Expression::Literal {
                    value: val,
                    span: Span::new(start, end),
                })
            }
            Some((Token::Number(n), _)) => {
                let val = n.parse::<f64>().unwrap_or(0.0);
                self.advance();
                let end = self.current_pos();
                Ok(Expression::Number {
                    value: val,
                    span: Span::new(start, end),
                })
            }
            Some((Token::Ident(i), _)) => {
                let name = i.to_string();
                self.advance();

                // Check for member access
                if self.match_token(Token::Dot) {
                    let property = self.expect_ident()?;
                    let end = self.current_pos();
                    Ok(Expression::Member {
                        object: Box::new(Expression::Variable {
                            name,
                            span: Span::new(start, end),
                        }),
                        property,
                        span: Span::new(start, end),
                    })
                } else {
                    let end = self.current_pos();
                    Ok(Expression::Variable {
                        name,
                        span: Span::new(start, end),
                    })
                }
            }
            Some((Token::LBrace, _)) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RBrace)?;
                Ok(expr)
            }
            _ => {
                // Default to empty literal
                Ok(Expression::Literal {
                    value: String::new(),
                    span: Span::new(start, start),
                })
            }
        }
    }

    // Helper methods

    fn peek(&self) -> Option<&(Token<'src>, std::ops::Range<usize>)> {
        self.tokens.get(self.pos)
    }

    fn peek_ahead(&self, offset: usize) -> Option<&(Token<'src>, std::ops::Range<usize>)> {
        self.tokens.get(self.pos + offset)
    }

    fn advance(&mut self) -> Option<&(Token<'src>, std::ops::Range<usize>)> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn check(&self, token: Token) -> bool {
        if let Some((t, _)) = self.peek() {
            std::mem::discriminant(t) == std::mem::discriminant(&token)
        } else {
            false
        }
    }

    fn match_token(&mut self, token: Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: Token) -> ParseResult<()> {
        if self.check(token.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::unexpected_token(
                self.current_pos(),
                format!("{:?}", token),
                format!("{:?}", self.peek()),
            ))
        }
    }

    fn expect_ident(&mut self) -> ParseResult<String> {
        match self.peek() {
            Some((Token::Ident(s), _)) => {
                let val = s.to_string();
                self.advance();
                Ok(val)
            }
            _ => Err(ParseError::unexpected_token(
                self.current_pos(),
                "identifier",
                format!("{:?}", self.peek()),
            )),
        }
    }

    fn expect_string(&mut self) -> ParseResult<String> {
        match self.peek() {
            Some((Token::String(s), _)) => {
                let val = s.trim_matches('"').to_string();
                self.advance();
                Ok(val)
            }
            _ => Err(ParseError::unexpected_token(
                self.current_pos(),
                "string",
                format!("{:?}", self.peek()),
            )),
        }
    }

    fn current_pos(&self) -> usize {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map(|(_, span)| span.start)
            .unwrap_or(0)
    }
}

pub fn parse(source: &str) -> ParseResult<Document> {
    let mut parser = Parser::new(source);
    parser.parse_document()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_component() {
        let source = r#"
            component Button {
                render button {
                    text "Click me"
                }
            }
        "#;

        let result = parse(source);
        if let Err(ref e) = result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
        assert_eq!(doc.components[0].name, "Button");
    }

    #[test]
    fn test_parse_component_with_style() {
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

        let result = parse(source);
        if let Err(ref e) = result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
    }

    #[test]
    fn test_parse_token() {
        let source = r#"
            public token primaryColor #3366FF
        "#;

        let result = parse(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.tokens.len(), 1);
        assert_eq!(doc.tokens[0].name, "primaryColor");
        assert_eq!(doc.tokens[0].value, "#3366FF");
    }
}
