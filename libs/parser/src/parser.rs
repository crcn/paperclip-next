//! Chumsky-based parser for Paperclip .pc files
//!
//! Uses parser combinators for:
//! - Composable grammar rules
//! - Excellent error recovery
//! - Automatic span tracking

use chumsky::prelude::*;
use paperclip_proto::ast::{
    self, Attribute, AttributeValue, BinaryOperator, Component, ComponentInstance,
    Condition, DocComment, Document, Element, Expression, FrameAnnotation, Import,
    Insert, Item, Prop, Repeat, RenderNode, SampleData, Slot, Span, Spanned,
    StyleBlock, StyleDeclaration, StyleDefinition, StyleValue, TextNode, TextPart,
    Token as AstToken, TokenValue, UnaryOperator, Variant,
};

use crate::lexer::{lex, Token, TokenSpan};
use crate::error::{ParseError, ParseErrors};

/// Parse a .pc file into an AST Document
pub fn parse(source: &str) -> Result<Document, ParseErrors> {
    // First, lex the source into tokens
    let tokens: Vec<_> = lex(source)
        .filter_map(|result| {
            match result {
                Ok(spanned) => {
                    // Skip comments for parsing (but we could preserve them for roundtrip)
                    match &spanned.token {
                        Token::LineComment(_) | Token::BlockComment(_) => None,
                        _ => Some((spanned.token, spanned.span)),
                    }
                }
                Err(_) => None, // TODO: collect lex errors
            }
        })
        .collect();

    // Create the parser
    let parser = document_parser();

    // Parse with recovery
    let (document, errors) = parser.parse_recovery(TokenStream::new(&tokens, source.len()));

    if !errors.is_empty() {
        let mut parse_errors = ParseErrors::new();
        for err in errors {
            parse_errors.push(convert_chumsky_error(err));
        }
        if document.is_none() {
            return Err(parse_errors);
        }
        // If we have a partial result, we could return it with warnings
    }

    Ok(document.unwrap_or_else(|| Document { items: vec![] }))
}

/// Token stream wrapper for chumsky
struct TokenStream<'src> {
    tokens: &'src [(Token<'src>, TokenSpan)],
    eoi: TokenSpan,
}

impl<'src> TokenStream<'src> {
    fn new(tokens: &'src [(Token<'src>, TokenSpan)], source_len: usize) -> Self {
        Self {
            tokens,
            eoi: TokenSpan {
                start: source_len,
                end: source_len,
            },
        }
    }
}

type ParserInput<'src> = chumsky::Stream<
    'src,
    Token<'src>,
    TokenSpan,
    std::iter::Map<
        std::slice::Iter<'src, (Token<'src>, TokenSpan)>,
        fn(&'src (Token<'src>, TokenSpan)) -> (Token<'src>, TokenSpan),
    >,
>;

fn convert_chumsky_error<'src>(
    err: Simple<Token<'src>, TokenSpan>,
) -> ParseError {
    let span = err.span();
    let expected: Vec<_> = err
        .expected()
        .filter_map(|e| e.as_ref().map(|t| format!("{:?}", t)))
        .collect();
    let found = err.found().map(|t| format!("{:?}", t)).unwrap_or_else(|| "end of input".to_string());

    ParseError::UnexpectedToken {
        span,
        expected: if expected.is_empty() {
            "something else".to_string()
        } else {
            expected.join(" or ")
        },
        found,
    }
}

/// Main document parser
fn document_parser<'src>() -> impl Parser<Token<'src>, Document, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    item_parser()
        .repeated()
        .map(|items| Document { items })
}

/// Parse top-level items
fn item_parser<'src>() -> impl Parser<Token<'src>, Spanned<Item>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    choice((
        import_parser().map(Item::Import),
        token_parser().map(Item::Token),
        style_def_parser().map(Item::Style),
        component_parser().map(Item::Component),
    ))
    .map_with_span(|item, span| Spanned::new(item, to_span(span)))
}

