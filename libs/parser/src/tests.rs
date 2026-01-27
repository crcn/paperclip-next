//! Parser tests

use crate::parse;
use paperclip_proto::ast::*;

#[test]
fn test_parse_full_example() {
    let source = r#"
import "./tokens.pc" as tokens

public token spacing 16px

public style buttonBase {
    padding: 8px 16px
    border-radius: 4px
}

/** @frame(x: 0, y: 0, width: 200, height: 100) */
public component Button {
    variant hover trigger { ":hover" }
    variant disabled

    render button {
        class="btn"
        style extends buttonBase {
            background: var(tokens.primary)
            color: #fff
        }
        style variant hover {
            background: var(tokens.primaryHover)
        }
        slot children {
            text "Click me"
        }
    }
}

component Card {
    render div {
        style {
            display: flex
            flex-direction: column
            gap: 16px
        }
        slot header
        slot children
    }
}
"#;

    let doc = parse(source).expect("Should parse successfully");
    
    // Should have: 1 import, 1 token, 1 style, 2 components
    assert_eq!(doc.items.len(), 5);
    
    // Check import
    match &doc.items[0].node {
        Item::Import(import) => {
            assert_eq!(import.path.node, "./tokens.pc");
            assert_eq!(import.alias.as_ref().unwrap().node, "tokens");
        }
        _ => panic!("Expected import"),
    }
    
    // Check token
    match &doc.items[1].node {
        Item::Token(token) => {
            assert!(token.is_public);
            assert_eq!(token.name.node, "spacing");
        }
        _ => panic!("Expected token"),
    }
    
    // Check style
    match &doc.items[2].node {
        Item::Style(style) => {
            assert!(style.is_public);
            assert_eq!(style.name.node, "buttonBase");
            assert_eq!(style.declarations.len(), 2);
        }
        _ => panic!("Expected style"),
    }
    
    // Check Button component
    match &doc.items[3].node {
        Item::Component(comp) => {
            assert!(comp.is_public);
            assert_eq!(comp.name.node, "Button");
            assert_eq!(comp.variants.len(), 2);
            
            // Check frame annotation
            let frame = comp.doc_comment.as_ref().unwrap().frame.as_ref().unwrap();
            assert_eq!(frame.x, 0.0);
            assert_eq!(frame.width, 200.0);
        }
        _ => panic!("Expected component"),
    }
}

#[test]
fn test_parse_expressions() {
    let source = r#"
component Test {
    render div {
        if items.length > 0 {
            text "Has items"
        }
        repeat items as item, index {
            text "Item"
        } empty {
            text "No items"
        }
    }
}
"#;

    let doc = parse(source).expect("Should parse");
    assert_eq!(doc.items.len(), 1);
}

#[test]
fn test_parse_component_instance() {
    let source = r#"
component Page {
    render div {
        Header(title="Welcome")
        Button(onClick={handleClick}, disabled=true)
    }
}
"#;

    let doc = parse(source).expect("Should parse");
    
    match &doc.items[0].node {
        Item::Component(comp) => {
            if let Some(render) = &comp.render {
                if let RenderNode::Element(el) = &render.node {
                    assert_eq!(el.children.len(), 2);
                    
                    // First child should be Header instance
                    match &el.children[0].node {
                        RenderNode::ComponentInstance(inst) => {
                            assert_eq!(inst.name.node, "Header");
                            assert_eq!(inst.props.len(), 1);
                        }
                        _ => panic!("Expected ComponentInstance"),
                    }
                }
            }
        }
        _ => panic!("Expected component"),
    }
}

#[test]
fn test_parse_nested_styles() {
    let source = r#"
component Card {
    render div {
        style {
            display: flex
            padding: 16px
            background: rgb(255, 255, 255)
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1)
        }
    }
}
"#;

    let doc = parse(source).expect("Should parse");
    
    match &doc.items[0].node {
        Item::Component(comp) => {
            if let Some(render) = &comp.render {
                if let RenderNode::Element(el) = &render.node {
                    assert_eq!(el.styles.len(), 1);
                    let style = &el.styles[0].node;
                    assert_eq!(style.declarations.len(), 4);
                }
            }
        }
        _ => panic!("Expected component"),
    }
}
