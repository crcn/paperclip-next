//! Parser benchmarks
//!
//! Target: Parse 1000-line file in <10ms

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use paperclip_parser::parse;

fn generate_large_file(num_components: usize) -> String {
    let mut source = String::new();
    
    // Add some imports
    source.push_str("import \"./tokens.pc\" as tokens\n\n");
    
    // Add some tokens
    for i in 0..10 {
        source.push_str(&format!("public token color{} #{}66FF\n", i, i));
    }
    source.push('\n');
    
    // Generate components
    for i in 0..num_components {
        source.push_str(&format!(r#"
/** @frame(x: {}, y: 0, width: 200, height: 100) */
public component Component{} {{
    variant hover trigger {{ ":hover" }}
    
    render div {{
        style {{
            display: flex
            flex-direction: column
            padding: 16px
            background: var(tokens.color0)
        }}
        style variant hover {{
            background: var(tokens.color1)
        }}
        div {{
            style {{
                font-size: 14px
                font-weight: bold
            }}
            text "Title {}"
        }}
        div {{
            style {{
                font-size: 12px
                color: #666
            }}
            text "Description for component {}"
        }}
        slot children {{
            text "Default content"
        }}
    }}
}}
"#, i * 250, i, i, i));
    }
    
    source
}

fn bench_parse_small(c: &mut Criterion) {
    let source = r#"
public component Button {
    variant hover trigger { ":hover" }
    
    render button {
        style {
            padding: 8px 16px
            background: #3366FF
        }
        text "Click me"
    }
}
"#;

    c.bench_function("parse_small_component", |b| {
        b.iter(|| parse(black_box(source)))
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    // ~100 lines
    let source = generate_large_file(3);
    
    c.bench_function("parse_medium_file_100_lines", |b| {
        b.iter(|| parse(black_box(&source)))
    });
}

fn bench_parse_large(c: &mut Criterion) {
    // ~1000 lines
    let source = generate_large_file(30);
    
    println!("Large file size: {} lines", source.lines().count());
    
    c.bench_function("parse_large_file_1000_lines", |b| {
        b.iter(|| parse(black_box(&source)))
    });
}

fn bench_lexer_only(c: &mut Criterion) {
    let source = generate_large_file(30);
    
    c.bench_function("lex_large_file_1000_lines", |b| {
        b.iter(|| {
            let tokens: Vec<_> = paperclip_parser::lexer::lex(black_box(&source)).collect();
            tokens
        })
    });
}

criterion_group!(benches, bench_parse_small, bench_parse_medium, bench_parse_large, bench_lexer_only);
criterion_main!(benches);