/// Parse import statement
fn import_parser<'src>() -> impl Parser<Token<'src>, Import, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    just(Token::Import)
        .ignore_then(string_parser())
        .then(
            just(Token::As)
                .ignore_then(ident_parser())
                .or_not()
        )
        .map(|(path, alias)| Import { path, alias })
}

/// Parse token definition
fn token_parser<'src>() -> impl Parser<Token<'src>, Token_, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    let is_public = just(Token::Public).or_not().map(|p| p.is_some());

    is_public
        .then_ignore(just(Token::TokenKw))
        .then(ident_parser())
        .then(token_value_parser())
        .map(|((is_public, name), value)| Token_ {
            is_public,
            name,
            value,
        })
}

// Type alias to avoid conflict with Token enum
type Token_ = ast::Token;

/// Parse token value
fn token_value_parser<'src>() -> impl Parser<Token<'src>, Spanned<TokenValue>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    choice((
        select! { Token::Color(c) => TokenValue::Color(c.to_string()) },
        select! { Token::Dimension(d) => {
            // Parse dimension into value and unit
            let (num, unit) = parse_dimension(d);
            TokenValue::Dimension(num, unit)
        }},
        select! { Token::Number(n) => TokenValue::Number(n.parse().unwrap_or(0.0)) },
        select! { Token::String(s) => TokenValue::String(s.to_string()) },
        select! { Token::SingleQuoteString(s) => TokenValue::String(s.to_string()) },
    ))
    .map_with_span(|v, span| Spanned::new(v, to_span(span)))
}

fn parse_dimension(s: &str) -> (f64, String) {
    let num_end = s.find(|c: char| c.is_alphabetic() || c == '%').unwrap_or(s.len());
    let num: f64 = s[..num_end].parse().unwrap_or(0.0);
    let unit = s[num_end..].to_string();
    (num, unit)
}

/// Parse style definition
fn style_def_parser<'src>() -> impl Parser<Token<'src>, StyleDefinition, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    let is_public = just(Token::Public).or_not().map(|p| p.is_some());

    is_public
        .then_ignore(just(Token::Style))
        .then(ident_parser())
        .then(
            style_declaration_parser()
                .repeated()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .map(|((is_public, name), declarations)| StyleDefinition {
            is_public,
            name,
            declarations,
        })
}

/// Parse style declaration
fn style_declaration_parser<'src>() -> impl Parser<Token<'src>, Spanned<StyleDeclaration>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    ident_parser()
        .then_ignore(just(Token::Colon))
        .then(style_value_parser())
        .then_ignore(just(Token::Semi).or_not()) // Optional semicolon
        .map_with_span(|(property, value), span| {
            Spanned::new(
                StyleDeclaration { property, value },
                to_span(span),
            )
        })
}

/// Parse style value
fn style_value_parser<'src>() -> impl Parser<Token<'src>, Spanned<StyleValue>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    recursive(|value| {
        let simple_value = choice((
            select! { Token::Color(c) => StyleValue::Color(c.to_string()) },
            select! { Token::Dimension(d) => {
                let (num, unit) = parse_dimension(d);
                StyleValue::Dimension(num, unit)
            }},
            select! { Token::Number(n) => StyleValue::Number(n.parse().unwrap_or(0.0)) },
            select! { Token::String(s) => StyleValue::String(s.to_string()) },
            select! { Token::SingleQuoteString(s) => StyleValue::String(s.to_string()) },
            select! { Token::Ident(i) => StyleValue::Keyword(i.to_string()) },
        ));

        // Function call like var(tokenName) or rgb(255, 0, 0)
        let func_call = select! { Token::Ident(name) => name.to_string() }
            .then(
                value
                    .clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LParen), just(Token::RParen))
            )
            .map(|(name, args)| {
                // Special case for var() references
                if name == "var" && args.len() == 1 {
                    if let StyleValue::Keyword(ref_name) = &args[0].node {
                        return StyleValue::Reference(ref_name.clone());
                    }
                }
                StyleValue::Function(name, args.into_iter().map(|a| a.node).collect())
            });

        choice((func_call, simple_value))
            .map_with_span(|v, span| Spanned::new(v, to_span(span)))
    })
}

