use paperclip_parser::parse_with_path;
use std::fs;

fn main() {
    let example_files = vec![
        "../evaluator/examples/test.pc",
        "../evaluator/examples/simple_test.pc",
        "../evaluator/examples/styled_test.pc",
        "../evaluator/examples/list_test.pc",
    ];

    for file_path in example_files {
        print!("Parsing {}... ", file_path);
        match fs::read_to_string(file_path) {
            Ok(source) => {
                match parse_with_path(&source, file_path) {
                    Ok(_doc) => println!("✅ Success"),
                    Err(e) => println!("❌ Parse error: {:?}", e),
                }
            }
            Err(e) => println!("❌ File error: {}", e),
        }
    }
}
