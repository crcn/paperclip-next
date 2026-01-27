use criterion::{black_box, criterion_group, criterion_main, Criterion};
use paperclip_evaluator::Evaluator;
use paperclip_parser::parse;

fn evaluate_simple_component(c: &mut Criterion) {
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

    let doc = parse(source).unwrap();

    c.bench_function("evaluate_simple_component", |b| {
        b.iter(|| {
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&doc))
        })
    });
}

fn evaluate_medium_component(c: &mut Criterion) {
    let source = r#"
        public token primaryColor #3366FF
        public token spacing 16px

        public component Card {
            render div {
                style {
                    padding: 16px
                    background: white
                    border-radius: 8px
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
                    text "Card description"
                }

                button {
                    style {
                        padding: 8px 16px
                        background: #3366FF
                        color: white
                        border: none
                    }
                    text "Action"
                }
            }
        }
    "#;

    let doc = parse(source).unwrap();

    c.bench_function("evaluate_medium_component", |b| {
        b.iter(|| {
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&doc))
        })
    });
}

fn evaluate_multiple_components(c: &mut Criterion) {
    let mut source = String::new();

    // Add multiple components
    for i in 0..10 {
        source.push_str(&format!(
            r#"
            public component Component{} {{
                render div {{
                    style {{
                        padding: 16px
                    }}
                    div {{
                        text "Component {}"
                    }}
                    button {{
                        text "Click"
                    }}
                }}
            }}
            "#,
            i, i
        ));
    }

    let doc = parse(&source).unwrap();

    c.bench_function("evaluate_10_components", |b| {
        b.iter(|| {
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&doc))
        })
    });
}

fn parse_and_evaluate(c: &mut Criterion) {
    let source = r#"
        public component Card {
            render div {
                style {
                    padding: 16px
                    background: white
                }
                div {
                    text "Hello"
                }
                button {
                    text "Click"
                }
            }
        }
    "#;

    c.bench_function("parse_and_evaluate_combined", |b| {
        b.iter(|| {
            let doc = parse(black_box(source)).unwrap();
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(&doc)
        })
    });
}

criterion_group!(
    benches,
    evaluate_simple_component,
    evaluate_medium_component,
    evaluate_multiple_components,
    parse_and_evaluate
);
criterion_main!(benches);