/// Parse component definition
fn component_parser<'src>() -> impl Parser<Token<'src>, Component, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    let doc_comment = select! { Token::DocComment(c) => parse_doc_comment(c) }.or_not();
    let is_public = just(Token::Public).or_not().map(|p| p.is_some());

    doc_comment
        .then(is_public)
        .then_ignore(just(Token::Component))
        .then(ident_parser())
        .then(
            component_body_parser()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .map(|(((doc_comment, is_public), name), (variants, render))| Component {
            is_public,
            name,
            doc_comment,
            variants,
            render,
        })
}

fn parse_doc_comment(content: &str) -> DocComment {
    // Strip /** and */
    let inner = content
        .trim_start_matches("/**")
        .trim_end_matches("*/")
        .trim();

    // Parse @frame annotation if present
    let frame = parse_frame_annotation(inner);
    let samples = parse_sample_annotations(inner);

    DocComment {
        content: inner.to_string(),
        frame,
        samples,
    }
}

fn parse_frame_annotation(content: &str) -> Option<FrameAnnotation> {
    // Look for @frame(x: 100, y: 50, width: 320, height: 480)
    let frame_start = content.find("@frame(")?;
    let frame_end = content[frame_start..].find(')')? + frame_start;
    let params = &content[frame_start + 7..frame_end];

    let mut x = 0.0;
    let mut y = 0.0;
    let mut width = 320.0;
    let mut height = None;

    for part in params.split(',') {
        let kv: Vec<_> = part.split(':').map(|s| s.trim()).collect();
        if kv.len() == 2 {
            let value: f64 = kv[1].parse().unwrap_or(0.0);
            match kv[0] {
                "x" => x = value,
                "y" => y = value,
                "width" => width = value,
                "height" => height = Some(value),
                _ => {}
            }
        }
    }

    Some(FrameAnnotation { x, y, width, height })
}

fn parse_sample_annotations(content: &str) -> Vec<SampleData> {
    // TODO: Parse @sample annotations
    vec![]
}

/// Parse component body (variants and render)
fn component_body_parser<'src>() -> impl Parser<Token<'src>, (Vec<Spanned<Variant>>, Option<Spanned<RenderNode>>), Error = Simple<Token<'src>, TokenSpan>> + Clone {
    let variant = variant_parser().map_with_span(|v, span| Spanned::new(v, to_span(span)));
    let render = render_parser().map_with_span(|r, span| Spanned::new(r, to_span(span)));

    variant
        .repeated()
        .then(render.or_not())
        .map(|(variants, render)| (variants, render))
}

/// Parse variant definition
fn variant_parser<'src>() -> impl Parser<Token<'src>, Variant, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    just(Token::Variant)
        .ignore_then(ident_parser())
        .then(
            just(Token::Trigger)
                .ignore_then(
                    string_parser()
                        .separated_by(just(Token::Comma))
                        .delimited_by(just(Token::LBrace), just(Token::RBrace))
                )
                .or_not()
        )
        .map(|(name, triggers)| Variant {
            name,
            triggers: triggers.unwrap_or_default(),
        })
}

/// Parse render block
fn render_parser<'src>() -> impl Parser<Token<'src>, RenderNode, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    just(Token::Render)
        .ignore_then(render_node_parser())
}

