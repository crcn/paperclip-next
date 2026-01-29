//! # Spike 0.3: Component Composition & Slots
//!
//! Validates that component composition works end-to-end:
//! - Component instances
//! - Default slot content
//! - Named slots
//! - Slot content projection
//! - Nested composition

use paperclip_parser::parse;

#[test]
fn test_basic_component_instance() {
    let source = r#"
        component Button {
            render button {
                text "Click me"
            }
        }

        component App {
            render div {
                Button()
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.components.len(), 2);

    // Check that App component contains a Button instance
    let app = &doc.components[1];
    if let Some(body) = &app.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            assert_eq!(children.len(), 1);
            assert!(matches!(
                children[0],
                paperclip_parser::ast::Element::Instance { .. }
            ));

            if let paperclip_parser::ast::Element::Instance { name, .. } = &children[0] {
                assert_eq!(name, "Button");
            }
        }
    }

    println!("✓ Basic component instance parses correctly");
}

#[test]
fn test_component_instance_with_attributes() {
    let source = r#"
        component Button {
            render button {
                text "Click"
            }
        }

        component App {
            render div {
                Button(id="submit-btn", class="primary")
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let app = &doc.components[1];

    if let Some(body) = &app.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Instance { props, .. } = &children[0] {
                assert_eq!(props.len(), 2);
                assert!(props.contains_key("id"));
                assert!(props.contains_key("class"));
            }
        }
    }

    println!("✓ Component instance with attributes parses correctly");
}

#[test]
fn test_default_slot() {
    let source = r#"
        component Card {
            slot content

            render div(class="card") {
                content
            }
        }

        component App {
            render div {
                Card() {
                    text "Card content"
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();

    // Check Card has a slot declaration
    let card = &doc.components[0];
    assert_eq!(card.slots.len(), 1);
    assert_eq!(card.slots[0].name, "content");

    // Check Card body has a SlotInsert
    if let Some(body) = &card.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            assert_eq!(children.len(), 1);
            assert!(matches!(
                children[0],
                paperclip_parser::ast::Element::SlotInsert { .. }
            ));

            if let paperclip_parser::ast::Element::SlotInsert { name, .. } = &children[0] {
                assert_eq!(name, "content");
            }
        }
    }

    // Check App has Card instance with children
    let app = &doc.components[1];
    if let Some(body) = &app.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Instance { children: slot_content, .. } =
                &children[0]
            {
                assert_eq!(slot_content.len(), 1);
                assert!(matches!(
                    slot_content[0],
                    paperclip_parser::ast::Element::Text { .. }
                ));
            }
        }
    }

    println!("✓ Default slot parses correctly");
}

#[test]
fn test_named_slots() {
    let source = r#"
        component Dialog {
            slot header
            slot body
            slot footer

            render div(class="dialog") {
                div(class="header") {
                    header
                }
                div(class="body") {
                    body
                }
                div(class="footer") {
                    footer
                }
            }
        }

        component App {
            render div {
                Dialog() {
                    insert header {
                        text "Title"
                    }
                    insert body {
                        text "Content"
                    }
                    insert footer {
                        text "Actions"
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();

    // Check Dialog has 3 named slots
    let dialog = &doc.components[0];
    assert_eq!(dialog.slots.len(), 3);
    assert_eq!(dialog.slots[0].name, "header");
    assert_eq!(dialog.slots[1].name, "body");
    assert_eq!(dialog.slots[2].name, "footer");

    // Check Dialog body has SlotInserts
    if let Some(body) = &dialog.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            // Should have 3 div children (header, body, footer)
            assert_eq!(children.len(), 3);

            // Each div should contain a SlotInsert
            for child in children {
                if let paperclip_parser::ast::Element::Tag { children: slot_children, .. } = child {
                    assert_eq!(slot_children.len(), 1);
                    assert!(matches!(
                        slot_children[0],
                        paperclip_parser::ast::Element::SlotInsert { .. }
                    ));
                }
            }
        }
    }

    // Check App has Dialog instance with insert directives
    let app = &doc.components[1];
    if let Some(body) = &app.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Instance { children: slot_content, .. } =
                &children[0]
            {
                assert_eq!(slot_content.len(), 3);

                // Each should be an Insert directive
                for content in slot_content {
                    assert!(matches!(
                        content,
                        paperclip_parser::ast::Element::Insert { .. }
                    ));
                }
            }
        }
    }

    println!("✓ Named slots parse correctly");
}

#[test]
fn test_default_slot_content() {
    let source = r#"
        component Button {
            slot content {
                text "Default Text"
            }

            render button {
                content
            }
        }

        component App {
            render div {
                Button()
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();

    // Check Button has slot with default content
    let button = &doc.components[0];
    assert_eq!(button.slots.len(), 1);
    assert_eq!(button.slots[0].default_content.len(), 1);
    assert!(matches!(
        button.slots[0].default_content[0],
        paperclip_parser::ast::Element::Text { .. }
    ));

    // Body should have a SlotInsert element
    if let Some(body) = &button.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            assert!(matches!(
                children[0],
                paperclip_parser::ast::Element::SlotInsert { .. }
            ));
        }
    }

    println!("✓ Default slot content parses correctly");
}

