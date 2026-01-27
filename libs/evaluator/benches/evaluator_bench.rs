//! Evaluator benchmarks
//!
//! Target: Evaluate medium component in <20ms

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use paperclip_evaluator::evaluate;

fn generate_component(num_elements: usize) -> String {
    let mut source = String::new();
    
    // Add some tokens
    source.push_str("public token primary #3366FF\n");
    source.push_str("public token spacing 16px\n\n");
    
    // Generate a component with nested elements
    source.push_str("public component TestComponent {\n");
    source.push_str("    render div {\n");
    source.push_str("        style {\n");
    source.push_str("            display: flex\n");
    source.push_str("            flex-direction: column\n");
    source.push_str("        }\n");
    
    for i in 0..num_elements {
        source.push_str(&format!(r#"
        div {{
            style {{
                padding: var(spacing)
                background: var(primary)
            }}
            text "Item {}"
        }}
"#, i));
    }
    
    source.push_str("    }\n");
    source.push_str("}\n");
    
    source
}

fn bench_evaluate_small(c: &mut Criterion) {
    let source = generate_component(5);
    
    c.bench_function("evaluate_small_component", |b| {
        b.iter(|| evaluate(black_box(&source), "test.pc"))
    });
}

fn bench_evaluate_medium(c: &mut Criterion) {
    let source = generate_component(50);
    
    c.bench_function("evaluate_medium_component", |b| {
        b.iter(|| evaluate(black_box(&source), "test.pc"))
    });
}

fn bench_evaluate_large(c: &mut Criterion) {
    let source = generate_component(200);
    
    c.bench_function("evaluate_large_component", |b| {
        b.iter(|| evaluate(black_box(&source), "test.pc"))
    });
}

criterion_group!(benches, bench_evaluate_small, bench_evaluate_medium, bench_evaluate_large);
criterion_main!(benches);