/// Parse a render node (element, text, slot, etc.)
fn render_node_parser<'src>() -> impl Parser<Token<'src>, RenderNode, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    recursive(|node| {
        let element = element_parser(node.clone());
        let text_node = text_node_parser();
        let slot = slot_parser(node.clone());
        let insert = insert_parser(node.clone());
        let condition = condition_parser(node.clone());
        let repeat = repeat_parser(node.clone());
        let component_instance = component_instance_parser(node.clone());

        choice((
            text_node.map(RenderNode::Text),
            slot.map(RenderNode::Slot),
            insert.map(RenderNode::Insert),
            condition.map(RenderNode::Condition),
            repeat.map(RenderNode::Repeat),
            // Try component instance before element (component names are capitalized)
            component_instance.map(RenderNode::ComponentInstance),
            element.map(RenderNode::Element),
        ))
    })
}

/// Parse element: `div { ... }`
fn element_parser<'src>(
    child: impl Parser<Token<'src>, RenderNode, Error = Simple<Token<'src>, TokenSpan>> + Clone,
) -> impl Parser<Token<'src>, Element, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    let tag = select! { Token::Ident(i) => i.to_string() }
        .map_with_span(|t, span| Spanned::new(t, to_span(span)));

    let attribute = attribute_parser();
    let style_block = style_block_parser();
    let child_node = child.map_with_span(|n, span| Spanned::new(n, to_span(span)));

    tag.then(
        choice((
            attribute.map(ElementPart::Attribute),
            style_block.map(ElementPart::Style),
            child_node.map(ElementPart::Child),
        ))
        .repeated()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .or_not()
    )
    .map(|(tag, parts)| {
        let parts = parts.unwrap_or_default();
        let mut attributes = vec![];
        let mut styles = vec![];
        let mut children = vec![];

        for part in parts {
            match part {
                ElementPart::Attribute(a) => attributes.push(a),
                ElementPart::Style(s) => styles.push(s),
                ElementPart::Child(c) => children.push(c),
            }
        }

        Element {
            tag,
            attributes,
            styles,
            children,
        }
    })
}

enum ElementPart {
    Attribute(Spanned<Attribute>),
    Style(Spanned<StyleBlock>),
    Child(Spanned<RenderNode>),
}

/// Parse attribute: `class="primary"` or `onClick={handler}`
fn attribute_parser<'src>() -> impl Parser<Token<'src>, Spanned<Attribute>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    ident_parser()
        .then_ignore(just(Token::Eq))
        .then(attribute_value_parser())
        .map_with_span(|(name, value), span| {
            Spanned::new(Attribute { name, value }, to_span(span))
        })
}

fn attribute_value_parser<'src>() -> impl Parser<Token<'src>, Spanned<AttributeValue>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    choice((
        string_parser().map(|s| AttributeValue::String(s.node)),
        expression_parser()
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .map(|e| AttributeValue::Expression(e.node)),
        just(Token::True).to(AttributeValue::Boolean(true)),
        just(Token::False).to(AttributeValue::Boolean(false)),
    ))
    .map_with_span(|v, span| Spanned::new(v, to_span(span)))
}

/// Parse style block within element
fn style_block_parser<'src>() -> impl Parser<Token<'src>, Spanned<StyleBlock>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    let extends = just(Token::Extends)
        .ignore_then(
            ident_parser()
                .separated_by(just(Token::Comma))
        )
        .or_not()
        .map(|e| e.unwrap_or_default());

    let variant = just(Token::Variant)
        .ignore_then(ident_parser())
        .or_not();

    just(Token::Style)
        .ignore_then(extends)
        .then(variant)
        .then(
            style_declaration_parser()
                .repeated()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .map_with_span(|((extends, variant), declarations), span| {
            Spanned::new(
                StyleBlock {
                    extends,
                    variant,
                    declarations,
                },
                to_span(span),
            )
        })
}

/// Parse text node
fn text_node_parser<'src>() -> impl Parser<Token<'src>, TextNode, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    just(Token::Text)
        .ignore_then(text_content_parser())
        .map(|parts| TextNode { parts })
}

