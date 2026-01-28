use criterion::{black_box, criterion_group, criterion_main, Criterion};
use paperclip_parser::parse;

fn parse_simple_component(c: &mut Criterion) {
    let source = r#"
        public component Button {
            render button {
                style {
                    padding: 8px 16px
                    background: #3366FF
                }
                text "Click me"
            }
        }
    "#;

    c.bench_function("parse_simple_component", |b| {
        b.iter(|| parse(black_box(source)))
    });
}

fn parse_medium_component(c: &mut Criterion) {
    let source = r#"
        public token primaryColor #3366FF
        public token spacing 16px

        public component Card {
            variant hover trigger { ":hover" }

            render div {
                style {
                    padding: var(spacing)
                    background: white
                    border-radius: 8px
                    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1)
                }

                style variant hover {
                    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15)
                }

                div {
                    style {
                        font-size: 24px
                        font-weight: bold
                        margin-bottom: 8px
                    }
                    text "Card Title"
                }

                div {
                    style {
                        color: #666
                        line-height: 1.5
                    }
                    text "Card description goes here"
                }

                button {
                    style {
                        padding: 8px 16px
                        background: var(primaryColor)
                        color: white
                        border: none
                        border-radius: 4px
                    }
                    text "Action"
                }
            }
        }
    "#;

    c.bench_function("parse_medium_component", |b| {
        b.iter(|| parse(black_box(source)))
    });
}

fn parse_large_file(c: &mut Criterion) {
    // Simulate a larger file with multiple components
    let mut source = String::new();

    // Add tokens
    for i in 0..10 {
        source.push_str(&format!("public token color{} #FF{:04X}\n", i, i * 1000));
    }

    // Add multiple components
    for i in 0..20 {
        source.push_str(&format!(
            r#"
            public component Component{} {{
                render div {{
                    style {{
                        padding: 16px
                        background: #F0F0F0
                    }}
                    div {{
                        text "Component {}"
                    }}
                    button {{
                        style {{
                            padding: 8px
                        }}
                        text "Click"
                    }}
                }}
            }}
            "#,
            i, i
        ));
    }

    c.bench_function("parse_large_file_1000_lines", |b| {
        b.iter(|| parse(black_box(&source)))
    });
}

fn tokenize_only(c: &mut Criterion) {
    use paperclip_parser::tokenize;

    let source = r#"
        public component Button {
            render button {
                style {
                    padding: 8px 16px
                    background: #3366FF
                }
                text "Click me"
            }
        }
    "#;

    c.bench_function("tokenize_only", |b| {
        b.iter(|| tokenize(black_box(source)))
    });
}

criterion_group!(
    benches,
    parse_simple_component,
    parse_medium_component,
    parse_large_file,
    tokenize_only
);
criterion_main!(benches);