#[test]
fn test_nested_component_composition() {
    let source = r#"
        component Icon {
            slot icon

            render span(class="icon") {
                icon
            }
        }

        component Button {
            slot label

            render button {
                Icon() {
                    text "→"
                }
                label
            }
        }

        component App {
            render div {
                Button() {
                    text "Next"
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.components.len(), 3);

    // Check Icon has a slot
    let icon = &doc.components[0];
    assert_eq!(icon.slots.len(), 1);
    assert_eq!(icon.slots[0].name, "icon");

    // Check Button has a slot and uses Icon
    let button = &doc.components[1];
    assert_eq!(button.slots.len(), 1);
    assert_eq!(button.slots[0].name, "label");

    if let Some(body) = &button.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            assert_eq!(children.len(), 2);
            assert!(matches!(
                children[0],
                paperclip_parser::ast::Element::Instance { .. }
            ));
            assert!(matches!(
                children[1],
                paperclip_parser::ast::Element::SlotInsert { .. }
            ));
        }
    }

    // Check App has Button instance
    let app = &doc.components[2];
    if let Some(body) = &app.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            assert!(matches!(
                children[0],
                paperclip_parser::ast::Element::Instance { .. }
            ));
        }
    }

    println!("✓ Nested component composition parses correctly");
}

#[test]
fn test_multiple_slot_contents() {
    let source = r#"
        component Card {
            slot content {
                text "Empty"
            }

            render div(class="card") {
                content
            }
        }

        component App {
            render div {
                Card() {
                    div(class="item") {
                        text "Item 1"
                    }
                    div(class="item") {
                        text "Item 2"
                    }
                    div(class="item") {
                        text "Item 3"
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let app = &doc.components[1];

    if let Some(body) = &app.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Instance { children: slot_content, .. } =
                &children[0]
            {
                // Should have 3 items in slot content
                assert_eq!(slot_content.len(), 3);
            }
        }
    }

    println!("✓ Multiple slot contents parse correctly");
}

#[test]
fn test_slots_with_conditional_content() {
    let source = r#"
        component Card {
            slot content

            render div(class="card") {
                content
            }
        }

        component App {
            render div {
                Card() {
                    if isVisible {
                        text "Conditional content"
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let app = &doc.components[1];

    if let Some(body) = &app.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Instance { children: slot_content, .. } =
                &children[0]
            {
                assert_eq!(slot_content.len(), 1);
                assert!(matches!(
                    slot_content[0],
                    paperclip_parser::ast::Element::Conditional { .. }
                ));
            }
        }
    }

    println!("✓ Slots with conditional content parse correctly");
}

#[test]
fn test_slots_with_repeat_content() {
    let source = r#"
        component List {
            slot items

            render ul {
                items
            }
        }

        component App {
            render div {
                List() {
                    repeat item in items {
                        li {
                            text item.name
                        }
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let app = &doc.components[1];

    if let Some(body) = &app.body {
        if let paperclip_parser::ast::Element::Tag { children, .. } = body {
            if let paperclip_parser::ast::Element::Instance { children: slot_content, .. } =
                &children[0]
            {
                assert_eq!(slot_content.len(), 1);
                assert!(matches!(
                    slot_content[0],
                    paperclip_parser::ast::Element::Repeat { .. }
                ));
            }
        }
    }

    println!("✓ Slots with repeat content parse correctly");
}

#[test]
fn test_component_composition_with_styles() {
    let source = r#"
        component Badge {
            slot label

            render span {
                style {
                    padding: 4px 8px
                    border-radius: 4px
                }
                label
            }
        }

        component App {
            render div {
                Badge() {
                    text "New"
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();

    // Check Badge has slot and styles
    let badge = &doc.components[0];
    assert_eq!(badge.slots.len(), 1);

    if let Some(body) = &badge.body {
        if let paperclip_parser::ast::Element::Tag { styles, children, .. } = body {
            assert_eq!(styles.len(), 1);
            assert!(!styles[0].properties.is_empty());
            assert_eq!(children.len(), 1);
            assert!(matches!(
                children[0],
                paperclip_parser::ast::Element::SlotInsert { .. }
            ));
        }
    }

    println!("✓ Component composition with styles parses correctly");
}

#[test]
fn test_real_world_layout_composition() {
    let source = r#"
        component Header {
            slot logo
            slot nav

            render header {
                logo
                nav
            }
        }

        component Sidebar {
            slot content

            render aside {
                content
            }
        }

        component Layout {
            slot sidebarContent
            slot mainContent

            render div(class="layout") {
                Header() {
                    insert logo {
                        text "Logo"
                    }
                    insert nav {
                        text "Navigation"
                    }
                }
                div(class="main") {
                    Sidebar() {
                        sidebarContent
                    }
                    div(class="content") {
                        mainContent
                    }
                }
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.components.len(), 3);

    // Check each component has correct slots
    assert_eq!(doc.components[0].slots.len(), 2); // Header
    assert_eq!(doc.components[1].slots.len(), 1); // Sidebar
    assert_eq!(doc.components[2].slots.len(), 2); // Layout

    println!("✓ Real-world layout composition parses correctly");
}