fn text_content_parser<'src>() -> impl Parser<Token<'src>, Vec<TextPart>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    // For now, just handle simple strings. TODO: interpolation
    string_parser()
        .map(|s| vec![TextPart::Literal(s.node)])
}

/// Parse slot definition
fn slot_parser<'src>(
    child: impl Parser<Token<'src>, RenderNode, Error = Simple<Token<'src>, TokenSpan>> + Clone,
) -> impl Parser<Token<'src>, Slot, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    just(Token::Slot)
        .ignore_then(ident_parser())
        .then(
            child
                .map_with_span(|n, span| Spanned::new(n, to_span(span)))
                .repeated()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
                .or_not()
        )
        .map(|(name, default)| Slot {
            name,
            default: default.unwrap_or_default(),
        })
}

/// Parse insert block
fn insert_parser<'src>(
    child: impl Parser<Token<'src>, RenderNode, Error = Simple<Token<'src>, TokenSpan>> + Clone,
) -> impl Parser<Token<'src>, Insert, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    just(Token::InsertKw)
        .ignore_then(ident_parser())
        .then(
            child
                .map_with_span(|n, span| Spanned::new(n, to_span(span)))
                .repeated()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .map(|(slot_name, children)| Insert { slot_name, children })
}

/// Parse condition: `if expr { ... } else { ... }`
fn condition_parser<'src>(
    child: impl Parser<Token<'src>, RenderNode, Error = Simple<Token<'src>, TokenSpan>> + Clone,
) -> impl Parser<Token<'src>, Condition, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    let then_branch = child
        .clone()
        .map_with_span(|n, span| Spanned::new(n, to_span(span)))
        .repeated()
        .delimited_by(just(Token::LBrace), just(Token::RBrace));

    let else_branch = just(Token::Else)
        .ignore_then(
            child
                .map_with_span(|n, span| Spanned::new(n, to_span(span)))
                .repeated()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .or_not();

    just(Token::If)
        .ignore_then(expression_parser())
        .then(then_branch)
        .then(else_branch)
        .map(|((condition, then_branch), else_branch)| Condition {
            condition,
            then_branch,
            else_branch,
        })
}

/// Parse repeat: `repeat items as item { ... }`
fn repeat_parser<'src>(
    child: impl Parser<Token<'src>, RenderNode, Error = Simple<Token<'src>, TokenSpan>> + Clone,
) -> impl Parser<Token<'src>, Repeat, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    let body = child
        .clone()
        .map_with_span(|n, span| Spanned::new(n, to_span(span)))
        .repeated()
        .delimited_by(just(Token::LBrace), just(Token::RBrace));

    let empty_block = just(Token::Empty)
        .ignore_then(
            child
                .map_with_span(|n, span| Spanned::new(n, to_span(span)))
                .repeated()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .or_not();

    just(Token::Repeat)
        .ignore_then(expression_parser())
        .then_ignore(just(Token::As))
        .then(ident_parser())
        .then(
            just(Token::Comma)
                .ignore_then(ident_parser())
                .or_not()
        )
        .then(body)
        .then(empty_block)
        .map(|((((source, iterator), index), body), empty)| Repeat {
            source,
            iterator,
            index,
            body,
            empty,
        })
}

/// Parse component instance: `Button(label="Save")`
fn component_instance_parser<'src>(
    child: impl Parser<Token<'src>, RenderNode, Error = Simple<Token<'src>, TokenSpan>> + Clone,
) -> impl Parser<Token<'src>, ComponentInstance, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    // Component names start with uppercase
    let component_name = select! {
        Token::Ident(i) if i.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) => i.to_string()
    }.map_with_span(|n, span| Spanned::new(n, to_span(span)));

    let namespace = ident_parser()
        .then_ignore(just(Token::Dot))
        .or_not();

    let prop = ident_parser()
        .then_ignore(just(Token::Eq))
        .then(expression_parser())
        .map_with_span(|(name, value), span| Spanned::new(Prop { name, value }, to_span(span)));

    let props = prop
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .or_not()
        .map(|p| p.unwrap_or_default());

    let inserts = insert_parser(child)
        .map_with_span(|i, span| Spanned::new(i, to_span(span)))
        .repeated()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .or_not()
        .map(|i| i.unwrap_or_default());

    namespace
        .then(component_name)
        .then(props)
        .then(inserts)
        .map(|(((namespace, name), props), inserts)| ComponentInstance {
            name,
            namespace,
            props,
            inserts,
        })
}

