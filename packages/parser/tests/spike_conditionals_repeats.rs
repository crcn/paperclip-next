//! # Spike 0.6: Conditional Rendering & Spike 0.7: Repeat/Loop Rendering
//!
//! Validates control flow constructs:
//! - if statements
//! - else branches (if implemented)
//! - repeat item in collection
//! - Nested conditionals and repeats

use paperclip_parser::parse;

// ========== Spike 0.6: Conditional Rendering ==========

#[test]
fn test_conditional_basic() {
    let source = r#"
        component Message {
            render div {
                if isVisible {
                    text "Hello World"
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            assert_eq!(children.len(), 1);

            if let paperclip_parser::ast::Element::Conditional { condition, then_branch, .. } =
                &children[0]
            {
                // Check condition expression
                if let paperclip_parser::ast::Expression::Variable { name, .. } = condition {
                    assert_eq!(name, "isVisible");
                }

                // Check then branch
                assert_eq!(then_branch.len(), 1);
            } else {
                panic!("Expected Conditional element");
            }
        }
    }

    println!("✓ Basic conditional rendering parses correctly");
}

#[test]
fn test_conditional_with_complex_expression() {
    let source = r#"
        component Card {
            render div {
                if isActive && isShown {
                    text "Content visible"
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Conditional { condition, .. } = &children[0] {
                // Should be a binary expression (&&)
                assert!(matches!(
                    condition,
                    paperclip_parser::ast::Expression::Binary { .. }
                ));
            }
        }
    }

    println!("✓ Conditional with complex expression parses correctly");
}

#[test]
fn test_conditional_with_multiple_children() {
    let source = r#"
        component List {
            render div {
                if hasItems {
                    div(class="header") {
                        text "Items"
                    }
                    div(class="content") {
                        text "Content here"
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Conditional { then_branch, .. } = &children[0] {
                // Should have 2 children in then branch
                assert_eq!(then_branch.len(), 2);
            }
        }
    }

    println!("✓ Conditional with multiple children parses correctly");
}

#[test]
fn test_nested_conditionals() {
    let source = r#"
        component Dashboard {
            render div {
                if isLoggedIn {
                    if isPremium {
                        text "Premium Content"
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Conditional { then_branch, .. } = &children[0] {
                // Outer conditional's then branch should contain another conditional
                assert_eq!(then_branch.len(), 1);
                assert!(matches!(
                    then_branch[0],
                    paperclip_parser::ast::Element::Conditional { .. }
                ));
            }
        }
    }

    println!("✓ Nested conditionals parse correctly");
}

// ========== Spike 0.7: Repeat/Loop Rendering ==========

#[test]
fn test_repeat_basic() {
    let source = r#"
        component TodoList {
            render ul {
                repeat todo in todos {
                    li {
                        text todo
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            assert_eq!(children.len(), 1);

            if let paperclip_parser::ast::Element::Repeat { item_name, collection, body, .. } =
                &children[0]
            {
                // Check item name
                assert_eq!(item_name, "todo");

                // Check collection
                if let paperclip_parser::ast::Expression::Variable { name, .. } = collection {
                    assert_eq!(name, "todos");
                }

                // Check body has one element
                assert_eq!(body.len(), 1);
            } else {
                panic!("Expected Repeat element");
            }
        }
    }

    println!("✓ Basic repeat renders correctly");
}

#[test]
fn test_repeat_with_member_access() {
    let source = r#"
        component UserList {
            render div {
                repeat user in users {
                    div(class="user-card") {
                        text user.name
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Repeat { item_name, body, .. } = &children[0] {
                assert_eq!(item_name, "user");
                assert_eq!(body.len(), 1);

                // Check that the text uses member access
                if let paperclip_parser::ast::Element::Tag { children, .. } = &body[0] {
                    if let paperclip_parser::ast::Element::Text { content, .. } = &children[0] {
                        assert!(matches!(
                            content,
                            paperclip_parser::ast::Expression::Member { .. }
                        ));
                    }
                }
            }
        }
    }

    println!("✓ Repeat with member access parses correctly");
}

#[test]
fn test_repeat_with_complex_body() {
    let source = r#"
        component ProductGrid {
            render div(class="grid") {
                repeat product in products {
                    div(class="card") {
                        div(class="image") {
                            text product.name
                        }
                        div(class="price") {
                            text product.price
                        }
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Repeat { body, .. } = &children[0] {
                // Repeat body should have a card with nested structure
                assert_eq!(body.len(), 1);

                if let paperclip_parser::ast::Element::Tag { children, .. } = &body[0] {
                    // Card should have 2 children (image and price divs)
                    assert_eq!(children.len(), 2);
                }
            }
        }
    }

    println!("✓ Repeat with complex body parses correctly");
}

#[test]
fn test_nested_repeats() {
    let source = r#"
        component Matrix {
            render div {
                repeat row in rows {
                    div(class="row") {
                        repeat cell in row {
                            div(class="cell") {
                                text cell
                            }
                        }
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Repeat { item_name, body, .. } = &children[0] {
                assert_eq!(item_name, "row");

                // Outer repeat body should contain a div
                if let paperclip_parser::ast::Element::Tag { children, .. } = &body[0] {
                    // Inner div should contain another repeat
                    assert_eq!(children.len(), 1);
                    assert!(matches!(
                        children[0],
                        paperclip_parser::ast::Element::Repeat { .. }
                    ));
                }
            }
        }
    }

    println!("✓ Nested repeats parse correctly");
}

// ========== Combined: Conditionals + Repeats ==========

#[test]
fn test_conditional_inside_repeat() {
    let source = r#"
        component TaskList {
            render ul {
                repeat task in tasks {
                    if task.isComplete {
                        li(class="completed") {
                            text task.title
                        }
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Repeat { body, .. } = &children[0] {
                // Repeat body should contain a conditional
                assert_eq!(body.len(), 1);
                assert!(matches!(
                    body[0],
                    paperclip_parser::ast::Element::Conditional { .. }
                ));
            }
        }
    }

    println!("✓ Conditional inside repeat parses correctly");
}

#[test]
fn test_repeat_inside_conditional() {
    let source = r#"
        component Inbox {
            render div {
                if hasMessages {
                    ul {
                        repeat message in messages {
                            li {
                                text message.subject
                            }
                        }
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Conditional { then_branch, .. } = &children[0] {
                // Conditional should contain a ul
                if let paperclip_parser::ast::Element::Tag { children, .. } = &then_branch[0] {
                    // ul should contain a repeat
                    assert_eq!(children.len(), 1);
                    assert!(matches!(
                        children[0],
                        paperclip_parser::ast::Element::Repeat { .. }
                    ));
                }
            }
        }
    }

    println!("✓ Repeat inside conditional parses correctly");
}

#[test]
fn test_complex_control_flow() {
    let source = r#"
        component Dashboard {
            render div {
                if isAuthenticated {
                    div(class="content") {
                        repeat section in sections {
                            if section.isVisible {
                                div(class="section") {
                                    text section.title
                                    repeat item in section.items {
                                        div(class="item") {
                                            text item.name
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.components.len(), 1);

    println!("✓ Complex nested control flow parses correctly");
}

#[test]
fn test_repeat_with_component_instances() {
    let source = r#"
        component Gallery {
            render div(class="gallery") {
                repeat photo in photos {
                    PhotoCard(src=photo.url, caption=photo.title)
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Repeat { body, .. } = &children[0] {
                // Repeat body should contain a component instance
                assert_eq!(body.len(), 1);
                assert!(matches!(
                    body[0],
                    paperclip_parser::ast::Element::Instance { .. }
                ));
            }
        }
    }

    println!("✓ Repeat with component instances parses correctly");
}

#[test]
fn test_conditional_with_styles() {
    let source = r#"
        component Banner {
            render div {
                if showBanner {
                    div(class="banner") {
                        style {
                            background: yellow
                            padding: 20px
                        }
                        text "Important message"
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let component = &doc.components[0];

    if let Some(body) = &component.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Conditional { then_branch, .. } = &children[0] {
                // Should have a div with styles
                if let paperclip_parser::ast::Element::Tag { styles, .. } = &then_branch[0] {
                    assert_eq!(styles.len(), 1);
                    assert!(!styles[0].properties.is_empty());
                }
            }
        }
    }

    println!("✓ Conditional with styles parses correctly");
}
