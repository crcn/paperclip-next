#[cfg(test)]
mod comprehensive_example_test {
    use crate::parse_with_path;
    use std::fs;

    #[test]
    fn test_parse_comprehensive_example() {
        let source = fs::read_to_string("../../examples/comprehensive-features.pc")
            .expect("Failed to read comprehensive example file");

        let result = parse_with_path(&source, "examples/comprehensive-features.pc");

        match &result {
            Ok(doc) => {
                println!("✓ Successfully parsed comprehensive example!");
                println!("  - {} imports", doc.imports.len());
                println!("  - {} tokens", doc.tokens.len());
                println!("  - {} triggers", doc.triggers.len());
                println!("  - {} styles", doc.styles.len());
                println!("  - {} components", doc.components.len());

                for component in &doc.components {
                    println!("\n  Component: {}", component.name);
                    if component.script.is_some() {
                        println!("    - Has script directive");
                    }
                    println!("    - {} variants", component.variants.len());
                    println!("    - {} slots", component.slots.len());
                }
            }
            Err(e) => {
                eprintln!("Parse error: {:?}", e);
                panic!("Failed to parse comprehensive example");
            }
        }

        assert!(
            result.is_ok(),
            "Comprehensive example should parse successfully"
        );
    }
}