/// Parse expression (formula-like)
fn expression_parser<'src>() -> impl Parser<Token<'src>, Spanned<Expression>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    recursive(|expr| {
        let literal = choice((
            select! { Token::Number(n) => Expression::Number(n.parse().unwrap_or(0.0)) },
            select! { Token::String(s) => Expression::String(s.to_string()) },
            select! { Token::SingleQuoteString(s) => Expression::String(s.to_string()) },
            just(Token::True).to(Expression::Boolean(true)),
            just(Token::False).to(Expression::Boolean(false)),
        ));

        let ident = select! { Token::Ident(i) => Expression::Identifier(i.to_string()) };

        // Parenthesized expression
        let paren_expr = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .map(|e| e.node);

        // Primary expression (literals, identifiers, parens)
        let primary = choice((literal, ident, paren_expr))
            .map_with_span(|e, span| Spanned::new(e, to_span(span)));

        // Member access and function calls
        let postfix = primary.then(
            choice((
                just(Token::Dot)
                    .ignore_then(ident_parser())
                    .map(PostfixOp::Member),
                expr
                    .clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .map(PostfixOp::Call),
            ))
            .repeated()
        ).foldl(|left, op| {
            let span = left.span;
            match op {
                PostfixOp::Member(member) => {
                    let new_span = Span::new(span.start, member.span.end);
                    Spanned::new(
                        Expression::MemberAccess(Box::new(left), member),
                        new_span,
                    )
                }
                PostfixOp::Call(args) => {
                    // Function call - extract name from left
                    if let Expression::Identifier(name) = &left.node {
                        Spanned::new(
                            Expression::FunctionCall(
                                Spanned::new(name.clone(), left.span),
                                args,
                            ),
                            span,
                        )
                    } else {
                        // Invalid function call syntax
                        left
                    }
                }
            }
        });

        // Unary operators
        let unary = just(Token::Bang)
            .to(UnaryOperator::Not)
            .or(just(Token::Minus).to(UnaryOperator::Neg))
            .repeated()
            .then(postfix)
            .foldr(|op, right| {
                let span = right.span;
                Spanned::new(
                    Expression::UnaryOp(op, Box::new(right)),
                    span,
                )
            });

        // Binary operators (simplified - just a few for now)
        let product = unary.clone().then(
            choice((
                just(Token::Star).to(BinaryOperator::Mul),
                just(Token::Slash).to(BinaryOperator::Div),
            ))
            .then(unary)
            .repeated()
        ).foldl(|left, (op, right)| {
            let span = Span::new(left.span.start, right.span.end);
            Spanned::new(
                Expression::BinaryOp(Box::new(left), op, Box::new(right)),
                span,
            )
        });

        let sum = product.clone().then(
            choice((
                just(Token::Plus).to(BinaryOperator::Add),
                just(Token::Minus).to(BinaryOperator::Sub),
            ))
            .then(product)
            .repeated()
        ).foldl(|left, (op, right)| {
            let span = Span::new(left.span.start, right.span.end);
            Spanned::new(
                Expression::BinaryOp(Box::new(left), op, Box::new(right)),
                span,
            )
        });

        let comparison = sum.clone().then(
            choice((
                just(Token::EqEq).to(BinaryOperator::Eq),
                just(Token::NotEq).to(BinaryOperator::NotEq),
                just(Token::Lte).to(BinaryOperator::Lte),
                just(Token::Lt).to(BinaryOperator::Lt),
                just(Token::Gte).to(BinaryOperator::Gte),
                just(Token::Gt).to(BinaryOperator::Gt),
            ))
            .then(sum)
            .repeated()
        ).foldl(|left, (op, right)| {
            let span = Span::new(left.span.start, right.span.end);
            Spanned::new(
                Expression::BinaryOp(Box::new(left), op, Box::new(right)),
                span,
            )
        });

        let logical = comparison.clone().then(
            choice((
                just(Token::And).to(BinaryOperator::And),
                just(Token::Or).to(BinaryOperator::Or),
            ))
            .then(comparison)
            .repeated()
        ).foldl(|left, (op, right)| {
            let span = Span::new(left.span.start, right.span.end);
            Spanned::new(
                Expression::BinaryOp(Box::new(left), op, Box::new(right)),
                span,
            )
        });

        logical
    })
}

