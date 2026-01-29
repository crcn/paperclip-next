use crate::ast::TriggerDecl;
use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::id_generator::IDGenerator;
use crate::tokenizer::{tokenize, Token};
use std::collections::HashMap;

/// Parser for Paperclip language
pub struct Parser<'src> {
    tokens: Vec<(Token<'src>, std::ops::Range<usize>)>,
    pos: usize,
    id_generator: IDGenerator,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, id_generator: IDGenerator) -> Self {
        let tokens = tokenize(source);
        Self {
            tokens,
            pos: 0,
            id_generator,
        }
    }

    #[cfg(test)]
    pub fn new_with_path(source: &'src str, path: &str) -> Self {
        Self::new(source, IDGenerator::new(path))
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
                        Some((Token::Trigger, _)) => {
                            doc.triggers.push(self.parse_trigger_decl(true)?);
                        }
                        Some((Token::Style, _)) => {
                            doc.styles.push(self.parse_style_decl(true)?);
                        }
                        Some((Token::Component, _)) => {
                            doc.components.push(self.parse_component(true)?);
                        }
                        _ => {
                            return Err(ParseError::invalid_syntax_span(
                                self.peek_span(),
                                "Expected 'token', 'trigger', 'style', or 'component' after 'public'",
                            ));
                        }
                    }
                }
                Some((Token::TokenKeyword, _)) => {
                    doc.tokens.push(self.parse_token_decl(false)?);
                }
                Some((Token::Trigger, _)) => {
                    doc.triggers.push(self.parse_trigger_decl(false)?);
                }
                Some((Token::Style, _)) => {
                    doc.styles.push(self.parse_style_decl(false)?);
                }
                Some((Token::Component, _)) => {
                    doc.components.push(self.parse_component(false)?);
                }
                _ => {
                    return Err(ParseError::invalid_syntax_span(
                        self.peek_span(),
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
            span: Span::new(start, end, self.id_generator.new_id()),
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
            span: Span::new(start, end, self.id_generator.new_id()),
        })
    }

    /// Parse a trigger declaration
    fn parse_trigger_decl(&mut self, public: bool) -> ParseResult<TriggerDecl> {
        let start = self.current_pos();
        self.expect(Token::Trigger)?;

        let name = self.expect_ident()?;

        self.expect(Token::LBrace)?;

        let mut selectors = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            let selector = self.expect_string()?;
            selectors.push(selector);

            if !self.check(Token::RBrace) {
                // Comma is optional between selectors
                self.match_token(Token::Comma);
            }
        }

        self.expect(Token::RBrace)?;

        let end = self.current_pos();
        Ok(TriggerDecl {
            public,
            name,
            selectors,
            span: Span::new(start, end, self.id_generator.new_id()),
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
            _ => Err(ParseError::invalid_syntax_span(
                self.peek_span(),
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
            // Parse potentially namespaced reference like "theme.fontRegular"
            extends.push(self.parse_namespaced_ref()?);
            while self.match_token(Token::Comma) {
                extends.push(self.parse_namespaced_ref()?);
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
            span: Span::new(start, end, self.id_generator.new_id()),
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
                    if token_count > 0
                        && self
                            .peek_ahead(1)
                            .map(|(t, _)| matches!(t, Token::Colon))
                            .unwrap_or(false)
                    {
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
                Some((Token::Dot, _)) => {
                    value.push('.');
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

        let mut script = None;
        let frame = None;
        let mut variants = Vec::new();
        let mut slots = Vec::new();
        let mut overrides = Vec::new();
        let mut body = None;

        while !self.check(Token::RBrace) && !self.is_at_end() {
            match self.peek() {
                Some((Token::Script, _)) => {
                    script = Some(self.parse_script_directive()?);
                }
                Some((Token::Variant, _)) => {
                    variants.push(self.parse_variant()?);
                }
                Some((Token::Slot, _)) => {
                    slots.push(self.parse_slot()?);
                }
                Some((Token::Override, _)) => {
                    overrides.push(self.parse_override()?);
                }
                Some((Token::Render, _)) => {
                    self.advance();
                    body = Some(self.parse_element()?);
                }
                _ => {
                    return Err(ParseError::invalid_syntax_span(
                        self.peek_span(),
                        "Expected 'script', 'variant', 'slot', 'override', or 'render'",
                    ));
                }
            }
        }

        self.expect(Token::RBrace)?;

        let end = self.current_pos();

        Ok(Component {
            public,
            name,
            script,
            frame,
            variants,
            slots,
            overrides,
            body,
            span: Span::new(start, end, self.id_generator.new_id()),
        })
    }

    /// Parse a script directive: script(src: "...", target: "react", name: "Name")
    fn parse_script_directive(&mut self) -> ParseResult<ScriptDirective> {
        let start = self.current_pos();
        self.expect(Token::Script)?;
        self.expect(Token::LParen)?;

        let mut src = None;
        let mut target = None;
        let mut name = None;

        while !self.check(Token::RParen) && !self.is_at_end() {
            let param_name = self.expect_ident()?;
            self.expect(Token::Colon)?;
            let param_value = self.expect_string()?;

            match param_name.as_str() {
                "src" => src = Some(param_value),
                "target" => target = Some(param_value),
                "name" => name = Some(param_value),
                _ => {
                    return Err(ParseError::invalid_syntax_span(
                        self.current_span(),
                        format!("Unknown script parameter: {}", param_name),
                    ));
                }
            }

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RParen)?;

        let end = self.current_pos();

        let src = src.ok_or_else(|| {
            ParseError::invalid_syntax_span(
                start..end,
                "Missing required 'src' parameter in script directive",
            )
        })?;

        let target = target.ok_or_else(|| {
            ParseError::invalid_syntax_span(
                start..end,
                "Missing required 'target' parameter in script directive",
            )
        })?;

        Ok(ScriptDirective {
            src,
            target,
            name,
            span: Span::new(start, end, self.id_generator.new_id()),
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
                // Variant triggers can be either:
                // - String literal (direct CSS selector): ":hover"
                // - Identifier (trigger reference): mobile
                let trigger = match self.peek() {
                    Some((Token::String(_), _)) => self.expect_string()?,
                    Some((Token::Ident(_), _)) => self.expect_ident()?,
                    _ => {
                        return Err(ParseError::invalid_syntax_span(
                            self.peek_span(),
                            "Expected trigger reference (identifier) or CSS selector (string)",
                        ))
                    }
                };
                triggers.push(trigger);
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
            span: Span::new(start, end, self.id_generator.new_id()),
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
            span: Span::new(start, end, self.id_generator.new_id()),
        })
    }

    /// Parse an override: override Button.Icon { style { ... } }
    fn parse_override(&mut self) -> ParseResult<Override> {
        let start = self.current_pos();
        self.expect(Token::Override)?;

        // Parse dot-separated path: Button.Icon.Path or div.span
        // Accept both identifiers and element keywords (div, span, button, img)
        let mut path = Vec::new();
        path.push(self.expect_ident_or_element_keyword()?);

        while self.match_token(Token::Dot) {
            path.push(self.expect_ident_or_element_keyword()?);
        }

        // Parse attributes in parentheses (new syntax): override div(id="custom")
        let mut attributes = std::collections::HashMap::new();
        if self.match_token(Token::LParen) {
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

        self.expect(Token::LBrace)?;

        // Inside braces: only styles allowed now
        let mut styles = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            if self.check(Token::Style) {
                styles.push(self.parse_style_block()?);
            } else {
                return Err(ParseError::invalid_syntax_span(
                    self.peek_span(),
                    "Expected 'style' block in override (attributes go in parentheses)",
                ));
            }
        }

        self.expect(Token::RBrace)?;

        let end = self.current_pos();

        Ok(Override {
            path,
            styles,
            attributes,
            span: Span::new(start, end, self.id_generator.new_id()),
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
                    span: Span::new(start, end, self.id_generator.new_id()),
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
            Some((Token::Insert, _)) => self.parse_insert(start),
            Some((Token::Ident(_), _)) => {
                let name = self.expect_ident()?;

                // Distinguish between HTML tags (lowercase) and Components (capitalized)
                let first_char = name.chars().next().unwrap_or('_');
                let is_html_tag = first_char.is_lowercase();

                if self.check(Token::LParen) || self.check(Token::LBrace) {
                    if is_html_tag {
                        // Lowercase identifier: parse as HTML tag element
                        self.parse_tag_element(name, start)
                    } else {
                        // Capitalized identifier: parse as component instance
                        self.parse_instance(name, start)
                    }
                } else {
                    // Treat as slot insert (bare identifier)
                    let end = self.current_pos();
                    Ok(Element::SlotInsert {
                        name,
                        span: Span::new(start, end, self.id_generator.new_id()),
                    })
                }
            }
            _ => Err(ParseError::invalid_syntax_span(
                self.peek_span(),
                "Expected element",
            )),
        }
    }

    /// Parse an insert directive: insert slotName { ... }
    fn parse_insert(&mut self, start: usize) -> ParseResult<Element> {
        self.expect(Token::Insert)?;
        let slot_name = self.expect_ident()?;

        self.expect(Token::LBrace)?;
        let mut content = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            content.push(self.parse_element()?);
        }
        self.expect(Token::RBrace)?;

        let end = self.current_pos();

        Ok(Element::Insert {
            slot_name,
            content,
            span: Span::new(start, end, self.id_generator.new_id()),
        })
    }

    /// Parse a tag element (div, span, etc.)
    /// Supports: div myName (attrs) { ... } or div (attrs) { ... }
    fn parse_tag_element(&mut self, tag_name: String, start: usize) -> ParseResult<Element> {
        let mut element_name = None;
        let mut attributes = HashMap::new();
        let mut styles = Vec::new();
        let mut children = Vec::new();

        // Check for optional element name (identifier before parens or brace)
        if let Some((Token::Ident(_), _)) = self.peek() {
            // Only consume if it's not followed by '=' (which would be an attribute)
            if !matches!(self.peek_ahead(1), Some((Token::Equals, _))) {
                element_name = Some(self.expect_ident()?);
            }
        }

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
            tag_name,
            name: element_name,
            attributes,
            styles,
            children,
            span: Span::new(start, end, self.id_generator.new_id()),
        })
    }

    /// Parse a style block
    /// Supports: style variant a + b + c { ... }
    /// Also supports: style extends baseStyle (without body)
    fn parse_style_block(&mut self) -> ParseResult<StyleBlock> {
        let start = self.current_pos();
        self.expect(Token::Style)?;

        let mut variants = Vec::new();

        // Check for variant combinations: variant a + b + c
        if self.match_token(Token::Variant) {
            variants.push(self.expect_ident()?);

            // Parse additional variants separated by '+'
            while self.match_token(Token::Plus) {
                variants.push(self.expect_ident()?);
            }
        }

        let mut extends = Vec::new();
        if self.match_token(Token::Extends) {
            // Parse potentially namespaced reference like "theme.fontRegular"
            extends.push(self.parse_namespaced_ref()?);
            while self.match_token(Token::Comma) {
                extends.push(self.parse_namespaced_ref()?);
            }
        }

        let mut properties = HashMap::new();

        // Body is optional if we have extends
        if self.match_token(Token::LBrace) {
            properties = self.parse_style_properties()?;
            self.expect(Token::RBrace)?;
        }

        let end = self.current_pos();

        Ok(StyleBlock {
            variants,
            extends,
            properties,
            span: Span::new(start, end, self.id_generator.new_id()),
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
            span: Span::new(start, end, self.id_generator.new_id()),
        })
    }

    /// Parse a repeat: repeat item in collection { ... }
    fn parse_repeat(&mut self, start: usize) -> ParseResult<Element> {
        self.expect(Token::Repeat)?;
        let item_name = self.expect_ident()?;
        self.expect(Token::In)?;
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
            span: Span::new(start, end, self.id_generator.new_id()),
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
            span: Span::new(start, end, self.id_generator.new_id()),
        })
    }

    /// Parse an expression with full operator precedence
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_or_expression()
    }

    /// Parse OR expression (lowest precedence)
    fn parse_or_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_pos();
        let mut left = self.parse_and_expression()?;

        while self.match_token(Token::Or) {
            let right = self.parse_and_expression()?;
            let end = self.current_pos();
            left = Expression::Binary {
                left: Box::new(left),
                operator: BinaryOp::Or,
                right: Box::new(right),
                span: Span::new(start, end, self.id_generator.new_id()),
            };
        }

        Ok(left)
    }

    /// Parse AND expression
    fn parse_and_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_pos();
        let mut left = self.parse_equality_expression()?;

        while self.match_token(Token::And) {
            let right = self.parse_equality_expression()?;
            let end = self.current_pos();
            left = Expression::Binary {
                left: Box::new(left),
                operator: BinaryOp::And,
                right: Box::new(right),
                span: Span::new(start, end, self.id_generator.new_id()),
            };
        }

        Ok(left)
    }

    /// Parse equality expression (== !=)
    fn parse_equality_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_pos();
        let mut left = self.parse_comparison_expression()?;

        while let Some(op) = self.match_equality_op() {
            let right = self.parse_comparison_expression()?;
            let end = self.current_pos();
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                span: Span::new(start, end, self.id_generator.new_id()),
            };
        }

        Ok(left)
    }

    /// Parse comparison expression (< > <= >=)
    fn parse_comparison_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_pos();
        let mut left = self.parse_additive_expression()?;

        while let Some(op) = self.match_comparison_op() {
            let right = self.parse_additive_expression()?;
            let end = self.current_pos();
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                span: Span::new(start, end, self.id_generator.new_id()),
            };
        }

        Ok(left)
    }

    /// Parse additive expression (+ -)
    fn parse_additive_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_pos();
        let mut left = self.parse_multiplicative_expression()?;

        while let Some(op) = self.match_additive_op() {
            let right = self.parse_multiplicative_expression()?;
            let end = self.current_pos();
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                span: Span::new(start, end, self.id_generator.new_id()),
            };
        }

        Ok(left)
    }

    /// Parse multiplicative expression (* /)
    fn parse_multiplicative_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_pos();
        let mut left = self.parse_primary_expression()?;

        while let Some(op) = self.match_multiplicative_op() {
            let right = self.parse_primary_expression()?;
            let end = self.current_pos();
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                span: Span::new(start, end, self.id_generator.new_id()),
            };
        }

        Ok(left)
    }

    /// Parse primary expression (literals, variables, member access, function calls)
    fn parse_primary_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_pos();

        match self.peek() {
            Some((Token::String(s), _)) => {
                let string_val = s.to_string();
                self.advance();
                let end = self.current_pos();

                // Check if this is a template string (contains ${...})
                if string_val.contains("${") {
                    self.parse_template_string(string_val, start, end)
                } else {
                    // Regular string literal
                    let val = string_val.trim_matches('"').to_string();
                    Ok(Expression::Literal {
                        value: val,
                        span: Span::new(start, end, self.id_generator.new_id()),
                    })
                }
            }
            Some((Token::Number(n), _)) => {
                let val = n.parse::<f64>().unwrap_or(0.0);
                self.advance();
                let end = self.current_pos();
                Ok(Expression::Number {
                    value: val,
                    span: Span::new(start, end, self.id_generator.new_id()),
                })
            }
            Some((Token::Ident(i), _)) => {
                let name = i.to_string();
                self.advance();

                // Check for function call
                if self.check(Token::LParen) {
                    return self.parse_function_call(name, start);
                }

                // Start with variable
                let expr = Expression::Variable {
                    name,
                    span: Span::new(start, self.current_pos(), self.id_generator.new_id()),
                };

                // Parse any postfix operations (member access, method calls)
                self.parse_postfix_operations(expr, start)
            }
            Some((Token::LParen, _)) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some((Token::LBrace, _)) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RBrace)?;
                Ok(expr)
            }
            _ => {
                // Default to empty literal for cases where expression is optional
                Ok(Expression::Literal {
                    value: String::new(),
                    span: Span::new(start, start, self.id_generator.new_id()),
                })
            }
        }
    }

    /// Parse function call: functionName(arg1, arg2, ...)
    fn parse_function_call(&mut self, function: String, start: usize) -> ParseResult<Expression> {
        self.expect(Token::LParen)?;

        let mut arguments = Vec::new();
        while !self.check(Token::RParen) && !self.is_at_end() {
            arguments.push(self.parse_expression()?);

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RParen)?;

        let mut expr = Expression::Call {
            function,
            arguments,
            span: Span::new(start, self.current_pos(), self.id_generator.new_id()),
        };

        // Check for member access and additional function calls: getUser().name.toUpperCase()
        expr = self.parse_postfix_operations(expr, start)?;

        Ok(expr)
    }

    /// Parse postfix operations (member access and method calls)
    /// Handles: obj.prop, obj.method(), obj.prop.method(), etc.
    fn parse_postfix_operations(
        &mut self,
        mut expr: Expression,
        start: usize,
    ) -> ParseResult<Expression> {
        loop {
            if self.match_token(Token::Dot) {
                let property = self.expect_ident()?;

                // Check if this is a method call
                if self.match_token(Token::LParen) {
                    // It's a method call like .toUpperCase()
                    let mut arguments = Vec::new();
                    while !self.check(Token::RParen) && !self.is_at_end() {
                        arguments.push(self.parse_expression()?);

                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;

                    // Create member access to the method, then wrap in call
                    let method_ref = Expression::Member {
                        object: Box::new(expr),
                        property,
                        span: Span::new(start, self.current_pos(), self.id_generator.new_id()),
                    };

                    expr = Expression::Call {
                        function: format!("method_{}", self.id_generator.new_id()),
                        arguments: vec![method_ref],
                        span: Span::new(start, self.current_pos(), self.id_generator.new_id()),
                    };
                } else {
                    // Just member access
                    let end = self.current_pos();
                    expr = Expression::Member {
                        object: Box::new(expr),
                        property,
                        span: Span::new(start, end, self.id_generator.new_id()),
                    };
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse template string with ${...} interpolation
    fn parse_template_string(
        &mut self,
        string_val: String,
        start: usize,
        end: usize,
    ) -> ParseResult<Expression> {
        let mut parts = Vec::new();
        let content = string_val.trim_matches('"');
        let mut current = String::new();
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // consume '{'

                // Save literal part before interpolation
                if !current.is_empty() {
                    parts.push(TemplatePart::Literal(current.clone()));
                    current.clear();
                }

                // Extract expression content
                let mut expr_str = String::new();
                let mut depth = 1;
                while depth > 0 {
                    match chars.next() {
                        Some('{') => {
                            depth += 1;
                            expr_str.push('{');
                        }
                        Some('}') => {
                            depth -= 1;
                            if depth > 0 {
                                expr_str.push('}');
                            }
                        }
                        Some(c) => expr_str.push(c),
                        None => break,
                    }
                }

                // Parse the expression
                let mut sub_parser = Parser::new(&expr_str, self.id_generator.clone());
                let expr = sub_parser.parse_expression()?;
                parts.push(TemplatePart::Expression(expr));
            } else if ch == '\\' && chars.peek().is_some() {
                // Handle escape sequences
                let next = chars.next().unwrap();
                match next {
                    'n' => current.push('\n'),
                    't' => current.push('\t'),
                    'r' => current.push('\r'),
                    '"' => current.push('"'),
                    '\\' => current.push('\\'),
                    _ => {
                        current.push('\\');
                        current.push(next);
                    }
                }
            } else {
                current.push(ch);
            }
        }

        // Save remaining literal part
        if !current.is_empty() {
            parts.push(TemplatePart::Literal(current));
        }

        Ok(Expression::Template {
            parts,
            span: Span::new(start, end, self.id_generator.new_id()),
        })
    }

    // Helper methods for matching operators

    fn match_equality_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(Token::EqualsEquals) {
            Some(BinaryOp::Equals)
        } else if self.match_token(Token::NotEquals) {
            Some(BinaryOp::NotEquals)
        } else {
            None
        }
    }

    fn match_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(Token::LessThanEquals) {
            Some(BinaryOp::LessThanOrEqual)
        } else if self.match_token(Token::GreaterThanEquals) {
            Some(BinaryOp::GreaterThanOrEqual)
        } else if self.match_token(Token::LAngle) {
            Some(BinaryOp::LessThan)
        } else if self.match_token(Token::RAngle) {
            Some(BinaryOp::GreaterThan)
        } else {
            None
        }
    }

    fn match_additive_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(Token::Plus) {
            Some(BinaryOp::Add)
        } else if self.match_token(Token::Minus) {
            Some(BinaryOp::Subtract)
        } else {
            None
        }
    }

    fn match_multiplicative_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(Token::Star) {
            Some(BinaryOp::Multiply)
        } else if self.match_token(Token::Slash) {
            Some(BinaryOp::Divide)
        } else {
            None
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
            Err(ParseError::unexpected_token_span(
                self.peek_span(),
                Self::format_expected_token(&token),
                Self::format_token(self.peek()),
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
            _ => Err(ParseError::unexpected_token_span(
                self.peek_span(),
                "identifier",
                Self::format_token(self.peek()),
            )),
        }
    }

    /// Accept either an identifier or an element keyword (div, span, button, img)
    /// Used for override paths where we need to target HTML elements
    fn expect_ident_or_element_keyword(&mut self) -> ParseResult<String> {
        match self.peek() {
            Some((Token::Ident(s), _)) => {
                let val = s.to_string();
                self.advance();
                Ok(val)
            }
            Some((Token::Div, _)) => {
                self.advance();
                Ok("div".to_string())
            }
            Some((Token::Span, _)) => {
                self.advance();
                Ok("span".to_string())
            }
            Some((Token::Button, _)) => {
                self.advance();
                Ok("button".to_string())
            }
            Some((Token::Img, _)) => {
                self.advance();
                Ok("img".to_string())
            }
            _ => Err(ParseError::unexpected_token_span(
                self.peek_span(),
                "identifier",
                Self::format_token(self.peek()),
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
            _ => Err(ParseError::unexpected_token_span(
                self.peek_span(),
                "string literal",
                Self::format_token(self.peek()),
            )),
        }
    }

    /// Parse a potentially namespaced reference like "theme.fontRegular"
    fn parse_namespaced_ref(&mut self) -> ParseResult<String> {
        let first = self.expect_ident()?;

        // Check for dot notation (namespace.name)
        if self.match_token(Token::Dot) {
            let second = self.expect_ident()?;
            Ok(format!("{}.{}", first, second))
        } else {
            Ok(first)
        }
    }

    fn current_pos(&self) -> usize {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map(|(_, span)| span.start)
            .unwrap_or(0)
    }

    /// Get the span of the current token (the one we just consumed)
    fn current_span(&self) -> std::ops::Range<usize> {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map(|(_, span)| span.clone())
            .unwrap_or(0..0)
    }

    /// Get the span of the next token (the one we're about to consume)
    fn peek_span(&self) -> std::ops::Range<usize> {
        self.tokens
            .get(self.pos)
            .map(|(_, span)| span.clone())
            .unwrap_or_else(|| {
                // If we're at EOF, use the end of the last token
                let end = self.tokens.last().map(|(_, span)| span.end).unwrap_or(0);
                end..end
            })
    }

    /// Format a token for display in error messages
    fn format_token(token: Option<&(Token, std::ops::Range<usize>)>) -> String {
        match token {
            None => "end of file".to_string(),
            Some((Token::Ident(s), _)) => format!("identifier '{}'", s),
            Some((Token::String(s), _)) => format!("string {}", s),
            Some((Token::Number(n), _)) => format!("number {}", n),
            Some((Token::LBrace, _)) => "'{'".to_string(),
            Some((Token::RBrace, _)) => "'}'".to_string(),
            Some((Token::LParen, _)) => "'('".to_string(),
            Some((Token::RParen, _)) => "')'".to_string(),
            Some((Token::Colon, _)) => "':'".to_string(),
            Some((Token::Semicolon, _)) => "';'".to_string(),
            Some((Token::Dot, _)) => "'.'".to_string(),
            Some((Token::Comma, _)) => "','".to_string(),
            Some((Token::Slash, _)) => "'/'".to_string(),
            Some((Token::Component, _)) => "keyword 'component'".to_string(),
            Some((Token::Style, _)) => "keyword 'style'".to_string(),
            Some((Token::TokenKeyword, _)) => "keyword 'token'".to_string(),
            Some((Token::Render, _)) => "keyword 'render'".to_string(),
            Some((Token::Public, _)) => "keyword 'public'".to_string(),
            Some((Token::Import, _)) => "keyword 'import'".to_string(),
            Some((Token::As, _)) => "keyword 'as'".to_string(),
            Some((Token::Variant, _)) => "keyword 'variant'".to_string(),
            Some((Token::Slot, _)) => "keyword 'slot'".to_string(),
            Some((Token::Text, _)) => "keyword 'text'".to_string(),
            Some((token, _)) => format!("{:?}", token),
        }
    }

    fn format_expected_token(token: &Token) -> String {
        match token {
            Token::LBrace => "'{'".to_string(),
            Token::RBrace => "'}'".to_string(),
            Token::LParen => "'('".to_string(),
            Token::RParen => "')'".to_string(),
            Token::Colon => "':'".to_string(),
            Token::Semicolon => "';'".to_string(),
            Token::Component => "keyword 'component'".to_string(),
            Token::Style => "keyword 'style'".to_string(),
            Token::Render => "keyword 'render'".to_string(),
            Token::Public => "keyword 'public'".to_string(),
            token => format!("{:?}", token),
        }
    }
}

pub fn parse(source: &str) -> ParseResult<Document> {
    parse_with_path(source, "<anonymous>")
}

pub fn parse_with_path(source: &str, path: &str) -> ParseResult<Document> {
    let id_generator = IDGenerator::new(path);
    let mut parser = Parser::new(source, id_generator);
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

    #[test]
    fn test_parse_override_simple() {
        let source = r#"
            component Card {
                render Button {}

                override Button {
                    style {
                        color: red
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.components.len(), 1);
        assert_eq!(doc.components[0].overrides.len(), 1);

        let override_def = &doc.components[0].overrides[0];
        assert_eq!(override_def.path, vec!["Button"]);
        assert_eq!(override_def.styles.len(), 1);
    }

    #[test]
    fn test_parse_override_deep_path() {
        let source = r#"
            component Page {
                render Card {}

                override Card.Button.Icon {
                    style {
                        fill: blue
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        let override_def = &doc.components[0].overrides[0];
        assert_eq!(override_def.path, vec!["Card", "Button", "Icon"]);
    }

    #[test]
    fn test_parse_override_with_attributes() {
        let source = r#"
            component Card {
                render Button {}

                override Button(id="custom-btn", class="primary") {
                    style {
                        color: red
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        let override_def = &doc.components[0].overrides[0];
        assert_eq!(override_def.attributes.len(), 2);
        assert!(override_def.attributes.contains_key("id"));
        assert!(override_def.attributes.contains_key("class"));
    }
}
