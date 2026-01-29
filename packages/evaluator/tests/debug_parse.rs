//! Debug test to understand parsing

use paperclip_parser::parse_with_path;

#[test]
fn debug_parse_ul() {
    let source = r#"
        public component TodoList {
            render ul {
                repeat todo in todos {
                    li {
                        text "Item"
                    }
                }
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();

    println!("\nComponent body: {:#?}", doc.components[0].body);
}

#[test]
fn debug_parse_div() {
    let source = r#"
        component Card {
            slot children {
                text "Default content"
            }

            render div {
                children
            }
        }
    "#;

    let doc = parse_with_path(source, "/test.pc").unwrap();

    println!("\nComponent body: {:#?}", doc.components[0].body);
}