enum PostfixOp {
    Member(Spanned<String>),
    Call(Vec<Spanned<Expression>>),
}

// Helper parsers

fn ident_parser<'src>() -> impl Parser<Token<'src>, Spanned<String>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    select! { Token::Ident(i) => i.to_string() }
        .map_with_span(|i, span| Spanned::new(i, to_span(span)))
}

fn string_parser<'src>() -> impl Parser<Token<'src>, Spanned<String>, Error = Simple<Token<'src>, TokenSpan>> + Clone {
    select! {
        Token::String(s) => s.to_string(),
        Token::SingleQuoteString(s) => s.to_string(),
    }
    .map_with_span(|s, span| Spanned::new(s, to_span(span)))
}

fn to_span(span: TokenSpan) -> Span {
    Span::new(span.start, span.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_import() {
        let source = r#"import "./theme.pc" as theme"#;
        let doc = parse(source).unwrap();
        
        assert_eq!(doc.items.len(), 1);
        match &doc.items[0].node {
            Item::Import(import) => {
                assert_eq!(import.path.node, "./theme.pc");
                assert_eq!(import.alias.as_ref().map(|a| &a.node), Some(&"theme".to_string()));
            }
            _ => panic!("Expected Import"),
        }
    }

    #[test]
    fn test_parse_token() {
        let source = "public token primaryColor #3366FF";
        let doc = parse(source).unwrap();
        
        assert_eq!(doc.items.len(), 1);
        match &doc.items[0].node {
            Item::Token(token) => {
                assert!(token.is_public);
                assert_eq!(token.name.node, "primaryColor");
                assert!(matches!(token.value.node, TokenValue::Color(_)));
            }
            _ => panic!("Expected Token"),
        }
    }

    #[test]
    fn test_parse_simple_component() {
        let source = r#"
public component Button {
    render button {
        text "Click me"
    }
}
"#;
        let doc = parse(source).unwrap();
        
        assert_eq!(doc.items.len(), 1);
        match &doc.items[0].node {
            Item::Component(comp) => {
                assert!(comp.is_public);
                assert_eq!(comp.name.node, "Button");
                assert!(comp.render.is_some());
            }
            _ => panic!("Expected Component"),
        }
    }

    #[test]
    fn test_parse_component_with_frame() {
        let source = r#"
/** @frame(x: 100, y: 50, width: 320, height: 480) */
public component Card {
    render div {
        style {
            padding: 16px
        }
    }
}
"#;
        let doc = parse(source).unwrap();
        
        match &doc.items[0].node {
            Item::Component(comp) => {
                let frame = comp.doc_comment.as_ref().unwrap().frame.as_ref().unwrap();
                assert_eq!(frame.x, 100.0);
                assert_eq!(frame.y, 50.0);
                assert_eq!(frame.width, 320.0);
                assert_eq!(frame.height, Some(480.0));
            }
            _ => panic!("Expected Component"),
        }
    }
}
